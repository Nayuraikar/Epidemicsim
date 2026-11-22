// Placement + per-location occupancy / infectious counts, location compaction, lambda upload.
#include "common.cuh"

// (a) SoA, one thread per agent, 32-bit global atomics (order-independent: addition commutes).
extern "C" __global__ void k_place_atomic(P p, const u32* __restrict__ id, const u8* __restrict__ state,
                                          const u32* __restrict__ home, const u32* __restrict__ work,
                                          u32* __restrict__ loc, u32* occ, u32* inf_a, u32* inf_i) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= p.n) return;
    u32 s = state[i];
    u32 l = place_loc(p, id[i], s, home[i], work[i]);
    loc[i] = l;
    if (l == NONE) return;
    atomicAdd(&occ[l], 1u);
    if (s == 2) atomicAdd(&inf_a[l], 1u);
    else if (s == 3) atomicAdd(&inf_i[l], 1u);
}

// Same, AoS layout (agent record read as one 24-byte struct, loc written into it).
extern "C" __global__ void k_place_aos(P p, AgentAoS* __restrict__ ag, u32* occ, u32* inf_a, u32* inf_i) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= p.n) return;
    AgentAoS a = ag[i];
    u32 l = place_loc(p, a.id, a.state, a.home, a.work);
    ag[i].loc = l;
    if (l == NONE) return;
    atomicAdd(&occ[l], 1u);
    if (a.state == 2) atomicAdd(&inf_a[l], 1u);
    else if (a.state == 3) atomicAdd(&inf_i[l], 1u);
}

// (b) Segmented reduction inside each warp: agents are location-sorted, so consecutive lanes
// usually share a location. Runs of equal `loc` are summed with a segmented shuffle scan and the
// last lane of each run issues one atomic per counter instead of one per agent. Correct for any
// order (unsorted input just gives short runs). All 32 lanes must execute the shuffles.
extern "C" __global__ void k_place_segmented(P p, const u32* __restrict__ id, const u8* __restrict__ state,
                                             const u32* __restrict__ home, const u32* __restrict__ work,
                                             u32* __restrict__ loc, u32* occ, u32* inf_a, u32* inf_i) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    u32 lane = threadIdx.x & 31;
    u32 l = NONE, s = 0;
    if (i < p.n) {
        s = state[i];
        l = place_loc(p, id[i], s, home[i], work[i]);
        loc[i] = l;
    }
    u32 prev = __shfl_up_sync(FULL, l, 1);
    bool head = (lane == 0) || (l != prev);
    u32 heads = __ballot_sync(FULL, head);
    u32 start = 31u - __clz(heads & (0xFFFFFFFFu >> (31u - lane)));   // lane index of my run's head
    u32 co = (l != NONE), ca = (l != NONE && s == 2), ci = (l != NONE && s == 3);
#pragma unroll
    for (u32 off = 1; off < 32; off <<= 1) {
        u32 o2 = __shfl_up_sync(FULL, co, off), a2 = __shfl_up_sync(FULL, ca, off), i2 = __shfl_up_sync(FULL, ci, off);
        if (lane - start >= off) { co += o2; ca += a2; ci += i2; }
    }
    u32 next = __shfl_down_sync(FULL, l, 1);
    bool tail = (lane == 31) || (next != l);
    if (tail && l != NONE) {
        atomicAdd(&occ[l], co);
        if (ca) atomicAdd(&inf_a[l], ca);
        if (ci) atomicAdd(&inf_i[l], ci);
    }
}

// Per-location pass: list locations with infectious agents for the host (which evaluates lambda
// with f64 exp), clear the lambda tables and the counters.
extern "C" __global__ void k_locs(P p, u32* occ, u32* inf_a, u32* inf_i, u64* lam_thr, u64* lam_bits,
                                  uint4* active, u32* nactive) {
    u32 l = blockIdx.x * blockDim.x + threadIdx.x;
    if (l >= p.nloc) return;
    u32 a = inf_a[l], i = inf_i[l], o = occ[l];
    lam_thr[l] = 0;
    lam_bits[l] = 0;
    if ((a | i) != 0) {
        u32 k = atomicAdd(nactive, 1u);
        active[k] = make_uint4(l, a, i, o);
    }
    occ[l] = 0; inf_a[l] = 0; inf_i[l] = 0;
}

// Scatter the host-computed lambda entries [loc, threshold, bits] x cnt into the tables.
extern "C" __global__ void k_apply_lam(const u64* __restrict__ ent, u32 cnt, u64* lam_thr, u64* lam_bits) {
    u32 k = blockIdx.x * blockDim.x + threadIdx.x;
    if (k >= cnt) return;
    u32 l = (u32)ent[3 * k];
    lam_thr[l] = ent[3 * k + 1];
    lam_bits[l] = ent[3 * k + 2];
}
