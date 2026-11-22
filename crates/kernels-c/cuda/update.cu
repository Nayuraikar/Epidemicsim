// Agent update: health-state / timer progression + infection sampling + vaccine clock, in
// several variants (see the table in README.md):
//   divergence : branchy (switch on health state) | predicated (branchless)
//   layout     : SoA | AoS
//   lambda data: global | shared-memory tile
//   decomposition: fused per agent | split into progress + one-thread-per-(susceptible,location) pairs
// Every variant makes the same Philox draws per agent, so all give identical results.
#include "common.cuh"

struct GlobalLam {
    const u64* thr; const u64* bits;
    __device__ __forceinline__ void get(u32 l, u64& t, u64& b) const { t = thr[l]; b = bits[l]; }
};
// Tile of lambda entries for locations [lo, lo+n) cached in shared memory; others read global.
struct SmemLam {
    const u64* thr; const u64* bits; const u64* sthr; const u64* sbits; u32 lo, n;
    __device__ __forceinline__ void get(u32 l, u64& t, u64& b) const {
        u32 k = l - lo;
        if (k < n) { t = sthr[k]; b = sbits[k]; } else { t = thr[l]; b = bits[l]; }
    }
};

// Progression of an agent in state 1..4 (E, A, I, H).
__device__ __forceinline__ void progress(const P& p, const u32* __restrict__ dtab, const u64* __restrict__ hosp,
                                         u32 id, u32 age, u8& state, u16& timer, u32 vs, u32 vd) {
    u32 s = state;
    u32 t0 = timer;
    u32 t = t0 > 0 ? t0 - 1 : 0;
    if (t > 0) { timer = (u16)t; return; }
    if (s == 2) { state = 5; timer = 0; return; }         // A -> R: no random number
    uint4 r = philox(id, p.step, 2u, 0u, p.k0, p.k1);
    u32 ns, tm = 0;
    if (s == 1) {
        if ((u64)r.x < p.thr_asym[age]) { ns = 2; tm = dur(p, dtab, 1, r.y); }
        else { ns = 3; tm = dur(p, dtab, 2, r.y); }
    } else if (s == 3) {
        u32 idx = vs != 0 ? vd + 1 : 0;
        if ((u64)r.x < hosp[age * p.vstride + idx]) { ns = 4; tm = dur(p, dtab, 3, r.y); }
        else ns = 5;
    } else {
        ns = (u64)r.x < p.thr_death[age] ? 6 : 5;
    }
    state = (u8)ns; timer = (u16)tm;
}

