//! Daily schedules: 4 slots/day (0 night, 1 morning, 2 day, 3 evening).
use crate::agents::state;
use crate::backend::InterventionState;
use crate::rng::purpose;
use crate::world::World;

pub const NIGHT: u32 = 0;
pub const MORNING: u32 = 1;
pub const DAY: u32 = 2;
pub const EVENING: u32 = 3;

/// Day 0 is a Monday.
pub fn is_weekend(day: u32) -> bool {
    day % 7 >= 5
}

/// Probability of visiting the neighbourhood market, and whether the agent
/// is at work/school in this slot.
fn slot_plan(day: u32, slot: u32, stays_home: bool) -> (f64, bool) {
    let weekend = is_weekend(day);
    match (slot, weekend) {
        (NIGHT, _) => (0.0, false),
        (MORNING, false) => (if stays_home { 0.10 } else { 0.0 }, true),
        (MORNING, true) => (0.05, false),
        (DAY, false) => (if stays_home { 0.20 } else { 0.0 }, true),
        (DAY, true) => (0.30, false),
        (_, false) => (0.20, false),
        (_, true) => (0.30, false),
    }
}

/// Location of agent `i` during this step. Dead agents must not be passed.
pub fn location_of(w: &World, i: usize, step: u32, iv: &InterventionState) -> u32 {
    let a = &w.agents;
    let home = a.home_loc[i];
    if a.state[i] == state::H {
        return w.layout.hospital_for(home);
    }
    let day = step / crate::world::STEPS_PER_DAY;
    let slot = step % crate::world::STEPS_PER_DAY;
    let work = a.work_loc[i];
    let (market_p, at_work) = slot_plan(day, slot, work == home);
    let loc = if market_p > 0.0 && w.philox.uniform(a.id[i], step, purpose::SCHEDULE) < market_p {
        w.layout.market_for(home)
    } else if at_work {
        work
    } else {
        home
    };
    if a.state[i] == state::I
        && iv.isolation_compliance > 0.0
        && w.philox.uniform(a.id[i], 0, purpose::ISOLATE) < iv.isolation_compliance
    {
        return home;
    }
    if iv.closed[w.layout.kind_of(loc) as usize]
        || (iv.colleges_closed && w.layout.colleges.contains(&loc))
    {
        home
    } else {
        loc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::Config;
    use crate::synth;
    use crate::world::LocKind;

    #[test]
    fn night_is_home_and_weekday_workers_go_to_work() {
        let cfg = Config {
            agents: 2000,
            ..Default::default()
        };
        let w = synth::generate(&cfg);
        let iv = InterventionState::default();
        let mut at_work = 0;
        for i in 0..w.agents.len() {
            assert_eq!(location_of(&w, i, NIGHT, &iv), w.agents.home_loc[i]);
            // Monday day slot, no market draw for workers.
            let l = location_of(&w, i, DAY, &iv);
            if w.layout.kind_of(l) == LocKind::Work {
                at_work += 1;
            }
        }
        assert!(at_work > 500);
    }

    #[test]
    fn closures_send_agents_home() {
        let cfg = Config {
            agents: 2000,
            ..Default::default()
        };
        let w = synth::generate(&cfg);
        let mut iv = InterventionState::default();
        iv.closed[LocKind::Work as usize] = true;
        for i in 0..w.agents.len() {
            let l = location_of(&w, i, DAY, &iv);
            assert_ne!(w.layout.kind_of(l), LocKind::Work);
        }
    }
}
