//! Backend registry: sequential reference, ILP variants, rayon, OpenMP (C) and SIMD.
// Index loops over fixed lane counts are deliberate (explicit unrolling in the kernels).
#![allow(clippy::needless_range_loop)]
pub mod engine;
pub mod kernels;
pub mod openmp;
pub mod par;
pub mod simd;

use engine::{Engine, Variant};
use epidemicsim_core::{Backend, SequentialBackend};
pub use kernels::Ilp;
pub use par::Schedule;
use par::{RayonPar, Serial};
pub use simd::SimdKind;

pub const NAMES: [&str; 5] = ["sequential", "ilp", "rayon", "openmp", "simd"];

/// Backend selection flags (each backend uses the ones that apply to it).
#[derive(Clone, Debug)]
pub struct BackendOpts {
    pub threads: usize,
    pub ilp: Ilp,
    pub schedule: Schedule,
    /// Chunk size; 0 = the schedule's default.
    pub chunk: usize,
    pub simd: SimdKind,
}

impl Default for BackendOpts {
    fn default() -> Self {
        Self {
            threads: 1,
            ilp: Ilp::Optimised,
            schedule: Schedule::Static,
            chunk: 0,
            simd: SimdKind::Scalar,
        }
    }
}

pub fn parse_ilp(s: &str) -> Result<Ilp, String> {
    match s {
        "naive" => Ok(Ilp::Naive),
        "optimised" | "optimized" => Ok(Ilp::Optimised),
        _ => Err(format!("unknown --ilp '{s}' (naive|optimised)")),
    }
}

pub fn parse_simd(s: &str) -> Result<SimdKind, String> {
    match s {
        "scalar" => Ok(SimdKind::Scalar),
        "neon" => Ok(SimdKind::Neon),
        "avx2" => Ok(SimdKind::Avx2),
        _ => Err(format!("unknown --simd '{s}' (scalar|neon|avx2)")),
    }
}

/// Short tag used in report file names, e.g. `rayon_dynamic_c1024`.
pub fn label(name: &str, o: &BackendOpts) -> String {
    let ilp = match o.ilp {
        Ilp::Naive => "naive",
        Ilp::Optimised => "optimised",
    };
    match name {
        "ilp" => format!("ilp_{ilp}"),
        "rayon" | "openmp" => {
            let c = if o.chunk == 0 {
                o.schedule.default_chunk()
            } else {
                o.chunk
            };
            format!("{name}_{}_c{c}", o.schedule.name())
        }
        "simd" => format!("simd_{}", o.simd.name()),
        _ => name.to_string(),
    }
}

/// Create a backend by name. `threads` is ignored by backends that are not multi-threaded.
pub fn make(name: &str, threads: usize) -> Result<Box<dyn Backend + Send>, String> {
    make_with(
        name,
        &BackendOpts {
            threads,
            ..Default::default()
        },
    )
}

pub fn make_with(name: &str, o: &BackendOpts) -> Result<Box<dyn Backend + Send>, String> {
    let variant = |ilp: Ilp| match ilp {
        Ilp::Naive => Variant::Naive,
        Ilp::Optimised => Variant::Optimised,
    };
    match name {
        "sequential" => Ok(Box::new(SequentialBackend::default())),
        "ilp" => {
            let n = if o.ilp == Ilp::Naive {
                "ilp-naive"
            } else {
                "ilp-optimised"
            };
            Ok(Box::new(Engine::new(Serial, variant(o.ilp), n)))
        }
        "rayon" => Ok(Box::new(Engine::new(
            RayonPar::new(o.threads, o.schedule, o.chunk)?,
            variant(o.ilp),
            "rayon",
        ))),
        "openmp" => Ok(Box::new(openmp::OpenMpBackend::new(
            o.threads, o.schedule, o.chunk,
        )?)),
        "simd" => {
            o.simd.check()?;
            Ok(Box::new(Engine::new(Serial, Variant::Simd(o.simd), "simd")))
        }
        _ => Err(format!(
            "unknown or not-yet-implemented backend '{name}' (available: {})",
            NAMES.join(", ")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry() {
        assert_eq!(make("sequential", 1).unwrap().name(), "sequential");
        assert!(make("cuda", 1).is_err());
    }
}
