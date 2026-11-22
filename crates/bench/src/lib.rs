//! Backend benchmark harness: warmup + repetitions per (agents, threads), JSON report.
pub mod layout;
pub mod roofline;
pub mod scaling;
pub mod tlp;

use epidemicsim_core::world::STEPS_PER_DAY;
use epidemicsim_core::{synth, Backend, Config};
use serde::Serialize;
use std::{error::Error, path::Path, process::Command, time::Instant};

/// Builds a fresh backend for a given thread count.
pub type Factory<'a> = &'a dyn Fn(usize) -> Result<Box<dyn Backend + Send>, String>;

#[derive(Clone, Debug)]
pub struct BenchOpts {
    pub agents: Vec<usize>,
    /// Thread counts to measure; the first entry (normally 1) is the speedup reference.
    pub threads: Vec<usize>,
    pub days: u32,
    pub warmup: usize,
    pub reps: usize,
    pub seed: u64,
}

impl Default for BenchOpts {
    fn default() -> Self {
        Self {
            agents: vec![10_000, 100_000, 1_000_000],
            threads: vec![1],
            days: 20,
            warmup: 1,
            reps: 5,
            seed: 42,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Timing {
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    /// Sample standard deviation (n - 1); 0 with fewer than two samples.
    pub std_dev: f64,
    /// Half-width of the 95% confidence interval of the mean (Student t); 0 with n < 2.
    pub ci95: f64,
    pub n: usize,
}

/// Two-sided 95% Student-t critical value for `df` degrees of freedom.
pub fn t_crit95(df: usize) -> f64 {
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160,
        2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074, 2.069, 2.064, 2.060, 2.056,
        2.052, 2.048, 2.045, 2.042,
    ];
    match df {
        0 => 0.0,
        1..=30 => T[df - 1],
        _ => 1.96,
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Point {
    pub agents: usize,
    pub threads: usize,
    pub steps: u32,
    /// Seconds for the whole `steps`-step run.
    pub wall_s: Timing,
    /// agents x steps / median wall time.
    pub agents_per_sec: f64,
    /// Agent-location visits (alive agents x steps, each mixed with its location) / median wall time.
    pub contacts_per_sec: f64,
    /// Median time at the reference thread count / median time at this thread count.
    pub speedup: f64,
    /// speedup / (threads / reference threads).
    pub efficiency: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Amdahl {
    pub agents: usize,
    /// Least-squares serial fraction s in speedup(t) = 1 / (s + (1 - s) / t).
    pub serial_fraction: f64,
    pub max_speedup: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Machine {
    pub os: String,
    pub arch: String,
    pub cpu: String,
    pub logical_cores: usize,
    pub memory_gb: Option<f64>,
}

/// Thermal state from `pmset -g therm` (macOS) at one moment; `None` where not detectable.
pub fn thermal_note() -> Option<String> {
    let t = cmd("pmset", &["-g", "therm"])?;
    let lines: Vec<&str> = t
        .lines()
        .filter(|l| !l.starts_with("Note:") && !l.contains("***"))
        .collect();
    Some(if lines.is_empty() {
        "no thermal/performance warning recorded (pmset -g therm); no die temperature available without sudo".into()
    } else {
        lines.join("; ")
    })
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub backend: String,
    pub machine: Machine,
    pub days: u32,
    pub warmup: usize,
    pub reps: usize,
    pub seed: u64,
    /// Thermal state before / after the run; throttling is only detectable as a recorded warning.
    pub thermal_before: Option<String>,
    pub thermal_after: Option<String>,
    pub points: Vec<Point>,
    /// Empty unless at least two thread counts (reference + one more) were measured.
    pub amdahl: Vec<Amdahl>,
}

fn cmd(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn machine_info() -> Machine {
    let cpu = cmd("sysctl", &["-n", "machdep.cpu.brand_string"])
        .or_else(|| {
            std::fs::read_to_string("/proc/cpuinfo").ok().and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("model name"))
                    .and_then(|l| l.split(':').nth(1))
                    .map(|v| v.trim().to_string())
            })
        })
        .unwrap_or_else(|| "unknown".into());
    let mem = cmd("sysctl", &["-n", "hw.memsize"])
        .and_then(|s| s.parse::<f64>().ok())
        .or_else(|| {
            std::fs::read_to_string("/proc/meminfo").ok().and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("MemTotal"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|v| v.parse::<f64>().ok())
                    .map(|kb| kb * 1024.0)
            })
        });
    Machine {
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        cpu,
        logical_cores: std::thread::available_parallelism().map_or(1, |n| n.get()),
        memory_gb: mem.map(|b| (b / 1e9 * 10.0).round() / 10.0),
    }
}

/// Lowercase, alphanumeric-and-underscore tag of the CPU name, e.g. `apple_m2_pro`.
pub fn hardware_tag(m: &Machine) -> String {
    let mut tag = String::new();
    for c in m.cpu.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            tag.push(c);
        } else if !tag.ends_with('_') {
            tag.push('_');
        }
    }
    let tag = tag.trim_matches('_');
    if tag.is_empty() {
        "unknown".into()
    } else {
        tag.into()
    }
}

