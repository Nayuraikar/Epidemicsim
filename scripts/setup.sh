#!/usr/bin/env bash
# One-shot setup for macOS and Linux: checks prerequisites, builds the Rust binary (release) and the web app.
# Usage: scripts/setup.sh [--cities bengaluru,mumbai,chennai,delhi]   (default: no city data, fast)
# Optional pieces (OpenMP, MPI, Metal, CUDA) never make this fail: they are skipped with a warning.
set -euo pipefail
cd "$(dirname "$0")/.."

CITIES=""
while [ $# -gt 0 ]; do
  case "$1" in
    --cities) CITIES="${2:-}"; shift 2 ;;
    --cities=*) CITIES="${1#*=}"; shift ;;
    -h|--help) sed -n 2,4p "$0"; exit 0 ;;
    *) echo "unknown option: $1 (see --help)" >&2; exit 2 ;;
  esac
done

ok()   { printf '  ok    %s\n' "$*"; }
warn() { printf '  WARN  %s\n' "$*"; }
die()  { printf '  FAIL  %s\n' "$*" >&2; exit 1; }
OS="$(uname -s)"

echo "== Prerequisites"
command -v rustc >/dev/null && command -v cargo >/dev/null || die "Rust not found. Install it from https://rustup.rs then re-run."
ok "rust $(rustc --version | cut -d' ' -f2)"
command -v node >/dev/null && command -v npm >/dev/null || die "Node.js/npm not found. Install Node 20.19+ or 22.12+ (https://nodejs.org)."
NODE_V="$(node -v | sed 's/^v//')"; NMAJ="${NODE_V%%.*}"; NMIN="$(echo "$NODE_V" | cut -d. -f2)"
if [ "$NMAJ" -lt 20 ] || { [ "$NMAJ" -eq 20 ] && [ "$NMIN" -lt 19 ]; } || { [ "$NMAJ" -eq 21 ]; } || { [ "$NMAJ" -eq 22 ] && [ "$NMIN" -lt 12 ]; }; then
  die "Node $NODE_V is too old for Vite 8 (need ^20.19 or >=22.12). Upgrade, e.g. 'brew install node' or nvm."
fi
ok "node $NODE_V"
command -v cc >/dev/null || command -v clang >/dev/null || die "No C compiler. macOS: xcode-select --install. Linux: install build-essential."

# OpenMP (optional): without it the OpenMP backend is simply left out.
if [ "$OS" = Darwin ]; then
  for p in "${LIBOMP_PREFIX:-}" "$(brew --prefix libomp 2>/dev/null || true)" /opt/homebrew/opt/libomp /usr/local/opt/libomp; do
    if [ -n "$p" ] && [ -f "$p/lib/libomp.dylib" ]; then export LIBOMP_PREFIX="$p"; ok "libomp at $p"; break; fi
  done
  [ -n "${LIBOMP_PREFIX:-}" ] || warn "libomp not found: OpenMP backend disabled. Enable with: brew install libomp"
else
  warn "OpenMP on Linux needs gcc + libgomp (package libgomp1 / gcc); the build skips it automatically if missing."
fi

# MPI (optional): only the multi-process `compare` / travel runs need it.
FEATURES=()
if command -v mpicc >/dev/null; then
  ok "MPI found ($(command -v mpicc))"
else
  warn "MPI (mpicc) not found: building without MPI. Enable with: brew install open-mpi  |  apt install libopenmpi-dev openmpi-bin libclang-dev"
  FEATURES=(--no-default-features)
fi
[ "$OS" = Darwin ] && ok "Metal GPU backend: built automatically on macOS" || warn "Metal backend is macOS only; CUDA needs 'cargo build --features cuda' and an NVIDIA GPU (not built here)."

echo "== Building the simulator (release; first build takes a few minutes)"
cargo build --release -q -p epidemicsim-cli ${FEATURES[@]+"${FEATURES[@]}"} 2>&1 | grep -v '^$' || true
[ -x target/release/epidemicsim ] || die "cargo build failed; re-run 'cargo build --release -p epidemicsim-cli' to see the error."
ok "target/release/epidemicsim"

echo "== Building the web app"
( cd web && npm ci --no-audit --no-fund --loglevel=error && npm run build --silent ) || die "web build failed (see above)."
ok "web/dist"

if [ -n "$CITIES" ]; then
  echo "== Building city data: $CITIES (downloads OpenStreetMap extracts; large and slow, see README)"
  ./target/release/epidemicsim pipeline ${CITIES//,/ } || warn "city build failed; the app still works for Lab and Replay."
else
  echo "== City data: skipped (Lab and Replay work without it). Run live needs: ./target/release/epidemicsim pipeline <city>"
fi

cat <<'M'

Done. Start the app (one process, one port):
  scripts/run.sh                 # http://127.0.0.1:8080
Dev mode (hot reload): scripts/run.sh in one terminal, then  cd web && npm run dev
M
