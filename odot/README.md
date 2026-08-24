---
title: odot
subtext: Todo app. Rust API + Next.js UI + Postgres.
order: 1
portfolioMode: summary
stack: [Rust, Next.js, PostgreSQL]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# odot

## Portfolio

**odot** is a full-stack todo app — Rust API, Next.js frontend, and Postgres — designed around a frictionless local dev loop. The goal was to wire a realistic three-tier setup without the usual pain of starting services manually, managing ports, or forgetting which terminal runs what.

The Rust backend owns persistence, validation, and HTTP routes. The Next.js UI consumes the API and handles interaction — lists, forms, and state on the client. Postgres sits underneath as the source of truth. Together they mirror how a small production app is actually structured, just scoped down to todos.

What makes it practical day-to-day is the orchestration: one `npm run dev` brings up the API, the UI, and the database. When you're done, Ctrl+C stops everything — apps, connections, and the local `odot` database — so you're not left with orphaned processes. Docker Compose and k8s manifests are there if you want to run it closer to a deployed environment.

I built this to explore Rust on the API side while keeping a familiar React/Next surface for the UI, and to practice owning the full vertical slice from schema to screen.

### Usage

```bash
npm run dev
```

- UI: http://localhost:3000
- API: http://localhost:8080

## Development

```bash
cp backend/.env.example backend/.env
cp frontend/.env.local.example frontend/.env.local
npm install
npm run install:all
```

```bash
docker compose up --build
cd backend && cargo test
kubectl apply -f k8s/
```
