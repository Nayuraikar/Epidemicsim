//! SIMD kernels for the agent update: timer/recovery and infection probability.
//!
//! * scalar reference (`timer_scalar`, `infect_scalar`)
//! * NEON, aarch64: 4-lane u32 Philox, 8-lane u16 timers (`std::arch::aarch64`)
//! * AVX2, x86_64: 8-lane u32 Philox, 16-lane u16 timers (compile-only so far: no x86 hardware
//!   was available, so these are checked for type-correctness, not for results).
//!
//! Exactness: no fast-math, no FMA. Uniforms are computed in f64 exactly as the scalar code
//! does ((x + 0.5) * 2^-32) and compared with `u < lambda * (1 - ve)`, so every backend makes
//! identical decisions.
// The vector kernels are `unsafe` only because of `target_feature`; the safety condition is
// always "the CPU supports the feature", enforced by `SimdKind::check`.
#![allow(clippy::missing_safety_doc)]
use crate::kernels::*;
use epidemicsim_core::agents::state;
use epidemicsim_core::rng::{purpose, to_unit};
use std::alloc::{alloc_zeroed, dealloc, Layout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimdKind {
    Scalar,
    Neon,
    Avx2,
}

impl SimdKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Neon => "neon",
            Self::Avx2 => "avx2",
        }
    }

    /// Err when this machine/build cannot run the kernel.
    pub fn check(self) -> Result<(), String> {
        match self {
            Self::Scalar => Ok(()),
            Self::Neon if cfg!(target_arch = "aarch64") => Ok(()),
            Self::Neon => Err("--simd neon needs an aarch64 build".into()),
            Self::Avx2 => {
                #[cfg(target_arch = "x86_64")]
                if std::is_x86_feature_detected!("avx2") {
                    return Ok(());
                }
                Err("--simd avx2 needs an x86_64 CPU with AVX2".into())
            }
        }
    }
}

// ------------------------------------------------------------------ aligned storage
/// Zero-initialised buffer aligned to 64 bytes (one cache line).
pub struct AlignedBuf<T: Copy> {
    ptr: *mut T,
    len: usize,
    layout: Layout,
}

impl<T: Copy> AlignedBuf<T> {
    pub fn new(len: usize) -> Self {
        let layout = Layout::from_size_align(len.max(1) * std::mem::size_of::<T>(), 64).unwrap();
        // SAFETY: non-zero size, valid layout; zeroed bytes are a valid value for the integer
        // element types this is used with.
        let ptr = unsafe { alloc_zeroed(layout) } as *mut T;
        assert!(!ptr.is_null());
        Self { ptr, len, layout }
    }
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: ptr is valid for len elements.
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: as above, and we hold &mut self.
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl<T: Copy> Drop for AlignedBuf<T> {
    fn drop(&mut self) {
        // SAFETY: allocated with this layout in `new`.
        unsafe { dealloc(self.ptr as *mut u8, self.layout) }
    }
}

// ------------------------------------------------------------------ scalar reference
/// Decrement timers of infected agents (states E, A, I, H) and list those that reached 0.
pub fn timer_scalar(st: &[u8], timer: &mut [u16], trans: &mut Vec<u32>) {
    for i in 0..st.len() {
        if st[i].wrapping_sub(1) < 4 {
            let t = timer[i].saturating_sub(1);
            timer[i] = t;
            if t == 0 {
                trans.push(i as u32);
            }
        }
    }
}

/// Infection decision for candidate agents; hits carry (index, second uniform).
pub fn infect_scalar(p: &UpdPlan, c: &Chunk, cands: &[u32], hits: &mut Vec<(u32, f64)>) {
    for &i in cands {
        let i = i as usize;
        let lam = p.lambda[c.loc[i] as usize];
        let ve = p.t.ve_inf(p.v, c.vstat[i], c.vdays[i]);
        let b = p.philox.block(c.id[i], p.step, purpose::INFECT);
        if to_unit(b[0]) < lam * (1.0 - ve) {
            hits.push((i as u32, to_unit(b[1])));
        }
    }
}

/// Stream kernel used by the alignment benchmark: first Philox word for each id.
pub fn stream_scalar(ids: &[u32], step: u32, key: [u32; 2], out: &mut [u32]) {
    for (o, &id) in out.iter_mut().zip(ids) {
        *o = epidemicsim_core::rng::philox4x32_10([id, step, purpose::INFECT, 0], key)[0];
    }
}

// ------------------------------------------------------------------ NEON (aarch64)
#[cfg(target_arch = "aarch64")]
pub mod neon {
    use super::*;
    use std::arch::aarch64::*;

