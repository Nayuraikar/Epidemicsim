# Progress

- [x] Phase 0: Scaffold (workspace, web, placeholders)
- [x] Phase 1: Sim core + `run`
  - core: SoA agents, Philox4x32-10 (KAT-tested), SEIAHRD model, schedules, SequentialBackend, synth city, Parquet+JSON output.
  - 100k agents x 120 days: ~1.6s release; peak A+I ~3.7k @ day 83, attack 37%, 263 deaths.
  - Golden hash test (3k agents, 40 days, seed 42) and 2-run determinism test pass.
  - Params uncalibrated; beta defaults hand-tuned.
- [x] Phase 2: Data crate + `pipeline`
  - OSM .pbf -> clip bbox -> homes/offices/schools/colleges/markets/hospitals; WorldPop or building-density weights.
  - Households, ages, gravity (1/(1+(d/3km)^2)) workplaces; folders per SCHEMA.md; `run --city`.
  - Built mumbai, bengaluru at 1M agents; `run --city` OK on both. OSM homes: mumbai 159k bldgs (under-mapped), bengaluru 780k.
  - WorldPop path untested on a real raster. Tiny fixture pbf: `python3 -I scripts/make_tiny_pbf.py`.
- [x] Phase 3: Scenarios + bench
  - scenarios: masks, vaccination (random/age/location), isolation, college/market closure, lockdown; time or threshold triggers.
  - Monte Carlo ensembles (rayon over runs, mean + p05/p95 bands); `compare` writes bands.parquet, summary.json, comparison.json.
  - bench: warmup + reps, median/min/max, agents/s, contacts/s (agent-location visits), speedup, efficiency, Amdahl fit; `backends` registry.
  - core: colleges sub-range in Layout, isolation + college closure in schedule, `World::reseed`.
  - Scenario parameters are illustrative, not calibrated.
- [x] Phase 4: CPU parallelism (all 7 steps done; golden table: every variant PASS vs sequential, with and without vaccination)
  - Apple M1 (4P+4E), medians. ILP optimised vs naive: 1.49x @100k, 1.38x @1M (0.236->0.158 s, 2.364->1.707 s); asm in results/ilp/.
  - Rayon (custom static/dynamic/guided scheduler) @1M: 8 threads 2.24x static, 1.87x dynamic, 2.63x guided vs own 1 thread.
  - OpenMP (C, FFI) @1M: 8 threads 3.10x static, 3.32x dynamic, 3.34x guided vs own 1 thread (its 1-thread time is 2.0 s vs rayon 1.6 s).
  - SIMD NEON vs scalar (same backend, 1 thread): 1.26x @100k (0.193->0.153 s), 1.05x @1M (1.613->1.535 s); kernels alone: timer 6.5x, Philox stream 2.3x; 64B alignment made no difference (<=3%). AVX2 compile-checked only (x86_64 target), untested.
  - Race: unsynchronised counting wrong in 200/200 (rayon) and 108/200 (OpenMP) runs; atomics fastest correct, used in production. Imbalance small (<=1.15); see results/tlp/analysis.md.
- [ ] Phase 5: Phase 5 (TODO: define)
- [ ] Phase 6: Phase 6 (TODO: define)
- [ ] Phase 7: Phase 7 (TODO: define)
- [ ] Phase 8: Phase 8 (TODO: define)

## Findings

- Phase 4: load imbalance is small on both cities (max/mean <= 1.15) and Mumbai/Chennai gave no measurable dense-vs-sparse difference; speedup is limited by cores (4P+4E) and memory, not scheduling.

