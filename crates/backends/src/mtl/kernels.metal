// Metal kernels for the epidemic step. Every comparison that the CPU does in f64 is turned
// into an exact integer test (see mod.rs): a uniform u = (x + 0.5) * 2^-32 is below a
// probability p  <=>  x < count_below(p), with the threshold computed on the host in f64.
// The one product that cannot be hoisted (lambda * (1 - ve) for vaccinated agents) is
// evaluated here by `lt_prod`, an exact emulation of the f64 multiply with 64-bit integers.
#include <metal_stdlib>
using namespace metal;

constant uint M0 = 0xD2511F53u;
constant uint M1 = 0xCD9E8D57u;
constant uint W0 = 0x9E3779B9u;
constant uint W1 = 0xBB67AE85u;
constant uint NONE = 0xFFFFFFFFu;

// Must match `Params` in mod.rs (repr(C), 8-byte fields first).
struct P {
    ulong thr_asym[5];
    ulong thr_death[5];
    ulong mk_n, hp_n, nh;
    ulong mp_stay_thr, mp_go_thr, iso_thr;
    uint n, nloc, step, k0, k1;
    uint homes_end, works_end, markets_end, mk_start, hp_start, col_lo, col_hi;
    uint at_work, mp_stay_pos, mp_go_pos, iso_on, closed_mask, col_closed, day_end;
    uint vstride, dcap;
    uint dcnt[4];
    uint don[4];
    uint pad;
};

// Philox4x32-10, bit-identical to core::rng::philox4x32_10 (same constants, same round).
inline uint4 philox(uint c0, uint c1, uint c2, uint c3, uint k0, uint k1) {
    for (uint r = 0; r < 10; r++) {
        if (r > 0) { k0 += W0; k1 += W1; }
        uint hi0 = mulhi(M0, c0), lo0 = M0 * c0;
        uint hi1 = mulhi(M1, c2), lo1 = M1 * c2;
        uint n0 = hi1 ^ c1 ^ k0;
        uint n2 = hi0 ^ c3 ^ k1;
        c0 = n0; c1 = lo1; c2 = n2; c3 = lo0;
    }
    return uint4(c0, c1, c2, c3);
}

// (x + 0.5) * 2^-32  <  fl(a * b) for positive normal doubles a < 1, b <= 1 given as bit
// patterns. The 106-bit product is rounded to 53 bits (ties to even) exactly like an f64 mul.
inline bool lt_prod(uint x, ulong abits, ulong bbits) {
    uint ea = uint((abits >> 52) & 0x7FFu), eb = uint((bbits >> 52) & 0x7FFu);
    if (ea == 0 || eb == 0) return false;
    ulong ma = (abits & 0xFFFFFFFFFFFFFUL) | (1UL << 52);
    ulong mb = (bbits & 0xFFFFFFFFFFFFFUL) | (1UL << 52);
    int e = int(ea) + int(eb) - 2150;
    ulong hi = mulhi(ma, mb), lo = ma * mb;      // product in [2^104, 2^106)
    int sh = (hi >> 41) ? 53 : 52;
    ulong mr = (hi << (64 - sh)) | (lo >> sh);
    ulong rem = lo & ((1UL << sh) - 1UL), half_ = 1UL << (sh - 1);
    if (rem > half_ || (rem == half_ && (mr & 1UL))) mr++;
    if (mr == (1UL << 53)) { mr >>= 1; sh++; }
    int s = -(e + sh + 33);                      // compare (2x+1) < mr * 2^-s
    ulong y = 2UL * ulong(x) + 1UL;
    if (s <= 0) return true;                     // unreachable for a < 1
    if (s >= 64) return false;
    if ((y >> (64 - s)) != 0) return false;
    return (y << s) < mr;
}

// Duration in steps from the host-built threshold table: 1 + #(entries <= x).
inline uint dur(constant P& p, device const uint* dtab, uint kind, uint x) {
    if (!p.don[kind]) return 0;
    device const uint* t = dtab + kind * p.dcap;
    uint lo = 0, hi = p.dcnt[kind];
    while (lo < hi) {
        uint mid = (lo + hi) >> 1;
        if (t[mid] <= x) lo = mid + 1; else hi = mid;
    }
    return 1 + lo;
}