    const M0: u32 = 0xD251_1F53;
    const M1: u32 = 0xCD9E_8D57;
    const W0: u32 = 0x9E37_79B9;
    const W1: u32 = 0xBB67_AE85;

    /// 32x32 -> (hi, lo) for four lanes: two `umull` on the low/high halves.
    #[inline]
    #[target_feature(enable = "neon")]
    unsafe fn mulhilo(m: uint32x4_t, a: uint32x4_t) -> (uint32x4_t, uint32x4_t) {
        let pl = vmull_u32(vget_low_u32(m), vget_low_u32(a));
        let ph = vmull_high_u32(m, a);
        (
            vcombine_u32(vshrn_n_u64::<32>(pl), vshrn_n_u64::<32>(ph)),
            vcombine_u32(vmovn_u64(pl), vmovn_u64(ph)),
        )
    }

    /// Philox4x32-10 on four independent counters (one per lane); returns the 4 output words.
    #[target_feature(enable = "neon")]
    pub unsafe fn philox4(
        mut c0: uint32x4_t,
        mut c1: uint32x4_t,
        mut c2: uint32x4_t,
        mut c3: uint32x4_t,
        key: [u32; 2],
    ) -> [uint32x4_t; 4] {
        let (mut k0, mut k1) = (key[0], key[1]);
        let (m0, m1) = (vdupq_n_u32(M0), vdupq_n_u32(M1));
        for r in 0..10 {
            if r > 0 {
                k0 = k0.wrapping_add(W0);
                k1 = k1.wrapping_add(W1);
            }
            let (hi0, lo0) = mulhilo(m0, c0);
            let (hi1, lo1) = mulhilo(m1, c2);
            let n0 = veorq_u32(veorq_u32(hi1, c1), vdupq_n_u32(k0));
            let n2 = veorq_u32(veorq_u32(hi0, c3), vdupq_n_u32(k1));
            c0 = n0;
            c1 = lo1;
            c2 = n2;
            c3 = lo0;
        }
        [c0, c1, c2, c3]
    }

    /// 8 states / 8 timers per iteration; saturating decrement under an "infected" mask.
    #[target_feature(enable = "neon")]
    pub unsafe fn timer(st: &[u8], timer: &mut [u16], trans: &mut Vec<u32>) {
        let n = st.len();
        let n8 = n & !7;
        let (one8, three8) = (vdup_n_u8(1), vdup_n_u8(3));
        let (one16, zero16) = (vdupq_n_u16(1), vdupq_n_u16(0));
        let mut i = 0;
        while i < n8 {
            let s = vld1_u8(st.as_ptr().add(i));
            // (state - 1) <= 3  <=>  state in {E,A,I,H}; state 0 wraps to 255 and fails.
            let act8 = vcle_u8(vsub_u8(s, one8), three8);
            let act = vreinterpretq_u16_s16(vmovl_s8(vreinterpret_s8_u8(act8)));
            let tm = vld1q_u16(timer.as_ptr().add(i));
            let t = vqsubq_u16(tm, one16);
            vst1q_u16(timer.as_mut_ptr().add(i), vbslq_u16(act, t, tm));
            let done = vandq_u16(act, vceqq_u16(t, zero16));
            if vmaxvq_u16(done) != 0 {
                let mut m = [0u16; 8];
                vst1q_u16(m.as_mut_ptr(), done);
                for (k, &x) in m.iter().enumerate() {
                    if x != 0 {
                        trans.push((i + k) as u32);
                    }
                }
            }
            i += 8;
        }
        for j in n8..n {
            if st[j].wrapping_sub(1) < 4 {
                let t = timer[j].saturating_sub(1);
                timer[j] = t;
                if t == 0 {
                    trans.push(j as u32);
                }
            }
        }
    }

