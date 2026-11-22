#!/usr/bin/env bash
# Compile every CUDA kernel file to PTX at several optimisation levels and count instruction
# categories. Needs nvcc (Kaggle / any CUDA toolkit); cannot run on a Mac.
#   scripts/ptx.sh            -> results/ptx/*.ptx, results/ptx/summary.json
# Env: ARCH=sm_75 (default: detected from nvidia-smi, else sm_75), OUT=results/ptx
# Levels: O0 O1 O2 O3, O3fm (-O3 --use_fast_math), and G (-G: device debug, no device
# optimisation) as an extra reference, because `nvcc -O<n>` mostly controls the HOST compiler and
# may leave the device PTX identical across levels; summary.json records file hashes so identical
# PTX is visible instead of assumed.
set -u
cd "$(dirname "$0")/.."
OUT="${OUT:-results/ptx}"
CU=crates/kernels-c/cuda
mkdir -p "$OUT"
if ! command -v nvcc >/dev/null 2>&1; then
  echo "nvcc not found: cannot generate PTX" >&2
  exit 1
fi
ARCH="${ARCH:-}"
if [ -z "$ARCH" ] && command -v nvidia-smi >/dev/null 2>&1; then
  cc=$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader 2>/dev/null | head -1 | tr -d '. ')
  [ -n "$cc" ] && ARCH="sm_$cc"
fi
ARCH="${ARCH:-sm_75}"
echo "nvcc: $(nvcc --version | tail -1)  arch=$ARCH" | tee "$OUT/nvcc_info.txt"
declare -A FLAGS=([O0]="-O0" [O1]="-O1" [O2]="-O2" [O3]="-O3" [O3fm]="-O3 --use_fast_math" [G]="-G")
fail=0
for f in place update stats; do
  for lvl in O0 O1 O2 O3 O3fm G; do
    # shellcheck disable=SC2086
    nvcc -ptx -arch="$ARCH" -I "$CU" ${FLAGS[$lvl]} "$CU/$f.cu" -o "$OUT/${f}_${lvl}.ptx" 2>"$OUT/${f}_${lvl}.log" \
      || { echo "FAILED: $f $lvl (see $OUT/${f}_${lvl}.log)"; fail=1; }
  done
done
python3 -I - "$OUT" "$ARCH" <<'PY'
import hashlib, json, os, subprocess, sys
out, arch = sys.argv[1], sys.argv[2]
count = os.path.join("scripts", "ptx_count.py")
summary = {"arch": arch, "files": {}}
for name in sorted(os.listdir(out)):
    if not name.endswith(".ptx"):
        continue
    path = os.path.join(out, name)
    data = open(path, "rb").read()
    entry = {"sha256": hashlib.sha256(data).hexdigest()[:16], "bytes": len(data)}
    try:
        r = subprocess.run([sys.executable, "-I", count, path], capture_output=True, text=True, check=True)
        entry.update(json.loads(r.stdout))
    except Exception as e:  # keep going: one bad file must not hide the rest
        entry["error"] = str(e)
    summary["files"][name[:-4]] = entry
json.dump(summary, open(os.path.join(out, "summary.json"), "w"), indent=2)
print("wrote", os.path.join(out, "summary.json"))
PY
exit $fail
