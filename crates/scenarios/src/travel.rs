//! Inter-city travel at the scenario level: several city worlds advance in lockstep and, at the
//! start of each day, infectious travellers from one city seed infections in another.
//!
//! No kernel is involved. Everything is a deterministic function of the run seed through the
//! Philox keying (`purpose::TRAVEL`, `purpose::TRAVEL_LAND`):
//!
//! * a source agent in state A or I travels with probability `total_rate * factor` (factor 1 for A,
//!   `symptomatic_factor` for I) and its destination is picked from the same uniform;
//! * only the per-destination traveller *counts* leave the city (`outgoing`), so ranks can
//!   exchange `n_cities` integers per day;
//! * the destination turns the k-th traveller from city s into a draw keyed by `(s, k, day)`:
//!   one random resident, infected only if currently susceptible (`land`).
//!
//! Flows are ASSUMED, illustrative rates (config/travel.toml), not data.
use crate::ensemble::WorldSource;
use crate::metrics::{Dist, RunMetrics};
use crate::spec::{Intervention, Scenario};
use crate::Controller;
use epidemicsim_core::agents::state;
use epidemicsim_core::rng::{purpose, to_unit};
use epidemicsim_core::world::STEPS_PER_DAY;
use epidemicsim_core::{Backend, StepStats, World};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Flow {
    pub from: String,
    pub to: String,
    /// Assumed per-agent probability of a trip from `from` to `to` on a given day.
    pub per_agent_per_day: f64,
}

#[derive(Clone, Debug, Deserialize)]
struct TravelFile {
    cities: Vec<String>,
    #[serde(default)]
    seed_cities: Vec<String>,
    #[serde(default = "default_sym")]
    symptomatic_factor: f64,
    #[serde(default = "default_thr")]
    start_threshold: f64,
    #[serde(default = "default_other")]
    other_initial_frac: f64,
    #[serde(default)]
    flows: Vec<Flow>,
}

fn default_sym() -> f64 {
    0.3
}
fn default_other() -> f64 {
    0.00001
}
fn default_thr() -> f64 {
    0.005
}

#[derive(Clone, Debug)]
pub struct Travel {
    pub cities: Vec<String>,
    /// Cities seeded with the full initial infected fraction at day 0; the rest get only a trace.
    pub seeded: Vec<bool>,
    pub symptomatic_factor: f64,
    pub start_threshold: f64,
    /// Initial infected fraction of the cities not in `seeded` (at least one agent), both arms.
    pub other_initial_frac: f64,
    /// `rate[from][to]`, zero on the diagonal.
    pub rate: Vec<Vec<f64>>,
    pub flows: Vec<Flow>,
}

impl Travel {
    pub fn from_file(p: &Path) -> Result<Self, Box<dyn Error>> {
        Self::from_toml_str(&std::fs::read_to_string(p)?)
    }

    pub fn from_toml_str(s: &str) -> Result<Self, Box<dyn Error>> {
        let f: TravelFile = toml::from_str(s)?;
        let n = f.cities.len();
        if n < 2 {
            return Err("travel needs at least two cities".into());
        }
        let idx = |name: &str| {
            f.cities
                .iter()
                .position(|c| c == name)
                .ok_or_else(|| format!("travel: unknown city '{name}'"))
        };
        let mut rate = vec![vec![0.0; n]; n];
        for fl in &f.flows {
            let (a, b) = (idx(&fl.from)?, idx(&fl.to)?);
            if a == b || !(0.0..=1.0).contains(&fl.per_agent_per_day) {
                return Err(format!("travel: bad flow {} -> {}", fl.from, fl.to).into());
            }
            rate[a][b] += fl.per_agent_per_day;
        }
        let mut seeded = vec![false; n];
        for c in &f.seed_cities {
            seeded[idx(c)?] = true;
        }
        if !seeded.iter().any(|&s| s) {
            return Err("travel: seed_cities must name at least one city".into());
        }
        Ok(Self {
            cities: f.cities,
            seeded,
            symptomatic_factor: f.symptomatic_factor,
            start_threshold: f.start_threshold,
            other_initial_frac: f.other_initial_frac,
            rate,
            flows: f.flows,
        })
    }

