//! MPI distribution of `compare`: the (scenario x run) tasks are spread over ranks.
//!
//! * `static`  : rank r runs the contiguous block of tasks `[r*T/P, (r+1)*T/P)`; results are
//!   gathered on rank 0 once at the end (one message per rank). No coordination while running.
//! * `dynamic` : rank 0 is a master holding the task queue; workers request a task, run it and
//!   send the result with their next request (so there is one message pair per task).
//!
//! Each task is one seeded `run_once`, executed serially on one core, so results do not depend
//! on which rank ran it. Rank 0 aggregates in task order with the same code as the
//! single-process path, which makes the output byte-identical.
use epidemicsim_core::{Backend, StepStats};
use epidemicsim_scenarios::compare::{write_all, CompareOpts};
use epidemicsim_scenarios::ensemble::{ensemble_from_results, run_once};
use epidemicsim_scenarios::travel::{
    build_report, run_coupled, ArmRuns, CityRun, Ctx, Exchange, TravelReport,
};
use epidemicsim_scenarios::{RunMetrics, Scenario, WorldSource};
use mpi::traits::*;
use serde::Serialize;
use std::{error::Error, path::Path, time::Instant};

pub mod scaling;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Static,
    Dynamic,
}

impl std::str::FromStr for Mode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "static" => Ok(Mode::Static),
            "dynamic" => Ok(Mode::Dynamic),
            _ => Err(format!("unknown MPI mode '{s}' (static|dynamic)")),
        }
    }
}

/// True when the process was started by `mpirun` (Open MPI or MPICH/PMI environment).
pub fn launched_by_mpirun() -> bool {
    ["OMPI_COMM_WORLD_SIZE", "PMI_SIZE", "PMIX_RANK"]
        .iter()
        .any(|k| std::env::var_os(k).is_some())
}

const WORDS_PER_STEP: usize = 10;
const TAG_REQ: i32 = 1;
const TAG_JOB: i32 = 2;
const TAG_GATHER: i32 = 3;
const NONE: u64 = u64::MAX;

/// Flat encoding of one run: 10 metric values then 10 words per step (counts x7,
/// new_infections, hospital_load, r_t bits). `step` is the index and `day` = step / 4.
pub fn pack(stats: &[StepStats], m: &RunMetrics) -> Vec<u64> {
    let mut v: Vec<u64> = Vec::with_capacity(10 + stats.len() * WORDS_PER_STEP);
    v.extend(m.values().map(f64::to_bits));
    for s in stats {
        v.extend(s.counts.map(u64::from));
        v.push(u64::from(s.new_infections));
        v.push(u64::from(s.hospital_load));
        v.push(s.r_t.to_bits());
    }
    v
}

