//! World: agents, location layout and simulation clock.
use crate::agents::{state, Agents};
use crate::params::{DiseaseParams, VaccineParams, N_AGE};
use crate::rng::{purpose, Philox};
use std::ops::Range;

pub const STEPS_PER_DAY: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LocKind {
    Home = 0,
    Work = 1,
    Market = 2,
    Hospital = 3,
}

/// Location ids are contiguous by kind: homes, work/school, markets, hospitals.
#[derive(Clone, Debug)]
pub struct Layout {
    pub homes: Range<u32>,
    pub works: Range<u32>,
    pub markets: Range<u32>,
    pub hospitals: Range<u32>,
    /// Colleges: a sub-range of `works` (empty when the city has none).
    pub colleges: Range<u32>,
}

impl Layout {
    pub fn new(n_homes: u32, n_works: u32, n_markets: u32, n_hospitals: u32) -> Self {
        let a = n_homes;
        let b = a + n_works;
        let c = b + n_markets;
        Self {
            homes: 0..a,
            works: a..b,
            markets: b..c,
            hospitals: c..c + n_hospitals,
            colleges: a..a,
        }
    }

    /// Mark `n_colleges` work locations starting at offset `skip` into `works` as colleges.
    pub fn with_colleges(mut self, skip: u32, n_colleges: u32) -> Self {
        let s = self.works.start + skip;
        self.colleges = s..(s + n_colleges).min(self.works.end);
        self
    }

    pub fn total(&self) -> usize {
        self.hospitals.end as usize
    }

    pub fn kind_of(&self, loc: u32) -> LocKind {
        if loc < self.homes.end {
            LocKind::Home
        } else if loc < self.works.end {
            LocKind::Work
        } else if loc < self.markets.end {
            LocKind::Market
        } else {
            LocKind::Hospital
        }
    }

    fn nearest(r: &Range<u32>, home: u32, n_homes: u32) -> u32 {
        let n = u64::from(r.end - r.start);
        r.start + (u64::from(home) * n / u64::from(n_homes.max(1))) as u32
    }

    /// Neighbourhood market for a home (homes are spatially ordered by id).
    pub fn market_for(&self, home: u32) -> u32 {
        Self::nearest(&self.markets, home, self.homes.end)
    }

    pub fn hospital_for(&self, home: u32) -> u32 {
        Self::nearest(&self.hospitals, home, self.homes.end)
    }
}

#[derive(Clone, Debug)]
pub struct World {
    pub seed: u64,
    pub agents: Agents,
    pub layout: Layout,
    pub disease: DiseaseParams,
    pub vaccine: VaccineParams,
    pub philox: Philox,
    /// Global step counter (STEPS_PER_DAY steps per day; day 0 is a Monday).
    pub step: u32,
    /// Expected infectiousness-weighted steps per case, used for R_t.
    pub infectious_steps_per_case: f64,
}

/// Duration in steps: uniform in [0.5, 1.5] x mean, at least one step.
pub fn duration_steps(mean_days: f64, u: f64) -> u16 {
    (mean_days * f64::from(STEPS_PER_DAY) * (0.5 + u))
        .round()
        .max(1.0) as u16
}

impl World {
    pub fn new(
        seed: u64,
        agents: Agents,
        layout: Layout,
        disease: DiseaseParams,
        vaccine: VaccineParams,
    ) -> Self {
        let mut per_age = [0f64; N_AGE];
        for &g in &agents.age_group {
            per_age[g as usize] += 1.0;
        }
        let n = agents.len().max(1) as f64;
        let steps: f64 = (0..N_AGE)
            .map(|g| {
                let p = disease.p_asymptomatic[g];
                per_age[g]
                    * (p * disease.rel_infectiousness_asymptomatic * disease.asymptomatic_days
                        + (1.0 - p) * disease.symptomatic_days)
            })
            .sum::<f64>()
            * f64::from(STEPS_PER_DAY)
            / n;
        Self {
            seed,
            agents,
            layout,
            disease,
            vaccine,
            philox: Philox::new(seed),
            step: 0,
            infectious_steps_per_case: steps,
        }
    }

    /// Restart the epidemic with a new seed: everyone back to S (vaccination kept), clock at 0.
    pub fn reseed(&mut self, seed: u64, infected_frac: f64) {
        self.seed = seed;
        self.philox = Philox::new(seed);
        self.step = 0;
        self.agents.state.fill(state::S);
        self.agents.timer.fill(0);
        self.seed_infections(infected_frac);
    }

    /// Infect a random fraction of agents (at least one) into state E.
    pub fn seed_infections(&mut self, frac: f64) {
        let mut any = false;
        for i in 0..self.agents.len() {
            let u = self.philox.uniforms(i as u32, 0, purpose::SEED_INFECT);
            if u[0] < frac {
                self.infect(i, u[1]);
                any = true;
            }
        }
        if !any && !self.agents.is_empty() {
            self.infect(0, 0.5);
        }
    }

    pub fn infect(&mut self, i: usize, u_duration: f64) {
        self.agents.state[i] = state::E;
        self.agents.timer[i] = duration_steps(self.disease.latent_days, u_duration);
    }
}