// One agent. `locp` is read lazily (dead agents have loc NONE). Returns true on a new infection.
template <bool BL, class L>
__device__ __forceinline__ bool agent_update(const P& p, const L& lam, const u32* __restrict__ dtab,
                                             const u64* __restrict__ hosp, const u64* __restrict__ binf,
                                             u32 id, u32 age, const u32* locp, u8& state, u16& timer,
                                             u32 vs, u32 vd) {
    if (!BL) {
        u32 s = state;
        if (s >= 1 && s <= 4) {
            progress(p, dtab, hosp, id, age, state, timer, vs, vd);
        } else if (s == 0) {
            u64 lt, lb;
            lam.get(*locp, lt, lb);
            if (lb != 0) {
                uint4 r = philox(id, p.step, 1u, 0u, p.k0, p.k1);
                bool inf = vs == 0 ? ((u64)r.x < lt) : lt_prod(r.x, lb, binf[vd]);
                if (inf) { state = 1; timer = (u16)dur(p, dtab, 0, r.y); return true; }
            }
        }
        return false;
    } else {
        // Branchless: every outcome is computed for every agent and selected with predicates.
        // Draws and table lookups are made unconditionally (indices clamped to stay in range).
        u32 s = state, t0 = timer;
        bool prog = (s - 1u) < 4u;
        u32 t = t0 > 0 ? t0 - 1 : 0;
        bool expire = prog && t == 0, keep = prog && t > 0, isS = s == 0;
        u64 lt, lb;
        lam.get(isS ? *locp : 0u, lt, lb);
        uint4 rp = philox(id, p.step, 2u, 0u, p.k0, p.k1);
        uint4 ri = philox(id, p.step, 1u, 0u, p.k0, p.k1);
        bool asym = (u64)rp.x < p.thr_asym[age];
        bool hosp_ = (u64)rp.x < hosp[age * p.vstride + (vs != 0 ? vd + 1 : 0)];
        bool dead = (u64)rp.x < p.thr_death[age];
        u32 ns = (s == 1) ? (asym ? 2u : 3u) : ((s == 3) ? (hosp_ ? 4u : 5u) : (dead ? 6u : 5u));
        u32 kind = (s == 1) ? (asym ? 1u : 2u) : 3u;
        u32 d = dur(p, dtab, kind, rp.y);
        u32 tm = ((s == 1) || (s == 3 && hosp_)) ? d : 0u;
        ns = (s == 2) ? 5u : ns;
        tm = (s == 2) ? 0u : tm;
        u64 bb = binf[vs != 0 ? vd : 0];
        bool inf_u = (u64)ri.x < lt, inf_v = lt_prod(ri.x, lb, bb);
        bool inf = isS && lb != 0 && (vs == 0 ? inf_u : inf_v);
        u32 dinf = dur(p, dtab, 0, ri.y);
        state = (u8)(expire ? ns : (inf ? 1u : s));
        timer = (u16)(keep ? t : (expire ? tm : (inf ? dinf : t0)));
        return inf;
    }
}

// New infections: warp-shuffle sum, one atomic per warp. All lanes must call this.
__device__ __forceinline__ void add_new(u32 ni, u32* stats) {
#pragma unroll
    for (u32 off = 16; off > 0; off >>= 1) ni += __shfl_down_sync(FULL, ni, off);
    if ((threadIdx.x & 31) == 0 && ni) atomicAdd(&stats[7], ni);
}

// ---------------------------------------------------------------- SoA
struct SoA {
    const u32* order; const u32* id; const u8* age; u8* state; u16* timer; const u8* vstat; u16* vdays;
    const u32* loc; const u64* hosp; const u64* binf; const u32* dtab;
};

template <bool BL, class L>
__device__ __forceinline__ u32 soa_one(const P& p, const SoA& a, const L& lam, u32 t) {
    u32 i = (p.flags & 1u) ? a.order[t] : t;
    u32 vs = a.vstat[i], vd = a.vdays[i];
    u8 st0 = a.state[i], st = st0;
    u16 tm0 = a.timer[i], tm = tm0;
    bool ni = agent_update<BL>(p, lam, a.dtab, a.hosp, a.binf, a.id[i], a.age[i], a.loc + i, st, tm, vs, vd);
    if (BL || st != st0) a.state[i] = st;
    if (BL || tm != tm0) a.timer[i] = tm;
    if (p.day_end != 0 && vs != 0) a.vdays[i] = (u16)min(vd + 1u, 65535u);
    return ni;
}

#define SOA_PARAMS P p, const u32* __restrict__ order, const u32* __restrict__ id, const u8* __restrict__ age, \
    u8* state, u16* timer, const u8* __restrict__ vstat, u16* vdays, const u32* __restrict__ loc,            \
    const u64* __restrict__ lam_thr, const u64* __restrict__ lam_bits, const u64* __restrict__ hosp,        \
    const u64* __restrict__ binf, const u32* __restrict__ dtab, u32* stats
#define SOA_INIT SoA a = {order, id, age, state, timer, vstat, vdays, loc, hosp, binf, dtab}

