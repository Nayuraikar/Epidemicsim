//! High-level run loop.
use crate::backend::{Backend, InterventionState, StepStats};
use crate::output::Summary;
use crate::params::Config;
use crate::synth;
use crate::world::{World, STEPS_PER_DAY};

pub struct RunOutput {
    pub stats: Vec<StepStats>,
    pub summary: Summary,
}

/// Generate a synthetic city and run it for `cfg.days` with no interventions.
pub fn run(cfg: &Config, backend: &mut dyn Backend) -> RunOutput {
    run_world(cfg, synth::generate(cfg), backend)
}

/// Run a prebuilt world (e.g. loaded from a city folder) for `cfg.days`.
pub fn run_world(cfg: &Config, mut world: World, backend: &mut dyn Backend) -> RunOutput {
    let iv = InterventionState::default();
    let steps = cfg.days * STEPS_PER_DAY;
    let stats: Vec<StepStats> = (0..steps).map(|_| backend.step(&mut world, &iv)).collect();
    backend.sync_to_host(&mut world);
    let summary = Summary::from_stats(cfg, backend.name(), &stats);
    RunOutput { stats, summary }
}
