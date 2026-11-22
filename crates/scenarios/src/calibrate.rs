//! Rough calibration (grid search on early growth) and one-at-a-time sensitivity analysis.
//! Scenario/ensemble level only: no kernel or engine changes.
use crate::ensemble::{run_ensemble, run_once, EnsembleOpts, WorldSource};
use crate::spec::{Intervention, Scenario};
use epidemicsim_core::{Config, SequentialBackend, StepStats};
use rayon::prelude::*;
use serde::Serialize;
use std::collections::BTreeMap;

const STEPS_PER_DAY: usize = 4;
/// Days (inclusive start, exclusive end) of the log-linear growth fit. The end is cut short when
/// cumulative infections reach [`SATURATION_STOP`] of the population (depletion bends the curve).
pub const FIT_WINDOW: (usize, usize) = (6, 30);
pub const SATURATION_STOP: f64 = 0.05;
const MIN_WINDOW_DAYS: usize = 6;

/// Exponential growth rate per day of daily new infections over [`FIT_WINDOW`] (least squares on
/// ln(max(count, 1))).
pub fn growth_rate(stats: &[StepStats]) -> f64 {
    let days = stats.len() / STEPS_PER_DAY;
    let n_pop: u32 = stats.first().map_or(1, |s| s.counts.iter().sum());
    let mut cum = 0u64;
    let mut stop = days;
    for (i, s) in stats.iter().enumerate() {
        cum += u64::from(s.new_infections);
        if cum as f64 >= SATURATION_STOP * f64::from(n_pop) {
            stop = i / STEPS_PER_DAY;
            break;
        }
    }
    let a = FIT_WINDOW.0;
    let b = FIT_WINDOW.1.min(days).min(stop).max(a + MIN_WINDOW_DAYS);
    let ys: Vec<f64> = (a..b)
        .map(|d| {
            let c: u32 = stats[d * STEPS_PER_DAY..(d + 1) * STEPS_PER_DAY]
                .iter()
                .map(|s| s.new_infections)
                .sum();
            f64::from(c.max(1)).ln()
        })
        .collect();
    let n = ys.len() as f64;
    let mx = (a..b).map(|d| d as f64).sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let (mut sxy, mut sxx) = (0.0, 0.0);
    for (i, y) in ys.iter().enumerate() {
        let dx = (a + i) as f64 - mx;
        sxy += dx * (y - my);
        sxx += dx * dx;
    }
    sxy / sxx
}

/// Population-weighted P(asymptomatic | infected) under the synthetic age distribution.
pub fn asym_share(cfg: &Config) -> f64 {
    let mut prev = 0.0;
    let mut s = 0.0;
    for g in 0..cfg.synth.age_cdf.len() {
        s += (cfg.synth.age_cdf[g] - prev) * cfg.disease.p_asymptomatic[g];
        prev = cfg.synth.age_cdf[g];
    }
    s
}

/// Scale the home/work/market betas (hospital stays as is) and every age's P(asymptomatic).
pub fn scaled(base: &Config, beta_scale: f64, asym_scale: f64) -> Config {
    let mut c = base.clone();
    for b in c.disease.beta.iter_mut().take(3) {
        *b *= beta_scale;
    }
    for p in c.disease.p_asymptomatic.iter_mut() {
        *p = (*p * asym_scale).min(0.95);
    }
    c
}

#[derive(Clone, Debug, Serialize)]
pub struct GridPoint {
    pub beta_scale: f64,
    pub asym_scale: f64,
    pub asym_share: f64,
    pub growth_mean: f64,
    pub doubling_days_mean: f64,
    pub doubling_days_sd: f64,
    pub attack_rate_at_horizon: f64,
    pub loss: f64,
}

#[derive(Clone, Debug)]
pub struct Targets {
    pub doubling_days: f64,
    pub asym_share: f64,
    /// Scales of the squared errors in the loss.
    pub doubling_tol: f64,
    pub share_tol: f64,
}

fn sd(v: &[f64]) -> f64 {
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len().max(2) - 1) as f64).sqrt()
}

