#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export DATABASE_URL="${DATABASE_URL:-postgres://op_exence:op_exence@localhost:5432/op_exence}"
export CORS_ORIGIN="${CORS_ORIGIN:-http://localhost:8081}"
export API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
export RUST_LOG="${RUST_LOG:-info,op_exence=debug}"

# Java 21 (Homebrew openjdk@21 is keg-only — set JAVA_HOME before Gradle)
source "$ROOT/scripts/ensure-java.sh"

cleanup() {
  if [[ "${CLEANED:-0}" == "1" ]]; then
    return
  fi
  CLEANED=1
  echo ""
  echo "Stopping..."
  bash "$ROOT/scripts/stop-dev.sh"
}

trap cleanup INT TERM

bash "$ROOT/scripts/kill-apps.sh"
bash "$ROOT/scripts/start-postgres.sh"

if ! command -v npx >/dev/null 2>&1; then
  echo "Node.js/npx required. Install Node.js or run: npm install" >&2
  exit 1
fi

echo ""
echo "Starting backend (Rust :8080) and frontend (Spring :8081)..."
echo "Dashboard: http://localhost:8081"
echo "Press Ctrl+C to stop everything"
echo ""

npx concurrently -k -n backend,frontend -c green,cyan \
  "npm run dev:backend" \
  "npm run dev:frontend"
