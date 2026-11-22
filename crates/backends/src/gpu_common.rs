//! Host-side maths shared by the GPU backends (Metal, CUDA). GPUs have no usable f64 `exp`
//! that is bit-identical to Rust's, so every f64 decision the CPU makes is turned into an
//! exact integer test: a uniform `u = (x + 0.5) * 2^-32` is below a probability `p` iff
//! `x < count_below(p)`. The one product that cannot be hoisted (lambda * (1 - ve) for a
//! vaccinated agent) is emulated exactly on the device (`lt_prod`). See `mtl/mod.rs`.
use crate::kernels::{dur, LamPlan};
use epidemicsim_core::backend::InterventionState;
use epidemicsim_core::params::{DiseaseParams, VaccineParams};
use epidemicsim_core::rng::to_unit;
use epidemicsim_core::world::{World, STEPS_PER_DAY};

pub const MAX_DUR: u32 = 4096;
pub const DAY_CAP: usize = 65_536;

/// Number of x in [0, 2^32) with `to_unit(x) < t` (the CPU predicate), exactly.
pub fn count_below(t: f64) -> u64 {
    if t.is_nan() || t <= 0.0 {
        return 0;
    }
    if t >= 1.0 {
        return 1 << 32;
    }
    let pred = |x: u64| to_unit(x as u32) < t;
    let mut c = ((t * 4_294_967_296.0).floor() as u64).min(1 << 32);
    while c > 0 && !pred(c - 1) {
        c -= 1;
    }
    while c < (1 << 32) && pred(c) {
        c += 1;
    }
    c
}

/// Switch points of `dur(mean4, u)`: entry k-2 is the smallest x with duration >= k.
pub fn dur_table(mean4: f64) -> Result<Vec<u32>, String> {
    if mean4 <= 0.0 {
        return Ok(Vec::new());
    }
    let f = |x: u32| u32::from(dur(mean4, to_unit(x)));
    let kmax = f(u32::MAX);
    if kmax > MAX_DUR {
        return Err(format!(
            "duration of {kmax} steps exceeds the GPU table limit"
        ));
    }
    Ok((2..=kmax)
        .map(|k| {
            let (mut lo, mut hi) = (0u32, u32::MAX);
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if f(mid) >= k {
                    hi = mid;
                } else {
                    lo = mid + 1;
                }
            }
            lo
        })
        .collect())
}

/// Kernel parameter block (`P` in the .metal / .cu sources): 8-byte fields first, 248 bytes.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Params {
    pub thr_asym: [u64; 5],
    pub thr_death: [u64; 5],
    pub mk_n: u64,
    pub hp_n: u64,
    pub nh: u64,
    pub mp_stay_thr: u64,
    pub mp_go_thr: u64,
    pub iso_thr: u64,
    pub n: u32,
    pub nloc: u32,
    pub step: u32,
    pub k0: u32,
    pub k1: u32,
    pub homes_end: u32,
    pub works_end: u32,
    pub markets_end: u32,
    pub mk_start: u32,
    pub hp_start: u32,
    pub col_lo: u32,
    pub col_hi: u32,
    pub at_work: u32,
    pub mp_stay_pos: u32,
    pub mp_go_pos: u32,
    pub iso_on: u32,
    pub closed_mask: u32,
    pub col_closed: u32,
    pub day_end: u32,
    pub vstride: u32,
    pub dcap: u32,
    pub dcnt: [u32; 4],
    pub don: [u32; 4],
    /// Metal: unused. CUDA: bit 0 = update kernels read agents through the `order` array.
    pub flags: u32,
}

/// Host-built tables that depend only on the disease / vaccine parameters.
pub struct Tables {
    pub sig: Vec<f64>,
    pub thr_asym: [u64; 5],
    pub thr_death: [u64; 5],
    pub dtab: Vec<u32>,
    pub dcnt: [u32; 4],
    pub don: [u32; 4],
    pub dcap: u32,
}

pub fn signature(d: &DiseaseParams, v: &VaccineParams) -> Vec<f64> {
    let mut s = Vec::new();
    s.extend(d.p_asymptomatic);
    s.extend(d.p_hospitalised);
    s.extend(d.p_death_hospitalised);
    s.extend([
        d.latent_days,
        d.asymptomatic_days,
        d.symptomatic_days,
        d.hospital_days,
    ]);
    s.extend([
        v.ve_infection,
        v.ve_severe,
        v.ramp_days,
        v.waning_half_life_days,
    ]);
    s
}

impl Tables {
    pub fn new(d: &DiseaseParams, v: &VaccineParams) -> Result<Self, String> {
        let s4 = f64::from(STEPS_PER_DAY);
        let means = [
            d.latent_days * s4,
            d.asymptomatic_days * s4,
            d.symptomatic_days * s4,
            d.hospital_days * s4,
        ];
        let tabs: Vec<Vec<u32>> = means
            .iter()
            .map(|&m| dur_table(m))
            .collect::<Result<_, _>>()?;
        let dcap = tabs.iter().map(Vec::len).max().unwrap_or(0).max(1);
        let mut dtab = vec![0u32; 4 * dcap];
        let mut dcnt = [0u32; 4];
        let mut don = [0u32; 4];
        for (k, t) in tabs.iter().enumerate() {
            dtab[k * dcap..k * dcap + t.len()].copy_from_slice(t);
            dcnt[k] = t.len() as u32;
            don[k] = u32::from(means[k] > 0.0);
        }
        Ok(Self {
            sig: signature(d, v),
            thr_asym: std::array::from_fn(|a| count_below(d.p_asymptomatic[a])),
            thr_death: std::array::from_fn(|a| count_below(d.p_death_hospitalised[a])),
            dtab,
            dcnt,
            don,
            dcap: dcap as u32,
        })
    }
}

