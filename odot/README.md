---
title: odot
subtext: Todo app. Rust API + Next.js UI + Postgres.
stack: [Rust, Next.js, PostgreSQL]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# odot

Todo app. Rust API + Next.js UI + Postgres.

## Setup

```bash
cp backend/.env.example backend/.env
cp frontend/.env.local.example frontend/.env.local
npm install
npm run install:all
```

## Dev

```bash
npm run dev
```

- UI: http://localhost:3000
- API: http://localhost:8080

In that same terminal, press **Ctrl+C** to stop. That kills the apps, drops the `odot` database, and stops Postgres.

You can also run `npm run stop` from another terminal.

## Other

```bash
docker compose up --build
cd backend && cargo test
kubectl apply -f k8s/
```
