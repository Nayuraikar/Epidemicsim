//! Thread-level-parallelism experiments: the counting race demo and the load-imbalance study.
use crate::{machine_info, timing, Machine, Timing};
use epidemicsim_backends::engine::{Engine, Variant};
use epidemicsim_backends::par::{Par, Phase, RayonPar, Schedule, SharedMut};
use epidemicsim_core::agents::state;
use epidemicsim_core::{schedule, synth, Backend, Config};
use epidemicsim_kernels_c as kc;
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};
use std::{error::Error, path::Path, time::Instant};

// ------------------------------------------------------------------ race demo
#[derive(Clone, Debug)]
pub struct RaceOpts {
    pub agents: usize,
    pub threads: usize,
    pub runs: usize,
    pub chunk: usize,
    /// Steps simulated before the snapshot (a weekend daytime step is used; day 82 is near the peak).
    pub steps: u32,
}

impl Default for RaceOpts {
    fn default() -> Self {
        Self {
            agents: 1_000_000,
            threads: 8,
            runs: 200,
            chunk: 64,
            steps: 330, // day 82 (a Saturday) daytime slot, near the epidemic peak
        }
    }
}

#[derive(Serialize)]
pub struct MethodResult {
    pub method: &'static str,
    /// True when every run matched the sequential counts.
    pub correct_in_all_runs: bool,
    pub runs: usize,
    pub wrong_runs: usize,
    /// Largest number of lost increments (sequential total - total counted) in any run.
    pub max_lost_increments: u64,
    pub time_ms: Timing,
}

#[derive(Serialize)]
pub struct BackendRace {
    pub backend: &'static str,
    pub methods: Vec<MethodResult>,
    /// Fastest method (median time) among those that were correct in every run; this is what
    /// the production backends use.
    pub fastest_correct: &'static str,
}

fn fastest_correct(m: &[MethodResult]) -> &'static str {
    m.iter()
        .filter(|r| r.correct_in_all_runs)
        .min_by(|a, b| a.time_ms.median.total_cmp(&b.time_ms.median))
        .map_or("none", |r| r.method)
}

#[derive(Serialize)]
pub struct Scenario {
    pub name: &'static str,
    pub description: &'static str,
    pub infectious_agents: usize,
    pub locations_with_infectious: usize,
    pub backends: Vec<BackendRace>,
}

#[derive(Serialize)]
pub struct RaceReport {
    pub machine: Machine,
    pub agents: usize,
    pub locations: usize,
    pub threads: usize,
    pub chunk: usize,
    pub schedule: &'static str,
    pub runs: usize,
    pub snapshot_step: u32,
    pub note: &'static str,
    pub scenarios: Vec<Scenario>,
}

/// Rust: unsynchronised "increment" written as relaxed atomic load + store. That is defined
/// behaviour (no UB) but still NOT an atomic read-modify-write: two threads can load the same
/// old value and both store old+1, losing one increment.
fn rust_unsync(par: &RayonPar, loc: &[u32], inf: &[u8], counts: &[AtomicU32]) {
    par.run(Phase::Update, loc.len(), &|_t, r| {
        for i in r {
            if inf[i] != 0 {
                let c = &counts[loc[i] as usize];
                c.store(c.load(Relaxed) + 1, Relaxed); // lost-update window between the two
            }
        }
    });
}

/// Rust: one atomic read-modify-write per increment: always correct.
fn rust_atomic(par: &RayonPar, loc: &[u32], inf: &[u8], counts: &[AtomicU32]) {
    par.run(Phase::Update, loc.len(), &|_t, r| {
        for i in r {
            if inf[i] != 0 {
                counts[loc[i] as usize].fetch_add(1, Relaxed);
            }
        }
    });
}