pub fn timing(samples: &[f64]) -> Timing {
    let mut v = samples.to_vec();
    v.sort_by(f64::total_cmp);
    let n = v.len();
    let median = if n == 0 {
        0.0
    } else if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    };
    let mean = if n == 0 {
        0.0
    } else {
        v.iter().sum::<f64>() / n as f64
    };
    let std_dev = if n < 2 {
        0.0
    } else {
        (v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt()
    };
    let ci95 = if n < 2 {
        0.0
    } else {
        t_crit95(n - 1) * std_dev / (n as f64).sqrt()
    };
    Timing {
        median,
        min: v.first().copied().unwrap_or(0.0),
        max: v.last().copied().unwrap_or(0.0),
        mean,
        std_dev,
        ci95,
        n,
    }
}

/// Fit the serial fraction from (threads, speedup) pairs; `None` with fewer than two points
/// beyond t = 1. Uses 1/S = 1/t + s (1 - 1/t), solved by least squares and clamped to [0, 1].
pub fn amdahl_fit(points: &[(f64, f64)]) -> Option<f64> {
    let (mut num, mut den, mut k) = (0.0, 0.0, 0);
    for &(t, s) in points {
        if t > 1.0 && s > 0.0 {
            let x = 1.0 - 1.0 / t;
            num += (1.0 / s - 1.0 / t) * x;
            den += x * x;
            k += 1;
        }
    }
    (k >= 1 && den > 0.0).then(|| (num / den).clamp(0.0, 1.0))
}

