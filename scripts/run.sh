#!/usr/bin/env bash
# Starts the Rust server, which also serves the built web app (web/dist): one command, one port.
# Usage: scripts/run.sh [port]      env: HOST (default 127.0.0.1), PORT (default 8080)
set -euo pipefail
cd "$(dirname "$0")/.."
HOST="${HOST:-127.0.0.1}"; PORT="${1:-${PORT:-8080}}"
[ -x target/release/epidemicsim ] || { echo "target/release/epidemicsim missing: run scripts/setup.sh first." >&2; exit 1; }
[ -f web/dist/index.html ] || { echo "web/dist missing: run scripts/setup.sh (or: cd web && npm ci && npm run build)." >&2; exit 1; }
if (exec 3<>"/dev/tcp/$HOST/$PORT") 2>/dev/null; then
  echo "Port $PORT on $HOST is already in use. Use another one: scripts/run.sh 8081 (or stop the other process)." >&2; exit 1
fi
echo "epidemicsim: http://$HOST:$PORT  (#/studio, #/lab)"
exec ./target/release/epidemicsim serve --addr "$HOST:$PORT"