/// Rust: a private counter array per thread, then a parallel reduction over locations.
fn rust_buffers(par: &RayonPar, loc: &[u32], inf: &[u8], nloc: usize, counts: &[AtomicU32]) {
    let t = par.threads();
    let mut bufs = vec![0u32; t * nloc];
    let sh = SharedMut::new(&mut bufs);
    par.run(Phase::Update, loc.len(), &|tid, r| {
        // SAFETY: thread `tid` only ever touches its own nloc-sized region.
        let mine = unsafe { sh.slice(tid * nloc..(tid + 1) * nloc) };
        for i in r {
            if inf[i] != 0 {
                mine[loc[i] as usize] += 1;
            }
        }
    });
    let bufs = &bufs;
    par.run(Phase::Location, nloc, &|_t, r| {
        for l in r {
            let s: u32 = (0..t).map(|k| bufs[k * nloc + l]).sum();
            counts[l].store(s, Relaxed);
        }
    });
}

fn c_race(method: i32, threads: usize, chunk: usize, loc: &[u32], inf: &[u8], out: &mut [u32]) {
    // SAFETY: slices outlive the call and have the lengths passed.
    unsafe {
        kc::ep_race(
            method,
            threads as i32,
            loc.len() as i64,
            out.len() as i64,
            loc.as_ptr(),
            inf.as_ptr(),
            chunk as i64,
            out.as_mut_ptr(),
        )
    }
}

fn evaluate(
    name: &'static str,
    runs: usize,
    want: &[u32],
    mut f: impl FnMut(&mut [u32]) -> f64,
) -> MethodResult {
    let total: u64 = want.iter().map(|&x| u64::from(x)).sum();
    let (mut wrong, mut lost) = (0usize, 0u64);
    let mut ms = Vec::new();
    let mut got = vec![0u32; want.len()];
    for _ in 0..runs {
        ms.push(f(&mut got));
        if got != want {
            wrong += 1;
            let g: u64 = got.iter().map(|&x| u64::from(x)).sum();
            lost = lost.max(total.saturating_sub(g));
        }
    }
    MethodResult {
        method: name,
        correct_in_all_runs: wrong == 0,
        runs,
        wrong_runs: wrong,
        max_lost_increments: lost,
        time_ms: timing(&ms),
    }
}