/// Evaluate every (beta_scale, asym_scale) pair with `runs` seeded synthetic cities each.
/// `base.agents`/`base.seed` define the sample; `days` is the run length (>= FIT_WINDOW.1).
pub fn grid_search(
    base: &Config,
    betas: &[f64],
    asyms: &[f64],
    runs: usize,
    days: u32,
    t: &Targets,
) -> Vec<GridPoint> {
    let baseline = Scenario::from_toml_str("name='baseline'").expect("static toml");
    let tasks: Vec<(usize, usize, usize)> = (0..betas.len())
        .flat_map(|i| (0..asyms.len()).flat_map(move |j| (0..runs).map(move |k| (i, j, k))))
        .collect();
    let res: Vec<(f64, f64)> = tasks
        .par_iter()
        .map(|&(i, j, k)| {
            let cfg = scaled(base, betas[i], asyms[j]);
            let src = WorldSource::Synth(Box::new(cfg.clone()));
            let o = EnsembleOpts {
                runs,
                days,
                base_seed: base.seed,
                hospital_capacity: 1,
            };
            let (stats, m) = run_once(
                &src,
                &baseline,
                base.seed + k as u64,
                &o,
                &mut SequentialBackend::default(),
            );
            (growth_rate(&stats), m.attack_rate)
        })
        .collect();
    let mut out = Vec::new();
    for (i, &b) in betas.iter().enumerate() {
        for (j, &a) in asyms.iter().enumerate() {
            let idx = (i * asyms.len() + j) * runs;
            let r = &res[idx..idx + runs];
            let dts: Vec<f64> = r.iter().map(|x| std::f64::consts::LN_2 / x.0).collect();
            let g = r.iter().map(|x| x.0).sum::<f64>() / runs as f64;
            let dt = dts.iter().sum::<f64>() / runs as f64;
            let share = asym_share(&scaled(base, b, a));
            out.push(GridPoint {
                beta_scale: b,
                asym_scale: a,
                asym_share: share,
                growth_mean: g,
                doubling_days_mean: dt,
                doubling_days_sd: sd(&dts),
                attack_rate_at_horizon: r.iter().map(|x| x.1).sum::<f64>() / runs as f64,
                loss: ((dt - t.doubling_days) / t.doubling_tol).powi(2)
                    + ((share - t.asym_share) / t.share_tol).powi(2),
            });
        }
    }
    out
}

// ---------------------------------------------------------------- sensitivity

