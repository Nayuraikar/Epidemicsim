# Thread-level parallelism: load imbalance and scaling (Apple M1, 4P+4E cores, 8 GB)

All numbers are measured (`results/tlp/imbalance_{mumbai,chennai}.json`, `results/tlp/race_demo.json`,
`results/bench/{rayon,openmp}_*_apple_m1.json`). Imbalance = max / mean of per-thread work time summed over the three
parallel regions (1.00 = perfectly balanced). `work_s` for rayon is time inside kernel calls; for OpenMP it is loop start to the
thread's last iteration (includes scheduling cost), so the two backends' imbalance values are not strictly like-for-like.
Imbalance runs: 1M agents, 30 simulated days, median of 3 repetitions after one warm-up.

## Imbalance at 8 threads (max/mean) and wall time (s)
| city | backend | static | dynamic | guided |
|---|---|---|---|---|
| Mumbai | rayon | 1.032 / 1.01 s | 1.005 / 1.25 s | 1.015 / 1.06 s |
| Mumbai | openmp | 1.097 / 1.16 s | 1.048 / 0.99 s | 1.037 / 1.09 s |
| Chennai | rayon | 1.020 / 1.04 s | 1.004 / 1.19 s | 1.010 / 0.91 s |
| Chennai | openmp | 1.147 / 1.05 s | 1.027 / 0.94 s | 1.077 / 1.27 s |

## Winner per city (lowest wall time at 8 threads)
- **Mumbai:** OpenMP dynamic (0.99 s); best rayon schedule was static (1.01 s).
- **Chennai:** rayon guided (0.91 s); best OpenMP schedule was dynamic (0.94 s).
- Differences between the winners and the runner-up are a few percent, within the run-to-run noise seen on this machine
  (e.g. OpenMP static at 4 threads on Mumbai took 1.97 s vs 1.83 s at 2 threads, which is not a real effect).
  Treat the winners as indicative, not decisive.

## Size of the imbalance and why
- The measured imbalance is small everywhere: at most 1.147 (OpenMP static, Chennai), usually 1.00-1.10. Static scheduling is
  the most imbalanced for OpenMP (1.10 / 1.15); dynamic and guided reduce it (1.03-1.08); rayon is within 1.00-1.03 for all three.
- Dynamic/guided balance the work better because idle threads take more chunks, while static fixes each thread's contiguous
  block of agents up front, so a block with more infected agents (extra Philox work) cannot be shared.
- Why it is small here: most per-agent work (placement, state scan) is uniform; the uneven part (Philox for infected or
  at-risk agents) is a small share of agents, and agent ids are spatially ordered but both synthetic-ish cities spread
  infection widely by day 30. Mumbai and Chennai have almost identical size (1M agents, ~320k locations), so the data show
  no measurable dense-vs-sparse difference in imbalance; I did not measure why beyond that.
- Imbalance is not what limits speedup. Wall time barely changes between schedules (see below); at 8 threads the 4 efficiency
  cores and shared memory bandwidth dominate.

## 1M agents, 80 steps: speedup vs own 1 thread (mean of 10, 95% interval in brackets; re-measured)
Delta-method interval from the 95% CIs of the 1-thread and N-thread mean times. Run: `scripts/bench_stats.sh`, 20 days, 1 warm-up.
| backend / schedule | 2 threads | 4 threads | 8 threads | 1-thread s | 8-thread s (mean +/- CI) |
|---|---|---|---|---|---|
| rayon static | 1.70 [1.66-1.74] | 2.16 [2.03-2.29] | 2.75 [2.65-2.86] | 1.605 | 0.583 +/- 0.019 |
| rayon dynamic (1024) | 1.36 [1.12-1.60] | 1.95 [1.81-2.10] | 2.28 [2.22-2.34] | 1.631 | 0.715 +/- 0.004 |
| rayon guided (min 64) | 1.70 [1.69-1.70] | 2.19 [2.07-2.32] | 2.93 [2.89-2.98] | 1.577 | 0.537 +/- 0.008 |
| openmp static | 1.52 [1.22-1.82] | 2.63 [2.36-2.91] | 3.24 [2.92-3.55] | 2.148 | 0.664 +/- 0.026 |
| openmp dynamic (1024) | 1.86 [1.79-1.93] | 2.39 [2.27-2.50] | 3.40 [3.37-3.42] | 1.993 | 0.587 +/- 0.003 |
| openmp guided (min 64) | 1.90 [1.90-1.91] | 2.64 [2.54-2.73] | 3.42 [3.41-3.43] | 1.986 | 0.581 +/- 0.001 |

Updated TLP finding (significance by non-overlapping 95% intervals):
- Highest 8-thread speedup is OpenMP guided (3.42x), but it is **not significantly better than OpenMP dynamic** (3.40x; intervals overlap). OpenMP static overlaps both.
- Among rayon schedules guided (2.93x) is significantly better than static (2.75x) and dynamic (2.28x).
- OpenMP's higher speedup is mostly its slower 1-thread baseline (1.99 s vs 1.58 s). In absolute time rayon guided is the fastest at 8 threads (0.537 +/- 0.008 s vs 0.581 +/- 0.001 s for OpenMP guided; non-overlapping), so "OpenMP scales better" holds for speedup only, not for time.
- The earlier 5-rep medians (rayon guided 2.63x, OpenMP dynamic 3.32x) were from single sessions; the re-measured values differ by up to 0.3x.
- Imbalance table above is from the earlier 3-rep runs and was not re-measured.

## Race demo (8 threads, 200 runs each, ~32.7k infectious agents near the epidemic peak)
Unsynchronised counting was wrong in 200/200 rayon runs (up to 1397 lost increments) and 108/200 OpenMP runs (up to 4 lost) on the
realistic snapshot; atomics and per-thread buffers were correct in every run. Median times (ms), realistic: rayon unsync 1.60,
atomic 1.68, buffers 2.22; openmp unsync 0.31, atomic 0.33, buffers 1.17. Atomic is the fastest correct method on the realistic
snapshot for both backends, which is what production uses. In the artificial 64-location hotspot, OpenMP per-thread buffers
(0.98 ms) beat atomics (1.21 ms); rayon atomic still won (1.78 vs 2.22 ms).
