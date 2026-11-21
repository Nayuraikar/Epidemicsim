//! Simulation core: agents, RNG, disease model, schedules, backends, output.
pub mod agents;
pub mod backend;
pub mod output;
pub mod params;
pub mod rng;
pub mod schedule;
pub mod sim;
pub mod synth;
pub mod world;

pub use backend::{Backend, InterventionState, SequentialBackend, StepStats};
pub use params::Config;
pub use world::World;