#[allow(clippy::chunks_exact_to_as_chunks)]
pub fn unpack(v: &[u64]) -> (Vec<StepStats>, RunMetrics) {
    let vals: [f64; 10] = std::array::from_fn(|i| f64::from_bits(v[i]));
    let stats = v[10..]
        .chunks_exact(WORDS_PER_STEP)
        .enumerate()
        .map(|(i, c)| StepStats {
            step: i as u32,
            day: i as u32 / epidemicsim_core::world::STEPS_PER_DAY,
            counts: std::array::from_fn(|k| c[k] as u32),
            new_infections: c[7] as u32,
            hospital_load: c[8] as u32,
            r_t: f64::from_bits(c[9]),
        })
        .collect();
    (stats, RunMetrics::from_values(vals))
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct RankTiming {
    pub rank: i32,
    pub tasks: usize,
    /// Seconds running simulations.
    pub compute_s: f64,
    /// Seconds in MPI send/receive calls (excluding waiting for other ranks).
    pub comm_s: f64,
    /// Static: time waiting at the barrier before the gather (load imbalance).
    pub idle_s: f64,
    pub bytes_sent: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct MpiReport {
    pub mode: Mode,
    pub ranks: i32,
    pub tasks: usize,
    /// Rank 0: scheduling + gather, until all results are on rank 0.
    pub run_s: f64,
    /// Rank 0: aggregation into bands/metrics + file output.
    pub aggregate_s: f64,
    pub per_rank: Vec<RankTiming>,
}

fn run_tasks<C: Communicator>(
    world: &C,
    mode: Mode,
    total: usize,
    exec: &dyn Fn(usize) -> Vec<u64>,
) -> (Vec<Vec<u64>>, RankTiming) {
    let (rank, size) = (world.rank(), world.size() as usize);
    let mut tm = RankTiming {
        rank,
        ..Default::default()
    };
    let mut results = vec![Vec::new(); total];
    let run = |t: usize, tm: &mut RankTiming| {
        let t0 = Instant::now();
        let r = exec(t);
        tm.compute_s += t0.elapsed().as_secs_f64();
        tm.tasks += 1;
        r
    };
    if size == 1 {
        for (t, slot) in results.iter_mut().enumerate() {
            *slot = run(t, &mut tm);
        }
        return (results, tm);
    }
    match mode {
        Mode::Static => {
            let block = |r: usize| (r * total / size)..((r + 1) * total / size);
            let mut mine: Vec<u64> = Vec::new();
            for t in block(rank as usize) {
                let r = run(t, &mut tm);
                mine.extend([t as u64, r.len() as u64]);
                mine.extend(r);
            }
            let t0 = Instant::now();
            world.barrier();
            tm.idle_s = t0.elapsed().as_secs_f64();
            let t0 = Instant::now();
            if rank == 0 {
                let mut msgs = vec![mine];
                for _ in 1..size {
                    let (m, _) = world.any_process().receive_vec_with_tag::<u64>(TAG_GATHER);
                    msgs.push(m);
                }
                for m in msgs {
                    let mut i = 0;
                    while i < m.len() {
                        let (t, n) = (m[i] as usize, m[i + 1] as usize);
                        results[t] = m[i + 2..i + 2 + n].to_vec();
                        i += 2 + n;
                    }
                }
            } else {
                tm.bytes_sent += mine.len() as u64 * 8;
                world
                    .process_at_rank(0)
                    .send_with_tag(&mine[..], TAG_GATHER);
            }
            tm.comm_s = t0.elapsed().as_secs_f64();
        }
        Mode::Dynamic => {
            if rank == 0 {
                let (mut next, mut live) = (0usize, size - 1);
                while live > 0 {
                    let t0 = Instant::now();
                    let (m, st) = world.any_process().receive_vec_with_tag::<u64>(TAG_REQ);
                    if m[0] != NONE {
                        results[m[0] as usize] = m[1..].to_vec();
                    }
                    let job = if next < total {
                        next += 1;
                        (next - 1) as u64
                    } else {
                        live -= 1;
                        NONE
                    };
                    world
                        .process_at_rank(st.source_rank())
                        .send_with_tag(&[job][..], TAG_JOB);
                    tm.comm_s += t0.elapsed().as_secs_f64();
                }
            } else {
                let mut msg = vec![NONE];
                loop {
                    let t0 = Instant::now();
                    tm.bytes_sent += msg.len() as u64 * 8;
                    world.process_at_rank(0).send_with_tag(&msg[..], TAG_REQ);
                    let (job, _) = world
                        .process_at_rank(0)
                        .receive_vec_with_tag::<u64>(TAG_JOB);
                    tm.comm_s += t0.elapsed().as_secs_f64();
                    if job[0] == NONE {
                        break;
                    }
                    let r = run(job[0] as usize, &mut tm);
                    msg = vec![job[0]];
                    msg.extend(r);
                }
            }
        }
    }
    (results, tm)
}

/// Run `scenarios` x `opts.ensemble.runs` over all MPI ranks and write the usual outputs on
/// rank 0. Every rank must call this with identical arguments. Returns the timing report
/// (complete on rank 0 only).
pub fn compare_mpi<B, F>(
    src: &WorldSource,
    scenarios: &[Scenario],
    opts: &CompareOpts,
    out_root: &Path,
    make_backend: F,
    mode: Mode,
) -> Result<MpiReport, Box<dyn Error>>
where
    B: Backend,
    F: Fn() -> B + Sync,
{
    let universe = mpi::initialize().ok_or("MPI was already initialised")?;
    let world = universe.world();
    let eo = &opts.ensemble;
    let total = scenarios.len() * eo.runs;
    let exec = |t: usize| {
        let (s, k) = (t / eo.runs, t % eo.runs);
        let mut b = make_backend();
        let (stats, m) = run_once(src, &scenarios[s], eo.base_seed + k as u64, eo, &mut b);
        pack(&stats, &m)
    };
    world.barrier();
    let t0 = Instant::now();
    let (results, tm) = run_tasks(&world, mode, total, &exec);
    let run_s = t0.elapsed().as_secs_f64();

    // Collect per-rank timings on rank 0 (small message, outside the timed region).
    let mine = [
        tm.tasks as f64,
        tm.compute_s,
        tm.comm_s,
        tm.idle_s,
        tm.bytes_sent as f64,
    ];
    let mut per_rank = vec![tm];
    if world.rank() == 0 {
        for _ in 1..world.size() {
            let (m, st) = world
                .any_process()
                .receive_vec_with_tag::<f64>(TAG_GATHER + 1);
            per_rank.push(RankTiming {
                rank: st.source_rank(),
                tasks: m[0] as usize,
                compute_s: m[1],
                comm_s: m[2],
                idle_s: m[3],
                bytes_sent: m[4] as u64,
            });
        }
        per_rank.sort_by_key(|r| r.rank);
    } else {
        world
            .process_at_rank(0)
            .send_with_tag(&mine[..], TAG_GATHER + 1);
    }
    let mut report = MpiReport {
        mode,
        ranks: world.size(),
        tasks: total,
        run_s,
        aggregate_s: 0.0,
        per_rank,
    };
    if world.rank() == 0 {
        let t1 = Instant::now();
        let ensembles: Vec<_> = scenarios
            .iter()
            .enumerate()
            .map(|(s, sc)| {
                let runs = (0..eo.runs)
                    .map(|k| unpack(&results[s * eo.runs + k]))
                    .collect();
                ensemble_from_results(sc, eo, runs)
            })
            .collect();
        write_all(src, scenarios, opts, out_root, &ensembles)?;
        report.aggregate_s = t1.elapsed().as_secs_f64();
    }
    world.barrier();
    Ok(report)
}

/// `Exchange` over MPI: every rank owns one city and contributes one row of traveller counts.
struct MpiExchange<'a, C: Communicator>(&'a C);

impl<C: Communicator> Exchange for MpiExchange<'_, C> {
    fn all_rows(&self, mine: &[Vec<u32>]) -> Vec<Vec<u32>> {
        let n = self.0.size() as usize;
        let mut buf = vec![0u32; n * n];
        self.0.all_gather_into(&mine[0][..], &mut buf[..]);
        buf.chunks(n).map(<[u32]>::to_vec).collect()
    }
}

