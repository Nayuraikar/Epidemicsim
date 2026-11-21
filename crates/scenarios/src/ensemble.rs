//! Monte Carlo ensembles: many seeded runs of one scenario, parallel over runs with rayon.
use crate::engine::Controller;
use crate::metrics::{quantile, Dist, RunMetrics};
use crate::spec::{Intervention, Scenario};
use epidemicsim_core::world::STEPS_PER_DAY;
use epidemicsim_core::{synth, Backend, Config, StepStats, World};
use rayon::prelude::*;
use std::collections::BTreeMap;

/// Where each run's world comes from.
pub enum WorldSource {
    /// Generate a synthetic city per run (the run seed also varies the city).
    Synth(Box<Config>),
    /// A prebuilt city; each run clones it and reseeds the epidemic.
    City {
        world: Box<World>,
        infected_frac: f64,
    },
}

impl WorldSource {
    pub fn agents(&self) -> usize {
        match self {
            WorldSource::Synth(c) => c.agents,
            WorldSource::City { world, .. } => world.agents.len(),
        }
    }

    pub fn make(&self, seed: u64) -> World {
        match self {
            WorldSource::Synth(c) => synth::generate(&Config {
                seed,
                ..(**c).clone()
            }),
            WorldSource::City {
                world,
                infected_frac,
            } => {
                let mut w = (**world).clone();
                w.reseed(seed, *infected_frac);
                w
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnsembleOpts {
    pub runs: usize,
    pub days: u32,
    /// Run `k` uses seed `base_seed + k`, identically across scenarios (paired comparison).
    pub base_seed: u64,
    pub hospital_capacity: u32,
}

/// Names of the per-step series banded in `bands.parquet`.
pub const SERIES: [&str; 10] = [
    "S",
    "E",
    "A",
    "I",
    "H",
    "R",
    "D",
    "new_infections",
    "hospital_load",
    "r_t",
];

fn series_value(s: &StepStats, k: usize) -> f64 {
    match k {
        0..=6 => f64::from(s.counts[k]),
        7 => f64::from(s.new_infections),
        8 => f64::from(s.hospital_load),
        _ => s.r_t,
    }
}

pub struct Band {
    pub mean: Vec<f64>,
    pub p05: Vec<f64>,
    pub p95: Vec<f64>,
}

pub struct Ensemble {
    pub scenario: String,
    pub runs: usize,
    /// One band per entry of [`SERIES`].
    pub bands: Vec<Band>,
    pub metrics: BTreeMap<String, Dist>,
    pub per_run: Vec<RunMetrics>,
    pub hash: Vec<String>,
}

/// Run one scenario once; returns per-step stats, metrics and the output hash.
pub fn run_once(
    src: &WorldSource,
    scenario: &Scenario,
    seed: u64,
    opts: &EnsembleOpts,
    backend: &mut dyn Backend,
) -> (Vec<StepStats>, RunMetrics) {
    let mut world = src.make(seed);
    let n = world.agents.len();
    let mut ctl = Controller::new(scenario, n, opts.hospital_capacity);
    let has_vax = scenario
        .interventions
        .iter()
        .any(|i| matches!(i, Intervention::Vaccination { .. }));
    let mut stats: Vec<StepStats> = Vec::with_capacity((opts.days * STEPS_PER_DAY) as usize);
    for _ in 0..opts.days * STEPS_PER_DAY {
        // Vaccination edits agents on the host: device-resident backends sync around it.
        let edits = has_vax && world.step.is_multiple_of(STEPS_PER_DAY);
        if edits {
            backend.sync_to_host(&mut world);
        }
        let iv = ctl.update(&mut world, stats.last()).clone();
        if edits {
            backend.host_changed();
        }
        let s = backend.step(&mut world, &iv);
        stats.push(s);
    }
    debug_assert_eq!(
        stats.last().map(|s| s.counts.iter().sum::<u32>() as usize),
        Some(n)
    );
    let m = RunMetrics::from_stats(&stats, n, opts.hospital_capacity, ctl.doses());
    (stats, m)
}

pub fn run_ensemble<B, F>(
    src: &WorldSource,
    scenario: &Scenario,
    opts: &EnsembleOpts,
    make_backend: F,
) -> Ensemble
where
    B: Backend,
    F: Fn() -> B + Sync,
{
    let results: Vec<(Vec<StepStats>, RunMetrics)> = (0..opts.runs)
        .into_par_iter()
        .map(|k| {
            let mut backend = make_backend();
            run_once(src, scenario, opts.base_seed + k as u64, opts, &mut backend)
        })
        .collect();

    ensemble_from_results(scenario, opts, results)
}

/// Aggregate per-run results (ordered by run index) into bands, metric distributions and hashes.
/// Pure and deterministic, so any way of producing `results` (rayon, MPI) gives identical output.
pub fn ensemble_from_results(
    scenario: &Scenario,
    opts: &EnsembleOpts,
    results: Vec<(Vec<StepStats>, RunMetrics)>,
) -> Ensemble {
    let steps = results.first().map_or(0, |r| r.0.len());
    let bands = (0..SERIES.len())
        .map(|k| {
            let mut b = Band {
                mean: Vec::with_capacity(steps),
                p05: Vec::with_capacity(steps),
                p95: Vec::with_capacity(steps),
            };
            let mut col = vec![0.0; results.len()];
            for t in 0..steps {
                for (c, r) in col.iter_mut().zip(&results) {
                    *c = series_value(&r.0[t], k);
                }
                col.sort_by(f64::total_cmp);
                b.mean
                    .push(col.iter().sum::<f64>() / col.len().max(1) as f64);
                b.p05.push(quantile(&col, 0.05));
                b.p95.push(quantile(&col, 0.95));
            }
            b
        })
        .collect();

    let per_run: Vec<RunMetrics> = results.iter().map(|r| r.1.clone()).collect();
    let metrics = RunMetrics::NAMES
        .iter()
        .enumerate()
        .map(|(j, name)| {
            let vals: Vec<f64> = per_run.iter().map(|m| m.values()[j]).collect();
            (name.to_string(), Dist::of(&vals))
        })
        .collect();
    let hash = results
        .iter()
        .map(|r| epidemicsim_core::output::hash_stats(&r.0))
        .collect();
    Ensemble {
        scenario: scenario.name.clone(),
        runs: opts.runs,
        bands,
        metrics,
        per_run,
        hash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use epidemicsim_core::SequentialBackend;

    fn src() -> WorldSource {
        WorldSource::Synth(Box::new(Config {
            agents: 5000,
            ..Default::default()
        }))
    }

    fn opts(runs: usize) -> EnsembleOpts {
        EnsembleOpts {
            runs,
            days: 40,
            base_seed: 1,
            hospital_capacity: 10,
        }
    }

    #[test]
    fn ensemble_is_deterministic_and_bands_are_ordered() {
        let sc = Scenario::from_toml_str("name='baseline'").unwrap();
        let a = run_ensemble(&src(), &sc, &opts(6), SequentialBackend::default);
        let b = run_ensemble(&src(), &sc, &opts(6), SequentialBackend::default);
        assert_eq!(a.hash, b.hash);
        assert_ne!(a.hash[0], a.hash[1], "runs must use different seeds");
        for band in &a.bands {
            for t in 0..band.mean.len() {
                assert!(band.p05[t] <= band.mean[t] + 1e-9 && band.mean[t] <= band.p95[t] + 1e-9);
            }
        }
        assert_eq!(a.bands[0].mean.len(), 160);
    }

    #[test]
    fn lockdown_beats_baseline_on_attack_rate() {
        let base = Scenario::from_toml_str("name='baseline'").unwrap();
        let lock =
            Scenario::from_toml_str("name='l'\n[[interventions]]\nkind='lockdown'\nstart_day=0\n")
                .unwrap();
        let b = run_ensemble(&src(), &base, &opts(4), SequentialBackend::default);
        let l = run_ensemble(&src(), &lock, &opts(4), SequentialBackend::default);
        assert!(l.metrics["attack_rate"].mean < b.metrics["attack_rate"].mean);
    }
}
