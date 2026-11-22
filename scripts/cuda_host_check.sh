#!/usr/bin/env bash
# Host-only sanity checks of the CUDA device code (no GPU, no nvcc): the intrinsics are emulated
# and the helpers in common.cuh / update.cu are compiled as plain C++. Checks Philox against the
# known-answer vector, `lt_prod` against f64 on 3M cases, and branchy == predicated update on 2M
# random agents. It does NOT check kernel launch logic, warp shuffles, atomics or shared memory.
set -euo pipefail
cd "$(dirname "$0")/.."
D=crates/kernels-c/cuda
T=$(mktemp -d)
for t in check_helpers check_predication; do
  "${CXX:-c++}" -std=c++14 -O1 -Wno-unknown-pragmas -I "$D" "$D/host_check/$t.cpp" -o "$T/$t"
  "$T/$t"
done
