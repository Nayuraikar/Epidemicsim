//! Builds the C kernels. OpenMP needs libomp on macOS (Homebrew: `brew install libomp`); if it
//! is missing we warn and skip the C kernels instead of failing the build.
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=openmp");
    println!("cargo:rerun-if-changed=cuda");
    println!("cargo:rerun-if-env-changed=LIBOMP_PREFIX");
    println!("cargo:rustc-check-cfg=cfg(has_openmp)");

    let mac = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|o| o == "macos");
    let prefix =
        std::env::var("LIBOMP_PREFIX").unwrap_or_else(|_| "/opt/homebrew/opt/libomp".into());
    let (inc, lib) = (format!("{prefix}/include"), format!("{prefix}/lib"));

    let mut b = cc::Build::new();
    b.file("openmp/kernels.c")
        .include("openmp")
        .opt_level(3)
        // No fused multiply-add and no fast-math: doubles must round exactly like Rust's.
        .flag("-ffp-contract=off")
        .warnings(true);
    if mac {
        if !Path::new(&inc).join("omp.h").exists() || !Path::new(&lib).join("libomp.dylib").exists()
        {
            println!(
                "cargo:warning=libomp not found under {prefix} (brew install libomp); skipping OpenMP kernels"
            );
            return;
        }
        // Apple clang has no built-in OpenMP: preprocess with -fopenmp and link libomp.
        b.include(&inc).flag("-Xpreprocessor").flag("-fopenmp");
        println!("cargo:rustc-link-search=native={lib}");
        println!("cargo:rustc-link-lib=omp");
    } else {
        b.flag("-fopenmp");
        println!("cargo:rustc-link-lib=gomp");
    }
    b.compile("epidemicsim_openmp");
    println!("cargo:rustc-cfg=has_openmp");
}