    /// u32 lanes -> two f64x2 uniforms in (0,1): (x + 0.5) * 2^-32, exactly as `to_unit`.
    #[inline]
    #[target_feature(enable = "neon")]
    unsafe fn unit(x: uint32x4_t) -> (float64x2_t, float64x2_t) {
        let (h, s) = (vdupq_n_f64(0.5), vdupq_n_f64(1.0 / 4_294_967_296.0));
        let lo = vcvtq_f64_u64(vmovl_u32(vget_low_u32(x)));
        let hi = vcvtq_f64_u64(vmovl_high_u32(x));
        (
            vmulq_f64(vaddq_f64(lo, h), s),
            vmulq_f64(vaddq_f64(hi, h), s),
        )
    }

    /// Four candidates per iteration: vector Philox + f64 compare; scalar gather of lambda/ve.
    #[target_feature(enable = "neon")]
    pub unsafe fn infect(p: &UpdPlan, c: &Chunk, cands: &[u32], hits: &mut Vec<(u32, f64)>) {
        let n4 = cands.len() & !3;
        let mut g = 0;
        while g < n4 {
            let ix = [
                cands[g] as usize,
                cands[g + 1] as usize,
                cands[g + 2] as usize,
                cands[g + 3] as usize,
            ];
            let ids = [c.id[ix[0]], c.id[ix[1]], c.id[ix[2]], c.id[ix[3]]];
            let mut peff = [0.0f64; 4];
            for k in 0..4 {
                let lam = p.lambda[c.loc[ix[k]] as usize];
                let ve = p.t.ve_inf(p.v, c.vstat[ix[k]], c.vdays[ix[k]]);
                peff[k] = lam * (1.0 - ve);
            }
            let r = philox4(
                vld1q_u32(ids.as_ptr()),
                vdupq_n_u32(p.step),
                vdupq_n_u32(purpose::INFECT),
                vdupq_n_u32(0),
                p.key,
            );
            let (ulo, uhi) = unit(r[0]);
            let mlo = vcltq_f64(ulo, vld1q_f64(peff.as_ptr()));
            let mhi = vcltq_f64(uhi, vld1q_f64(peff.as_ptr().add(2)));
            let m = [
                vgetq_lane_u64::<0>(mlo),
                vgetq_lane_u64::<1>(mlo),
                vgetq_lane_u64::<0>(mhi),
                vgetq_lane_u64::<1>(mhi),
            ];
            if (m[0] | m[1] | m[2] | m[3]) != 0 {
                let mut w1 = [0u32; 4];
                vst1q_u32(w1.as_mut_ptr(), r[1]);
                for k in 0..4 {
                    if m[k] != 0 {
                        hits.push((ix[k] as u32, to_unit(w1[k])));
                    }
                }
            }
            g += 4;
        }
        infect_scalar(p, c, &cands[n4..], hits);
    }

    /// Alignment-benchmark stream kernel: 4 Philox blocks per iteration, load/store of u32x4.
    #[target_feature(enable = "neon")]
    pub unsafe fn stream(ids: &[u32], step: u32, key: [u32; 2], out: &mut [u32]) {
        let n4 = ids.len() & !3;
        let mut i = 0;
        while i < n4 {
            let r = philox4(
                vld1q_u32(ids.as_ptr().add(i)),
                vdupq_n_u32(step),
                vdupq_n_u32(purpose::INFECT),
                vdupq_n_u32(0),
                key,
            );
            vst1q_u32(out.as_mut_ptr().add(i), r[0]);
            i += 4;
        }
        stream_scalar(&ids[n4..], step, key, &mut out[n4..]);
    }
}

// ------------------------------------------------------------------ AVX2 (x86_64, compile-only)
#[cfg(target_arch = "x86_64")]
pub mod avx2 {
    use super::*;
    use std::arch::x86_64::*;

    const M0: u32 = 0xD251_1F53;
    const M1: u32 = 0xCD9E_8D57;
    const W0: u32 = 0x9E37_79B9;
    const W1: u32 = 0xBB67_AE85;

