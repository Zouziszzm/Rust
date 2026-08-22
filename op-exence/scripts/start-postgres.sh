#!/usr/bin/env bash
set -euo pipefail

# Start local Postgres (Homebrew) and ensure op_exence database exists.

DB_NAME="op_exence"
DB_USER="op_exence"
DB_PASS="op_exence"

if ! command -v brew >/dev/null 2>&1; then
  echo "Homebrew not found. Install PostgreSQL locally and create:" >&2
  echo "  user=$DB_USER password=$DB_PASS database=$DB_NAME" >&2
  exit 1
fi

if brew services list 2>/dev/null | grep -q 'postgresql@16'; then
  brew services start postgresql@16 >/dev/null
elif brew services list 2>/dev/null | grep -q '^postgresql '; then
  brew services start postgresql >/dev/null
else
  echo "PostgreSQL not found via Homebrew. Install: brew install postgresql@16" >&2
  exit 1
fi

for _ in $(seq 1 30); do
  if pg_isready -h 127.0.0.1 -p 5432 >/dev/null 2>&1; then
    break
  fi
  sleep 0.3
done

if ! pg_isready -h 127.0.0.1 -p 5432 >/dev/null 2>&1; then
  echo "PostgreSQL did not become ready on localhost:5432" >&2
  exit 1
fi

psql -h 127.0.0.1 -d postgres -v ON_ERROR_STOP=1 <<SQL >/dev/null
DO \$\$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '${DB_USER}') THEN
    CREATE ROLE ${DB_USER} LOGIN PASSWORD '${DB_PASS}';
  END IF;
END
\$\$;

SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = '${DB_NAME}' AND pid <> pg_backend_pid();

DROP DATABASE IF EXISTS ${DB_NAME};
CREATE DATABASE ${DB_NAME} OWNER ${DB_USER};
SQL

echo "PostgreSQL ready (database: ${DB_NAME}, user: ${DB_USER})"