/// `hosp[age][idx]` (idx 0 = unvaccinated, idx d+1 = vaccinated d days ago) as thresholds, and
/// `binf[d]` = bits of `1 - ve_infection(d)`.
pub fn vtables(w: &World, cap: usize) -> (Vec<u64>, Vec<u64>) {
    let stride = cap + 1;
    let mut hosp = vec![0u64; 5 * stride];
    let mut binf = vec![0u64; cap];
    for a in 0..5 {
        let p = w.disease.p_hospitalised[a];
        hosp[a * stride] = count_below(p * 1.0);
        for d in 0..cap {
            let b = 1.0 - w.vaccine.eff_severe(1, d as u16);
            hosp[a * stride + d + 1] = count_below(p * b);
        }
    }
    for (d, b) in binf.iter_mut().enumerate() {
        *b = (1.0 - w.vaccine.eff_infection(1, d as u16)).to_bits();
    }
    (hosp, binf)
}

/// Same table as `schedule::slot_plan` (market probability, at-work flag).
pub fn slot_plan(day: u32, slot: u32, stays_home: bool) -> (f64, bool) {
    use epidemicsim_core::schedule::{is_weekend, DAY, MORNING, NIGHT};
    let weekend = is_weekend(day);
    match (slot, weekend) {
        (NIGHT, _) => (0.0, false),
        (MORNING, false) => (if stays_home { 0.10 } else { 0.0 }, true),
        (MORNING, true) => (0.05, false),
        (DAY, false) => (if stays_home { 0.20 } else { 0.0 }, true),
        (DAY, true) => (0.30, false),
        (_, false) => (0.20, false),
        (_, true) => (0.30, false),
    }
}

pub fn build_params(
    w: &World,
    iv: &InterventionState,
    tb: &Tables,
    n: usize,
    nloc: usize,
    vcap: usize,
) -> Params {
    let step = w.step;
    let day = step / STEPS_PER_DAY;
    let slot = step % STEPS_PER_DAY;
    let (mp_stay, at_work) = slot_plan(day, slot, true);
    let (mp_go, _) = slot_plan(day, slot, false);
    let l = &w.layout;
    let mut closed_mask = 0;
    for (k, &c) in iv.closed.iter().enumerate() {
        closed_mask |= u32::from(c) << k;
    }
    Params {
        thr_asym: tb.thr_asym,
        thr_death: tb.thr_death,
        mk_n: u64::from(l.markets.end - l.markets.start),
        hp_n: u64::from(l.hospitals.end - l.hospitals.start),
        nh: u64::from(l.homes.end.max(1)),
        mp_stay_thr: count_below(mp_stay),
        mp_go_thr: count_below(mp_go),
        iso_thr: count_below(iv.isolation_compliance),
        n: n as u32,
        nloc: nloc as u32,
        step,
        k0: w.seed as u32,
        k1: (w.seed >> 32) as u32,
        homes_end: l.homes.end,
        works_end: l.works.end,
        markets_end: l.markets.end,
        mk_start: l.markets.start,
        hp_start: l.hospitals.start,
        col_lo: l.colleges.start,
        col_hi: l.colleges.end,
        at_work: u32::from(at_work),
        mp_stay_pos: u32::from(mp_stay > 0.0),
        mp_go_pos: u32::from(mp_go > 0.0),
        iso_on: u32::from(iv.isolation_compliance > 0.0),
        closed_mask,
        col_closed: u32::from(iv.colleges_closed),
        day_end: u32::from(step % STEPS_PER_DAY == STEPS_PER_DAY - 1),
        vstride: vcap as u32 + 1,
        dcap: tb.dcap,
        dcnt: tb.dcnt,
        don: tb.don,
        flags: 0,
    }
}

/// CPU lambda stage: for every active location `[loc, n_asym, n_sympt, occupancy]` compute the
/// force of infection in f64 (same expression and operand order as `kernels::lambda_chunk`) and
/// hand `(loc, count_below(lam), lam.to_bits())` to `put`. Returns the total infectious weight
/// used for R_t.
pub fn lambda_stage(
    w: &World,
    iv: &InterventionState,
    list: &[[u32; 4]],
    mut put: impl FnMut(u32, u64, u64),
) -> f64 {
    let mut beta = w.disease.beta;
    for b in &mut beta {
        *b *= iv.beta_multiplier;
    }
    let rel_a = w.disease.rel_infectiousness_asymptomatic;
    let (mut sum_a, mut sum_i) = (0u64, 0u64);
    for &[l, a, i, o] in list {
        sum_a += u64::from(a);
        sum_i += u64::from(i);
        let infectious = f64::from(a) * rel_a + f64::from(i);
        if infectious > 0.0 {
            let kind = w.layout.kind_of(l) as usize;
            let occ = f64::from(o.max(1));
            let lam = 1.0 - (-beta[kind] * infectious / occ.sqrt()).exp();
            if lam > 0.0 {
                put(l, count_below(lam), lam.to_bits());
            }
        }
    }
    LamPlan::new(w, iv).total(sum_a, sum_i)
}