pub fn run(name: &str, factory: Factory, o: &BenchOpts) -> Result<Report, Box<dyn Error>> {
    let steps = o.days * STEPS_PER_DAY;
    let thermal_before = thermal_note();
    let mut points: Vec<Point> = Vec::new();
    let mut amdahl = Vec::new();
    for &n in &o.agents {
        let world = synth::generate(&Config {
            agents: n,
            seed: o.seed,
            ..Default::default()
        });
        let first = points.len();
        let mut reference: Option<(usize, f64)> = None;
        for &t in &o.threads {
            let mut samples = Vec::new();
            let mut visits = 0u64;
            for rep in 0..o.warmup + o.reps {
                let mut backend = factory(t)?;
                let mut w = world.clone();
                let iv = Default::default();
                let mut v = 0u64;
                let start = Instant::now();
                for _ in 0..steps {
                    let s = backend.step(&mut w, &iv);
                    v += n as u64 - u64::from(s.counts[6]);
                }
                let secs = start.elapsed().as_secs_f64();
                if rep >= o.warmup {
                    samples.push(secs);
                    visits = v;
                }
            }
            let wall = timing(&samples);
            let (rt, rm) = *reference.get_or_insert((t, wall.median));
            let speedup = rm / wall.median;
            points.push(Point {
                agents: n,
                threads: t,
                steps,
                agents_per_sec: n as f64 * f64::from(steps) / wall.median,
                contacts_per_sec: visits as f64 / wall.median,
                speedup,
                efficiency: speedup / (t as f64 / rt as f64),
                wall_s: wall,
            });
            let p = points.last().expect("just pushed");
            eprintln!(
                "{name} agents={n} threads={t}: median {:.3}s  {:.2}M agents/s  speedup {:.2}",
                p.wall_s.median,
                p.agents_per_sec / 1e6,
                p.speedup
            );
        }
        let pairs: Vec<(f64, f64)> = points[first..]
            .iter()
            .map(|p| (p.threads as f64, p.speedup))
            .collect();
        if let Some(s) = amdahl_fit(&pairs) {
            amdahl.push(Amdahl {
                agents: n,
                serial_fraction: s,
                max_speedup: if s > 0.0 { 1.0 / s } else { f64::INFINITY },
            });
        }
    }
    Ok(Report {
        backend: name.into(),
        machine: machine_info(),
        days: o.days,
        warmup: o.warmup,
        reps: o.reps,
        seed: o.seed,
        thermal_before,
        thermal_after: thermal_note(),
        points,
        amdahl,
    })
}

/// Write `<dir>/<backend>_<hardware>.json`; returns the path.
pub fn write_report(dir: &Path, r: &Report) -> Result<std::path::PathBuf, Box<dyn Error>> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("{}_{}.json", r.backend, hardware_tag(&r.machine)));
    // serde_json writes non-finite floats as null, which is fine for max_speedup.
    std::fs::write(&path, serde_json::to_string_pretty(r)? + "\n")?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use epidemicsim_core::SequentialBackend;

    #[test]
    fn median_min_max() {
        let t = timing(&[3.0, 1.0, 2.0, 10.0]);
        assert_eq!((t.median, t.min, t.max), (2.5, 1.0, 10.0));
        assert_eq!(t.mean, 4.0);
        assert!((t.std_dev - 4.0825).abs() < 1e-3);
        assert!((t.ci95 - 3.182 * t.std_dev / 2.0).abs() < 1e-9);
        assert_eq!(timing(&[5.0, 1.0, 3.0]).median, 3.0);
    }

    #[test]
    fn amdahl_recovers_serial_fraction() {
        let s = 0.1;
        let pts: Vec<(f64, f64)> = [1.0, 2.0, 4.0, 8.0]
            .iter()
            .map(|&t| (t, 1.0 / (s + (1.0 - s) / t)))
            .collect();
        assert!((amdahl_fit(&pts).unwrap() - s).abs() < 1e-9);
        assert!(amdahl_fit(&[(1.0, 1.0)]).is_none());
    }

    #[test]
    fn hardware_tag_is_filename_safe() {
        let mut m = machine_info();
        m.cpu = "Apple M2 Pro (x)".into();
        assert_eq!(hardware_tag(&m), "apple_m2_pro_x");
    }

    #[test]
    fn small_bench_runs() {
        let o = BenchOpts {
            agents: vec![500],
            days: 2,
            warmup: 1,
            reps: 3,
            ..Default::default()
        };
        let f = |_t: usize| -> Result<Box<dyn Backend + Send>, String> {
            Ok(Box::new(SequentialBackend::default()))
        };
        let r = run("sequential", &f, &o).unwrap();
        assert_eq!(r.points.len(), 1);
        let p = &r.points[0];
        assert!(p.agents_per_sec > 0.0 && p.contacts_per_sec > 0.0);
        assert_eq!(p.speedup, 1.0);
        assert!(r.amdahl.is_empty());
    }
}