#[derive(Clone, Debug, Serialize)]
pub struct Outcome {
    pub attack_rate: f64,
    pub attack_rate_p05: f64,
    pub attack_rate_p95: f64,
    pub peak_infections: f64,
    pub peak_infections_p05: f64,
    pub peak_infections_p95: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct SensRow {
    pub parameter: String,
    /// Scenario used to exercise the parameter.
    pub scenario: String,
    pub base_value: f64,
    pub low_value: f64,
    pub high_value: f64,
    pub base: Outcome,
    pub low: Outcome,
    pub high: Outcome,
}

fn outcome(src: &WorldSource, sc: &Scenario, o: &EnsembleOpts) -> Outcome {
    let e = run_ensemble(src, sc, o, SequentialBackend::default);
    let (a, p) = (&e.metrics["attack_rate"], &e.metrics["peak_infections"]);
    Outcome {
        attack_rate: a.mean,
        attack_rate_p05: a.p05,
        attack_rate_p95: a.p95,
        peak_infections: p.mean,
        peak_infections_p05: p.p05,
        peak_infections_p95: p.p95,
    }
}

fn first_masks_efficacy(s: &Scenario) -> f64 {
    s.interventions
        .iter()
        .find_map(|i| match i {
            Intervention::Masks { efficacy, .. } => Some(*efficacy),
            _ => None,
        })
        .unwrap_or(0.0)
}

fn first_isolation(s: &Scenario) -> f64 {
    s.interventions
        .iter()
        .find_map(|i| match i {
            Intervention::Isolation { compliance, .. } => Some(*compliance),
            _ => None,
        })
        .unwrap_or(0.0)
}

fn mean_of(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// One-at-a-time +-`delta` (relative) sensitivity on a synthetic sample. `scenarios` must contain
/// baseline, vax_random, masks and isolation. Runs are paired: run k uses the same seed everywhere.
pub fn sensitivity(
    base: &Config,
    scenarios: &[Scenario],
    delta: f64,
    o: &EnsembleOpts,
) -> Vec<SensRow> {
    let find = |n: &str| {
        scenarios
            .iter()
            .find(|s| s.name == n)
            .unwrap_or_else(|| panic!("scenario {n} missing"))
            .clone()
    };
    let (baseline, vax, masks, iso) = (
        find("baseline"),
        find("vax_random"),
        find("masks"),
        find("isolation"),
    );
    type Apply = Box<dyn Fn(&mut Config, &mut Scenario, f64)>;
    let clip = |v: f64| v.min(1.0);
    let mut specs: Vec<(&str, &Scenario, f64, Apply)> = vec![
        (
            "beta (home/work/market)",
            &baseline,
            mean_of(&base.disease.beta[..3]),
            Box::new(|c, _, f| c.disease.beta.iter_mut().take(3).for_each(|b| *b *= f)),
        ),
        (
            "asymptomatic fraction",
            &baseline,
            asym_share(base),
            Box::new(move |c, _, f| {
                c.disease
                    .p_asymptomatic
                    .iter_mut()
                    .for_each(|p| *p = (*p * f).min(0.95))
            }),
        ),
        (
            "hospitalisation rate",
            &baseline,
            mean_of(&base.disease.p_hospitalised),
            Box::new(move |c, _, f| {
                c.disease
                    .p_hospitalised
                    .iter_mut()
                    .for_each(|p| *p = clip(*p * f))
            }),
        ),
        (
            "fatality rate (given hospitalised)",
            &baseline,
            mean_of(&base.disease.p_death_hospitalised),
            Box::new(move |c, _, f| {
                c.disease
                    .p_death_hospitalised
                    .iter_mut()
                    .for_each(|p| *p = clip(*p * f))
            }),
        ),
        (
            "vaccine efficacy (infection)",
            &vax,
            base.vaccine.ve_infection,
            Box::new(|c, _, f| c.vaccine.ve_infection = (c.vaccine.ve_infection * f).min(1.0)),
        ),
    ];
    let me = first_masks_efficacy(&masks);
    specs.push((
        "mask efficacy",
        &masks,
        me,
        Box::new(|_, s, f| {
            for i in &mut s.interventions {
                if let Intervention::Masks { efficacy, .. } = i {
                    *efficacy = (*efficacy * f).min(1.0);
                }
            }
        }),
    ));
    let ic = first_isolation(&iso);
    specs.push((
        "isolation compliance",
        &iso,
        ic,
        Box::new(|_, s, f| {
            for i in &mut s.interventions {
                if let Intervention::Isolation { compliance, .. } = i {
                    *compliance = (*compliance * f).min(1.0);
                }
            }
        }),
    ));

    let mut base_cache: BTreeMap<String, Outcome> = BTreeMap::new();
    let mut rows = Vec::new();
    for (name, sc, central, apply) in &specs {
        let run = |f: f64| {
            let mut c = base.clone();
            let mut s = (*sc).clone();
            apply(&mut c, &mut s, f);
            outcome(&WorldSource::Synth(Box::new(c)), &s, o)
        };
        let b = base_cache
            .entry(sc.name.clone())
            .or_insert_with(|| outcome(&WorldSource::Synth(Box::new(base.clone())), sc, o))
            .clone();
        rows.push(SensRow {
            parameter: (*name).to_string(),
            scenario: sc.name.clone(),
            base_value: *central,
            low_value: central * (1.0 - delta),
            high_value: central * (1.0 + delta),
            base: b,
            low: run(1.0 - delta),
            high: run(1.0 + delta),
        });
    }
    rows
}

// ---------------------------------------------------------------- drivers (write files)

pub struct CalibrateOpts {
    pub runs: usize,
    pub days: u32,
    pub targets: Targets,
    pub toml_out: std::path::PathBuf,
    pub json_out: std::path::PathBuf,
}

#[derive(Serialize)]
struct CalibrationFile<'a> {
    note: &'a str,
    agents: usize,
    days: u32,
    runs_per_point: usize,
    base_seed: u64,
    fit_window_days: [usize; 2],
    target_doubling_days: f64,
    target_asym_share: f64,
    default_asym_share: f64,
    stages: Vec<(&'a str, &'a [GridPoint])>,
    best: &'a GridPoint,
    default_doubling_days: f64,
}

/// Two-stage grid search (coarse, then beta refined around the best coarse point); writes the
/// fitted `[disease]` TOML and the full grid as JSON. Returns the best point.
pub fn run_calibration(base: &Config, o: &CalibrateOpts) -> std::io::Result<GridPoint> {
    let asyms = [0.6, 0.8, 1.0, 1.1, 1.2, 1.4];
    let coarse_b = [1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 5.0];
    let coarse = grid_search(base, &coarse_b, &asyms, o.runs, o.days, &o.targets);
    let best_c = best(&coarse);
    let fine_b: Vec<f64> = (-5..=5)
        .map(|k| ((best_c.beta_scale + 0.1 * f64::from(k)) * 100.0).round() / 100.0)
        .filter(|b| *b > 0.0)
        .collect();
    let fine = grid_search(base, &fine_b, &asyms, o.runs, o.days, &o.targets);
    let b = best(&fine.iter().chain(&coarse).cloned().collect::<Vec<_>>());
    let default = grid_search(base, &[1.0], &[1.0], o.runs, o.days, &o.targets);

    let fit = scaled(base, b.beta_scale, b.asym_scale);
    let d = &fit.disease;
    let toml = format!(
        "# Rough calibration, NOT a forecast. See docs/CALIBRATION.md.\n\
         # Fitted on {} synthetic agents, {} runs per grid point, early growth from day {} (up to day {} or {}% cumulative attack).\n\
         # beta_scale {} and asym_scale {} multiply the built-in defaults.\n\
         [disease]\nbeta = [{:.5}, {:.5}, {:.5}, {:.5}]\np_asymptomatic = [{}]\n",
        base.agents,
        o.runs,
        FIT_WINDOW.0,
        FIT_WINDOW.1,
        SATURATION_STOP * 100.0,
        b.beta_scale,
        b.asym_scale,
        d.beta[0],
        d.beta[1],
        d.beta[2],
        d.beta[3],
        d.p_asymptomatic
            .iter()
            .map(|p| format!("{p:.4}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    if let Some(p) = o.toml_out.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(&o.toml_out, toml)?;
    let file = CalibrationFile {
        note: "Rough calibration to an assumed, illustrative target; not a forecast.",
        agents: base.agents,
        days: o.days,
        runs_per_point: o.runs,
        base_seed: base.seed,
        fit_window_days: [FIT_WINDOW.0, FIT_WINDOW.1],
        target_doubling_days: o.targets.doubling_days,
        target_asym_share: o.targets.asym_share,
        default_asym_share: asym_share(base),
        stages: vec![("coarse", &coarse), ("fine", &fine)],
        best: &b,
        default_doubling_days: default[0].doubling_days_mean,
    };
    if let Some(p) = o.json_out.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(&o.json_out, serde_json::to_string_pretty(&file)?)?;
    Ok(b)
}

fn best(v: &[GridPoint]) -> GridPoint {
    v.iter()
        .filter(|g| g.loss.is_finite())
        .min_by(|a, b| a.loss.total_cmp(&b.loss))
        .expect("non-empty grid")
        .clone()
}

#[derive(Serialize)]
struct SensFile<'a> {
    city: &'a str,
    params: &'a str,
    note: &'a str,
    agents: usize,
    days: u32,
    runs: usize,
    base_seed: u64,
    backend: &'a str,
    delta: f64,
    rows: &'a [SensRow],
}

/// Run [`sensitivity`] and write `results/sensitivity/<name>.json`.
pub fn run_sensitivity(
    base: &Config,
    scenarios: &[Scenario],
    delta: f64,
    o: &EnsembleOpts,
    params: &str,
    out: &std::path::Path,
) -> std::io::Result<Vec<SensRow>> {
    let rows = sensitivity(base, scenarios, delta, o);
    let f = SensFile {
        city: "synthetic",
        params,
        note: "One-at-a-time +-delta relative change of each parameter, paired seeds. Each parameter is exercised in the scenario named in its row; effects are relative to that scenario's own central run.",
        agents: base.agents,
        days: o.days,
        runs: o.runs,
        base_seed: o.base_seed,
        backend: "sequential",
        delta,
        rows: &rows,
    };
    if let Some(p) = out.parent() {
        std::fs::create_dir_all(p)?;
    }
    std::fs::write(out, serde_json::to_string_pretty(&f)?)?;
    Ok(rows)
}
