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
    for simd in [SimdKind::Scalar, SimdKind::Neon, SimdKind::Avx2] {
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

/// Runs only on x86_64; fails (instead of skipping) when `EPI_REQUIRE_AVX2=1` and the CPU lacks AVX2.
#[cfg(target_arch = "x86_64")]
#[test]
fn avx2_matches_on_x86_64() {
    if !std::is_x86_feature_detected!("avx2") {
        assert!(
            std::env::var("EPI_REQUIRE_AVX2").is_err(),
            "EPI_REQUIRE_AVX2 set but this CPU has no AVX2"
        );
        eprintln!("no AVX2: skipping");
        return;
    }
    check(
        "simd",
        &BackendOpts {
            simd: SimdKind::Avx2,
            ..Default::default()
        },
    );
}

/// CUDA golden test (feature `cuda`): every kernel variant and block size must reproduce the
/// sequential reference bit for bit. With `EPI_REQUIRE_CUDA=1` a missing GPU is a failure.
#[cfg(feature = "cuda")]
#[test]
fn cuda_variants_match() {
    if !epidemicsim_backends::cuda::available() {
        assert!(
            std::env::var("EPI_REQUIRE_CUDA").is_err(),
            "EPI_REQUIRE_CUDA set but no CUDA device/driver"
        );
        eprintln!("no CUDA device: skipping");
        return;
    }
    let o = |layout: &str, count: &str, update: &str, block: u32, part: u32| BackendOpts {
        cuda_layout: layout.into(),
        cuda_count: count.into(),
        cuda_update: update.into(),
        cuda_block: block,
        cuda_partition: part,
        ..Default::default()
    };
    for block in [64, 128, 256, 512] {
        check("cuda", &o("soa", "atomic", "branch", block, 0));
    }
    for count in ["atomic", "segmented"] {
        for update in ["branch", "pred", "smem", "pairs"] {
            check("cuda", &o("soa", count, update, 128, 0));
        }
    }
    for update in ["branch", "pred"] {
        check("cuda", &o("aos", "atomic", update, 256, 0));
    }
    for update in ["branch", "pred", "smem"] {
        for part in [1, 3] {
            check("cuda", &o("soa", "segmented", update, 128, part));
        }
    }
}
