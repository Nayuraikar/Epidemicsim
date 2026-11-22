// Shared device code for the CUDA backend. No system headers: the Rust backend compiles these
// sources with NVRTC (which has none), and scripts/ptx.sh compiles them with `nvcc -ptx`.
// Exactness design is the same as the Metal kernels: every f64 comparison the CPU makes is an
// integer threshold computed on the host; `lt_prod` emulates the one f64 product on the device.
#ifndef EPI_COMMON_CUH
#define EPI_COMMON_CUH

typedef unsigned int u32;
typedef unsigned long long u64;
typedef unsigned short u16;
typedef unsigned char u8;

#define NONE 0xFFFFFFFFu
#define FULL 0xFFFFFFFFu
#define SM_CAP 512      // per-block cache of per-location lambda entries (smem variant)

#define M0 0xD2511F53u
#define M1 0xCD9E8D57u
#define W0 0x9E3779B9u
#define W1 0xBB67AE85u

// Must match `Params` in crates/backends/src/gpu_common.rs (repr(C), 248 bytes).
struct P {
    u64 thr_asym[5];
    u64 thr_death[5];
    u64 mk_n, hp_n, nh;
    u64 mp_stay_thr, mp_go_thr, iso_thr;
    u32 n, nloc, step, k0, k1;
    u32 homes_end, works_end, markets_end, mk_start, hp_start, col_lo, col_hi;
    u32 at_work, mp_stay_pos, mp_go_pos, iso_on, closed_mask, col_closed, day_end;
    u32 vstride, dcap;
    u32 dcnt[4];
    u32 don[4];
    u32 flags;          // bit 0: update kernels read agents through `order`
};

// AoS agent record for the layout comparison (24 bytes; the SoA arrays hold the same fields).
struct alignas(8) AgentAoS {
    u32 id, home, work, loc;
    u16 timer, vdays;
    u8 age, state, vs, pad;
};

// Philox4x32-10, bit-identical to core::rng::philox4x32_10.
__device__ __forceinline__ uint4 philox(u32 c0, u32 c1, u32 c2, u32 c3, u32 k0, u32 k1) {
#pragma unroll
    for (int r = 0; r < 10; r++) {
        if (r > 0) { k0 += W0; k1 += W1; }
        u32 hi0 = __umulhi(M0, c0), lo0 = M0 * c0;
        u32 hi1 = __umulhi(M1, c2), lo1 = M1 * c2;
        u32 n0 = hi1 ^ c1 ^ k0;
        u32 n2 = hi0 ^ c3 ^ k1;
        c0 = n0; c1 = lo1; c2 = n2; c3 = lo0;
    }
    return make_uint4(c0, c1, c2, c3);
}

// (x + 0.5) * 2^-32  <  fl(a * b) for positive normal doubles a < 1, b <= 1 given as bit
// patterns; the 106-bit product is rounded to 53 bits (ties to even) like an f64 multiply.
__device__ __forceinline__ bool lt_prod(u32 x, u64 abits, u64 bbits) {
    u32 ea = (u32)((abits >> 52) & 0x7FFu), eb = (u32)((bbits >> 52) & 0x7FFu);
    if (ea == 0 || eb == 0) return false;
    u64 ma = (abits & 0xFFFFFFFFFFFFFULL) | (1ULL << 52);
    u64 mb = (bbits & 0xFFFFFFFFFFFFFULL) | (1ULL << 52);
    int e = (int)ea + (int)eb - 2150;
    u64 hi = __umul64hi(ma, mb), lo = ma * mb;      // product in [2^104, 2^106)
    int sh = (hi >> 41) ? 53 : 52;
    u64 mr = (hi << (64 - sh)) | (lo >> sh);
    u64 rem = lo & ((1ULL << sh) - 1ULL), half_ = 1ULL << (sh - 1);
    if (rem > half_ || (rem == half_ && (mr & 1ULL))) mr++;
    if (mr == (1ULL << 53)) { mr >>= 1; sh++; }
    int s = -(e + sh + 33);                          // compare (2x+1) < mr * 2^-s
    u64 y = 2ULL * (u64)x + 1ULL;
    if (s <= 0) return true;                         // unreachable for a < 1
    if (s >= 64) return false;
    if ((y >> (64 - s)) != 0) return false;
    return (y << s) < mr;
}

// Duration in steps from the host-built threshold table: 1 + #(entries <= x).
__device__ __forceinline__ u32 dur(const P& p, const u32* __restrict__ dtab, u32 kind, u32 x) {
    if (!p.don[kind]) return 0;
    const u32* t = dtab + kind * p.dcap;
    u32 lo = 0, hi = p.dcnt[kind];
    while (lo < hi) {
        u32 mid = (lo + hi) >> 1;
        if (t[mid] <= x) lo = mid + 1; else hi = mid;
    }
    return 1 + lo;
}

// Location of agent `i` this step (NONE if dead), from the schedule + interventions.
__device__ __forceinline__ u32 place_loc(const P& p, u32 id, u32 s, u32 h, u32 w) {
    if (s == 6) return NONE;
    if (s == 4) return p.hp_start + (u32)((u64)h * p.hp_n / p.nh);
    bool stay = (w == h);
    u64 thr = stay ? p.mp_stay_thr : p.mp_go_thr;
    bool pos = stay ? (p.mp_stay_pos != 0) : (p.mp_go_pos != 0);
    bool visit = false;
    if (pos) visit = (u64)philox(id, p.step, 3u, 0u, p.k0, p.k1).x < thr;
    u32 l0 = visit ? p.mk_start + (u32)((u64)h * p.mk_n / p.nh) : (p.at_work ? w : h);
    u32 l = l0;
    if (s == 3 && p.iso_on != 0 && (u64)philox(id, 0u, 8u, 0u, p.k0, p.k1).x < p.iso_thr) {
        l = h;
    } else {
        u32 kind = (u32)(l0 >= p.homes_end) + (u32)(l0 >= p.works_end) + (u32)(l0 >= p.markets_end);
        bool col = p.col_closed != 0 && l0 >= p.col_lo && l0 < p.col_hi;
        if (((p.closed_mask >> kind) & 1u) != 0 || col) l = h;
    }
    return l;
}

#endif