/// Inter-city travel with one MPI rank per city (`mpirun -n <cities>`). Each rank simulates its
/// own city and, once a day, all-gathers the per-destination traveller counts; rank 0 collects
/// the per-run results and builds the same report as the single-process path (byte-identical
/// JSON). Returns the report on rank 0, `None` elsewhere.
pub fn travel_mpi<B, F>(
    ctx: &Ctx,
    params: &str,
    runs: usize,
    base_seed: u64,
    make_backend: F,
) -> Result<Option<TravelReport>, Box<dyn Error>>
where
    B: Backend,
    F: Fn() -> B,
{
    let universe = mpi::initialize().ok_or("MPI was already initialised")?;
    let world = universe.world();
    let n = ctx.travel.n();
    if world.size() as usize != n {
        return Err(format!(
            "travel needs one MPI rank per city: {n} cities but {} ranks (mpirun -n {n})",
            world.size()
        )
        .into());
    }
    let rank = world.rank() as usize;
    let ex = MpiExchange(&world);
    // [arm][run] flat words: pack(stats, metrics) followed by the import count.
    let mut mine: Vec<Vec<u64>> = Vec::new();
    for coupled in [false, true] {
        for k in 0..runs {
            let mut bs = [make_backend()];
            let r = run_coupled(ctx, &[rank], coupled, base_seed + k as u64, &mut bs, &ex)
                .pop()
                .expect("one owned city");
            let mut v = pack(&r.stats, &r.metrics);
            v.push(u64::from(r.imports));
            mine.push(v);
        }
    }
    if rank != 0 {
        let flat: Vec<u64> = mine
            .iter()
            .flat_map(|v| std::iter::once(v.len() as u64).chain(v.iter().copied()))
            .collect();
        world
            .process_at_rank(0)
            .send_with_tag(&flat[..], TAG_GATHER);
        world.barrier();
        return Ok(None);
    }
    let mut per_city: Vec<Vec<Vec<u64>>> = vec![Vec::new(); n];
    per_city[0] = mine;
    for _ in 1..n {
        let (m, st) = world.any_process().receive_vec_with_tag::<u64>(TAG_GATHER);
        let mut i = 0;
        let mut v = Vec::new();
        while i < m.len() {
            let len = m[i] as usize;
            v.push(m[i + 1..i + 1 + len].to_vec());
            i += 1 + len;
        }
        per_city[st.source_rank() as usize] = v;
    }
    let arm = |a: usize| -> ArmRuns {
        (0..runs)
            .map(|k| {
                (0..n)
                    .map(|c| {
                        let v = &per_city[c][a * runs + k];
                        let (stats, metrics) = unpack(&v[..v.len() - 1]);
                        CityRun {
                            city: c,
                            stats,
                            metrics,
                            imports: v[v.len() - 1] as u32,
                        }
                    })
                    .collect()
            })
            .collect()
    };
    let report = build_report(ctx, params, runs, base_seed, &arm(0), &arm(1));
    world.barrier();
    Ok(Some(report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_roundtrip() {
        let stats: Vec<StepStats> = (0..9)
            .map(|i| StepStats {
                step: i,
                day: i / 4,
                counts: [i, 2, 3, 4, 5, 6, 7],
                new_infections: i * 3,
                hospital_load: 5,
                r_t: f64::from(i) * 0.1,
            })
            .collect();
        let m = RunMetrics::from_stats(&stats, 100, 3, 17);
        let (s2, m2) = unpack(&pack(&stats, &m));
        assert_eq!(stats, s2);
        assert_eq!(m.values(), m2.values());
    }
}
