# PTX analysis (template: fill in after running `scripts/ptx.sh` and the benchmarks on a GPU)

Nothing below has been measured yet. `results/ptx/summary.json` holds the instruction counts
(per kernel and totals) for each file x level; `sha256` fields show whether two levels produced
identical PTX.

## Setup
- GPU / arch / nvcc version: TODO
- Levels: O0, O1, O2, O3, O3fm (`-O3 --use_fast_math`), G (`-G`, device debug)

## Instruction mix (copy from summary.json: totals per file)
| file | level | total | ld.global | st.global | ld.shared | st.shared | fma/mad | setp | selp | bra | atom/red | shfl |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| update | O0 | TODO | | | | | | | | | | |
| update | O3 | TODO | | | | | | | | | | |
| update | G  | TODO | | | | | | | | | | |

## Timings per level (rebuild the CUDA backend with each PTX if you want end-to-end numbers)
| level | update kernel s | total s | notes |
|---|---|---|---|
| O0 | TODO | TODO | |
| O3 | TODO | TODO | |
| O3fm | TODO | TODO | integer-only kernels: expect no difference |

## Branchy vs predicated (`k_update_soa_branch` vs `k_update_soa_pred`)
- `bra` / `setp` / `selp` counts: TODO
- Does the predicated kernel really contain fewer branches? TODO
- Static instruction count vs measured time (see results/gpu/cuda_warp.json): TODO

## Observations
- Were PTX files identical across O1-O3? TODO (check sha256)
- Anything surprising (local-memory spills `ld.local`/`st.local`, `call`s, 64-bit multiply expansion in `lt_prod`): TODO