extern "C" __global__ void k_update_soa_branch(SOA_PARAMS) {
    SOA_INIT; GlobalLam lam = {lam_thr, lam_bits};
    u32 t = blockIdx.x * blockDim.x + threadIdx.x, ni = 0;
    if (t < p.n) ni = soa_one<false>(p, a, lam, t);
    add_new(ni, stats);
}

extern "C" __global__ void k_update_soa_pred(SOA_PARAMS) {
    SOA_INIT; GlobalLam lam = {lam_thr, lam_bits};
    u32 t = blockIdx.x * blockDim.x + threadIdx.x, ni = 0;
    if (t < p.n) ni = soa_one<true>(p, a, lam, t);
    add_new(ni, stats);
}

// Branchy + shared-memory tile of lambda entries. The block first finds the range of locations
// its susceptible agents sit in (cheap when agents are location-sorted); if the range fits in
// SM_CAP entries the tile is loaded cooperatively once. stats[8]/[9] count blocks using the tile / all blocks.
extern "C" __global__ void k_update_soa_smem(SOA_PARAMS) {
    __shared__ u64 s_thr[SM_CAP], s_bits[SM_CAP];
    __shared__ u32 s_lo, s_hi;
    SOA_INIT;
    u32 t = blockIdx.x * blockDim.x + threadIdx.x;
    if (threadIdx.x == 0) { s_lo = NONE; s_hi = 0; }
    __syncthreads();
    if (t < p.n) {
        u32 i = (p.flags & 1u) ? order[t] : t;
        if (state[i] == 0) { u32 l = loc[i]; atomicMin(&s_lo, l); atomicMax(&s_hi, l + 1); }
    }
    __syncthreads();
    u32 lo = s_lo, hi = s_hi;
    bool use = hi > lo && (hi - lo) <= SM_CAP;
    if (use) {
        for (u32 k = threadIdx.x; k < hi - lo; k += blockDim.x) { s_thr[k] = lam_thr[lo + k]; s_bits[k] = lam_bits[lo + k]; }
    }
    __syncthreads();
    if (threadIdx.x == 0) { atomicAdd(&stats[9], 1u); if (use) atomicAdd(&stats[8], 1u); }
    SmemLam lam = {lam_thr, lam_bits, s_thr, s_bits, lo, use ? hi - lo : 0u};
    u32 ni = 0;
    if (t < p.n) ni = soa_one<false>(p, a, lam, t);
    add_new(ni, stats);
}

// ---------------------------------------------------------------- AoS
template <bool BL>
__device__ __forceinline__ u32 aos_one(const P& p, AgentAoS* ag, const u32* order, const u64* hosp,
                                       const u64* binf, const u32* dtab, const GlobalLam& lam, u32 t) {
    u32 i = (p.flags & 1u) ? order[t] : t;
    AgentAoS r = ag[i];
    u8 st = r.state; u16 tm = r.timer;
    bool ni = agent_update<BL>(p, lam, dtab, hosp, binf, r.id, r.age, &ag[i].loc, st, tm, r.vs, r.vdays);
    if (BL || st != r.state) ag[i].state = st;
    if (BL || tm != r.timer) ag[i].timer = tm;
    if (p.day_end != 0 && r.vs != 0) ag[i].vdays = (u16)min((u32)r.vdays + 1u, 65535u);
    return ni;
}

extern "C" __global__ void k_update_aos_branch(P p, AgentAoS* ag, const u32* __restrict__ order,
                                               const u64* __restrict__ lam_thr, const u64* __restrict__ lam_bits,
                                               const u64* __restrict__ hosp, const u64* __restrict__ binf,
                                               const u32* __restrict__ dtab, u32* stats) {
    GlobalLam lam = {lam_thr, lam_bits};
    u32 t = blockIdx.x * blockDim.x + threadIdx.x, ni = 0;
    if (t < p.n) ni = aos_one<false>(p, ag, order, hosp, binf, dtab, lam, t);
    add_new(ni, stats);
}