    pub fn n(&self) -> usize {
        self.cities.len()
    }

    /// Travellers leaving city `c` today, per destination. Reads agent states only.
    pub fn outgoing(&self, w: &World, c: usize, day: u32) -> Vec<u32> {
        let mut out = vec![0u32; self.n()];
        let total: f64 = self.rate[c].iter().sum();
        if total <= 0.0 {
            return out;
        }
        for i in 0..w.agents.len() {
            let f = match w.agents.state[i] {
                state::A => 1.0,
                state::I => self.symptomatic_factor,
                _ => continue,
            };
            let u = w.philox.uniform(w.agents.id[i], day, purpose::TRAVEL);
            let p = total * f;
            if u >= p {
                continue;
            }
            let v = u / p * total;
            let mut acc = 0.0;
            for (j, r) in self.rate[c].iter().enumerate() {
                acc += r;
                if v < acc || j + 1 == out.len() {
                    out[j] += 1;
                    break;
                }
            }
        }
        out
    }

    /// Apply today's arrivals to city `c` (`counts[s][c]` travellers came from `s`).
    /// Returns how many arrivals infected a susceptible resident.
    pub fn land(&self, w: &mut World, c: usize, day: u32, counts: &[Vec<u32>]) -> u32 {
        let n = w.agents.len() as u64;
        let mut hit = 0;
        for (s, row) in counts.iter().enumerate() {
            for k in 0..row[c] {
                let b = w.philox.block(
                    ((s as u32) << 24) | (k & 0xff_ffff),
                    day,
                    purpose::TRAVEL_LAND,
                );
                let i = ((u64::from(b[0]) * n) >> 32) as usize;
                if w.agents.state[i] == state::S {
                    w.infect(i, to_unit(b[1]));
                    hit += 1;
                }
            }
        }
        hit
    }
}

/// Moves the per-day traveller-count rows between whoever owns the cities.
pub trait Exchange {
    /// `mine` holds one row per city this process owns (in city order); returns the full matrix.
    fn all_rows(&self, mine: &[Vec<u32>]) -> Vec<Vec<u32>>;
}

/// One process owns every city.
pub struct LocalExchange;

impl Exchange for LocalExchange {
    fn all_rows(&self, mine: &[Vec<u32>]) -> Vec<Vec<u32>> {
        mine.to_vec()
    }
}

pub struct Ctx<'a> {
    pub srcs: &'a [WorldSource],
    /// Hospital beds per city.
    pub capacities: &'a [u32],
    pub travel: &'a Travel,
    pub scenario: &'a Scenario,
    pub days: u32,
}

pub struct CityRun {
    pub city: usize,
    pub stats: Vec<StepStats>,
    pub metrics: RunMetrics,
    /// Arrivals that infected a susceptible resident.
    pub imports: u32,
}

/// City `c` of run seed `seed`: independent worlds and epidemics per city, same across arms.
pub fn city_seed(seed: u64, c: usize) -> u64 {
    seed.wrapping_add(c as u64 * 1_000_003)
}

