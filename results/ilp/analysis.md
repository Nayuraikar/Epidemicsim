# ILP analysis: `naive` vs `optimised` agent kernels

Source: `crates/backends/src/kernels.rs`. Assembly (Apple M1, aarch64, `--release`, cargo-show-asm):
`naive.asm` = `place_naive` + `update_naive`; `optimised.asm` = `place_optimised` + `update_optimised` + `flush_batch`.
All numbers below are **static** counts read from those files (not dynamic instruction counts).

| function | instructions | cond. branches | `umull` | `udiv` | csel/cset/fcsel | calls (non-panic) |
|---|---|---|---|---|---|---|
| place_naive | 171 | 22 | 0 | 0 | 1 | `schedule::location_of` x2 |
| place_optimised | 1219 | 68 | 89 | 4 | 22 | `philox4x32_10` x2 (scalar tail / isolation), memset |
| update_naive | 440 | 40 | 0 | 0 | 7 | `philox4x32_10` x2, `exp2` x2 |
| update_optimised | 761 | 97 | 0 | 0 | 14 | `flush_batch` x6 |
| flush_batch (Philox x4) | 2171 | 189 | 72 | 0 | 27 | `philox4x32_10` x1, `exp2` x10 |

## What the assembly shows
- **Unrolling / interleaving.** `place_optimised` has 89 scalar `umull` (64-bit products) and `flush_batch` 72, straight-line, versus 0 in the
  naive kernels, which call the out-of-line `philox4x32_10` (one block per call). The four-lane Philox rounds are emitted
  inline as independent scalar chains (no NEON instructions: the multiplies are `umull x, w, w`), so the core can overlap the chains.
- **Hoisting.** `place_naive` calls `location_of` per agent, so the slot/weekend decision, layout range checks and market/hospital
  geometry are re-derived inside the callee every time. `place_optimised` contains no call to `location_of`; those values are read from
  the precomputed plan. Its 4 `udiv` are the market/hospital index computations (one pair in the 4-wide body, one in the tail loop).
- **Branch reduction.** csel/cset/fcsel count rises 1 -> 22 (placement) and 7 -> 14 (update): location kind is computed as a sum of
  comparisons, closed-location test and visit/at-work choice are selects. In `update_optimised` the A->R transition no longer reaches Philox.
  Static conditional-branch counts went *up* (code is unrolled and duplicated for the tail loop), so this file does not show fewer
  branches executed; only the select substitutions above are visible.
- **Lookup tables.** `update_naive` calls `exp2` twice (the vaccine waning factor, `0.5.powf` is lowered to `exp2`, via `eff_infection/eff_severe`).
  In the optimised path vaccine efficacy comes from a per-day table; `exp2` remains in `flush_batch` only as the fallback for days >= 1024.
  Per-age probabilities and mean durations (x4 steps) are also precomputed in `Tables`.
- **Code size cost.** 2171 + 761 instructions for the optimised update vs 440 for the naive one (about 6.7x); `flush_batch` is kept out of line
  (`#[inline(never)]`) because inlining it at 6 sites produced a ~10.8k-line function.

## Measured effect (median of 5, 80 steps, `results/bench/ilp_*_apple_m1.json`)
| agents | naive (s) | optimised (s) | optimised vs naive |
|---|---|---|---|
| 100k | 0.236 | 0.158 | 1.49x |
| 1M | 2.364 | 1.707 | 1.38x |

Run-to-run variation on this 8 GB machine is visible (optimised 1M measured 1.58-1.74 s across repeats).
