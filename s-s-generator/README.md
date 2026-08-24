---
title: Static Site Generator
subtext: A small static site generator written in Rust with markdown, YAML front-matter, and HTML templates.
order: 4
portfolioMode: metadata-only
stack: [Rust, pulldown-cmark, serde, clap]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# s-s-generator

## Portfolio

A small static site generator written in Rust. Reads markdown files, parses YAML front-matter, renders HTML with a layout template, and walks a content directory to produce a `dist/` output.

## Development

```bash
cargo run -- build --clean
cargo test
```

## Stack

- **Rust** (edition 2021)
- **pulldown-cmark** — markdown to HTML
- **serde + serde_yaml** — front-matter deserialization
- **walkdir** — recursive directory traversal
- **clap** — CLI
- **anyhow** — error handling
