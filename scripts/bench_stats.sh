#!/usr/bin/env bash
# Re-run the key CPU/Metal benchmarks with --reps 10 (mean/median/std/95% CI). Run alone, machine idle.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build -q --release
E=target/release/epidemicsim
R="--reps 10 --warmup 1 --days 20"
$E bench --backend sequential $R
$E bench --backend ilp --ilp naive --agents 100000,1000000 $R
$E bench --backend ilp --ilp optimised --agents 100000,1000000 $R
for s in static dynamic guided; do
  $E bench --backend rayon --schedule $s --agents 1000000 --threads 1,2,4,8 $R
  $E bench --backend openmp --schedule $s --agents 1000000 --threads 1,2,4,8 $R
done
$E bench --backend simd --simd scalar --agents 100000,1000000 $R
$E bench --backend simd --simd neon --agents 100000,1000000 $R
$E bench --backend metal --agents 100000,1000000,10000000 $R
