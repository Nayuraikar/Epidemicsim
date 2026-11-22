//! Scaling driver: launches `mpirun -n P epidemicsim compare ...` for each process count and
//! mode, collects the timing report each run writes, and writes results/mpi/scaling.json and
//! results/bench/mpi_apple_m1.json.
use crate::MpiReport;
use serde::Serialize;
use std::{error::Error, path::Path, process::Command};

pub struct Opts {
    pub exe: std::path::PathBuf,
    pub procs: Vec<usize>,
    pub reps: usize,
    pub agents: usize,
    pub days: u32,
    pub runs: usize,
    pub scenarios_dir: String,
    pub backend: String,
    pub work_dir: std::path::PathBuf,
}

#[derive(Serialize)]
pub struct Point {
    pub mode: String,
    pub procs: usize,
    pub wall_s: crate::scaling::Median,
    /// Median wall at 1 process (same mode) / median wall here.
    pub speedup: f64,
    pub efficiency: f64,
    /// Sum over ranks of compute seconds / (procs x run_s): fraction of core-time computing.
    pub utilisation: f64,
    /// Rank-summed seconds in MPI calls, as a fraction of rank-summed run time.
    pub comm_fraction: f64,
    pub comm_s_total: f64,
    pub idle_s_total: f64,
    pub bytes_sent_total: u64,
    /// max / mean of per-rank compute seconds (1.0 = perfectly balanced).
    pub compute_imbalance: f64,
    pub tasks_per_rank: Vec<usize>,
}

#[derive(Serialize, Clone, Copy)]
pub struct Median {
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    /// Half-width of the 95% CI of the mean (Student t).
    pub ci95: f64,
    pub n: usize,
}

#[derive(Serialize)]
pub struct Scaling {
    pub machine: String,
    pub backend: String,
    pub agents: usize,
    pub days: u32,
    pub runs_per_scenario: usize,
    pub scenarios: usize,
    pub tasks: usize,
    pub reps: usize,
    pub note: &'static str,
    pub points: Vec<Point>,
}

fn med(v: &mut [f64]) -> Median {
    const T: [f64; 12] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179,
    ];
    v.sort_by(f64::total_cmp);
    let n = v.len();
    let mean = v.iter().sum::<f64>() / n as f64;
    let std_dev = if n < 2 {
        0.0
    } else {
        (v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt()
    };
    let t = if n < 2 {
        0.0
    } else {
        T.get(n - 2).copied().unwrap_or(1.96)
    };
    Median {
        median: if n % 2 == 1 {
            v[n / 2]
        } else {
            (v[n / 2 - 1] + v[n / 2]) / 2.0
        },
        min: v[0],
        max: v[n - 1],
        mean,
        std_dev,
        ci95: t * std_dev / (n as f64).sqrt(),
        n,
    }
}

fn one(o: &Opts, p: usize, mode: &str, tag: &str) -> Result<MpiReport, Box<dyn Error>> {
    let out = o.work_dir.join(tag);
    let timing = o.work_dir.join(format!("{tag}.json"));
    let st = Command::new("mpirun")
        .args(["-n", &p.to_string()])
        .arg(&o.exe)
        .args(["compare", "--backend", &o.backend, "--agents"])
        .arg(o.agents.to_string())
        .args(["--days", &o.days.to_string(), "--runs", &o.runs.to_string()])
        .args(["--scenarios-dir", &o.scenarios_dir, "--mpi", mode, "--out"])
        .arg(&out)
        .arg("--mpi-timing")
        .arg(&timing)
        .output()?;
    if !st.status.success() {
        return Err(format!(
            "mpirun -n {p} failed: {}",
            String::from_utf8_lossy(&st.stderr)
        )
        .into());
    }
    let r = serde_json::from_slice(&std::fs::read(&timing)?)?;
    let _ = std::fs::remove_dir_all(&out);
    Ok(r)
}