    /// 32x32 -> (hi, lo) for eight lanes: `mul_epu32` multiplies even lanes only, so the odd
    /// lanes are shifted down, multiplied, and the halves re-interleaved with blends.
    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn mulhilo(m: u32, a: __m256i) -> (__m256i, __m256i) {
        let mv = _mm256_set1_epi32(m as i32);
        let even = _mm256_mul_epu32(a, mv);
        let odd = _mm256_mul_epu32(_mm256_srli_epi64::<32>(a), mv);
        let lo = _mm256_blend_epi32::<0xAA>(even, _mm256_slli_epi64::<32>(odd));
        let hi = _mm256_blend_epi32::<0xAA>(_mm256_srli_epi64::<32>(even), odd);
        (hi, lo)
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn philox8(
        mut c0: __m256i,
        mut c1: __m256i,
        mut c2: __m256i,
        mut c3: __m256i,
        key: [u32; 2],
    ) -> [__m256i; 4] {
        let (mut k0, mut k1) = (key[0], key[1]);
        for r in 0..10 {
            if r > 0 {
                k0 = k0.wrapping_add(W0);
                k1 = k1.wrapping_add(W1);
            }
            let (hi0, lo0) = mulhilo(M0, c0);
            let (hi1, lo1) = mulhilo(M1, c2);
            let n0 = _mm256_xor_si256(_mm256_xor_si256(hi1, c1), _mm256_set1_epi32(k0 as i32));
            let n2 = _mm256_xor_si256(_mm256_xor_si256(hi0, c3), _mm256_set1_epi32(k1 as i32));
            c0 = n0;
            c1 = lo1;
            c2 = n2;
            c3 = lo0;
        }
        [c0, c1, c2, c3]
    }

    /// 16 states / 16 timers per iteration.
    #[target_feature(enable = "avx2")]
    pub unsafe fn timer(st: &[u8], timer: &mut [u16], trans: &mut Vec<u32>) {
        let n = st.len();
        let n16 = n & !15;
        let (one8, three8) = (_mm_set1_epi8(1), _mm_set1_epi8(3));
        let (one16, zero16) = (_mm256_set1_epi16(1), _mm256_setzero_si256());
        let mut i = 0;
        while i < n16 {
            let s = _mm_loadu_si128(st.as_ptr().add(i) as *const __m128i);
            let d = _mm_sub_epi8(s, one8);
            // d <= 3 (unsigned)  <=>  min(d, 3) == d
            let act8 = _mm_cmpeq_epi8(_mm_min_epu8(d, three8), d);
            let act = _mm256_cvtepi8_epi16(act8);
            let tp = timer.as_mut_ptr().add(i) as *mut __m256i;
            let tm = _mm256_loadu_si256(tp);
            let t = _mm256_subs_epu16(tm, one16);
            _mm256_storeu_si256(tp, _mm256_blendv_epi8(tm, t, act));
            let done = _mm256_and_si256(act, _mm256_cmpeq_epi16(t, zero16));
            if _mm256_movemask_epi8(done) != 0 {
                let mut m = [0u16; 16];
                _mm256_storeu_si256(m.as_mut_ptr() as *mut __m256i, done);
                for (k, &x) in m.iter().enumerate() {
                    if x != 0 {
                        trans.push((i + k) as u32);
                    }
                }
            }
            i += 16;
        }
        for j in n16..n {
            if st[j].wrapping_sub(1) < 4 {
                let t = timer[j].saturating_sub(1);
                timer[j] = t;
                if t == 0 {
                    trans.push(j as u32);
                }
            }
        }
    }

    /// u32 -> f64 exactly (AVX2 has only signed int->double): flip the sign bit, convert, add 2^31.
    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn unit4(x: __m128i) -> __m256d {
        let flip = _mm_xor_si128(x, _mm_set1_epi32(i32::MIN));
        let f = _mm256_add_pd(_mm256_cvtepi32_pd(flip), _mm256_set1_pd(2_147_483_648.0));
        _mm256_mul_pd(
            _mm256_add_pd(f, _mm256_set1_pd(0.5)),
            _mm256_set1_pd(1.0 / 4_294_967_296.0),
        )
    }

    /// Eight candidates per iteration.
    #[target_feature(enable = "avx2")]
    pub unsafe fn infect(p: &UpdPlan, c: &Chunk, cands: &[u32], hits: &mut Vec<(u32, f64)>) {
        let n8 = cands.len() & !7;
        let mut g = 0;
        while g < n8 {
            let mut ix = [0usize; 8];
            let mut ids = [0u32; 8];
            let mut peff = [0.0f64; 8];
            for k in 0..8 {
                ix[k] = cands[g + k] as usize;
                ids[k] = c.id[ix[k]];
                let lam = p.lambda[c.loc[ix[k]] as usize];
                let ve = p.t.ve_inf(p.v, c.vstat[ix[k]], c.vdays[ix[k]]);
                peff[k] = lam * (1.0 - ve);
            }
            let r = philox8(
                _mm256_loadu_si256(ids.as_ptr() as *const __m256i),
                _mm256_set1_epi32(p.step as i32),
                _mm256_set1_epi32(purpose::INFECT as i32),
                _mm256_setzero_si256(),
                p.key,
            );
            let u_lo = unit4(_mm256_castsi256_si128(r[0]));
            let u_hi = unit4(_mm256_extracti128_si256::<1>(r[0]));
            let m_lo = _mm256_movemask_pd(_mm256_cmp_pd::<_CMP_LT_OQ>(
                u_lo,
                _mm256_loadu_pd(peff.as_ptr()),
            ));
            let m_hi = _mm256_movemask_pd(_mm256_cmp_pd::<_CMP_LT_OQ>(
                u_hi,
                _mm256_loadu_pd(peff.as_ptr().add(4)),
            ));
            let m = (m_lo | (m_hi << 4)) as u32;
            if m != 0 {
                let mut w1 = [0u32; 8];
                _mm256_storeu_si256(w1.as_mut_ptr() as *mut __m256i, r[1]);
                for k in 0..8 {
                    if m >> k & 1 != 0 {
                        hits.push((ix[k] as u32, to_unit(w1[k])));
                    }
                }
            }
            g += 8;
        }
        infect_scalar(p, c, &cands[n8..], hits);
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn stream(ids: &[u32], step: u32, key: [u32; 2], out: &mut [u32]) {
        let n8 = ids.len() & !7;
        let mut i = 0;
        while i < n8 {
            let r = philox8(
                _mm256_loadu_si256(ids.as_ptr().add(i) as *const __m256i),
                _mm256_set1_epi32(step as i32),
                _mm256_set1_epi32(purpose::INFECT as i32),
                _mm256_setzero_si256(),
                key,
            );
            _mm256_storeu_si256(out.as_mut_ptr().add(i) as *mut __m256i, r[0]);
            i += 8;
        }
        stream_scalar(&ids[n8..], step, key, &mut out[n8..]);
    }
}

// ------------------------------------------------------------------ dispatch
fn timer_k(k: SimdKind, st: &[u8], tm: &mut [u16], trans: &mut Vec<u32>) {
    match k {
        // SAFETY (all arms): `check()` verified the instruction set before the backend was built.
        #[cfg(target_arch = "aarch64")]
        SimdKind::Neon => unsafe { neon::timer(st, tm, trans) },
        #[cfg(target_arch = "x86_64")]
        SimdKind::Avx2 => unsafe { avx2::timer(st, tm, trans) },
        _ => timer_scalar(st, tm, trans),
    }
}

fn infect_k(k: SimdKind, p: &UpdPlan, c: &Chunk, cands: &[u32], hits: &mut Vec<(u32, f64)>) {
    match k {
        #[cfg(target_arch = "aarch64")]
        SimdKind::Neon => unsafe { neon::infect(p, c, cands, hits) },
        #[cfg(target_arch = "x86_64")]
        SimdKind::Avx2 => unsafe { avx2::infect(p, c, cands, hits) },
        _ => infect_scalar(p, c, cands, hits),
    }
}

/// Agent update built from the two SIMD kernels (scalar for `SimdKind::Scalar`):
/// 1. timer kernel: decrement infected agents' timers, list those that expired;
/// 2. candidate scan: susceptible agents at a location with force of infection > 0;
/// 3. infection kernel over the candidates;
/// 4. scalar progression of expired agents, apply infections, vaccine clock + counts.
pub fn update(kind: SimdKind, p: &UpdPlan, c: &mut Chunk) -> UpdResult {
    let n = c.state.len();
    let mut trans: Vec<u32> = Vec::new();
    let mut cands: Vec<u32> = Vec::new();
    let mut hits: Vec<(u32, f64)> = Vec::new();
    timer_k(kind, c.state, c.timer, &mut trans);
    for i in 0..n {
        if c.state[i] == state::S && p.lambda[c.loc[i] as usize] > 0.0 {
            cands.push(i as u32);
        }
    }
    infect_k(kind, p, c, &cands, &mut hits);
    for &i in &trans {
        let i = i as usize;
        if c.state[i] == state::A {
            c.state[i] = state::R;
            c.timer[i] = 0;
        } else {
            let b = p.philox.block(c.id[i], p.step, purpose::TRANSITION);
            apply_trans(p, c, i, to_unit(b[0]), to_unit(b[1]));
        }
    }
    for &(i, u1) in &hits {
        c.state[i as usize] = state::E;
        c.timer[i as usize] = dur(p.t.latent4, u1);
    }
    let counts = finish_chunk(p, c);
    (counts, hits.len() as u32)
}

/// Timer kernel by kind (used by the alignment benchmark).
pub fn timer_dispatch(k: SimdKind, st: &[u8], tm: &mut [u16], trans: &mut Vec<u32>) {
    timer_k(k, st, tm, trans)
}

/// Philox stream kernel by kind (used by the alignment benchmark).
pub fn stream_dispatch(k: SimdKind, ids: &[u32], step: u32, key: [u32; 2], out: &mut [u32]) {
    match k {
        // SAFETY: `check()` verified the instruction set.
        #[cfg(target_arch = "aarch64")]
        SimdKind::Neon => unsafe { neon::stream(ids, step, key, out) },
        #[cfg(target_arch = "x86_64")]
        SimdKind::Avx2 => unsafe { avx2::stream(ids, step, key, out) },
        _ => stream_scalar(ids, step, key, out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use epidemicsim_core::rng::philox4x32_10;

    fn lcg(s: &mut u64) -> u32 {
        *s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*s >> 32) as u32
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn neon_philox_matches_scalar_lane_by_lane() {
        use std::arch::aarch64::*;
        let mut s = 7u64;
        for _ in 0..200 {
            let key = [lcg(&mut s), lcg(&mut s)];
            let c: [[u32; 4]; 4] = std::array::from_fn(|_| std::array::from_fn(|_| lcg(&mut s)));
            // SAFETY: NEON is baseline on aarch64.
            let r = unsafe {
                let r = neon::philox4(
                    vld1q_u32(c[0].as_ptr()),
                    vld1q_u32(c[1].as_ptr()),
                    vld1q_u32(c[2].as_ptr()),
                    vld1q_u32(c[3].as_ptr()),
                    key,
                );
                let mut o = [[0u32; 4]; 4];
                for w in 0..4 {
                    vst1q_u32(o[w].as_mut_ptr(), r[w]);
                }
                o
            };
            for l in 0..4 {
                let want = philox4x32_10([c[0][l], c[1][l], c[2][l], c[3][l]], key);
                assert_eq!([r[0][l], r[1][l], r[2][l], r[3][l]], want, "lane {l}");
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn neon_timer_and_stream_match_scalar() {
        let mut s = 99u64;
        let n = 1003; // not a multiple of the lane count: exercises the tails
        let st: Vec<u8> = (0..n).map(|_| (lcg(&mut s) % 7) as u8).collect();
        let tm: Vec<u16> = (0..n).map(|_| (lcg(&mut s) % 3) as u16).collect();
        let (mut t1, mut t2) = (tm.clone(), tm);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        timer_scalar(&st, &mut t1, &mut a);
        unsafe { neon::timer(&st, &mut t2, &mut b) };
        assert_eq!((t1, a), (t2, b));
        let ids: Vec<u32> = (0..n as u32).collect();
        let (mut o1, mut o2) = (vec![0; n], vec![0; n]);
        stream_scalar(&ids, 5, [1, 2], &mut o1);
        unsafe { neon::stream(&ids, 5, [1, 2], &mut o2) };
        assert_eq!(o1, o2);
    }
}
