/* OpenMP versions of the epidemic step's two hot kernels, in plain C:
 *   1. agent update   (placement, progression + infection, vaccine clock, counts)
 *   2. location contacts (force of infection per location)
 * Results are bit-identical to the Rust sequential backend:
 *   - Philox4x32-10 below mirrors core::rng (same constants, same counter layout);
 *   - build with -ffp-contract=off (no fused multiply-add) and no fast-math, so the double
 *     arithmetic rounds exactly as Rust's does;
 *   - infectious weight per location is (#A * rel_a + #I), exact for rel_a = 0.5.
 * The loop schedule (static | dynamic | guided) and chunk size are chosen at run time. */
#include <math.h>
#include <omp.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "ep.h"

#define NONE 0xFFFFFFFFu
enum { S_ = 0, E_, A_, I_, H_, R_, D_ };
enum { PURPOSE_INFECT = 1, PURPOSE_TRANSITION = 2, PURPOSE_SCHEDULE = 3, PURPOSE_ISOLATE = 8 };

/* ---------------------------------------------------------------- Philox4x32-10 */
static inline void philox(uint32_t c0, uint32_t c1, uint32_t c2, uint32_t c3, uint32_t k0,
                          uint32_t k1, uint32_t out[4]) {
  for (int r = 0; r < 10; r++) {
    if (r > 0) {
      k0 += 0x9E3779B9u;
      k1 += 0xBB67AE85u;
    }
    uint64_t p0 = (uint64_t)0xD2511F53u * c0;
    uint64_t p1 = (uint64_t)0xCD9E8D57u * c2;
    uint32_t n0 = (uint32_t)(p1 >> 32) ^ c1 ^ k0;
    uint32_t n1 = (uint32_t)p1;
    uint32_t n2 = (uint32_t)(p0 >> 32) ^ c3 ^ k1;
    uint32_t n3 = (uint32_t)p0;
    c0 = n0; c1 = n1; c2 = n2; c3 = n3;
  }
  out[0] = c0; out[1] = c1; out[2] = c2; out[3] = c3;
}

static inline double to_unit(uint32_t x) { return ((double)x + 0.5) * (1.0 / 4294967296.0); }

static inline void uniforms(const EpCtx *c, uint32_t id, uint32_t step, uint32_t purpose,
                            double *u0, double *u1) {
  uint32_t b[4];
  philox(id, step, purpose, 0, c->key0, c->key1, b);
  *u0 = to_unit(b[0]);
  *u1 = to_unit(b[1]);
}

/* duration in steps: uniform in [0.5, 1.5] x mean, at least one step (matches duration_steps) */
static inline uint16_t duration_steps(double mean_days, double u) {
  double v = round(mean_days * 4.0 * (0.5 + u));
  if (v < 1.0) v = 1.0;
  if (v > 65535.0) v = 65535.0;
  return (uint16_t)v;
}

/* ---------------------------------------------------------------- vaccine efficacy */
static inline double waning(const EpCtx *c, uint16_t days) {
  double d = (double)days;
  double ramp = c->ramp_days > 0.0 ? fmin(d / c->ramp_days, 1.0) : 1.0;
  /* Rust compiles 0.5.powf(y) to exp2(-y); mirror that for days beyond the table. */
  return ramp * exp2(-(fmax(d - c->ramp_days, 0.0) / c->half_life));
}
static inline double eff_inf(const EpCtx *c, uint8_t status, uint16_t days) {
  if (status == 0) return 0.0;
  if ((int64_t)days < c->lut_n) return c->ve_inf_lut[days];
  return c->ve_inf * waning(c, days);
}
static inline double eff_sev(const EpCtx *c, uint8_t status, uint16_t days) {
  if (status == 0) return 0.0;
  if ((int64_t)days < c->lut_n) return c->ve_sev_lut[days];
  return c->ve_sev * waning(c, days);
}

/* ---------------------------------------------------------------- schedule */
static inline void slot_plan(uint32_t day, uint32_t slot, int stays_home, double *mp, int *at_work) {
  int weekend = (day % 7) >= 5;
  if (slot == 0) { *mp = 0.0; *at_work = 0; }
  else if (slot == 1) {
    if (!weekend) { *mp = stays_home ? 0.10 : 0.0; *at_work = 1; }
    else { *mp = 0.05; *at_work = 0; }
  } else if (slot == 2) {
    if (!weekend) { *mp = stays_home ? 0.20 : 0.0; *at_work = 1; }
    else { *mp = 0.30; *at_work = 0; }
  } else { *mp = weekend ? 0.30 : 0.20; *at_work = 0; }
}

static inline int kind_of(const EpCtx *c, uint32_t l) {
  if (l < c->homes_end) return 0;
  if (l < c->works_end) return 1;
  if (l < c->markets_end) return 2;
  return 3;
}

