//! Backend registry: sequential reference, ILP variants, rayon, OpenMP (C) and SIMD.
// Index loops over fixed lane counts are deliberate (explicit unrolling in the kernels).
#![allow(clippy::needless_range_loop)]
#[cfg(feature = "cuda")]
pub mod cuda;
pub mod engine;
pub mod gpu_common;
pub mod kernels;
#[cfg(target_os = "macos")]
pub mod mtl;
pub mod openmp;
pub mod par;
pub mod simd;

use engine::{Engine, Variant};
use epidemicsim_core::{Backend, SequentialBackend};
pub use kernels::Ilp;
pub use par::Schedule;
use par::{RayonPar, Serial};
pub use simd::SimdKind;

pub const NAMES: [&str; 7] = [
    "sequential",
    "ilp",
    "rayon",
    "openmp",
    "simd",
    "metal",
    "cuda",
];

/// Backend selection flags (each backend uses the ones that apply to it).
#[derive(Clone, Debug)]
pub struct BackendOpts {
    pub threads: usize,
    pub ilp: Ilp,
    pub schedule: Schedule,
    /// Chunk size; 0 = the schedule's default.
    pub chunk: usize,
    pub simd: SimdKind,
    /// Metal threads per threadgroup (32, 64, 128, 256).
    pub metal_tg: usize,
    /// Metal: private (GPU-only) agent buffers instead of shared.
    pub metal_private: bool,
    /// CUDA (feature `cuda`): block size, count mode, update kernel, layout, partition period.
    pub cuda_block: u32,
    pub cuda_count: String,
    pub cuda_update: String,
    pub cuda_layout: String,
    pub cuda_partition: u32,
}

impl Default for BackendOpts {
    fn default() -> Self {
        Self {
            threads: 1,
            ilp: Ilp::Optimised,
            schedule: Schedule::Static,
            chunk: 0,
            simd: SimdKind::Scalar,
            metal_tg: 64,
            metal_private: false,
            cuda_block: 256,
            cuda_count: "atomic".into(),
            cuda_update: "branch".into(),
            cuda_layout: "soa".into(),
            cuda_partition: 0,
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
        "metal" => {
            let d = BackendOpts::default();
            if o.metal_tg == d.metal_tg && o.metal_private == d.metal_private {
                "metal".to_string()
            } else {
                let m = if o.metal_private { "private" } else { "shared" };
                format!("metal_{m}_tg{}", o.metal_tg)
            }
        }
        "cuda" => {
            let d = BackendOpts::default();
            let same = o.cuda_block == d.cuda_block
                && o.cuda_count == d.cuda_count
                && o.cuda_update == d.cuda_update
                && o.cuda_layout == d.cuda_layout
                && o.cuda_partition == d.cuda_partition;
            if same {
                "cuda".to_string()
            } else {
                format!(
                    "cuda_{}_{}_{}_b{}_p{}",
                    o.cuda_layout, o.cuda_count, o.cuda_update, o.cuda_block, o.cuda_partition
                )
            }
        }
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
        #[cfg(target_os = "macos")]
        "metal" => Ok(Box::new(mtl::MetalBackend::new(mtl::MetalOpts {
            threadgroup: o.metal_tg,
            private: o.metal_private,
        })?)),
        #[cfg(feature = "cuda")]
        "cuda" => Ok(Box::new(cuda::CudaBackend::new(cuda::CudaOpts {
            block: o.cuda_block,
            count: cuda::parse_count(&o.cuda_count)?,
            update: cuda::parse_update(&o.cuda_update)?,
            layout: cuda::parse_layout(&o.cuda_layout)?,
            partition_every: o.cuda_partition,
        })?)),
        #[cfg(not(feature = "cuda"))]
        "cuda" => Err("the cuda backend needs `--features cuda` (and an NVIDIA GPU)".into()),
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
