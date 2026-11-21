//! Run several scenarios and write results/<city>/<scenario>/{bands.parquet,summary.json}
//! plus results/<city>/comparison.json.
use crate::ensemble::{run_ensemble, Ensemble, EnsembleOpts, WorldSource, SERIES};
use crate::metrics::Dist;
use crate::spec::Scenario;
use epidemicsim_core::output::write_f64_columns;
use epidemicsim_core::world::STEPS_PER_DAY;
use epidemicsim_core::Backend;
use serde::Serialize;
use std::{collections::BTreeMap, error::Error, path::Path};

pub struct CompareOpts {
    pub city: String,
    pub backend: String,
    pub ensemble: EnsembleOpts,
}

#[derive(Serialize)]
struct Summary<'a> {
    city: &'a str,
    scenario: &'a str,
    description: &'a str,
    backend: &'a str,
    runs: usize,
    agents: usize,
    days: u32,
    base_seed: u64,
    hospital_capacity: u32,
    interventions: &'a [crate::spec::Intervention],
    /// Per-metric mean and 5/95 percentiles across runs.
    metrics: &'a BTreeMap<String, Dist>,
    /// FNV-1a hash of each run's output (reproducibility check).
    run_hashes: &'a [String],
}

#[derive(Serialize)]
struct Row<'a> {
    scenario: &'a str,
    metrics: &'a BTreeMap<String, Dist>,
    /// Relative change of the mean vs baseline (negative = better); absent without a baseline.
    vs_baseline: Option<BTreeMap<&'static str, f64>>,
}

#[derive(Serialize)]
struct Comparison<'a> {
    city: &'a str,
    backend: &'a str,
    runs: usize,
    agents: usize,
    days: u32,
    hospital_capacity: u32,
    scenarios: Vec<Row<'a>>,
}

const DELTA_METRICS: [&str; 5] = [
    "peak_infections",
    "attack_rate",
    "deaths",
    "peak_hospital_load",
    "days_over_capacity",
];

fn write_bands(path: &Path, e: &Ensemble) -> Result<(), Box<dyn Error>> {
    let mut cols = Vec::new();
    for (name, b) in SERIES.iter().zip(&e.bands) {
        cols.push((format!("{name}_mean"), b.mean.clone()));
        cols.push((format!("{name}_p05"), b.p05.clone()));
        cols.push((format!("{name}_p95"), b.p95.clone()));
    }
    write_f64_columns(path, STEPS_PER_DAY, &cols)
}

/// Run every scenario (baseline should come first) and write all outputs under `out_root`.
pub fn compare<B, F>(
    src: &WorldSource,
    scenarios: &[Scenario],
    opts: &CompareOpts,
    out_root: &Path,
    make_backend: F,
) -> Result<(), Box<dyn Error>>
where
    B: Backend,
    F: Fn() -> B + Sync,
{
    let city_dir = out_root.join(&opts.city);
    let eo = &opts.ensemble;
    let mut ensembles = Vec::new();
    for sc in scenarios {
        let t = std::time::Instant::now();
        let e = run_ensemble(src, sc, eo, &make_backend);
        let dir = city_dir.join(&sc.name);
        std::fs::create_dir_all(&dir)?;
        write_bands(&dir.join("bands.parquet"), &e)?;
        let summary = Summary {
            city: &opts.city,
            scenario: &sc.name,
            description: &sc.description,
            backend: &opts.backend,
            runs: eo.runs,
            agents: src.agents(),
            days: eo.days,
            base_seed: eo.base_seed,
            hospital_capacity: eo.hospital_capacity,
            interventions: &sc.interventions,
            metrics: &e.metrics,
            run_hashes: &e.hash,
        };
        std::fs::write(
            dir.join("summary.json"),
            serde_json::to_string_pretty(&summary)? + "\n",
        )?;
        eprintln!(
            "{:<14} {} runs in {:.1}s  peak={:.0} attack={:.1}% deaths={:.0}",
            sc.name,
            eo.runs,
            t.elapsed().as_secs_f64(),
            e.metrics["peak_infections"].mean,
            e.metrics["attack_rate"].mean * 100.0,
            e.metrics["deaths"].mean
        );
        ensembles.push(e);
    }

    let base = scenarios
        .iter()
        .position(|s| s.name == "baseline")
        .map(|i| &ensembles[i]);
    let rows = ensembles
        .iter()
        .map(|e| Row {
            scenario: &e.scenario,
            metrics: &e.metrics,
            vs_baseline: base.map(|b| {
                DELTA_METRICS
                    .iter()
                    .map(|&m| {
                        let (x, y) = (e.metrics[m].mean, b.metrics[m].mean);
                        (m, if y > 0.0 { (x - y) / y } else { 0.0 })
                    })
                    .collect()
            }),
        })
        .collect();
    let cmp = Comparison {
        city: &opts.city,
        backend: &opts.backend,
        runs: eo.runs,
        agents: src.agents(),
        days: eo.days,
        hospital_capacity: eo.hospital_capacity,
        scenarios: rows,
    };
    std::fs::create_dir_all(&city_dir)?;
    std::fs::write(
        city_dir.join("comparison.json"),
        serde_json::to_string_pretty(&cmp)? + "\n",
    )?;
    Ok(())
}
