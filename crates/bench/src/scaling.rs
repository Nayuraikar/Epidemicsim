//! Weak scaling: agents per thread fixed, threads 1..N. Also the Gustafson's-law fit.
use crate::{machine_info, thermal_note, timing, Factory, Machine, Timing};
use epidemicsim_core::world::STEPS_PER_DAY;
use epidemicsim_core::{synth, Config};
use serde::Serialize;
use std::{error::Error, time::Instant};

#[derive(Clone, Debug)]
pub struct WeakOpts {
    pub per_thread: usize,
    pub threads: Vec<usize>,
    pub days: u32,
    pub warmup: usize,
    pub reps: usize,
    pub seed: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct WeakPoint {
    pub threads: usize,
    pub agents: usize,
    pub wall_s: Timing,
    /// Median time at the reference thread count / median time here (1.0 = perfect weak scaling).
    pub weak_efficiency: f64,
    /// Scaled speedup = (threads / ref threads) x weak_efficiency.
    pub scaled_speedup: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Gustafson {
    /// Least-squares serial fraction alpha in S(t) = t - alpha (t - 1).
    pub serial_fraction: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct WeakReport {
    pub backend: String,
    pub machine: Machine,
    pub per_thread_agents: usize,
    pub days: u32,
    pub steps: u32,
    pub warmup: usize,
    pub reps: usize,
    pub seed: u64,
    pub thermal_before: Option<String>,
    pub thermal_after: Option<String>,
    pub points: Vec<WeakPoint>,
    pub gustafson: Option<Gustafson>,
}

/// Fit alpha from (threads, scaled speedup) pairs: t - S = alpha (t - 1), least squares,
/// clamped to [0, 1]. `None` unless some t > 1 is present.
pub fn gustafson_fit(points: &[(f64, f64)]) -> Option<f64> {
    let (mut num, mut den) = (0.0, 0.0);
    for &(t, s) in points {
        if t > 1.0 {
            num += (t - s) * (t - 1.0);
            den += (t - 1.0) * (t - 1.0);
        }
    }
    (den > 0.0).then(|| (num / den).clamp(0.0, 1.0))
}

pub fn weak(name: &str, factory: Factory, o: &WeakOpts) -> Result<WeakReport, Box<dyn Error>> {
    let steps = o.days * STEPS_PER_DAY;
    let thermal_before = thermal_note();
    let mut points: Vec<WeakPoint> = Vec::new();
    let mut reference: Option<(usize, f64)> = None;
    for &t in &o.threads {
        let n = o.per_thread * t;
        let world = synth::generate(&Config {
            agents: n,
            seed: o.seed,
            ..Default::default()
        });
        let mut samples = Vec::new();
        for rep in 0..o.warmup + o.reps {
            let mut backend = factory(t)?;
            let mut w = world.clone();
            let iv = Default::default();
            let start = Instant::now();
            for _ in 0..steps {
                backend.step(&mut w, &iv);
            }
            if rep >= o.warmup {
                samples.push(start.elapsed().as_secs_f64());
            }
        }
        let wall = timing(&samples);
        let (rt, rm) = *reference.get_or_insert((t, wall.median));
        let eff = rm / wall.median;
        eprintln!(
            "{name} weak threads={t} agents={n}: median {:.3}s +-{:.3} (95% CI)  efficiency {eff:.2}",
            wall.median, wall.ci95
        );
        points.push(WeakPoint {
            threads: t,
            agents: n,
            wall_s: wall,
            weak_efficiency: eff,
            scaled_speedup: eff * t as f64 / rt as f64,
        });
    }
    let pairs: Vec<(f64, f64)> = points
        .iter()
        .map(|p| (p.threads as f64, p.scaled_speedup))
        .collect();
    Ok(WeakReport {
        backend: name.into(),
        machine: machine_info(),
        per_thread_agents: o.per_thread,
        days: o.days,
        steps,
        warmup: o.warmup,
        reps: o.reps,
        seed: o.seed,
        thermal_before,
        thermal_after: thermal_note(),
        points,
        gustafson: gustafson_fit(&pairs).map(|serial_fraction| Gustafson { serial_fraction }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gustafson_recovers_alpha() {
        let a = 0.2;
        let pts: Vec<(f64, f64)> = [1.0, 2.0, 4.0, 8.0]
            .iter()
            .map(|&t| (t, t - a * (t - 1.0)))
            .collect();
        assert!((gustafson_fit(&pts).unwrap() - a).abs() < 1e-12);
        assert!(gustafson_fit(&[(1.0, 1.0)]).is_none());
    }
}