pub fn race(o: &RaceOpts) -> Result<RaceReport, Box<dyn Error>> {
    let mut world = synth::generate(&Config {
        agents: o.agents,
        ..Default::default()
    });
    let mut engine = Engine::new(
        RayonPar::new(o.threads, Schedule::Static, 0)?,
        Variant::Optimised,
        "snapshot",
    );
    let iv = Default::default();
    for _ in 0..o.steps {
        engine.step(&mut world, &iv);
    }
    let n = world.agents.len();
    let nloc = world.layout.total();
    let infectious: Vec<u8> = world
        .agents
        .state
        .iter()
        .map(|&s| u8::from(s == state::A || s == state::I))
        .collect();
    let realistic: Vec<u32> = (0..n)
        .map(|i| {
            if world.agents.state[i] == state::D {
                0
            } else {
                schedule::location_of(&world, i, world.step, &iv)
            }
        })
        .collect();
    // Stress case: the same infectious agents, all crammed into 64 locations.
    let hotspot: Vec<u32> = (0..n).map(|i| (i % 64) as u32).collect();

    let par = RayonPar::new(o.threads, Schedule::Dynamic, o.chunk)?;
    let atomics: Vec<AtomicU32> = (0..nloc).map(|_| AtomicU32::new(0)).collect();
    let mut scenarios = Vec::new();
    for (name, description, loc) in [
        (
            "realistic",
            "simulated placement at a Saturday daytime step near the epidemic peak (market visits spread infectious agents over shared locations)",
            &realistic,
        ),
        (
            "hotspot",
            "synthetic stress case: the same infectious agents all mapped to 64 locations",
            &hotspot,
        ),
    ] {
        let mut want = vec![0u32; nloc];
        for i in 0..n {
            if infectious[i] != 0 {
                want[loc[i] as usize] += 1;
            }
        }
        let snap = |out: &mut [u32]| {
            for (o, a) in out.iter_mut().zip(&atomics) {
                *o = a.load(Relaxed);
            }
        };
        let zero = || atomics.iter().for_each(|a| a.store(0, Relaxed));
        let mut rust = Vec::new();
        rust.push(evaluate("unsynchronised", o.runs, &want, |out| {
            let t0 = Instant::now();
            zero();
            rust_unsync(&par, loc, &infectious, &atomics);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            snap(out);
            ms
        }));
        rust.push(evaluate("atomic", o.runs, &want, |out| {
            let t0 = Instant::now();
            zero();
            rust_atomic(&par, loc, &infectious, &atomics);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            snap(out);
            ms
        }));
        rust.push(evaluate("per_thread_buffers", o.runs, &want, |out| {
            let t0 = Instant::now();
            rust_buffers(&par, loc, &infectious, nloc, &atomics);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            snap(out);
            ms
        }));
        let mut backends = vec![BackendRace {
            backend: "rayon",
            fastest_correct: fastest_correct(&rust),
            methods: rust,
        }];
        if kc::HAS_OPENMP {
            let mut c = Vec::new();
            for (m, label) in [(0, "unsynchronised"), (1, "atomic"), (2, "per_thread_buffers")] {
                c.push(evaluate(label, o.runs, &want, |out| {
                    let t0 = Instant::now();
                    c_race(m, o.threads, o.chunk, loc, &infectious, out);
                    t0.elapsed().as_secs_f64() * 1e3
                }));
            }
            backends.push(BackendRace {
                backend: "openmp",
                fastest_correct: fastest_correct(&c),
                methods: c,
            });
        }
        scenarios.push(Scenario {
            name,
            description,
            infectious_agents: infectious.iter().map(|&x| usize::from(x)).sum(),
            locations_with_infectious: want.iter().filter(|&&x| x > 0).count(),
            backends,
        });
    }
    Ok(RaceReport {
        machine: machine_info(),
        agents: n,
        locations: nloc,
        threads: o.threads,
        chunk: o.chunk,
        schedule: "dynamic",
        runs: o.runs,
        snapshot_step: world.step,
        note: "unsynchronised = lost-update race (Rust: relaxed load+store, C: plain ++). \
               wrong_runs counts runs whose per-location counts differ from the sequential count.",
        scenarios,
    })
}

pub fn write_json<T: Serialize>(path: &Path, v: &T) -> Result<(), Box<dyn Error>> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(v)? + "\n")?;
    Ok(())
}

// ------------------------------------------------------------------ load imbalance
use epidemicsim_backends::openmp::OpenMpBackend;
use epidemicsim_backends::par::PhaseStat;
use epidemicsim_core::world::STEPS_PER_DAY;
use epidemicsim_core::World;

#[derive(Clone, Debug)]
pub struct ImbalanceOpts {
    pub city: String,
    pub threads: Vec<usize>,
    pub days: u32,
    pub reps: usize,
    pub chunk: usize,
}

#[derive(Serialize)]
pub struct PhaseImbalance {
    pub phase: &'static str,
    pub per_thread: Vec<epidemicsim_backends::par::ThreadStat>,
    /// max / mean of per-thread work time (1.0 = perfectly balanced).
    pub time_imbalance: f64,
    /// max / mean of per-thread item counts.
    pub items_imbalance: f64,
}

#[derive(Serialize)]
pub struct ImbalancePoint {
    pub backend: &'static str,
    pub schedule: &'static str,
    pub chunk: usize,
    pub threads: usize,
    pub wall_s: f64,
    pub phases: Vec<PhaseImbalance>,
    /// Per-thread work summed over the three phases.
    pub total_work_s: Vec<f64>,
    pub total_time_imbalance: f64,
}

#[derive(Serialize)]
pub struct ImbalanceReport {
    pub machine: Machine,
    pub city: String,
    pub agents: usize,
    pub locations: usize,
    pub days: u32,
    pub steps: u32,
    pub reps: usize,
    pub note: &'static str,
    pub points: Vec<ImbalancePoint>,
}

fn max_over_mean(v: &[f64]) -> f64 {
    let mean = v.iter().sum::<f64>() / v.len().max(1) as f64;
    if mean > 0.0 {
        v.iter().cloned().fold(0.0, f64::max) / mean
    } else {
        1.0
    }
}

