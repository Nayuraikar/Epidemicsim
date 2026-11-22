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
- [x] Phase 5: Metal (M1) + MPI. Golden: Metal (shared/private, tg 32-256), all scenario files, and MPI (static/dynamic, 3-4 ranks, byte-identical outputs) PASS.
  - Metal: `backends/src/mtl` (MSL Philox, atomics counts, simdgroup reduction). f64 tests become integer thresholds; vaccinated lambda*(1-ve) emulated exactly in u64; lambda's f64 `exp` runs on the CPU for active locations only.
  - Backend trait got `sync_to_host`/`host_changed` (scenario vaccination edits agents on the host).
  - MPI: `crates/mpi` (rsmpi 0.8 + Homebrew OpenMPI 5, builds fine); `compare --mpi auto|static|dynamic`; `bench --suite mpi|metal` regenerate results.
- [x] Phase 6 (code complete, NOTHING RUN ON A GPU): CUDA backend `--features cuda` (cudarc+NVRTC; kernels in `crates/kernels-c/cuda`; atomics/segmented counts, branch/pred/smem/pairs, AoS/SoA, health-state partition, exact host lambda like Metal), block sweep + warp + variant benches, `scripts/ptx.sh`, AVX2 golden test + bench path, OpenACC source, `scripts/kaggle_run.sh`, `docs/KAGGLE.md`.
  - Verified here: Mac build/tests/clippy (default and `--features cuda`), host emulation of Philox/`lt_prod`/predication (`scripts/cuda_host_check.sh`), x86_64 compile of AVX2 tests. Metal refactored onto `gpu_common.rs`, golden still PASS.
  - Unverified: NVRTC/nvcc compile, every GPU result (golden, speedups, ncu), PTX counts, AVX2 on real x86, OpenACC on a GPU. First check = `docs/KAGGLE.md`.
- [x] Phase 7: Server + studio (`epidemicsim serve|bake`, web/ with maplibre + deck.gl + Recharts)
  - REST (cities, scenarios, comparison, bands, summary, baked frames) + `/ws/live` (start/pause/resume/stop/speed/inspect); live capped at 250k agents via household subsample (non-home beta x k^0.6, tuned on Mumbai only).
  - Infection chains recorded in the server (flag, no engine change): sources = co-located infectious agents, not a unique infector; inspector works in live runs only.
  - Tested headless Chrome (CDP): replay, live, pause, inspector click, 2-run compare, JSON/PNG export, not-built city. Untested: real GPU/Metal backend live, other browsers, touch, phone layout.
- [x] Phase 8: PAGP Lab, landing, report, README
  - server `/api/lab/index` + `/api/lab/file/<path>` (whitelisted results roots); web `#/` landing, `#/studio`, `#/lab` (10 tabs, all from results files), `#/demo` (auto-replay).
  - `scripts/gen_report.py` -> `docs/REPORT.md` (9 PAGP sections + Limitations); README with mermaid, setup, demo script; golden_table skips NEON off aarch64.
  - Tested headless Chrome: all 10 tabs render without console errors, demo replay starts. Untested: Safari/Firefox, phone layout, PTX diff on the largest pairs (-G vs -O3 aligns only if <30M cells).
  - Notes: kaggle/ptx/analysis.md and ptx/analysis.md are unfilled templates (not used); Mac golden status comes from this file (logs not saved).

## Findings

- Phase 4: load imbalance is small on both cities (max/mean <= 1.15) and Mumbai/Chennai gave no measurable dense-vs-sparse difference; speedup is limited by cores (4P+4E) and memory, not scheduling.
- Phase 5 Metal vs best CPU (rayon guided 8t, same session, 80 steps): 100k 0.079 s vs 0.071 s (0.90x, Metal slower); 1M 0.233 vs 0.636 s (2.73x); 10M 1.666 vs 6.743 s (4.05x). results/gpu/metal_vs_cpu.json.
- Phase 5 Metal breakdown @1M shared tg64: kernels 0.141 s, dispatch/commit/wait overhead 0.072 s, CPU lambda 0.004 s, readback 0.0001 s of 0.218 s; @100k overhead (0.055 s) exceeds kernels (0.021 s). Two command buffers per step cost ~0.7 ms each step.
- Phase 5 shared vs private and tg 32/64/128/256 differ by <=13% at 1M (best: shared tg64 0.218 s; others 0.227-0.247 s), close to run-to-run noise; unified memory makes private no faster.
- Phase 5 MPI (50k agents, 8 scenarios x 6 runs, sequential backend, 1 core per rank): static 1/2/4/8 procs speedup 1.00/1.90/2.88/4.26 (30.3 s -> 7.1 s); dynamic 1.00/0.98/1.84/3.42 (pure master => procs-1 workers). Comm <0.1% of run time for static; dynamic comm figures include the master waiting on requests.
- Phase 5 caveats: 10M Metal/CPU results are not checked against sequential (too slow); 100k and 1M are (identical per-step stats).
