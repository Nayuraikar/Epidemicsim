//! Prints a pass/fail table: every Phase-4 variant vs the sequential reference
//! (identical per-step stats and run hash, with and without vaccination).
//! Run: cargo test -q -p epidemicsim-backends --release --test golden_table -- --nocapture
use epidemicsim_backends::{make_with, BackendOpts, Ilp, Schedule, SimdKind};
use epidemicsim_core::{sim, Config, SequentialBackend};

fn cfg(vax: f64) -> Config {
    let mut c = Config {
        agents: 3000,
        days: 40,
        seed: 42,
        ..Default::default()
    };
    c.synth.initial_vax_frac = vax;
    c
}

#[test]
fn golden_table() {
    let mut rows: Vec<(String, &str, BackendOpts)> = vec![
        ("sequential".into(), "sequential", BackendOpts::default()),
        (
            "ilp optimised".into(),
            "ilp",
            BackendOpts {
                ilp: Ilp::Optimised,
                ..Default::default()
            },
        ),
    ];
    for s in [Schedule::Static, Schedule::Dynamic, Schedule::Guided] {
        let o = BackendOpts {
            threads: 4,
            schedule: s,
            chunk: 37,
            ..Default::default()
        };
        rows.push((format!("rayon {} (4t)", s.name()), "rayon", o.clone()));
    }
    for s in [Schedule::Static, Schedule::Dynamic, Schedule::Guided] {
        let o = BackendOpts {
            threads: 4,
            schedule: s,
            chunk: 37,
            ..Default::default()
        };
        rows.push((format!("openmp {} (4t)", s.name()), "openmp", o));
    }
    rows.push((
        "simd neon".into(),
        "simd",
        BackendOpts {
            simd: SimdKind::Neon,
            ..Default::default()
        },
    ));
    println!("\n{:<22} {:<6} {:<6}", "variant", "novax", "vax");
    let mut all = true;
    for (label, name, o) in rows {
        let mut res = [false; 2];
        for (k, vax) in [0.0, 0.4].into_iter().enumerate() {
            let want = sim::run(&cfg(vax), &mut SequentialBackend::default());
            let got = sim::run(&cfg(vax), &mut make_with(name, &o).unwrap());
            res[k] = want.stats == got.stats && want.summary.hash == got.summary.hash;
        }
        all &= res[0] && res[1];
        let f = |b| if b { "PASS" } else { "FAIL" };
        println!("{:<22} {:<6} {:<6}", label, f(res[0]), f(res[1]));
    }
    assert!(all);
}
