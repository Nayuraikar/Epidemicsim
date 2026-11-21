//! Parallel-for executors: serial, and a rayon-based one with a hand-written chunk scheduler
//! that imitates OpenMP's `schedule(static|dynamic|guided)`.
use serde::Serialize;
use std::ops::Range;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

/// The three parallel regions of a step (used to key per-thread statistics).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Placement of agents + occupancy counting (agent-parallel).
    Place = 0,
    /// Force of infection per location (location-parallel).
    Location = 1,
    /// Progression, infection, counts (agent-parallel).
    Update = 2,
}
pub const PHASES: usize = 3;
pub const PHASE_NAMES: [&str; PHASES] = ["place_agents", "location_contacts", "update_agents"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Schedule {
    Static,
    Dynamic,
    Guided,
}

impl FromStr for Schedule {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "static" => Ok(Self::Static),
            "dynamic" => Ok(Self::Dynamic),
            "guided" => Ok(Self::Guided),
            _ => Err(format!("unknown schedule '{s}' (static|dynamic|guided)")),
        }
    }
}

impl Schedule {
    pub fn name(self) -> &'static str {
        match self {
            Self::Static => "static",
            Self::Dynamic => "dynamic",
            Self::Guided => "guided",
        }
    }

    /// Chunk size used when `--chunk` is 0/absent: 0 means "one contiguous block per thread"
    /// for static; dynamic and guided need a (minimum) chunk to be useful.
    pub fn default_chunk(self) -> usize {
        match self {
            Self::Static => 0,
            Self::Dynamic => 1024,
            Self::Guided => 64,
        }
    }
}

/// Raw view of a slice that threads write at disjoint indices.
#[derive(Clone, Copy)]
pub struct SharedMut<T> {
    ptr: *mut T,
    len: usize,
}
// SAFETY: access is only through `slice`, whose contract requires disjoint ranges.
unsafe impl<T: Send> Send for SharedMut<T> {}
unsafe impl<T: Send> Sync for SharedMut<T> {}

impl<T> SharedMut<T> {
    pub fn new(s: &mut [T]) -> Self {
        Self {
            ptr: s.as_mut_ptr(),
            len: s.len(),
        }
    }

    /// # Safety
    /// No two live slices obtained from the same `SharedMut` may overlap, and the original
    /// slice must outlive them. The schedulers below hand every index to exactly one thread.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn slice(&self, r: Range<usize>) -> &mut [T] {
        assert!(r.start <= r.end && r.end <= self.len);
        std::slice::from_raw_parts_mut(self.ptr.add(r.start), r.end - r.start)
    }
}

/// Runs `f(thread_id, range)` over `0..n`; every index is covered by exactly one call.
pub trait Par: Sync {
    fn threads(&self) -> usize;
    /// True when `f` is only ever called from one thread (accumulators need no atomics).
    fn serial(&self) -> bool {
        self.threads() == 1
    }
    fn run(&self, phase: Phase, n: usize, f: &(dyn Fn(usize, Range<usize>) + Sync));
}

pub struct Serial;

impl Par for Serial {
    fn threads(&self) -> usize {
        1
    }
    fn run(&self, _phase: Phase, n: usize, f: &(dyn Fn(usize, Range<usize>) + Sync)) {
        f(0, 0..n);
    }
}

/// Per-thread counters, padded to a cache line so threads do not false-share.
#[repr(align(64))]
#[derive(Default)]
struct Cell {
    ns: AtomicU64,
    items: AtomicU64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ThreadStat {
    pub thread: usize,
    /// Seconds spent inside kernel calls on this thread (rayon) or from the start of the
    /// loop to this thread's last iteration (OpenMP).
    pub work_s: f64,
    pub items: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PhaseStat {
    pub phase: &'static str,
    pub threads: Vec<ThreadStat>,
}

pub struct RayonPar {
    pool: rayon::ThreadPool,
    threads: usize,
    pub schedule: Schedule,
    pub chunk: usize,
    stats: Vec<Cell>,
}

impl RayonPar {
    pub fn new(threads: usize, schedule: Schedule, chunk: usize) -> Result<Self, String> {
        let threads = threads.max(1);
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            pool,
            threads,
            schedule,
            chunk: if chunk == 0 {
                schedule.default_chunk()
            } else {
                chunk
            },
            stats: (0..PHASES * threads).map(|_| Cell::default()).collect(),
        })
    }

