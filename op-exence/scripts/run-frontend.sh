#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
source "$ROOT/scripts/ensure-java.sh"
cd "$ROOT/frontend"
exec ./gradlew bootRun --no-daemon
