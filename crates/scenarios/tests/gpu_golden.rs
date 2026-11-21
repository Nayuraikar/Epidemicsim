//! Every scenario file (masks, vaccination, isolation, closures, lockdown, combo) must give
//! identical per-step stats on the Metal backend and the sequential reference.
use epidemicsim_core::{Config, SequentialBackend};
use epidemicsim_scenarios::{ensemble::run_once, load_all, EnsembleOpts, WorldSource};

#[cfg(target_os = "macos")]
#[test]
fn metal_matches_sequential_on_all_scenarios() {
    if !epidemicsim_backends::mtl::available() {
        return;
    }
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios");
    let src = WorldSource::Synth(Box::new(Config {
        agents: 6000,
        ..Default::default()
    }));
    let opts = EnsembleOpts {
        runs: 1,
        days: 60,
        base_seed: 11,
        hospital_capacity: 20,
    };
    for s in load_all(&dir).unwrap() {
        let (want, _) = run_once(&src, &s, 11, &opts, &mut SequentialBackend::default());
        let mut m = epidemicsim_backends::make("metal", 1).unwrap();
        let (got, _) = run_once(&src, &s, 11, &opts, &mut m);
        assert_eq!(want, got, "scenario {}", s.name);
    }
}