impl<'de> serde::Deserialize<'de> for MpiReport {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::Deserialize;
        #[derive(Deserialize)]
        struct R {
            mode: String,
            ranks: i32,
            tasks: usize,
            run_s: f64,
            aggregate_s: f64,
            per_rank: Vec<RT>,
        }
        #[derive(Deserialize)]
        struct RT {
            rank: i32,
            tasks: usize,
            compute_s: f64,
            comm_s: f64,
            idle_s: f64,
            bytes_sent: u64,
        }
        let r = R::deserialize(d)?;
        Ok(MpiReport {
            mode: if r.mode == "dynamic" {
                crate::Mode::Dynamic
            } else {
                crate::Mode::Static
            },
            ranks: r.ranks,
            tasks: r.tasks,
            run_s: r.run_s,
            aggregate_s: r.aggregate_s,
            per_rank: r
                .per_rank
                .into_iter()
                .map(|x| crate::RankTiming {
                    rank: x.rank,
                    tasks: x.tasks,
                    compute_s: x.compute_s,
                    comm_s: x.comm_s,
                    idle_s: x.idle_s,
                    bytes_sent: x.bytes_sent,
                })
                .collect(),
        })
    }
}

pub fn run(o: &Opts, machine: &str, scenarios: usize) -> Result<Scaling, Box<dyn Error>> {
    std::fs::create_dir_all(&o.work_dir)?;
    let mut points: Vec<Point> = Vec::new();
    for mode in ["static", "dynamic"] {
        let mut base = 0.0;
        for &p in &o.procs {
            let mut reps: Vec<MpiReport> = Vec::new();
            for r in 0..o.reps + 1 {
                // The first repetition is a warm-up (page cache, CPU clocks) and is dropped.
                let rep = one(o, p, mode, &format!("{mode}_{p}_{r}"))?;
                if r > 0 {
                    reps.push(rep);
                }
            }
            let mut walls: Vec<f64> = reps.iter().map(|r| r.run_s).collect();
            let wall = med(&mut walls);
            if base == 0.0 {
                base = wall.median;
            }
            // Per-rank breakdown from the repetition closest to the median wall time.
            let rep = reps
                .iter()
                .min_by(|a, b| {
                    (a.run_s - wall.median)
                        .abs()
                        .total_cmp(&(b.run_s - wall.median).abs())
                })
                .expect("reps >= 1");
            let compute: Vec<f64> = rep.per_rank.iter().map(|r| r.compute_s).collect();
            let mean = compute.iter().sum::<f64>() / compute.len() as f64;
            let comm: f64 = rep.per_rank.iter().map(|r| r.comm_s).sum();
            let idle: f64 = rep.per_rank.iter().map(|r| r.idle_s).sum();
            let speedup = base / wall.median;
            points.push(Point {
                mode: mode.into(),
                procs: p,
                wall_s: wall,
                speedup,
                efficiency: speedup / (p as f64 / o.procs[0] as f64),
                utilisation: compute.iter().sum::<f64>() / (p as f64 * rep.run_s),
                comm_fraction: comm / (p as f64 * rep.run_s),
                comm_s_total: comm,
                idle_s_total: idle,
                bytes_sent_total: rep.per_rank.iter().map(|r| r.bytes_sent).sum(),
                compute_imbalance: compute.iter().copied().fold(0.0, f64::max) / mean.max(1e-12),
                tasks_per_rank: rep.per_rank.iter().map(|r| r.tasks).collect(),
            });
            let q = points.last().expect("pushed");
            eprintln!(
                "mpi {mode} n={p}: median {:.3}s speedup {:.2} util {:.2} comm {:.4} imbalance {:.2}",
                q.wall_s.median, q.speedup, q.utilisation, q.comm_fraction, q.compute_imbalance
            );
        }
    }
    Ok(Scaling {
        machine: machine.into(),
        backend: o.backend.clone(),
        agents: o.agents,
        days: o.days,
        runs_per_scenario: o.runs,
        scenarios,
        tasks: scenarios * o.runs,
        reps: o.reps,
        note: "wall_s = rank-0 time from first dispatch until all results are gathered (excludes aggregation and file output). dynamic uses rank 0 as a pure master, so it has procs-1 workers.",
        points,
    })
}