/// Run one seeded run of all `owned` cities in lockstep. `coupled = false` is the control arm
/// (same worlds and seeds, no travel). `backends[k]` serves `owned[k]`.
pub fn run_coupled<B: Backend>(
    ctx: &Ctx,
    owned: &[usize],
    coupled: bool,
    seed: u64,
    backends: &mut [B],
    ex: &dyn Exchange,
) -> Vec<CityRun> {
    let has_vax = ctx
        .scenario
        .interventions
        .iter()
        .any(|i| matches!(i, Intervention::Vaccination { .. }));
    let mut worlds: Vec<World> = owned
        .iter()
        .map(|&c| {
            let mut w = ctx.srcs[c].make(city_seed(seed, c));
            if !ctx.travel.seeded[c] {
                w.agents.state.fill(state::S);
                w.agents.timer.fill(0);
                w.seed_infections(ctx.travel.other_initial_frac);
            }
            w
        })
        .collect();
    let mut ctls: Vec<Controller> = owned
        .iter()
        .zip(&worlds)
        .map(|(&c, w)| Controller::new(ctx.scenario, w.agents.len(), ctx.capacities[c]))
        .collect();
    let mut stats: Vec<Vec<StepStats>> = owned
        .iter()
        .map(|_| Vec::with_capacity((ctx.days * STEPS_PER_DAY) as usize))
        .collect();
    let mut imports = vec![0u32; owned.len()];
    for step in 0..ctx.days * STEPS_PER_DAY {
        let boundary = step.is_multiple_of(STEPS_PER_DAY);
        let edits = boundary && (has_vax || coupled);
        if edits {
            for (b, w) in backends.iter_mut().zip(worlds.iter_mut()) {
                b.sync_to_host(w);
            }
        }
        if boundary && coupled {
            let day = step / STEPS_PER_DAY;
            let mine: Vec<Vec<u32>> = owned
                .iter()
                .zip(&worlds)
                .map(|(&c, w)| ctx.travel.outgoing(w, c, day))
                .collect();
            let all = ex.all_rows(&mine);
            for (k, &c) in owned.iter().enumerate() {
                imports[k] += ctx.travel.land(&mut worlds[k], c, day, &all);
            }
        }
        for k in 0..owned.len() {
            let iv = ctls[k].update(&mut worlds[k], stats[k].last()).clone();
            if edits {
                backends[k].host_changed();
            }
            let s = backends[k].step(&mut worlds[k], &iv);
            stats[k].push(s);
        }
    }
    owned
        .iter()
        .enumerate()
        .zip(stats)
        .map(|((k, &c), st)| {
            let n = worlds[k].agents.len();
            let metrics = RunMetrics::from_stats(&st, n, ctx.capacities[c], ctls[k].doses());
            CityRun {
                city: c,
                stats: st,
                metrics,
                imports: imports[k],
            }
        })
        .collect()
}

/// `[run][city]` results of one arm.
pub type ArmRuns = Vec<Vec<CityRun>>;

/// Run both arms (control first) in one process, parallel over runs.
pub fn run_single<B, F>(
    ctx: &Ctx,
    runs: usize,
    base_seed: u64,
    make_backend: F,
) -> (ArmRuns, ArmRuns)
where
    B: Backend,
    F: Fn() -> B + Sync,
{
    let all: Vec<usize> = (0..ctx.travel.n()).collect();
    let arm = |coupled: bool| -> ArmRuns {
        (0..runs)
            .into_par_iter()
            .map(|k| {
                let mut bs: Vec<B> = all.iter().map(|_| make_backend()).collect();
                run_coupled(
                    ctx,
                    &all,
                    coupled,
                    base_seed + k as u64,
                    &mut bs,
                    &LocalExchange,
                )
            })
            .collect()
    };
    (arm(false), arm(true))
}

#[derive(Serialize)]
pub struct CityArm {
    pub city: String,
    /// Mean over runs of (N - S) / N at the end of each day.
    pub cum_frac_mean: Vec<f64>,
    /// Per run: first day the cumulative infected fraction reached the threshold.
    pub start_days: Vec<Option<u32>>,
    pub reached: usize,
    pub start_median: Option<f64>,
    pub attack_rate: Dist,
    pub imports_mean: f64,
    pub run_hashes: Vec<String>,
}

#[derive(Serialize)]
pub struct Arm {
    pub travel: bool,
    pub cities: Vec<CityArm>,
}

/// With vs without travel, per city, over the same seeds.
#[derive(Serialize)]
pub struct Paired {
    pub city: String,
    /// Runs where the city's epidemic started earlier / later / the same day with travel.
    pub earlier: usize,
    pub later: usize,
    pub same: usize,
    /// Reached the threshold only with travel (never without) / only without.
    pub only_with_travel: usize,
    pub only_without_travel: usize,
    /// Mean (with - without) start day over runs where both reached it (negative = earlier).
    pub mean_shift_days: Option<f64>,
}

