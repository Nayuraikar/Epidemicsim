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

## 1M agents, 80 steps: speedup / efficiency vs 1 thread (median of 5)
| backend / schedule | 2 threads | 4 threads | 8 threads |
|---|---|---|---|
| rayon static | 1.72 / 0.86 | 2.00 / 0.50 | 2.24 / 0.28 |
| rayon dynamic (1024) | 1.54 / 0.77 | 1.76 / 0.44 | 1.87 / 0.23 |
| rayon guided (min 64) | 1.70 / 0.85 | 2.19 / 0.55 | 2.63 / 0.33 |
| openmp static | 1.89 / 0.94 | 2.58 / 0.65 | 3.10 / 0.39 |
| openmp dynamic (1024) | 1.89 / 0.95 | 2.50 / 0.62 | 3.32 / 0.42 |
| openmp guided (min 64) | 1.90 / 0.95 | 2.69 / 0.67 | 3.34 / 0.42 |

Single-thread times: rayon ~1.6 s, openmp ~2.0 s (the literal C code is slower than the Rust optimised kernels single-threaded),
so absolute 8-thread times are close (rayon guided 0.60 s, openmp guided 0.60 s).

## Race demo (8 threads, 200 runs each, ~32.7k infectious agents near the epidemic peak)
Unsynchronised counting was wrong in 200/200 rayon runs (up to 1397 lost increments) and 108/200 OpenMP runs (up to 4 lost) on the
realistic snapshot; atomics and per-thread buffers were correct in every run. Median times (ms), realistic: rayon unsync 1.60,
atomic 1.68, buffers 2.22; openmp unsync 0.31, atomic 0.33, buffers 1.17. Atomic is the fastest correct method on the realistic
snapshot for both backends, which is what production uses. In the artificial 64-location hotspot, OpenMP per-thread buffers
(0.98 ms) beat atomics (1.21 ms); rayon atomic still won (1.78 vs 2.22 ms).
