#!/usr/bin/env bash
# One-shot CUDA / warp / PTX / AVX2 run for a Kaggle notebook (Ubuntu, T4/P100, nvcc, internet on).
# Every step is logged to results/kaggle/logs/<step>.log and a failing step never stops the rest
# (no `set -e`); results/kaggle/steps.tsv lists each step's exit code. Ends with a zip of
# results/kaggle/. See docs/KAGGLE.md. Env knobs: QUICK=1 (fewer reps/agents), REPO_URL,
# STEP_TIMEOUT (seconds per step, default 3000), TRY_NVHPC=1 (attempt the NVHPC install).
set -u

# ---- locate (or unpack / clone) the repo, in a writable directory ----
SRC="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." 2>/dev/null && pwd)"
if [ -n "$SRC" ] && [ -f "$SRC/Cargo.toml" ] && [ -d "$SRC/crates" ]; then
  ROOT="$SRC"
  if ! touch "$ROOT/.w" 2>/dev/null; then          # read-only (e.g. /kaggle/input): copy out
    ROOT=/kaggle/working/epidemicsim
    mkdir -p "$ROOT" && cp -r "$SRC/." "$ROOT/"
  else
    rm -f "$ROOT/.w"
  fi
else
  ROOT=/kaggle/working/epidemicsim
  mkdir -p "$ROOT"
  ZIP=$(ls /kaggle/input/*/*.zip /kaggle/input/*.zip 2>/dev/null | head -1)
  if [ -n "$ZIP" ]; then
    unzip -q -o "$ZIP" -d "$ROOT"
  elif [ -n "${REPO_URL:-}" ]; then
    git clone --depth 1 "$REPO_URL" "$ROOT"
  fi
  # a zip may contain one top-level folder
  [ -f "$ROOT/Cargo.toml" ] || ROOT=$(dirname "$(find "$ROOT" -maxdepth 2 -name Cargo.toml | head -1)")
fi
cd "$ROOT" || { echo "no repo found (set REPO_URL or attach a zip dataset)"; exit 1; }

OUT=results/kaggle
mkdir -p "$OUT/logs" "$OUT/bench" "$OUT/gpu"
: > "$OUT/steps.tsv"
T="${STEP_TIMEOUT:-3000}"
if [ "${QUICK:-0}" = 1 ]; then REPS=2; DAYS=10; AG="100000"; else REPS=5; DAYS=20; AG="100000,1000000"; fi

# step NAME CMD...: run, log, record exit code; never abort the script.
step() {
  local name="$1"; shift
  echo "=== $name: $*"
  local t0=$SECONDS
  timeout "$T" "$@" >"$OUT/logs/$name.log" 2>&1
  local rc=$?
  printf '%s\t%s\t%ss\n' "$name" "$rc" "$((SECONDS - t0))" >> "$OUT/steps.tsv"
  [ $rc -ne 0 ] && echo "    FAILED rc=$rc (see $OUT/logs/$name.log)"
  return 0
}
sh_step() { local name="$1"; shift; step "$name" bash -c "$*"; }

# ---- 0. machine info ----
sh_step machine_info 'python3 -I - <<PY
import json, subprocess, platform
def run(c):
    try: return subprocess.run(c, shell=True, capture_output=True, text=True, timeout=60).stdout.strip()
    except Exception as e: return "error: %s" % e
info = {
  "gpu": run("nvidia-smi --query-gpu=name,memory.total,driver_version,compute_cap --format=csv,noheader"),
  "nvcc": run("nvcc --version | tail -1"),
  "cuda_driver_api": run("nvidia-smi | grep -o \"CUDA Version: [0-9.]*\""),
  "cpu": run("grep -m1 \"model name\" /proc/cpuinfo | cut -d: -f2").strip(),
  "cpu_cores": run("nproc"),
  "avx2": "avx2" in run("grep -m1 flags /proc/cpuinfo"),
  "os": platform.platform(),
  "ncu": run("command -v ncu"),
  "nvc": run("command -v nvc"),
  "date_utc": run("date -u +%FT%TZ"),
}
json.dump(info, open("'"$OUT"'/machine.json", "w"), indent=2)
print(json.dumps(info, indent=2))
PY'
sh_step nvidia_smi 'nvidia-smi; nvidia-smi -q | head -60'
sh_step nvcc_version 'nvcc --version'

# ---- 1. toolchain ----
sh_step apt_deps 'apt-get update -qq && apt-get install -y -qq build-essential pkg-config curl zip unzip python3 libssl-dev'
if ! command -v cargo >/dev/null 2>&1 && [ ! -x "$HOME/.cargo/bin/cargo" ]; then
  sh_step rustup 'curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable'
fi
# shellcheck disable=SC1091
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
step rust_version rustc --version

# ---- 2. build with the cuda feature (no MPI: needs an MPI install, not useful here) ----
step build cargo build --release -p epidemicsim-cli --no-default-features --features cuda
BIN=target/release/epidemicsim

# ---- 3. correctness: golden tests (any variant not matching sequential fails here) ----
step test_cuda_unit env EPI_REQUIRE_CUDA=1 cargo test --release -p epidemicsim-backends --features cuda --lib cuda -- --nocapture
step test_cuda_golden env EPI_REQUIRE_CUDA=1 cargo test --release -p epidemicsim-backends --features cuda --test golden cuda -- --nocapture
step test_golden_table env EPI_REQUIRE_CUDA=1 cargo test --release -p epidemicsim-backends --features cuda --test golden_table -- --nocapture
step test_avx2_golden env EPI_REQUIRE_AVX2=1 cargo test --release -p epidemicsim-backends --test golden avx2 -- --nocapture

# ---- 4. synthetic city (generated in-process; no .pbf downloads): cuda vs sequential ----
step synth_run_seq "$BIN" run --backend sequential --agents 200000 --days 60 --out "$OUT/synth_sequential"
step synth_run_cuda "$BIN" run --backend cuda --agents 200000 --days 60 --out "$OUT/synth_cuda"
sh_step synth_compare 'python3 -I - <<PY
import json
a = json.load(open("'"$OUT"'/synth_sequential/summary.json")); b = json.load(open("'"$OUT"'/synth_cuda/summary.json"))
for d in (a, b): d.pop("backend", None)
same = a == b
print("sequential vs cuda summary identical:", same)
json.dump({"identical": same, "sequential": a, "cuda": b}, open("'"$OUT"'/synth_compare.json", "w"), indent=2)
raise SystemExit(0 if same else 1)
PY'

# ---- 5. benchmarks ----
step bench_cuda_blocks "$BIN" bench --suite cuda --agents "$AG" --days "$DAYS" --reps "$REPS" --gpu-out "$OUT/gpu"
step bench_cuda_variants "$BIN" bench --suite cuda-variants --agents 1000000 --cuda-block 256 --days "$DAYS" --reps "$REPS" --gpu-out "$OUT/gpu"
step bench_warp "$BIN" bench --suite warp --agents 1000000 --cuda-block 256 --days "$DAYS" --reps "$REPS" --gpu-out "$OUT/gpu"
# CPU baselines on the same machine (for honest GPU-vs-CPU ratios) and the AVX2 kernel.
step bench_sequential "$BIN" bench --backend sequential --agents "$AG" --days "$DAYS" --reps "$REPS" --out "$OUT/bench"
step bench_rayon "$BIN" bench --backend rayon --schedule guided --chunk 64 --threads 1,"$(nproc)" --agents "$AG" --days "$DAYS" --reps "$REPS" --out "$OUT/bench"
step bench_simd_scalar "$BIN" bench --backend simd --simd scalar --agents "$AG" --days "$DAYS" --reps "$REPS" --out "$OUT/bench"
step bench_simd_avx2 "$BIN" bench --backend simd --simd avx2 --agents "$AG" --days "$DAYS" --reps "$REPS" --out "$OUT/bench"
step bench_cuda_default "$BIN" bench --backend cuda --agents "$AG" --days "$DAYS" --reps "$REPS" --out "$OUT/bench"

# ---- 6. ncu (branch efficiency / warp execution efficiency); expected to fail without profiler permission ----
if command -v ncu >/dev/null 2>&1; then
  METRICS=smsp__sass_average_branch_targets_threads_uniform.pct,smsp__thread_inst_executed_per_inst_executed.ratio,smsp__warps_launched.sum,gpu__time_duration.sum
  for v in "branch:0" "pred:0" "branch:4"; do
    u=${v%%:*}; p=${v##*:}
    step "ncu_${u}_part${p}" ncu --csv --metrics "$METRICS" -k "regex:k_update" --launch-count 12 \
      "$BIN" run --backend cuda --cuda-update "$u" --cuda-partition "$p" --agents 500000 --days 6 --out "$OUT/ncu_run"
    cp "$OUT/logs/ncu_${u}_part${p}.log" "$OUT/ncu_${u}_part${p}.csv" 2>/dev/null
  done
else
  echo "ncu not installed: branch / warp execution efficiency NOT collected" | tee "$OUT/ncu_notes.txt"
fi

# ---- 7. PTX at several optimisation levels ----
step ptx env OUT="$OUT/ptx" bash scripts/ptx.sh
cp results/ptx/analysis.md "$OUT/ptx/analysis.md" 2>/dev/null

# ---- 8. OpenACC (bonus): try only if a compiler is present or TRY_NVHPC=1 ----
{
  echo "# OpenACC attempt"; echo; echo "Source: crates/kernels-c/openacc/acc_bench.c"; echo
} > "$OUT/openacc_notes.md"
ACC=""
if command -v nvc >/dev/null 2>&1; then
  ACC=nvc; ACCCMD=(nvc -acc -gpu=cc75 -O3 -Minfo=accel crates/kernels-c/openacc/acc_bench.c -o "$OUT/acc_bench")
elif [ "${TRY_NVHPC:-0}" = 1 ]; then
  echo "TRY_NVHPC=1: attempting the NVHPC apt repository install (large download)." >> "$OUT/openacc_notes.md"
  sh_step nvhpc_install 'curl -fsSL https://developer.download.nvidia.com/hpc-sdk/ubuntu/DEB-GPG-KEY-NVIDIA-HPC-SDK | gpg --dearmor -o /usr/share/keyrings/nvidia-hpcsdk-archive-keyring.gpg &&
    echo "deb [signed-by=/usr/share/keyrings/nvidia-hpcsdk-archive-keyring.gpg] https://developer.download.nvidia.com/hpc-sdk/ubuntu/amd64 /" > /etc/apt/sources.list.d/nvhpc.list &&
    apt-get update -qq && apt-get install -y -qq nvhpc-24-11'
  NVC=$(ls /opt/nvidia/hpc_sdk/Linux_x86_64/*/compilers/bin/nvc 2>/dev/null | head -1)
  [ -n "$NVC" ] && { ACC=nvc; ACCCMD=("$NVC" -acc -gpu=cc75 -O3 -Minfo=accel crates/kernels-c/openacc/acc_bench.c -o "$OUT/acc_bench"); }
