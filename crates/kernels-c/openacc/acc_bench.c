/* Standalone OpenACC kernels (bonus, lowest priority; NOT wired into the Rust simulation).
 *
 * Mirrors the shape of the epidemic step's two data-parallel loops:
 *   acc_update : per-agent timer countdown + health-state transition with a Philox4x32-10 draw
 *                compared against integer thresholds (E->A|I, A->R, I->H|R, H->D|R). Durations
 *                use a fixed number of steps per state instead of the exact duration tables, so
 *                this is "kernel-shaped", not bit-comparable with the Rust backends.
 *   acc_stats  : S/E/A/I/H/R/D counts with `reduction(+:...)`.
 * Build (NVHPC):  nvc -acc -gpu=cc75 -O3 -Minfo=accel acc_bench.c -o acc_bench
 * Build (host):   cc -O2 acc_bench.c -o acc_bench   (pragmas ignored; checks the logic)
 * Run: ./acc_bench [agents=1000000] [steps=80]; prints timing and the final counts, and
 * checks the accelerated run against a serial reference loop in the same binary.
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define M0 0xD2511F53u
#define M1 0xCD9E8D57u
#define W0 0x9E3779B9u
#define W1 0xBB67AE85u

static inline uint32_t philox_x(uint32_t c0, uint32_t c1, uint32_t c2, uint32_t c3, uint32_t k0, uint32_t k1) {
    for (int r = 0; r < 10; r++) {
        if (r > 0) { k0 += W0; k1 += W1; }
        uint64_t p0 = (uint64_t)M0 * c0, p1 = (uint64_t)M1 * c2;
        uint32_t n0 = (uint32_t)(p1 >> 32) ^ c1 ^ k0;
        uint32_t n2 = (uint32_t)(p0 >> 32) ^ c3 ^ k1;
        c0 = n0; c1 = (uint32_t)p1; c2 = n2; c3 = (uint32_t)p0;
    }
    return c0;
}

/* thresholds on the 32-bit draw: P(asymptomatic)=0.4, P(hospitalised)=0.1, P(death|H)=0.2 */
#define THR_ASYM 1717986918u
#define THR_HOSP 429496729u
#define THR_DEATH 858993459u
#define DUR_E 12u
#define DUR_A 20u
#define DUR_I 24u
#define DUR_H 40u

#pragma acc routine seq
static inline void update_one(uint32_t i, uint32_t step, uint8_t *state, uint16_t *timer) {
    uint8_t s = state[i];
    if (s < 1 || s > 4) return;
    uint16_t t0 = timer[i];
    uint16_t t = t0 > 0 ? t0 - 1 : 0;
    if (t > 0) { timer[i] = t; return; }
    if (s == 2) { state[i] = 5; timer[i] = 0; return; }
    uint32_t x = philox_x(i, step, 2u, 0u, 0xa4093822u, 0x299f31d0u);
    uint8_t ns; uint16_t tm = 0;
    if (s == 1)      { if (x < THR_ASYM) { ns = 2; tm = DUR_A; } else { ns = 3; tm = DUR_I; } }
    else if (s == 3) { if (x < THR_HOSP) { ns = 4; tm = DUR_H; } else ns = 5; }
    else             { ns = x < THR_DEATH ? 6 : 5; }
    state[i] = ns; timer[i] = tm;
}

void acc_update(uint32_t n, uint32_t step, uint8_t *state, uint16_t *timer) {
    #pragma acc parallel loop present(state[0:n], timer[0:n])
    for (uint32_t i = 0; i < n; i++) update_one(i, step, state, timer);
}

void acc_stats(uint32_t n, const uint8_t *state, uint32_t counts[7]) {
    uint32_t c0 = 0, c1 = 0, c2 = 0, c3 = 0, c4 = 0, c5 = 0, c6 = 0;
    #pragma acc parallel loop present(state[0:n]) reduction(+:c0,c1,c2,c3,c4,c5,c6)
    for (uint32_t i = 0; i < n; i++) {
        uint8_t s = state[i];
        c0 += (s == 0); c1 += (s == 1); c2 += (s == 2); c3 += (s == 3);
        c4 += (s == 4); c5 += (s == 5); c6 += (s == 6);
    }
    counts[0] = c0; counts[1] = c1; counts[2] = c2; counts[3] = c3;
    counts[4] = c4; counts[5] = c5; counts[6] = c6;
}

static double now(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec + 1e-9 * t.tv_nsec; }

int main(int argc, char **argv) {
    uint32_t n = argc > 1 ? (uint32_t)atol(argv[1]) : 1000000u, steps = argc > 2 ? (uint32_t)atol(argv[2]) : 80u;
    uint8_t *st = malloc(n), *ref = malloc(n);
    uint16_t *tm = malloc(n * 2), *rtm = malloc(n * 2);
    for (uint32_t i = 0; i < n; i++) {   /* 2% exposed, rest susceptible */
        uint32_t x = philox_x(i, 0, 9u, 0u, 1u, 2u);
        st[i] = x < 85899345u ? 1 : 0;
        tm[i] = st[i] ? 1 + (x % DUR_E) : 0;
    }
    memcpy(ref, st, n); memcpy(rtm, tm, n * 2);
    for (uint32_t s = 0; s < steps; s++)   /* serial reference */
        for (uint32_t i = 0; i < n; i++) update_one(i, s, ref, rtm);
    uint32_t want[7] = {0}, got[7];
    for (uint32_t i = 0; i < n; i++) want[ref[i]]++;
    double t0 = now(), tk = 0;
    #pragma acc data copy(st[0:n], tm[0:n])
    {
        for (uint32_t s = 0; s < steps; s++) {
            double a = now();
            acc_update(n, s, st, tm);
            acc_stats(n, st, got);
            tk += now() - a;
        }
    }
    double total = now() - t0;
    int ok = memcmp(st, ref, n) == 0 && memcmp(tm, rtm, n * 2) == 0;
    for (int k = 0; k < 7; k++) ok = ok && got[k] == want[k];
    printf("agents=%u steps=%u total_s=%.4f loops_s=%.4f counts=%u,%u,%u,%u,%u,%u,%u matches_serial=%s\n", n, steps,
           total, tk, got[0], got[1], got[2], got[3], got[4], got[5], got[6], ok ? "yes" : "NO");
    return ok ? 0 : 1;
}
