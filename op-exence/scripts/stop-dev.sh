#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

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

DB_NAME="op_exence"

if pg_isready -h 127.0.0.1 -p 5432 >/dev/null 2>&1; then
  psql -h 127.0.0.1 -d postgres -v ON_ERROR_STOP=1 <<SQL >/dev/null 2>&1 || true
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = '${DB_NAME}' AND pid <> pg_backend_pid();

DROP DATABASE IF EXISTS ${DB_NAME};
SQL
  echo "Dropped database: ${DB_NAME}"
fi

if command -v brew >/dev/null 2>&1; then
  if brew services list 2>/dev/null | grep -q 'postgresql@16.*started'; then
    echo "Stopping postgresql@16"
    brew services stop postgresql@16 >/dev/null
  elif brew services list 2>/dev/null | grep -q 'postgresql.*started'; then
    echo "Stopping postgresql"
    brew services stop postgresql >/dev/null || true
  fi
fi

echo "Stopped"
