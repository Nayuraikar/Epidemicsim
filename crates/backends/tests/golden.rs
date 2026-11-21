//! Every backend must reproduce the sequential reference exactly (stats and hash).
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

fn check(name: &str, o: &BackendOpts) {
    for vax in [0.0, 0.4] {
        let want = sim::run(&cfg(vax), &mut SequentialBackend::default());
        let mut b = make_with(name, o).unwrap();
        let got = sim::run(&cfg(vax), &mut b);
        assert_eq!(want.stats, got.stats, "{name} {o:?} vax={vax}");
        assert_eq!(want.summary.hash, got.summary.hash);
    }
}

#[test]
fn ilp_variants_match() {
    for ilp in [Ilp::Naive, Ilp::Optimised] {
        check(
            "ilp",
            &BackendOpts {
                ilp,
                ..Default::default()
            },
        );
    }
}

#[test]
fn rayon_schedules_match() {
    for schedule in [Schedule::Static, Schedule::Dynamic, Schedule::Guided] {
        for (threads, chunk) in [(1, 0), (4, 0), (4, 37), (3, 1000)] {
            let o = BackendOpts {
                threads,
                schedule,
                chunk,
                ..Default::default()
            };
            check("rayon", &o);
        }
    }
}

#[test]
fn openmp_schedules_match() {
    if !epidemicsim_backends::openmp::available() {
        eprintln!("libomp missing: skipping");
        return;
    }
    for schedule in [Schedule::Static, Schedule::Dynamic, Schedule::Guided] {
        for (threads, chunk) in [(1, 0), (4, 0), (4, 37), (3, 1000)] {
            let o = BackendOpts {
                threads,
                schedule,
                chunk,
                ..Default::default()
            };
            check("openmp", &o);
        }
    }
}

#[test]
fn simd_kernels_match() {
    for simd in [SimdKind::Scalar, SimdKind::Neon] {
        if simd.check().is_err() {
            continue;
        }
        check(
            "simd",
            &BackendOpts {
                simd,
                ..Default::default()
            },
        );
    }
}

#[cfg(target_os = "macos")]
#[test]
fn metal_variants_match() {
    if !epidemicsim_backends::mtl::available() {
        eprintln!("no Metal device: skipping");
        return;
    }
    for private in [false, true] {
        for metal_tg in [32, 64, 128, 256] {
            check(
                "metal",
                &BackendOpts {
                    metal_tg,
                    metal_private: private,
                    ..Default::default()
                },
            );
        }
    }
}