static inline uint32_t location_of(const EpCtx *c, int64_t i, uint8_t st) {
  uint32_t home = c->home[i];
  if (st == H_) return c->hp_start + (uint32_t)((uint64_t)home * c->hp_n / c->nh);
  uint32_t work = c->work[i];
  double mp; int at_work;
  slot_plan(c->day, c->slot, work == home, &mp, &at_work);
  uint32_t l;
  double u0, u1;
  if (mp > 0.0) uniforms(c, c->id[i], c->step, PURPOSE_SCHEDULE, &u0, &u1);
  if (mp > 0.0 && u0 < mp) l = c->mk_start + (uint32_t)((uint64_t)home * c->mk_n / c->nh);
  else if (at_work) l = work;
  else l = home;
  if (st == I_ && c->iso > 0.0) {
    uniforms(c, c->id[i], 0, PURPOSE_ISOLATE, &u0, &u1);
    if (u0 < c->iso) return home;
  }
  if (c->closed[kind_of(c, l)] || (c->col_closed && l >= c->col_lo && l < c->col_hi)) return home;
  return l;
}

/* ---------------------------------------------------------------- loop bodies */
static inline void place_one(const EpCtx *c, int64_t i) {
  uint8_t st = c->state[i];
  if (st == D_) { c->loc[i] = NONE; return; }
  uint32_t l = location_of(c, i, st);
  c->loc[i] = l;
  /* Several threads can hit the same location: the increments MUST be atomic. */
#pragma omp atomic
  c->occ[l]++;
  if (st == A_) {
#pragma omp atomic
    c->inf_a[l]++;
  } else if (st == I_) {
#pragma omp atomic
    c->inf_i[l]++;
  }
}

static inline void lambda_one(const EpCtx *c, int64_t l, uint64_t *sa, uint64_t *si) {
  uint32_t a = c->inf_a[l], i = c->inf_i[l];
  double infectious = (double)a * c->rel_a + (double)i;
  if (infectious > 0.0) {
    uint32_t occ = c->occ[l] > 1 ? c->occ[l] : 1;
    c->lambda[l] = 1.0 - exp(-c->beta[kind_of(c, (uint32_t)l)] * infectious / sqrt((double)occ));
  } else {
    c->lambda[l] = 0.0;
  }
  *sa += a; *si += i;
  c->inf_a[l] = 0; c->inf_i[l] = 0; c->occ[l] = 0; /* reset for the next step */
}

static inline void update_one(EpCtx *c, int64_t i, uint32_t h[7], uint32_t *ni) {
  uint8_t st = c->state[i];
  double u0, u1;
  if (st == S_) {
    double p = c->lambda[c->loc[i]];
    if (p > 0.0) {
      double ve = eff_inf(c, c->vstat[i], c->vdays[i]);
      uniforms(c, c->id[i], c->step, PURPOSE_INFECT, &u0, &u1);
      if (u0 < p * (1.0 - ve)) {
        c->state[i] = E_;
        c->timer[i] = duration_steps(c->latent_days, u1);
        (*ni)++;
      }
    }
  } else if (st == E_ || st == A_ || st == I_ || st == H_) {
    uint16_t t = c->timer[i] > 0 ? (uint16_t)(c->timer[i] - 1) : 0;
    if (t > 0) {
      c->timer[i] = t;
    } else {
      uniforms(c, c->id[i], c->step, PURPOSE_TRANSITION, &u0, &u1);
      int age = c->age[i];
      uint8_t ns; double mean;
      if (st == E_) {
        if (u0 < c->p_asym[age]) { ns = A_; mean = c->asym_days; } else { ns = I_; mean = c->sym_days; }
      } else if (st == A_) { ns = R_; mean = 0.0; }
      else if (st == I_) {
        double ve = eff_sev(c, c->vstat[i], c->vdays[i]);
        if (u0 < c->p_hosp[age] * (1.0 - ve)) { ns = H_; mean = c->hosp_days; } else { ns = R_; mean = 0.0; }
      } else {
        if (u0 < c->p_death[age]) ns = D_; else ns = R_;
        mean = 0.0;
      }
      c->state[i] = ns;
      c->timer[i] = mean > 0.0 ? duration_steps(mean, u1) : 0;
    }
  }
  if (c->day_end && c->vstat[i] != 0 && c->vdays[i] != 65535) c->vdays[i]++;
  h[c->state[i]]++;
}

/* ---------------------------------------------------------------- scheduling
 * Runs `CALL` for I in [0, N) with the schedule chosen in c->sched (runtime) inside an
 * enclosing `omp parallel`. `nowait` lets us time each thread's own share: work_s is the
 * time from the common start (barrier) to the thread's last iteration, items its count. */