fn summarise(stats: Vec<PhaseStat>) -> (Vec<PhaseImbalance>, Vec<f64>, f64) {
    let threads = stats[0].threads.len();
    let mut total = vec![0.0; threads];
    let phases: Vec<PhaseImbalance> = stats
        .into_iter()
        .map(|p| {
            let t: Vec<f64> = p.threads.iter().map(|x| x.work_s).collect();
            let it: Vec<f64> = p.threads.iter().map(|x| x.items as f64).collect();
            for (a, b) in total.iter_mut().zip(&t) {
                *a += b;
            }
            PhaseImbalance {
                phase: p.phase,
                time_imbalance: max_over_mean(&t),
                items_imbalance: max_over_mean(&it),
                per_thread: p.threads,
            }
        })
        .collect();
    let imb = max_over_mean(&total);
    (phases, total, imb)
}

/// One timed run of `steps` steps on a clone of `world`; returns (wall seconds, per-thread stats).
fn timed<B: Backend>(
    b: &mut B,
    world: &World,
    steps: u32,
    reset: &mut dyn FnMut(&mut B),
    stats: &dyn Fn(&B) -> Vec<PhaseStat>,
) -> (f64, Vec<PhaseStat>) {
    let mut w = world.clone();
    let iv = Default::default();
    reset(b);
    let t0 = Instant::now();
    for _ in 0..steps {
        b.step(&mut w, &iv);
    }
    (t0.elapsed().as_secs_f64(), stats(b))
}

pub fn imbalance(world: &World, o: &ImbalanceOpts) -> Result<ImbalanceReport, Box<dyn Error>> {
    let steps = o.days * STEPS_PER_DAY;
    let mut points = Vec::new();
    for backend in ["rayon", "openmp"] {
        if backend == "openmp" && !kc::HAS_OPENMP {
            continue;
        }
        for sched in [Schedule::Static, Schedule::Dynamic, Schedule::Guided] {
            for &t in &o.threads {
                let mut runs: Vec<(f64, Vec<PhaseStat>)> = Vec::new();
                for _ in 0..o.reps.max(1) + 1 {
                    // First repetition is a warm-up and is discarded below.
                    let r = if backend == "rayon" {
                        let mut e = Engine::new(
                            RayonPar::new(t, sched, o.chunk)?,
                            Variant::Optimised,
                            "rayon",
                        );
                        timed(&mut e, world, steps, &mut |e| e.par.reset_stats(), &|e| {
                            e.par.stats()
                        })
                    } else {
                        let mut e = OpenMpBackend::new(t, sched, o.chunk)?;
                        timed(&mut e, world, steps, &mut |e| e.reset_stats(), &|e| {
                            e.stats()
                        })
                    };
                    runs.push(r);
                }
                runs.remove(0);
                runs.sort_by(|a, b| a.0.total_cmp(&b.0));
                let (wall, stats) = runs.swap_remove(runs.len() / 2);
                let (phases, total, imb) = summarise(stats);
                let chunk = if o.chunk == 0 {
                    sched.default_chunk()
                } else {
                    o.chunk
                };
                eprintln!(
                    "{} {} threads={t}: wall {wall:.3}s  imbalance(max/mean) {imb:.3}",
                    backend,
                    sched.name()
                );
                points.push(ImbalancePoint {
                    backend: if backend == "rayon" {
                        "rayon"
                    } else {
                        "openmp"
                    },
                    schedule: sched.name(),
                    chunk,
                    threads: t,
                    wall_s: wall,
                    phases,
                    total_work_s: total,
                    total_time_imbalance: imb,
                });
            }
        }
    }
    Ok(ImbalanceReport {
        machine: machine_info(),
        city: o.city.clone(),
        agents: world.agents.len(),
        locations: world.layout.total(),
        days: o.days,
        steps,
        reps: o.reps,
        note: "work_s: rayon = time inside kernel calls; openmp = from loop start to the thread's last \
               iteration (includes scheduling overhead). Stats are the repetition with median wall time.",
        points,
    })
}

