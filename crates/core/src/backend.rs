//! Backend trait and the sequential reference implementation.
use crate::agents::state::{self, COUNT};
use crate::rng::purpose;
use crate::schedule;
use crate::world::{duration_steps, World, STEPS_PER_DAY};

/// Interventions applied during a step (extended in later phases).
#[derive(Clone, Debug)]
pub struct InterventionState {
    /// Scales the transmission coefficient everywhere.
    pub beta_multiplier: f64,
    /// Closed location kinds, indexed by `LocKind`; occupants stay home.
    pub closed: [bool; 4],
    /// Colleges closed (a subset of work locations); occupants stay home.
    pub colleges_closed: bool,
    /// Probability a symptomatic (I) agent stays home (fixed per agent).
    pub isolation_compliance: f64,
}

impl Default for InterventionState {
    fn default() -> Self {
        Self {
            beta_multiplier: 1.0,
            closed: [false; 4],
            colleges_closed: false,
            isolation_compliance: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StepStats {
    pub step: u32,
    pub day: u32,
    /// State counts at the end of the step, indexed S,E,A,I,H,R,D.
    pub counts: [u32; COUNT],
    pub new_infections: u32,
    /// Agents in state H (hospital occupancy).
    pub hospital_load: u32,
    pub r_t: f64,
}

pub trait Backend {
    fn name(&self) -> &'static str;
    fn step(&mut self, world: &mut World, iv: &InterventionState) -> StepStats;
    /// Copy device-resident agent state back into `world` (no-op for host backends).
    fn sync_to_host(&mut self, _world: &mut World) {}
    /// The host edited `world.agents` (e.g. vaccination): re-upload before the next step.
    fn host_changed(&mut self) {}
}

impl<B: Backend + ?Sized> Backend for Box<B> {
    fn name(&self) -> &'static str {
        (**self).name()
    }
    fn step(&mut self, world: &mut World, iv: &InterventionState) -> StepStats {
        (**self).step(world, iv)
    }
    fn sync_to_host(&mut self, world: &mut World) {
        (**self).sync_to_host(world)
    }
    fn host_changed(&mut self) {
        (**self).host_changed()
    }
}

const NONE: u32 = u32::MAX;

#[derive(Default)]
pub struct SequentialBackend {
    loc: Vec<u32>,
    occupancy: Vec<u32>,
    infectious: Vec<f64>,
    lambda: Vec<f64>,
}

impl Backend for SequentialBackend {
    fn name(&self) -> &'static str {
        "sequential"
    }

    fn step(&mut self, w: &mut World, iv: &InterventionState) -> StepStats {
        let step = w.step;
        let n = w.agents.len();
        let nloc = w.layout.total();
        self.loc.clear();
        self.loc.resize(n, NONE);
        self.occupancy.clear();
        self.occupancy.resize(nloc, 0);
        self.infectious.clear();
        self.infectious.resize(nloc, 0.0);
        self.lambda.clear();
        self.lambda.resize(nloc, 0.0);

        // 1. Place agents and bucket by location (O(N)).
        let rel_a = w.disease.rel_infectiousness_asymptomatic;
        let mut infectious_total = 0.0;
        for i in 0..n {
            let st = w.agents.state[i];
            if st == state::D {
                continue;
            }
            let l = schedule::location_of(w, i, step, iv);
            self.loc[i] = l;
            self.occupancy[l as usize] += 1;
            let wgt = match st {
                state::A => rel_a,
                state::I => 1.0,
                _ => 0.0,
            };
            self.infectious[l as usize] += wgt;
            infectious_total += wgt;
        }

        // 2. Force of infection per location: 1 - exp(-beta * infectious / sqrt(occupancy)).
        for l in 0..nloc {
            if self.infectious[l] > 0.0 {
                let beta = w.disease.beta[w.layout.kind_of(l as u32) as usize] * iv.beta_multiplier;
                let occ = f64::from(self.occupancy[l].max(1));
                self.lambda[l] = 1.0 - (-beta * self.infectious[l] / occ.sqrt()).exp();
            }
        }

        // 3. Progression of already-infected agents (decided from start-of-step states).
        for i in 0..n {
            let st = w.agents.state[i];
            if matches!(st, state::S | state::R | state::D) {
                continue;
            }
            let t = w.agents.timer[i].saturating_sub(1);
            if t > 0 {
                w.agents.timer[i] = t;
                continue;
            }
            let u = w.philox.uniforms(w.agents.id[i], step, purpose::TRANSITION);
            let age = w.agents.age_group[i] as usize;
            let d = &w.disease;
            let (new_state, mean_days) = match st {
                state::E if u[0] < d.p_asymptomatic[age] => (state::A, d.asymptomatic_days),
                state::E => (state::I, d.symptomatic_days),
                state::A => (state::R, 0.0),
                state::I => {
                    let ve = w
                        .vaccine
                        .eff_severe(w.agents.vax_status[i], w.agents.vax_days[i]);
                    if u[0] < d.p_hospitalised[age] * (1.0 - ve) {
                        (state::H, d.hospital_days)
                    } else {
                        (state::R, 0.0)
                    }
                }
                _ => {
                    if u[0] < d.p_death_hospitalised[age] {
                        (state::D, 0.0)
                    } else {
                        (state::R, 0.0)
                    }
                }
            };
            w.agents.state[i] = new_state;
            w.agents.timer[i] = if mean_days > 0.0 {
                duration_steps(mean_days, u[1])
            } else {
                0
            };
        }

        // 4. New infections among susceptibles.
        let mut new_infections = 0u32;
        for i in 0..n {
            if w.agents.state[i] != state::S {
                continue;
            }
            let p = self.lambda[self.loc[i] as usize];
            if p <= 0.0 {
                continue;
            }
            let ve = w
                .vaccine
                .eff_infection(w.agents.vax_status[i], w.agents.vax_days[i]);
            let u = w.philox.uniforms(w.agents.id[i], step, purpose::INFECT);
            if u[0] < p * (1.0 - ve) {
                w.infect(i, u[1]);
                new_infections += 1;
            }
        }

        // 5. Vaccine clocks tick once per day.
        if step % STEPS_PER_DAY == STEPS_PER_DAY - 1 {
            for i in 0..n {
                if w.agents.vax_status[i] != 0 {
                    w.agents.vax_days[i] = w.agents.vax_days[i].saturating_add(1);
                }
            }
        }
        w.step += 1;

        let counts = w.agents.counts();
        let r_t = if infectious_total > 0.0 {
            f64::from(new_infections) / infectious_total * w.infectious_steps_per_case
        } else {
            0.0
        };
        StepStats {
            step,
            day: step / STEPS_PER_DAY,
            counts,
            new_infections,
            hospital_load: counts[state::H as usize],
            r_t,
        }
    }
}
