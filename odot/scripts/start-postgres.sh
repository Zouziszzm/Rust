#!/usr/bin/env bash
set -euo pipefail

# Start Postgres and recreate empty odot DB for local dev.

if ! command -v brew >/dev/null 2>&1; then
  echo "brew not found; start Postgres yourself" >&2
  exit 1
fi

if brew services list 2>/dev/null | grep -q 'postgresql@16'; then
  brew services start postgresql@16 >/dev/null
elif brew services list 2>/dev/null | grep -q '^postgresql '; then
  brew services start postgresql >/dev/null
fi

# Wait until ready
for _ in $(seq 1 30); do
  if pg_isready -h 127.0.0.1 -p 5432 >/dev/null 2>&1; then
    break
  fi
  sleep 0.3
done

if ! pg_isready -h 127.0.0.1 -p 5432 >/dev/null 2>&1; then
  echo "postgres did not become ready" >&2
  exit 1
fi

psql -h 127.0.0.1 -d postgres -v ON_ERROR_STOP=1 <<'SQL' >/dev/null
DO $$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'odot') THEN
    CREATE ROLE odot LOGIN PASSWORD 'odot' SUPERUSER;
  END IF;
END
$$;

SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = 'odot' AND pid <> pg_backend_pid();

DROP DATABASE IF EXISTS odot;
CREATE DATABASE odot OWNER odot;
SQL

echo "postgres ready (empty odot db)"