fi
if [ -z "$ACC" ] && gcc -fopenacc -foffload=nvptx-none -x c /dev/null -c -o /dev/null 2>/dev/null; then
  ACC=gcc; ACCCMD=(gcc -O3 -fopenacc -foffload=nvptx-none crates/kernels-c/openacc/acc_bench.c -o "$OUT/acc_bench")
fi
if [ -n "$ACC" ]; then
  step openacc_build "${ACCCMD[@]}"
  step openacc_run "$OUT/acc_bench" 1000000 80
  { echo "Compiler: $ACC. Build/run logs: logs/openacc_build.log, logs/openacc_run.log (exit codes in steps.tsv)."; } >> "$OUT/openacc_notes.md"
else
  {
    echo "SKIPPED: no OpenACC compiler available. \`nvc\` (NVHPC) is not installed on the Kaggle image and"
    echo "the GCC nvptx offload compiler (apt \`gcc-offload-nvptx\`) was not usable. NVHPC is a multi-GB"
    echo "download; re-run with TRY_NVHPC=1 to attempt it. The source was only checked on the host"
    echo "(pragmas ignored): it compiles and matches its serial reference. No GPU result exists."
  } >> "$OUT/openacc_notes.md"
fi

# ---- 9. zip ----
rm -f results/kaggle.zip
if command -v zip >/dev/null 2>&1; then
  zip -qr results/kaggle.zip "$OUT"
else
  python3 -I -c "import shutil; shutil.make_archive('results/kaggle', 'zip', 'results', 'kaggle')"
fi
[ -d /kaggle/working ] && cp results/kaggle.zip /kaggle/working/epidemicsim_kaggle_results.zip
echo "=== done. steps (name, exit code, seconds):"; cat "$OUT/steps.tsv"
echo "zip: $ROOT/results/kaggle.zip"
