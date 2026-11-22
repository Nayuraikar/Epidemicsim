//! FFI to the C kernels (OpenMP today; the CUDA folder is a placeholder).
//! `HAS_OPENMP` is false when libomp was not found at build time; the wrappers then panic.

/// True when the OpenMP kernels were compiled and linked.
pub const HAS_OPENMP: bool = cfg!(has_openmp);

pub const MAX_THREADS: usize = 64;
pub const PHASES: usize = 3;

/// Mirror of `EpCtx` in `openmp/ep.h`. Field order and types must match exactly.
#[repr(C)]
pub struct EpCtx {
    pub n: i64,
    pub nloc: i64,
    pub id: *const u32,
    pub age: *const u8,
    pub state: *mut u8,
    pub timer: *mut u16,
    pub home: *const u32,
    pub work: *const u32,
    pub vstat: *const u8,
    pub vdays: *mut u16,
    pub loc: *mut u32,
    pub occ: *mut u32,
    pub inf_a: *mut u32,
    pub inf_i: *mut u32,
    pub lambda: *mut f64,
    pub ve_inf_lut: *const f64,
    pub ve_sev_lut: *const f64,
    pub lut_n: i64,
    pub work_s: *mut f64,
    pub items: *mut i64,
    pub chunk: i64,
    pub nh: u64,
    pub mk_n: u64,
    pub hp_n: u64,
    pub sum_a: u64,
    pub sum_i: u64,
    pub beta: [f64; 4],
    pub rel_a: f64,
    pub p_asym: [f64; 5],
    pub p_hosp: [f64; 5],
    pub p_death: [f64; 5],
    pub latent_days: f64,
    pub asym_days: f64,
    pub sym_days: f64,
    pub hosp_days: f64,
    pub ve_inf: f64,
    pub ve_sev: f64,
    pub ramp_days: f64,
    pub half_life: f64,
    pub iso: f64,
    pub step: u32,
    pub day: u32,
    pub slot: u32,
    pub key0: u32,
    pub key1: u32,
    pub day_end: u32,
    pub homes_end: u32,
    pub works_end: u32,
    pub markets_end: u32,
    pub mk_start: u32,
    pub hp_start: u32,
    pub col_lo: u32,
    pub col_hi: u32,
    /// 0 static, 1 dynamic, 2 guided.
    pub sched: i32,
    pub nthreads: i32,
    pub counts: [u32; 7],
    pub new_inf: u32,
    pub closed: [u8; 4],
    pub col_closed: u8,
    pub pad: [u8; 3],
}

#[cfg(has_openmp)]
mod ffi {
    use super::EpCtx;
    extern "C" {
        pub fn ep_ctx_size() -> usize;
        pub fn ep_omp_max_threads() -> i32;
        pub fn ep_step(c: *mut EpCtx);
        pub fn ep_race(
            method: i32,
            nthreads: i32,
            n: i64,
            nloc: i64,
            loc: *const u32,
            infectious: *const u8,
            chunk: i64,
            counts: *mut u32,
        );
    }
}

#[cfg(has_openmp)]
pub use ffi::{ep_omp_max_threads, ep_race, ep_step};

/// Panics if the Rust mirror and the C struct disagree in size.
#[cfg(has_openmp)]
pub fn check_layout() {
    // SAFETY: pure function returning sizeof(EpCtx).
    let c = unsafe { ffi::ep_ctx_size() };
    assert_eq!(c, std::mem::size_of::<EpCtx>(), "EpCtx layout mismatch");
}

#[cfg(not(has_openmp))]
pub fn check_layout() {}

/// # Safety
/// `c` must describe valid, correctly sized buffers (see `openmp/ep.h`).
#[cfg(not(has_openmp))]
pub unsafe fn ep_step(_c: *mut EpCtx) {
    panic!("OpenMP kernels not built (libomp missing)");
}

/// # Safety
/// Buffers must be valid for `n`/`nloc` elements.
#[cfg(not(has_openmp))]
#[allow(clippy::too_many_arguments)]
pub unsafe fn ep_race(
    _: i32,
    _: i32,
    _: i64,
    _: i64,
    _: *const u32,
    _: *const u8,
    _: i64,
    _: *mut u32,
) {
    panic!("OpenMP kernels not built (libomp missing)");
}

#[cfg(not(has_openmp))]
pub fn ep_omp_max_threads() -> i32 {
    1
}

#[cfg(test)]
mod tests {
    #[test]
    fn layout_matches_c() {
        super::check_layout();
    }
}
