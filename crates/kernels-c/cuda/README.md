# CUDA kernels

Loaded by `crates/backends/src/cuda` (feature `cuda`, backend `cuda`, via `cudarc` + NVRTC; the
`.cu` files are concatenated with `common.cuh` at run time). `scripts/ptx.sh` also compiles each
with `nvcc -ptx`. **None of this has been compiled or run on a GPU yet** (no NVIDIA GPU on the
development machine); see `docs/KAGGLE.md`.

| file | kernels |
|---|---|
| `common.cuh` | Philox4x32-10, `lt_prod` (exact f64 product compare), duration table, placement rule, `P`, `AgentAoS` |
| `place.cu` | `k_place_atomic` (global atomics), `k_place_segmented` (warp segmented reduction over location-sorted agents), `k_place_aos`, `k_locs`, `k_apply_lam` |
| `update.cu` | `k_update_soa_branch`, `k_update_soa_pred`, `k_update_soa_smem`, `k_update_aos_branch/_pred`, pair split: `k_pairs_build` + `k_infect_pairs` + `k_progress` + `k_vclock` |
| `stats.cu` | `k_stats` (warp shuffle + block reduction), `k_part_offsets`/`k_part_scatter` (health-state partition), test hooks |

Exactness: lambda (f64 `exp`) is computed on the host for active locations and uploaded as exact
integer thresholds, exactly as in the Metal backend, so results match the sequential reference.
"Pair" kernels: the model is mean-field per location (no pairwise contacts), so a pair is
(susceptible agent, its location's lambda). The "sorted segmented reduction" is per warp (runs of
equal location among adjacent agents), not a global sort by location.

Host-only checks (intrinsics emulated, no GPU): `scripts/cuda_host_check.sh` (Philox KAT, `lt_prod` vs f64, branchy == predicated). Not checked anywhere: kernel launches, shuffles, atomics, shared memory, NVRTC/nvcc compilation.
