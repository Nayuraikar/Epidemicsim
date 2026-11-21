/* Shared context between Rust (#[repr(C)] EpCtx in src/lib.rs) and the OpenMP kernels.
 * Field order/types must match the Rust struct exactly (ep_ctx_size() is asserted). */
#ifndef EP_H
#define EP_H
#include <stdint.h>

#define EP_MAX_THREADS 64
#define EP_PHASES 3

typedef struct {
  /* ---- 8-byte fields ---- */
  int64_t n, nloc;
  const uint32_t *id;
  const uint8_t *age;
  uint8_t *state;
  uint16_t *timer;
  const uint32_t *home;
  const uint32_t *work;
  const uint8_t *vstat;
  uint16_t *vdays;
  uint32_t *loc;
  uint32_t *occ;
  uint32_t *inf_a;
  uint32_t *inf_i;
  double *lambda;
  const double *ve_inf_lut; /* eff_infection(1, d) for d < lut_n (computed by Rust) */
  const double *ve_sev_lut;
  int64_t lut_n;
  double *work_s; /* [EP_PHASES][EP_MAX_THREADS] cumulative per-thread seconds */
  int64_t *items; /* [EP_PHASES][EP_MAX_THREADS] cumulative per-thread items */
  int64_t chunk;
  uint64_t nh, mk_n, hp_n;
  uint64_t sum_a, sum_i; /* out: infectious counts */
  double beta[4];        /* already multiplied by the intervention multiplier */
  double rel_a;
  double p_asym[5], p_hosp[5], p_death[5];
  double latent_days, asym_days, sym_days, hosp_days;
  double ve_inf, ve_sev, ramp_days, half_life;
  double iso;
  /* ---- 4-byte fields ---- */
  uint32_t step, day, slot, key0, key1, day_end;
  uint32_t homes_end, works_end, markets_end, mk_start, hp_start, col_lo, col_hi;
  int32_t sched; /* 0 static, 1 dynamic, 2 guided */
  int32_t nthreads;
  uint32_t counts[7]; /* out */
  uint32_t new_inf;   /* out */
  /* ---- 1-byte fields ---- */
  uint8_t closed[4];
  uint8_t col_closed;
  uint8_t pad_[3];
} EpCtx;

#endif
