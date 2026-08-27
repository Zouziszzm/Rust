---
title: op-exence
subtext: Expense tracker with a Rust API, Java/Spring frontend, and Postgres. Supports categories, shops, and monthly summaries.
order: 2
portfolioMode: summary
date: Feb 2026
stack: [Rust, Java, Spring, PostgreSQL]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# op-exence

## Portfolio

**op-exence** is a personal expense tracker built with a Rust API, Java/Spring frontend, and Postgres. It exists to make everyday spending visible — what you bought, where, under which category, and how it adds up month over month.

On the backend, Rust handles the API layer: categories, shops, expenses, and aggregated summaries. The Spring frontend renders server-side pages for listing, creating, and editing records — forms, tables, and a dashboard view without a separate SPA build step. Postgres stores everything relationally so filters and monthly rollups stay straightforward.

The split stack was intentional: keep the data path fast and typed in Rust, while leaning on Spring's mature templating and form handling for the UI. Categories group spending (food, transport, bills), shops tag where money went, and monthly summaries give a quick read on habits without exporting to a spreadsheet.

It's a practical exercise in connecting two ecosystems — Rust and the JVM — around one database, with a dev workflow that still feels like a single project (`npm run dev` at the root).

### Usage

```bash
npm run dev
```

- UI: http://localhost:8081
- API: http://localhost:8080

## Development

```bash
docker compose up --build
cd backend && cargo test
```
