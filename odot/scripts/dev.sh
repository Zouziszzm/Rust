#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cleanup() {
  # Avoid running cleanup twice (INT + EXIT)
  if [[ "${CLEANED:-0}" == "1" ]]; then
    return
  fi
  CLEANED=1
  echo ""
  echo "stopping…"
  bash "$ROOT/scripts/stop-dev.sh"
}

trap cleanup INT TERM EXIT

bash "$ROOT/scripts/stop-dev.sh"
bash "$ROOT/scripts/start-postgres.sh"

npx concurrently -k -n backend,frontend -c green,cyan \
  "npm run dev:backend" \
  "npm run dev:frontend"
