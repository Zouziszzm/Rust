---
title: Lumen
subtext: Rust engine for Lumen — Anki import, SQLite persistence, FSRS scheduling, card templates, and HTML rendering.
order: 7
portfolioMode: summary
date: Jan 2026
stack: [Rust, SQLite, FSRS]
extent: [Develop]
contribution: Solo Developer
category: Personal
relatedProjects:
  - id: tauri--lumen
    label: Tauri
    role: frontend
  - id: flutter--lumen
    label: Flutter
    role: frontend
---

# lumen-core

## Portfolio

**lumen-core** is the shared Rust engine behind [Lumen](https://github.com/Zouziszzm/Tauri/tree/main/lumen) — a local-first spaced-repetition app. It handles everything below the UI: SQLite persistence, Anki `.apkg` / `.colpkg` import (ZIP/zstd), card templates, HTML rendering, and **FSRS** scheduling via `rs-fsrs`.

The crate is consumed by the Tauri macOS shell and mirrors the same domain model the Flutter app implements independently in Dart. Store layer, scheduler, import pipeline, and template engine all live here — so the desktop app stays thin: Tauri commands call into `lumen-core`, and the vanilla HTML UI just renders state.

This is an independent implementation of the public Anki package format. No Anki source code. Scheduling starts fresh after import in v1.

Frontends for this engine are available as **Tauri** (macOS desktop) and **Flutter** (iOS + macOS).

### Usage

```bash
cargo test
cargo doc --open --no-deps
```

## Development

```bash
cargo test -p lumen-core
```

**Modules:** `db`, `import`, `store`, `scheduler`, `template`, `html`, `models`.

**Stack:** Rust, rusqlite, rs-fsrs, serde, zip, zstd, regex, chrono.