#[derive(Serialize)]
pub struct TravelReport {
    pub note: &'static str,
    pub params: String,
    pub scenario: String,
    pub cities: Vec<String>,
    pub seeded: Vec<bool>,
    pub agents: Vec<usize>,
    pub runs: usize,
    pub days: u32,
    pub base_seed: u64,
    pub start_threshold: f64,
    pub other_initial_frac: f64,
    pub symptomatic_factor: f64,
    pub flows: Vec<Flow>,
    pub without_travel: Arm,
    pub with_travel: Arm,
    pub paired: Vec<Paired>,
}

const NOTE: &str =
    "ASSUMED illustrative flows (config/travel.toml), not measured commuter data; not a forecast.";

fn arm_summary(t: &Travel, agents: &[usize], days: u32, coupled: bool, runs: &ArmRuns) -> Arm {
    let cities = (0..t.n())
        .map(|c| {
            let n = agents[c] as f64;
            let curves: Vec<Vec<f64>> = runs
                .iter()
                .map(|r| {
                    (0..days as usize)
                        .map(|d| {
                            let s = &r[c].stats[d * STEPS_PER_DAY as usize + 3];
                            1.0 - f64::from(s.counts[state::S as usize]) / n
                        })
                        .collect()
                })
                .collect();
            let mean = (0..days as usize)
                .map(|d| curves.iter().map(|cv| cv[d]).sum::<f64>() / curves.len().max(1) as f64)
                .collect();
            let start_days: Vec<Option<u32>> = curves
                .iter()
                .map(|cv| {
                    cv.iter()
                        .position(|&v| v >= t.start_threshold)
                        .map(|d| d as u32)
                })
                .collect();
            let mut reached: Vec<f64> =
                start_days.iter().flatten().map(|&d| f64::from(d)).collect();
            reached.sort_by(f64::total_cmp);
            let attack: Vec<f64> = runs.iter().map(|r| r[c].metrics.attack_rate).collect();
            CityArm {
                city: t.cities[c].clone(),
                cum_frac_mean: mean,
                start_median: (!reached.is_empty())
                    .then(|| crate::metrics::quantile(&reached, 0.5)),
                reached: reached.len(),
                start_days,
                attack_rate: Dist::of(&attack),
                imports_mean: runs.iter().map(|r| f64::from(r[c].imports)).sum::<f64>()
                    / runs.len().max(1) as f64,
                run_hashes: runs
                    .iter()
                    .map(|r| epidemicsim_core::output::hash_stats(&r[c].stats))
                    .collect(),
            }
        })
        .collect();
    Arm {
        travel: coupled,
        cities,
    }
}

fn paired(without: &Arm, with: &Arm) -> Vec<Paired> {
    without
        .cities
        .iter()
        .zip(&with.cities)
        .map(|(a, b)| {
            let mut p = Paired {
                city: a.city.clone(),
                earlier: 0,
                later: 0,
                same: 0,
                only_with_travel: 0,
                only_without_travel: 0,
                mean_shift_days: None,
            };
            let mut shifts = Vec::new();
            for (x, y) in a.start_days.iter().zip(&b.start_days) {
                match (x, y) {
                    (Some(x), Some(y)) => {
                        shifts.push(f64::from(*y) - f64::from(*x));
                        match y.cmp(x) {
                            std::cmp::Ordering::Less => p.earlier += 1,
                            std::cmp::Ordering::Greater => p.later += 1,
                            std::cmp::Ordering::Equal => p.same += 1,
                        }
                    }
                    (None, Some(_)) => p.only_with_travel += 1,
                    (Some(_), None) => p.only_without_travel += 1,
                    (None, None) => {}
                }
            }
            if !shifts.is_empty() {
                p.mean_shift_days = Some(shifts.iter().sum::<f64>() / shifts.len() as f64);
            }
            p
        })
        .collect()
}