    pub fn reset_stats(&self) {
        for c in &self.stats {
            c.ns.store(0, Relaxed);
            c.items.store(0, Relaxed);
        }
    }

    pub fn stats(&self) -> Vec<PhaseStat> {
        (0..PHASES)
            .map(|p| PhaseStat {
                phase: PHASE_NAMES[p],
                threads: (0..self.threads)
                    .map(|t| {
                        let c = &self.stats[p * self.threads + t];
                        ThreadStat {
                            thread: t,
                            work_s: c.ns.load(Relaxed) as f64 * 1e-9,
                            items: c.items.load(Relaxed),
                        }
                    })
                    .collect(),
            })
            .collect()
    }
}

impl Par for RayonPar {
    fn threads(&self) -> usize {
        self.threads
    }

    fn run(&self, phase: Phase, n: usize, f: &(dyn Fn(usize, Range<usize>) + Sync)) {
        let t = self.threads;
        let (sched, chunk) = (self.schedule, self.chunk);
        // Shared cursor: the next unclaimed index (dynamic and guided).
        let next = AtomicUsize::new(0);
        // `broadcast` runs the closure once on every pool thread, like an OpenMP parallel
        // region; the scheduling inside is entirely ours.
        self.pool.broadcast(|ctx| {
            let tid = ctx.index();
            let (mut ns, mut items) = (0u64, 0u64);
            let mut work = |r: Range<usize>| {
                let t0 = Instant::now();
                items += (r.end - r.start) as u64;
                f(tid, r);
                ns += t0.elapsed().as_nanos() as u64;
            };
            match sched {
                // static: iteration space split before the loop starts, no shared state.
                Schedule::Static if chunk == 0 => {
                    let (lo, hi) = (tid * n / t, (tid + 1) * n / t);
                    if lo < hi {
                        work(lo..hi);
                    }
                }
                // static,chunk: chunks dealt round-robin (thread tid gets chunks tid, tid+t, ...).
                Schedule::Static => {
                    let mut s = tid * chunk;
                    while s < n {
                        work(s..(s + chunk).min(n));
                        s += t * chunk;
                    }
                }
                // dynamic: threads grab fixed-size chunks from one shared atomic counter, so
                // fast threads take more chunks (balances uneven work, costs one RMW per chunk).
                Schedule::Dynamic => loop {
                    let s = next.fetch_add(chunk, Relaxed);
                    if s >= n {
                        break;
                    }
                    work(s..(s + chunk).min(n));
                },
                // guided: chunk = remaining / threads, shrinking towards `chunk` (the minimum);
                // big early chunks keep overhead low, small late chunks even out the tail.
                Schedule::Guided => loop {
                    let cur = next.load(Relaxed);
                    if cur >= n {
                        break;
                    }
                    let rem = n - cur;
                    let size = (rem / t).max(chunk).min(rem);
                    if next
                        .compare_exchange_weak(cur, cur + size, Relaxed, Relaxed)
                        .is_ok()
                    {
                        work(cur..cur + size);
                    }
                },
            }
            let cell = &self.stats[phase as usize * t + tid];
            cell.ns.fetch_add(ns, Relaxed);
            cell.items.fetch_add(items, Relaxed);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[test]
    fn every_index_covered_exactly_once() {
        for sched in [Schedule::Static, Schedule::Dynamic, Schedule::Guided] {
            for chunk in [0, 1, 7, 1000] {
                for n in [0usize, 1, 5, 999, 10_000] {
                    let p = RayonPar::new(4, sched, chunk).unwrap();
                    let hits: Vec<AtomicU32> = (0..n).map(|_| AtomicU32::new(0)).collect();
                    p.run(Phase::Update, n, &|_t, r| {
                        for i in r {
                            hits[i].fetch_add(1, Relaxed);
                        }
                    });
                    assert!(
                        hits.iter().all(|h| h.load(Relaxed) == 1),
                        "{sched:?} {chunk} {n}"
                    );
                    let items: u64 = p.stats()[2].threads.iter().map(|t| t.items).sum();
                    assert_eq!(items, n as u64);
                }
            }
        }
    }
}