// (a) placement + per-location occupancy / infectious counts. One thread per agent; the
// counters are 32-bit atomics (relaxed): addition commutes, so the result is order-independent.
kernel void k_place(constant P& p [[buffer(0)]],
                    device const uint* id [[buffer(1)]],
                    device const uchar* state [[buffer(2)]],
                    device const uint* home [[buffer(3)]],
                    device const uint* work [[buffer(4)]],
                    device uint* loc [[buffer(5)]],
                    device atomic_uint* occ [[buffer(6)]],
                    device atomic_uint* inf_a [[buffer(7)]],
                    device atomic_uint* inf_i [[buffer(8)]],
                    uint i [[thread_position_in_grid]]) {
    if (i >= p.n) return;
    uchar s = state[i];
    if (s == 6) { loc[i] = NONE; return; }
    uint h = home[i];
    uint l;
    if (s == 4) {
        l = p.hp_start + uint(ulong(h) * p.hp_n / p.nh);
    } else {
        uint w = work[i];
        bool stay = (w == h);
        ulong thr = stay ? p.mp_stay_thr : p.mp_go_thr;
        bool pos = stay ? (p.mp_stay_pos != 0) : (p.mp_go_pos != 0);
        uint ag = id[i];
        bool visit = false;
        if (pos) visit = ulong(philox(ag, p.step, 3u, 0u, p.k0, p.k1).x) < thr;
        uint l0 = visit ? p.mk_start + uint(ulong(h) * p.mk_n / p.nh) : (p.at_work ? w : h);
        l = l0;
        if (s == 3 && p.iso_on != 0 && ulong(philox(ag, 0u, 8u, 0u, p.k0, p.k1).x) < p.iso_thr) {
            l = h;
        } else {
            uint kind = uint(l0 >= p.homes_end) + uint(l0 >= p.works_end) + uint(l0 >= p.markets_end);
            bool col = p.col_closed != 0 && l0 >= p.col_lo && l0 < p.col_hi;
            if (((p.closed_mask >> kind) & 1u) != 0 || col) l = h;
        }
    }
    loc[i] = l;
    atomic_fetch_add_explicit(&occ[l], 1u, memory_order_relaxed);
    if (s == 2) atomic_fetch_add_explicit(&inf_a[l], 1u, memory_order_relaxed);
    else if (s == 3) atomic_fetch_add_explicit(&inf_i[l], 1u, memory_order_relaxed);
}

// Per-location pass: compact the locations with infectious agents into a list for the host
// (which evaluates lambda with f64 exp), clear the lambda tables and reset the counters.
kernel void k_locs(constant P& p [[buffer(0)]],
                   device atomic_uint* occ [[buffer(1)]],
                   device atomic_uint* inf_a [[buffer(2)]],
                   device atomic_uint* inf_i [[buffer(3)]],
                   device ulong* lam_thr [[buffer(4)]],
                   device ulong* lam_bits [[buffer(5)]],
                   device uint4* active [[buffer(6)]],
                   device atomic_uint* nactive [[buffer(7)]],
                   uint l [[thread_position_in_grid]]) {
    if (l >= p.nloc) return;
    uint a = atomic_load_explicit(&inf_a[l], memory_order_relaxed);
    uint i = atomic_load_explicit(&inf_i[l], memory_order_relaxed);
    uint o = atomic_load_explicit(&occ[l], memory_order_relaxed);
    lam_thr[l] = 0;
    lam_bits[l] = 0;
    if ((a | i) != 0) {
        uint k = atomic_fetch_add_explicit(nactive, 1u, memory_order_relaxed);
        active[k] = uint4(l, a, i, o);
    }
    atomic_store_explicit(&occ[l], 0u, memory_order_relaxed);
    atomic_store_explicit(&inf_a[l], 0u, memory_order_relaxed);
    atomic_store_explicit(&inf_i[l], 0u, memory_order_relaxed);
}