extern "C" __global__ void k_update_aos_pred(P p, AgentAoS* ag, const u32* __restrict__ order,
                                             const u64* __restrict__ lam_thr, const u64* __restrict__ lam_bits,
                                             const u64* __restrict__ hosp, const u64* __restrict__ binf,
                                             const u32* __restrict__ dtab, u32* stats) {
    GlobalLam lam = {lam_thr, lam_bits};
    u32 t = blockIdx.x * blockDim.x + threadIdx.x, ni = 0;
    if (t < p.n) ni = aos_one<true>(p, ag, order, hosp, binf, dtab, lam, t);
    add_new(ni, stats);
}

// ---------------------------------------------------------------- pair decomposition
// The model is mean-field per location (no pairwise contacts), so the "contact pair" unit of
// work is (susceptible agent, its location's force of infection). Stage 1 compacts those pairs
// with warp-aggregated appends; stage 2 runs one thread per pair; progression of E/A/I/H agents
// is a separate kernel; the vaccine clock a third. The three touch disjoint agents/fields.
extern "C" __global__ void k_pairs_build(P p, const u8* __restrict__ state, const u32* __restrict__ loc,
                                         const u64* __restrict__ lam_bits, u32* pairs, u32* npairs) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x, lane = threadIdx.x & 31;
    bool want = false;
    if (i < p.n && state[i] == 0) want = lam_bits[loc[i]] != 0;
    u32 m = __ballot_sync(FULL, want);
    if (m) {
        u32 leader = __ffs(m) - 1u, base = 0;
        if (lane == leader) base = atomicAdd(npairs, __popc(m));
        base = __shfl_sync(FULL, base, leader);
        if (want) pairs[base + __popc(m & ((1u << lane) - 1u))] = i;
    }
}

extern "C" __global__ void k_infect_pairs(P p, const u32* __restrict__ pairs, const u32* __restrict__ npairs,
                                          const u32* __restrict__ id, u8* state, u16* timer,
                                          const u8* __restrict__ vstat, const u16* __restrict__ vdays,
                                          const u32* __restrict__ loc, const u64* __restrict__ lam_thr,
                                          const u64* __restrict__ lam_bits, const u64* __restrict__ binf,
                                          const u32* __restrict__ dtab, u32* stats) {
    u32 t = blockIdx.x * blockDim.x + threadIdx.x, ni = 0;
    if (t < *npairs) {
        u32 i = pairs[t], l = loc[i];
        uint4 r = philox(id[i], p.step, 1u, 0u, p.k0, p.k1);
        u64 lb = lam_bits[l];
        bool inf = vstat[i] == 0 ? ((u64)r.x < lam_thr[l]) : lt_prod(r.x, lb, binf[vdays[i]]);
        if (inf) { state[i] = 1; timer[i] = (u16)dur(p, dtab, 0, r.y); ni = 1; }
    }
    add_new(ni, stats);
}

extern "C" __global__ void k_progress(P p, const u32* __restrict__ order, const u32* __restrict__ id,
                                      const u8* __restrict__ age, u8* state, u16* timer,
                                      const u8* __restrict__ vstat, const u16* __restrict__ vdays,
                                      const u64* __restrict__ hosp, const u32* __restrict__ dtab) {
    u32 t = blockIdx.x * blockDim.x + threadIdx.x;
    if (t >= p.n) return;
    u32 i = (p.flags & 1u) ? order[t] : t;
    u8 st = state[i];
    if (st >= 1 && st <= 4) {
        u16 tm = timer[i];
        progress(p, dtab, hosp, id[i], age[i], st, tm, vstat[i], vdays[i]);
        state[i] = st; timer[i] = tm;
    }
}

extern "C" __global__ void k_vclock(P p, const u8* __restrict__ vstat, u16* vdays) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < p.n && vstat[i] != 0) vdays[i] = (u16)min((u32)vdays[i] + 1u, 65535u);
}
