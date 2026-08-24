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

Todo app. Rust API + Next.js UI + Postgres.

## Development

```bash
cp backend/.env.example backend/.env
cp frontend/.env.local.example frontend/.env.local
npm install
npm run install:all
npm run dev
```

- UI: http://localhost:3000
- API: http://localhost:8080

```bash
docker compose up --build
cd backend && cargo test
kubectl apply -f k8s/
```