// (b)+(c) health-state / timer update, infection sampling and vaccine clock, fused per agent.
// New infections are summed with simdgroup reductions (one atomic per simdgroup).
kernel void k_update(constant P& p [[buffer(0)]],
                     device const uint* id [[buffer(1)]],
                     device const uchar* age [[buffer(2)]],
                     device uchar* state [[buffer(3)]],
                     device ushort* timer [[buffer(4)]],
                     device const uchar* vstat [[buffer(5)]],
                     device ushort* vdays [[buffer(6)]],
                     device const uint* loc [[buffer(7)]],
                     device const ulong* lam_thr [[buffer(8)]],
                     device const ulong* lam_bits [[buffer(9)]],
                     device const ulong* hosp [[buffer(10)]],
                     device const ulong* binf [[buffer(11)]],
                     device const uint* dtab [[buffer(12)]],
                     device atomic_uint* stats [[buffer(13)]],
                     uint i [[thread_position_in_grid]],
                     uint lane [[thread_index_in_simdgroup]]) {
    uint ni = 0;
    if (i < p.n) {
        uchar s = state[i];
        uint vs = vstat[i];
        uint vd = vdays[i];
        if (s == 1 || s == 2 || s == 3 || s == 4) {
            uint t0 = timer[i];
            uint t = t0 > 0 ? t0 - 1 : 0;
            if (t > 0) {
                timer[i] = ushort(t);
            } else if (s == 2) {
                state[i] = 5; timer[i] = 0;                  // A -> R: no random number
            } else {
                uint4 r = philox(id[i], p.step, 2u, 0u, p.k0, p.k1);
                uint a = age[i];
                uchar ns; uint tm = 0;
                if (s == 1) {
                    if (ulong(r.x) < p.thr_asym[a]) { ns = 2; tm = dur(p, dtab, 1, r.y); }
                    else { ns = 3; tm = dur(p, dtab, 2, r.y); }
                } else if (s == 3) {
                    uint idx = vs != 0 ? vd + 1 : 0;
                    if (ulong(r.x) < hosp[a * p.vstride + idx]) { ns = 4; tm = dur(p, dtab, 3, r.y); }
                    else ns = 5;
                } else {
                    ns = ulong(r.x) < p.thr_death[a] ? 6 : 5;
                }
                state[i] = ns; timer[i] = ushort(tm);
            }
        } else if (s == 0) {
            uint l = loc[i];
            ulong lb = lam_bits[l];
            if (lb != 0) {
                uint4 r = philox(id[i], p.step, 1u, 0u, p.k0, p.k1);
                bool inf = vs == 0 ? (ulong(r.x) < lam_thr[l]) : lt_prod(r.x, lb, binf[vd]);
                if (inf) {
                    state[i] = 1;
                    timer[i] = ushort(dur(p, dtab, 0, r.y));
                    ni = 1;
                }
            }
        }
        if (p.day_end != 0 && vs != 0) vdays[i] = ushort(min(vd + 1u, 65535u));
    }
    uint tot = simd_sum(ni);
    if (lane == 0 && tot != 0) atomic_fetch_add_explicit(&stats[7], tot, memory_order_relaxed);
}

// (d) population statistics: grid-stride loop, simdgroup reduction per state, then a
// threadgroup-memory reduction across simdgroups, then one atomic per state per threadgroup.
kernel void k_stats(constant P& p [[buffer(0)]],
                    device const uchar* state [[buffer(1)]],
                    device atomic_uint* stats [[buffer(2)]],
                    uint gid [[thread_position_in_grid]],
                    uint gsz [[threads_per_grid]],
                    uint lid [[thread_position_in_threadgroup]],
                    uint sg [[simdgroup_index_in_threadgroup]],
                    uint nsg [[simdgroups_per_threadgroup]],
                    uint lane [[thread_index_in_simdgroup]]) {
    threadgroup uint part[32 * 7];
    uint c0 = 0, c1 = 0, c2 = 0, c3 = 0, c4 = 0, c5 = 0, c6 = 0;
    for (uint i = gid; i < p.n; i += gsz) {
        uchar s = state[i];
        c0 += (s == 0); c1 += (s == 1); c2 += (s == 2); c3 += (s == 3);
        c4 += (s == 4); c5 += (s == 5); c6 += (s == 6);
    }
    uint v[7] = {simd_sum(c0), simd_sum(c1), simd_sum(c2), simd_sum(c3),
                 simd_sum(c4), simd_sum(c5), simd_sum(c6)};
    if (lane == 0) for (uint s = 0; s < 7; s++) part[sg * 7 + s] = v[s];
    threadgroup_barrier(mem_flags::mem_threadgroup);
    if (lid < 7) {
        uint t = 0;
        for (uint g = 0; g < nsg; g++) t += part[g * 7 + lid];
        if (t != 0) atomic_fetch_add_explicit(&stats[lid], t, memory_order_relaxed);
    }
}

// Test hook: exact emulation of `(x+0.5)*2^-32 < a*b` against the host.
kernel void k_test_lt_prod(device const uint* xs [[buffer(0)]],
                           device const ulong* a [[buffer(1)]],
                           device const ulong* b [[buffer(2)]],
                           device uchar* out [[buffer(3)]],
                           uint i [[thread_position_in_grid]]) {
    out[i] = lt_prod(xs[i], a[i], b[i]) ? 1 : 0;
}

// Test hook: Philox output words for (id, step, purpose).
kernel void k_test_philox(device const uint4* in [[buffer(0)]],
                          device const uint2* key [[buffer(1)]],
                          device uint4* out [[buffer(2)]],
                          uint i [[thread_position_in_grid]]) {
    uint4 c = in[i];
    out[i] = philox(c.x, c.y, c.z, c.w, key[0].x, key[0].y);
}