// ------------------------------------------------------------------ SIMD alignment study
use crate::hardware_tag;
use epidemicsim_backends::simd::{self, AlignedBuf};

#[derive(Serialize)]
pub struct AlignRow {
    pub kernel: &'static str,
    pub variant: &'static str,
    pub alignment: &'static str,
    pub elements: usize,
    /// Median nanoseconds per element over the repetitions.
    pub ns_per_element: f64,
}

#[derive(Serialize)]
pub struct AlignReport {
    pub machine: Machine,
    pub elements: usize,
    pub passes_per_sample: usize,
    pub reps: usize,
    pub note: &'static str,
    pub rows: Vec<AlignRow>,
}

/// Timer kernel and Philox stream kernel, scalar vs vector, on 64-byte-aligned buffers vs the
/// same data shifted by one element (misaligned, vector loads straddle cache lines).
pub fn align_bench(out_dir: &Path, reps: usize) -> Result<std::path::PathBuf, Box<dyn Error>> {
    const N: usize = 1 << 20;
    const PASSES: usize = 20;
    let mut rows = Vec::new();
    let mut st = AlignedBuf::<u8>::new(N + 64);
    let mut tm = AlignedBuf::<u16>::new(N + 64);
    let mut ids = AlignedBuf::<u32>::new(N + 64);
    let mut out = AlignedBuf::<u32>::new(N + 64);
    for i in 0..N + 64 {
        st.as_mut_slice()[i] = 1 + (i % 4) as u8; // always infected
        tm.as_mut_slice()[i] = u16::MAX; // never expires during the benchmark
        ids.as_mut_slice()[i] = i as u32;
    }
    for (alignment, off) in [("aligned_64B", 0usize), ("misaligned_+1_element", 1)] {
        let mut kinds = vec![simd::SimdKind::Scalar];
        if simd::SimdKind::Neon.check().is_ok() {
            kinds.push(simd::SimdKind::Neon);
        }
        if simd::SimdKind::Avx2.check().is_ok() {
            kinds.push(simd::SimdKind::Avx2);
        }
        for k in kinds {
            // timer kernel
            let mut ms = Vec::new();
            for _ in 0..reps {
                let mut trans = Vec::new();
                let t0 = Instant::now();
                for _ in 0..PASSES {
                    simd::timer_dispatch(
                        k,
                        &st.as_slice()[off..off + N],
                        &mut tm.as_mut_slice()[off..off + N],
                        &mut trans,
                    );
                }
                ms.push(t0.elapsed().as_secs_f64() * 1e9 / (PASSES * N) as f64);
            }
            rows.push(AlignRow {
                kernel: "timer_recovery",
                variant: k.name(),
                alignment,
                elements: N,
                ns_per_element: timing(&ms).median,
            });
            // philox stream kernel
            let mut ms = Vec::new();
            for _ in 0..reps {
                let t0 = Instant::now();
                for _ in 0..PASSES {
                    simd::stream_dispatch(
                        k,
                        &ids.as_slice()[off..off + N],
                        3,
                        [42, 0],
                        &mut out.as_mut_slice()[off..off + N],
                    );
                }
                ms.push(t0.elapsed().as_secs_f64() * 1e9 / (PASSES * N) as f64);
            }
            rows.push(AlignRow {
                kernel: "philox_stream",
                variant: k.name(),
                alignment,
                elements: N,
                ns_per_element: timing(&ms).median,
            });
        }
    }
    let machine = machine_info();
    let report = AlignReport {
        machine: machine.clone(),
        elements: N,
        passes_per_sample: PASSES,
        reps,
        note: "aligned: slices start on a 64-byte boundary; misaligned: start +1 element \
               (+1 byte for u8 states, +2 for u16 timers, +4 for u32). Single thread.",
        rows,
    };
    let path = out_dir.join(format!("simd_alignment_{}.json", hardware_tag(&machine)));
    write_json(&path, &report)?;
    Ok(path)
}
