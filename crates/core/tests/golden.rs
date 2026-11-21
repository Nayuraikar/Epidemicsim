use epidemicsim_core::{sim, Config, SequentialBackend};

fn cfg() -> Config {
    Config {
        agents: 3000,
        days: 40,
        seed: 42,
        ..Default::default()
    }
}

#[test]
fn runs_are_identical() {
    let a = sim::run(&cfg(), &mut SequentialBackend::default());
    let b = sim::run(&cfg(), &mut SequentialBackend::default());
    assert_eq!(a.stats, b.stats);
    assert_eq!(a.summary.hash, b.summary.hash);
}

#[test]
fn population_is_conserved() {
    let r = sim::run(&cfg(), &mut SequentialBackend::default());
    assert!(r.stats.iter().all(|s| s.counts.iter().sum::<u32>() == 3000));
}

/// Golden hash: update only when a model change is intentional.
#[test]
fn golden_hash() {
    let r = sim::run(&cfg(), &mut SequentialBackend::default());
    assert_eq!(r.summary.hash, "6d0d8ee04a4f99c5");
}