#[derive(Serialize)]
pub struct WeakPoint {
    pub procs: usize,
    pub tasks: usize,
    pub wall_s: Median,
    pub weak_efficiency: f64,
    pub scaled_speedup: f64,
    pub comm_fraction: f64,
}

#[derive(Serialize)]
pub struct WeakScaling {
    pub machine: String,
    pub backend: String,
    pub mode: &'static str,
    pub agents: usize,
    pub days: u32,
    pub tasks_per_rank: usize,
    pub reps: usize,
    pub note: &'static str,
    pub points: Vec<WeakPoint>,
    pub gustafson_serial_fraction: Option<f64>,
}

/// Weak scaling: static mode, tasks per rank fixed (`scenarios` x runs / procs), so runs per
/// scenario = procs x tasks_per_rank / scenarios. `o.runs` is ignored.
pub fn run_weak(
    o: &Opts,
    machine: &str,
    scenarios: usize,
    tasks_per_rank: usize,
) -> Result<WeakScaling, Box<dyn Error>> {
    std::fs::create_dir_all(&o.work_dir)?;
    let mut points: Vec<WeakPoint> = Vec::new();
    let mut base = 0.0;
    for &p in &o.procs {
        let runs = (p * tasks_per_rank).div_ceil(scenarios).max(1);
        let oo = Opts {
            runs,
            procs: o.procs.clone(),
            exe: o.exe.clone(),
            scenarios_dir: o.scenarios_dir.clone(),
            backend: o.backend.clone(),
            work_dir: o.work_dir.clone(),
            ..*o
        };
        let mut reps = Vec::new();
        for r in 0..o.reps + 1 {
            let rep = one(&oo, p, "static", &format!("weak_{p}_{r}"))?;
            if r > 0 {
                reps.push(rep);
            }
        }
        let mut walls: Vec<f64> = reps.iter().map(|r| r.run_s).collect();
        let wall = med(&mut walls);
        if base == 0.0 {
            base = wall.median;
        }
        let comm: f64 = reps
            .iter()
            .map(|r| r.per_rank.iter().map(|x| x.comm_s).sum::<f64>() / (p as f64 * r.run_s))
            .sum::<f64>()
            / reps.len() as f64;
        let eff = base / wall.median;
        eprintln!(
            "mpi weak n={p} tasks={}: median {:.3}s +-{:.3} efficiency {eff:.2}",
            scenarios * runs,
            wall.median,
            wall.ci95
        );
        points.push(WeakPoint {
            procs: p,
            tasks: scenarios * runs,
            wall_s: wall,
            weak_efficiency: eff,
            scaled_speedup: eff * p as f64 / o.procs[0] as f64,
            comm_fraction: comm,
        });
    }
    let g = {
        let (mut num, mut den) = (0.0, 0.0);
        for q in &points {
            let t = q.procs as f64 / o.procs[0] as f64;
            if t > 1.0 {
                num += (t - q.scaled_speedup) * (t - 1.0);
                den += (t - 1.0) * (t - 1.0);
            }
        }
        (den > 0.0).then(|| (num / den).clamp(0.0, 1.0))
    };
    Ok(WeakScaling {
        machine: machine.into(),
        backend: o.backend.clone(),
        mode: "static",
        agents: o.agents,
        days: o.days,
        tasks_per_rank,
        reps: o.reps,
        note: "work per rank fixed: runs per scenario = procs x tasks_per_rank / scenarios; wall_s = rank-0 dispatch-to-gather time; ranks beyond the 4P+4E cores of this machine share cores.",
        points,
        gustafson_serial_fraction: g,
    })
}

pub fn write(path: &Path, v: &impl Serialize) -> Result<(), Box<dyn Error>> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(v)? + "\n")?;
    Ok(())
}