#define PAR_FOR(PH, N, I, CALL)                                                      \
  do {                                                                               \
    const int tid_ = omp_get_thread_num();                                           \
    int64_t cnt_ = 0;                                                                \
    const int64_t chunk_ = c->chunk;                                                 \
    _Pragma("omp barrier") double t0_ = omp_get_wtime();                             \
    if (c->sched == 1) {                                                             \
      _Pragma("omp for schedule(dynamic, chunk_) nowait")                            \
      for (int64_t I = 0; I < (N); I++) { cnt_++; CALL; }                            \
    } else if (c->sched == 2) {                                                      \
      _Pragma("omp for schedule(guided, chunk_) nowait")                             \
      for (int64_t I = 0; I < (N); I++) { cnt_++; CALL; }                            \
    } else if (chunk_ > 0) {                                                         \
      _Pragma("omp for schedule(static, chunk_) nowait")                             \
      for (int64_t I = 0; I < (N); I++) { cnt_++; CALL; }                            \
    } else {                                                                         \
      _Pragma("omp for schedule(static) nowait")                                     \
      for (int64_t I = 0; I < (N); I++) { cnt_++; CALL; }                            \
    }                                                                                \
    c->work_s[(PH) * EP_MAX_THREADS + tid_] += omp_get_wtime() - t0_;                \
    c->items[(PH) * EP_MAX_THREADS + tid_] += cnt_;                                  \
  } while (0)

size_t ep_ctx_size(void) { return sizeof(EpCtx); }
int ep_omp_max_threads(void) { return omp_get_max_threads(); }

void ep_step(EpCtx *c) {
  if (c->nthreads > EP_MAX_THREADS) c->nthreads = EP_MAX_THREADS;
  /* P1: place agents, count occupancy / infectious per location */
#pragma omp parallel num_threads(c->nthreads)
  { PAR_FOR(0, c->n, i, place_one(c, i)); }

  /* P2: per-location contact processing -> force of infection */
  uint64_t sa = 0, si = 0;
#pragma omp parallel num_threads(c->nthreads)
  {
    uint64_t la = 0, li = 0;
    PAR_FOR(1, c->nloc, l, lambda_one(c, l, &la, &li));
#pragma omp atomic
    sa += la;
#pragma omp atomic
    si += li;
  }
  c->sum_a = sa; c->sum_i = si;

  /* P3: progression + new infections + vaccine clock + counts, fused per agent */
  uint32_t total[7] = {0}, ni_total = 0;
#pragma omp parallel num_threads(c->nthreads)
  {
    uint32_t h[7] = {0}, ni = 0;
    PAR_FOR(2, c->n, i, update_one(c, i, h, &ni));
    for (int s = 0; s < 7; s++) {
#pragma omp atomic
      total[s] += h[s];
    }
#pragma omp atomic
    ni_total += ni;
  }
  memcpy(c->counts, total, sizeof total);
  c->new_inf = ni_total;
}

/* ---------------------------------------------------------------- race demo
 * Count infectious agents per location, three ways. `method`:
 *   0 UNSYNCHRONISED: plain `counts[l]++` from many threads. This is a data race (undefined
 *     behaviour in C): the read-modify-write of two threads can interleave, so increments are
 *     lost and the result is wrong and varies from run to run.
 *   1 ATOMIC: `#pragma omp atomic` makes each increment indivisible: always correct.
 *   2 BUFFERS: one private counter array per thread, then a parallel reduction over locations:
 *     correct and lock-free in the hot loop, at the cost of T x nloc memory + a reduction pass. */
void ep_race(int method, int nthreads, int64_t n, int64_t nloc, const uint32_t *loc,
             const uint8_t *infectious, int64_t chunk, uint32_t *counts) {
  memset(counts, 0, (size_t)nloc * sizeof(uint32_t));
  if (method == 0) {
#pragma omp parallel for num_threads(nthreads) schedule(dynamic, chunk)
    for (int64_t i = 0; i < n; i++)
      if (infectious[i]) counts[loc[i]]++; /* intentionally racy */
  } else if (method == 1) {
#pragma omp parallel for num_threads(nthreads) schedule(dynamic, chunk)
    for (int64_t i = 0; i < n; i++)
      if (infectious[i]) {
#pragma omp atomic
        counts[loc[i]]++;
      }
  } else {
    uint32_t *buf = (uint32_t *)calloc((size_t)nthreads * (size_t)nloc, sizeof(uint32_t));
#pragma omp parallel num_threads(nthreads)
    {
      uint32_t *mine = buf + (size_t)omp_get_thread_num() * (size_t)nloc;
#pragma omp for schedule(dynamic, chunk)
      for (int64_t i = 0; i < n; i++)
        if (infectious[i]) mine[loc[i]]++;
#pragma omp for schedule(static)
      for (int64_t l = 0; l < nloc; l++) {
        uint32_t s = 0;
        for (int t = 0; t < nthreads; t++) s += buf[(size_t)t * (size_t)nloc + (size_t)l];
        counts[l] = s;
      }
    }
    free(buf);
  }
}