/// Build the report from both arms. Pure and deterministic, so the single-process and MPI paths
/// produce byte-identical JSON.
pub fn build_report(
    ctx: &Ctx,
    params: &str,
    runs: usize,
    base_seed: u64,
    without: &ArmRuns,
    with: &ArmRuns,
) -> TravelReport {
    let t = ctx.travel;
    let agents: Vec<usize> = ctx.srcs.iter().map(WorldSource::agents).collect();
    let wo = arm_summary(t, &agents, ctx.days, false, without);
    let wi = arm_summary(t, &agents, ctx.days, true, with);
    TravelReport {
        note: NOTE,
        params: params.to_string(),
        scenario: ctx.scenario.name.clone(),
        cities: t.cities.clone(),
        seeded: t.seeded.clone(),
        agents,
        runs,
        days: ctx.days,
        base_seed,
        start_threshold: t.start_threshold,
        other_initial_frac: t.other_initial_frac,
        symptomatic_factor: t.symptomatic_factor,
        flows: t.flows.clone(),
        paired: paired(&wo, &wi),
        without_travel: wo,
        with_travel: wi,
    }
}

pub fn write_report(path: &Path, r: &TravelReport) -> Result<(), Box<dyn Error>> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(r)? + "\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use epidemicsim_core::{Config, SequentialBackend};

    const TOML: &str = "cities=['a','b']\nseed_cities=['a']\n[[flows]]\nfrom='a'\nto='b'\nper_agent_per_day=0.01\n[[flows]]\nfrom='b'\nto='a'\nper_agent_per_day=0.01\n";

    fn srcs() -> Vec<WorldSource> {
        (0..2)
            .map(|_| {
                let mut cfg = Config {
                    agents: 3000,
                    ..Default::default()
                };
                cfg.disease.initial_infected_frac = 0.02;
                WorldSource::Synth(Box::new(cfg))
            })
            .collect()
    }

    fn go(coupled: bool, seed: u64) -> Vec<CityRun> {
        let t = Travel::from_toml_str(TOML).unwrap();
        let sc = Scenario::from_toml_str("name='baseline'").unwrap();
        let s = srcs();
        let ctx = Ctx {
            srcs: &s,
            capacities: &[10, 10],
            travel: &t,
            scenario: &sc,
            days: 40,
        };
        let mut bs = [SequentialBackend::default(), SequentialBackend::default()];
        run_coupled(&ctx, &[0, 1], coupled, seed, &mut bs, &LocalExchange)
    }

    #[test]
    fn rejects_bad_files() {
        assert!(Travel::from_toml_str("cities=['a']").is_err());
        assert!(
            Travel::from_toml_str("cities=['a','b']").is_err(),
            "no seeded city"
        );
        assert!(Travel::from_toml_str(
            "cities=['a','b']\nseed_cities=['a']\n[[flows]]\nfrom='a'\nto='z'\nper_agent_per_day=0.1\n"
        )
        .is_err());
    }

    #[test]
    fn deterministic_and_travel_seeds_the_empty_city() {
        let (a, b) = (go(true, 5), go(true, 5));
        for (x, y) in a.iter().zip(&b) {
            assert_eq!(x.stats, y.stats);
            assert_eq!(x.imports, y.imports);
        }
        let ctrl = go(false, 5);
        // Control: city b holds only a trace seed.
        assert!(ctrl[1].stats[0].counts[state::E as usize] <= 1);
        assert_eq!(ctrl[1].imports, 0);
        // With travel: imports happen and b gets infected.
        assert!(a[1].imports > 0);
        assert!(
            a[1].stats[40 * 4 - 1].counts[state::S as usize]
                < ctrl[1].stats[40 * 4 - 1].counts[state::S as usize]
        );
        // Source city is the same world in both arms until travel feeds back.
        assert_eq!(a[0].stats[0], ctrl[0].stats[0]);
    }
}
