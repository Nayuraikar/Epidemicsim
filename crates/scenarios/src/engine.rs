//! Runtime controller: turns a scenario into an `InterventionState` (and world edits) each step.
use crate::spec::{Intervention, Metric, Scenario, Strategy, When};
use epidemicsim_core::agents::state;
use epidemicsim_core::rng::purpose;
use epidemicsim_core::world::{LocKind, STEPS_PER_DAY};
use epidemicsim_core::{InterventionState, StepStats, World};

struct Item {
    spec: Intervention,
    started: Option<u32>,
    doses: u64,
}

/// Agents grouped by block of homes, built lazily for location-targeted vaccination.
struct Blocks {
    block_homes: u32,
    /// Agent indices sorted by block.
    order: Vec<u32>,
    /// `order[off[b]..off[b + 1]]` are the agents of block `b`.
    off: Vec<usize>,
}

pub struct Controller {
    items: Vec<Item>,
    n: f64,
    capacity: f64,
    iv: InterventionState,
    blocks: Option<Blocks>,
}

impl Controller {
    pub fn new(scenario: &Scenario, agents: usize, hospital_capacity: u32) -> Self {
        Self {
            items: scenario
                .interventions
                .iter()
                .map(|s| Item {
                    spec: s.clone(),
                    started: None,
                    doses: 0,
                })
                .collect(),
            n: agents.max(1) as f64,
            capacity: f64::from(hospital_capacity.max(1)),
            iv: InterventionState::default(),
            blocks: None,
        }
    }

    /// Total vaccine doses given so far.
    pub fn doses(&self) -> u64 {
        self.items.iter().map(|i| i.doses).sum()
    }

    fn metric(&self, m: Metric, last: Option<&StepStats>) -> f64 {
        let Some(s) = last else { return 0.0 };
        match m {
            Metric::Infectious => {
                f64::from(s.counts[state::A as usize] + s.counts[state::I as usize]) / self.n
            }
            Metric::Hospital => f64::from(s.hospital_load) / self.capacity,
            Metric::Cumulative => 1.0 - f64::from(s.counts[state::S as usize]) / self.n,
        }
    }

    /// Call before every backend step with the previous step's stats. Decisions are made once
    /// per day (at the first step of the day); in between the cached state is returned.
    pub fn update(&mut self, w: &mut World, last: Option<&StepStats>) -> &InterventionState {
        if !w.step.is_multiple_of(STEPS_PER_DAY) {
            return &self.iv;
        }
        let day = w.step / STEPS_PER_DAY;
        let mut iv = InterventionState::default();
        for k in 0..self.items.len() {
            let when: When = *self.items[k].spec.when();
            if self.items[k].started.is_none() {
                let fire = match (when.start_day, when.trigger) {
                    (Some(d), _) => day >= d,
                    (None, Some(t)) => self.metric(t.metric, last) >= t.above,
                    (None, None) => true,
                };
                if fire {
                    self.items[k].started = Some(day);
                }
            }
            let Some(t0) = self.items[k].started else {
                continue;
            };
            if when.duration_days.is_some_and(|d| day >= t0 + d) {
                continue;
            }
            match self.items[k].spec.clone() {
                Intervention::Masks {
                    compliance,
                    efficacy,
                    ..
                } => iv.beta_multiplier *= 1.0 - compliance * efficacy,
                Intervention::Isolation { compliance, .. } => {
                    iv.isolation_compliance = iv.isolation_compliance.max(compliance);
                }
                Intervention::CollegeClosure { .. } => iv.colleges_closed = true,
                Intervention::MarketClosure { .. } => iv.closed[LocKind::Market as usize] = true,
                Intervention::Lockdown { .. } => {
                    iv.closed[LocKind::Work as usize] = true;
                    iv.closed[LocKind::Market as usize] = true;
                    iv.colleges_closed = true;
                }
                Intervention::Vaccination {
                    strategy,
                    coverage,
                    rate_per_day,
                    ages,
                    block_homes,
                    ..
                } => {
                    let left = (coverage * self.n - self.items[k].doses as f64).max(0.0);
                    let quota = (rate_per_day * self.n).min(left).round() as u64;
                    let given = self.vaccinate(w, day, strategy, quota, &ages, block_homes);
                    self.items[k].doses += given;
                }
            }
        }
        self.iv = iv;
        &self.iv
    }

