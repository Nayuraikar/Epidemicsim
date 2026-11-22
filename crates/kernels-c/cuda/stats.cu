// Population statistics, health-state partitioning, and test hooks.
#include "common.cuh"

// S/E/A/I/H/R/D counts: grid-stride loop, warp-shuffle reduction per state, block reduction
// through shared memory, one atomic per state per block. stats[0..7) are the counts.
extern "C" __global__ void k_stats(P p, const u8* __restrict__ state, u32* stats) {
    __shared__ u32 part[32 * 7];
    u32 c[7] = {0, 0, 0, 0, 0, 0, 0};
    for (u32 i = blockIdx.x * blockDim.x + threadIdx.x; i < p.n; i += gridDim.x * blockDim.x) {
        u32 s = state[i];
#pragma unroll
        for (u32 k = 0; k < 7; k++) c[k] += (s == k);
    }
    u32 lane = threadIdx.x & 31, wid = threadIdx.x >> 5, nw = (blockDim.x + 31) >> 5;
#pragma unroll
    for (u32 k = 0; k < 7; k++) {
#pragma unroll
        for (u32 off = 16; off > 0; off >>= 1) c[k] += __shfl_down_sync(FULL, c[k], off);
    }
    if (lane == 0) {
#pragma unroll
        for (u32 k = 0; k < 7; k++) part[wid * 7 + k] = c[k];
    }
    __syncthreads();
    if (threadIdx.x < 7) {
        u32 t = 0;
        for (u32 w = 0; w < nw; w++) t += part[w * 7 + threadIdx.x];
        if (t) atomicAdd(&stats[threadIdx.x], t);
    }
}

// ---- partition agents by health state (counting sort; `order` is a permutation of agent indices) ----
// cursor[s] = number of agents with state < s, computed from the counts in `cnt`.
extern "C" __global__ void k_part_offsets(const u32* cnt, u32* cursor) {
    u32 acc = 0;
    for (int s = 0; s < 7; s++) { cursor[s] = acc; acc += cnt[s]; }
}

// Warp-aggregated scatter: one atomic per (warp, state).
extern "C" __global__ void k_part_scatter(P p, const u8* __restrict__ state, u32* cursor, u32* __restrict__ order) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    u32 lane = threadIdx.x & 31;
    u32 s = (i < p.n) ? state[i] : 255u;
    for (u32 k = 0; k < 7; k++) {
        u32 m = __ballot_sync(FULL, s == k);
        if (m == 0) continue;
        u32 leader = __ffs(m) - 1u, base = 0;
        if (lane == leader) base = atomicAdd(&cursor[k], __popc(m));
        base = __shfl_sync(FULL, base, leader);
        if (s == k) order[base + __popc(m & ((1u << lane) - 1u))] = i;
    }
}

// ---- test hooks ----
extern "C" __global__ void k_test_lt_prod(const u32* xs, const u64* a, const u64* b, u8* out, u32 n) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < n) out[i] = lt_prod(xs[i], a[i], b[i]) ? 1 : 0;
}

extern "C" __global__ void k_test_philox(const uint4* in, const uint2* key, uint4* out, u32 n) {
    u32 i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < n) { uint4 c = in[i]; out[i] = philox(c.x, c.y, c.z, c.w, key[0].x, key[0].y); }
}

// Same as k_stats for the AoS layout (strided read of the state byte).
extern "C" __global__ void k_stats_aos(P p, const AgentAoS* __restrict__ ag, u32* stats) {
    __shared__ u32 part[32 * 7];
    u32 c[7] = {0, 0, 0, 0, 0, 0, 0};
    for (u32 i = blockIdx.x * blockDim.x + threadIdx.x; i < p.n; i += gridDim.x * blockDim.x) {
        u32 s = ag[i].state;
#pragma unroll
        for (u32 k = 0; k < 7; k++) c[k] += (s == k);
    }
    u32 lane = threadIdx.x & 31, wid = threadIdx.x >> 5, nw = (blockDim.x + 31) >> 5;
#pragma unroll
    for (u32 k = 0; k < 7; k++) {
#pragma unroll
        for (u32 off = 16; off > 0; off >>= 1) c[k] += __shfl_down_sync(FULL, c[k], off);
    }
    if (lane == 0) {
#pragma unroll
        for (u32 k = 0; k < 7; k++) part[wid * 7 + k] = c[k];
    }
    __syncthreads();
    if (threadIdx.x < 7) {
        u32 t = 0;
        for (u32 w = 0; w < nw; w++) t += part[w * 7 + threadIdx.x];
        if (t) atomicAdd(&stats[threadIdx.x], t);
    }
}
