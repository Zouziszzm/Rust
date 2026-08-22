#!/usr/bin/env bash
set -euo pipefail

kill_port() {
  local port="$1"
  local pids
  pids=$(lsof -ti tcp:"$port" -sTCP:LISTEN 2>/dev/null || true)
  if [[ -n "${pids}" ]]; then
    echo "Stopping :$port ($pids)"
    kill $pids 2>/dev/null || true
    sleep 0.4
    pids=$(lsof -ti tcp:"$port" -sTCP:LISTEN 2>/dev/null || true)
    if [[ -n "${pids}" ]]; then
      kill -9 $pids 2>/dev/null || true
    fi
  fi
}

kill_port 8080
kill_port 8081