    fn vaccinate(
        &mut self,
        w: &mut World,
        day: u32,
        strategy: Strategy,
        quota: u64,
        ages: &[u8],
        block_homes: u32,
    ) -> u64 {
        if quota == 0 {
            return 0;
        }
        let eligible = |w: &World, i: usize| {
            let a = &w.agents;
            a.vax_status[i] == 0
                && !matches!(a.state[i], state::D | state::H)
                && (strategy != Strategy::Age || ages.contains(&a.age_group[i]))
        };
        let mut given = 0u64;
        match strategy {
            Strategy::Random | Strategy::Age => {
                let n_elig = (0..w.agents.len()).filter(|&i| eligible(w, i)).count() as f64;
                if n_elig == 0.0 {
                    return 0;
                }
                let p = quota as f64 / n_elig;
                for i in 0..w.agents.len() {
                    if given >= quota {
                        break;
                    }
                    if eligible(w, i)
                        && w.philox.uniform(w.agents.id[i], day, purpose::VACCINATE) < p
                    {
                        give(w, i);
                        given += 1;
                    }
                }
            }
            Strategy::Location => {
                if self.blocks.as_ref().map(|b| b.block_homes) != Some(block_homes) {
                    self.blocks = Some(build_blocks(w, block_homes));
                }
                let b = self.blocks.as_ref().expect("built above");
                let nb = b.off.len() - 1;
                let mut score = vec![0u32; nb];
                for (bi, s) in score.iter_mut().enumerate() {
                    *s = b.order[b.off[bi]..b.off[bi + 1]]
                        .iter()
                        .filter(|&&i| {
                            matches!(
                                w.agents.state[i as usize],
                                state::E | state::A | state::I | state::H
                            )
                        })
                        .count() as u32;
                }
                let mut rank: Vec<usize> = (0..nb).collect();
                rank.sort_by_key(|&bi| (std::cmp::Reverse(score[bi]), bi));
                'outer: for bi in rank {
                    for &i in &b.order[b.off[bi]..b.off[bi + 1]] {
                        if given >= quota {
                            break 'outer;
                        }
                        if eligible(w, i as usize) {
                            give(w, i as usize);
                            given += 1;
                        }
                    }
                }
            }
        }
        given
    }
}

fn give(w: &mut World, i: usize) {
    w.agents.vax_status[i] = 1;
    w.agents.vax_days[i] = 0;
}

fn build_blocks(w: &World, block_homes: u32) -> Blocks {
    let nb = (w.layout.homes.end / block_homes + 1) as usize;
    let block = |i: usize| (w.agents.home_loc[i] / block_homes) as usize;
    let mut off = vec![0usize; nb + 1];
    for i in 0..w.agents.len() {
        off[block(i) + 1] += 1;
    }
    for b in 0..nb {
        off[b + 1] += off[b];
    }
    let mut cursor = off.clone();
    let mut order = vec![0u32; w.agents.len()];
    for i in 0..w.agents.len() {
        let b = block(i);
        order[cursor[b]] = i as u32;
        cursor[b] += 1;
    }
    Blocks {
        block_homes,
        order,
        off,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use epidemicsim_core::{synth, Config};

    fn world() -> World {
        synth::generate(&Config {
            agents: 4000,
            ..Default::default()
        })
    }

    fn scen(toml: &str) -> Scenario {
        Scenario::from_toml_str(toml).unwrap()
    }

    #[test]
    fn masks_start_on_day_and_end_after_duration() {
        let s = scen("name='m'\n[[interventions]]\nkind='masks'\nstart_day=2\nduration_days=3\ncompliance=0.5\nefficacy=0.4\n");
        let mut c = Controller::new(&s, 4000, 10);
        let mut w = world();
        let mut seen = vec![];
        for day in 0..7 {
            w.step = day * STEPS_PER_DAY;
            seen.push(c.update(&mut w, None).beta_multiplier);
        }
        let m = 1.0 - 0.5 * 0.4;
        assert_eq!(seen, vec![1.0, 1.0, m, m, m, 1.0, 1.0]);
    }

    #[test]
    fn threshold_trigger_fires_once_metric_crosses() {
        let s = scen(
            "name='l'\n[[interventions]]\nkind='lockdown'\ntrigger={metric='hospital',above=0.5}\n",
        );
        let mut c = Controller::new(&s, 4000, 10);
        let mut w = world();
        let mut st = StepStats {
            step: 0,
            day: 0,
            counts: [0; 7],
            new_infections: 0,
            hospital_load: 4,
            r_t: 0.0,
        };
        assert!(!c.update(&mut w, Some(&st)).closed[LocKind::Work as usize]);
        w.step = 4;
        st.hospital_load = 5;
        assert!(c.update(&mut w, Some(&st)).closed[LocKind::Work as usize]);
        // Stays on after the metric falls again (no duration set).
        w.step = 8;
        st.hospital_load = 0;
        assert!(c.update(&mut w, Some(&st)).closed[LocKind::Market as usize]);
    }

    fn doses_for(toml: &str) -> (World, u64) {
        let s = scen(toml);
        let mut c = Controller::new(&s, 4000, 10);
        let mut w = world();
        for day in 0..10 {
            w.step = day * STEPS_PER_DAY;
            c.update(&mut w, None);
        }
        let n = w.agents.vax_status.iter().filter(|&&v| v == 1).count() as u64;
        assert_eq!(n, c.doses());
        (w, n)
    }

    #[test]
    fn vaccination_respects_coverage_cap_and_strategy() {
        let (_, n) = doses_for("name='v'\n[[interventions]]\nkind='vaccination'\nstrategy='random'\ncoverage=0.1\nrate_per_day=0.05\n");
        assert_eq!(n, 400);
        let (w, n) = doses_for("name='v'\n[[interventions]]\nkind='vaccination'\nstrategy='age'\nages=[4]\ncoverage=0.5\nrate_per_day=0.1\n");
        assert!(n > 0);
        for i in 0..w.agents.len() {
            if w.agents.vax_status[i] == 1 {
                assert_eq!(w.agents.age_group[i], 4);
            }
        }
        let (_, n) = doses_for("name='v'\n[[interventions]]\nkind='vaccination'\nstrategy='location'\ncoverage=0.2\nrate_per_day=0.1\nblock_homes=100\n");
        assert_eq!(n, 800);
    }
}
