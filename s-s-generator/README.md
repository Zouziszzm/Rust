---
title: Static Site Generator
subtext: A small static site generator written in Rust with markdown, YAML front-matter, and HTML templates.
order: 4
portfolioMode: summary
stack: [Rust, pulldown-cmark, serde, clap]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# s-s-generator

## Portfolio

**s-s-generator** is a small static site generator written in Rust. You write content as markdown files with YAML front-matter, point the tool at a template, and it walks your content tree and writes HTML into `dist/` — the same mental model as Hugo or Eleventy, but small enough to read the whole implementation in an afternoon.

The pipeline is deliberately simple: parse front-matter with `serde_yaml`, convert markdown body to HTML with `pulldown-cmark`, inject `{{title}}`, `{{date}}`, and `{{content}}` into an HTML layout, and mirror the folder structure under `dist/`. Static assets like CSS sitting next to markdown get copied through unchanged. A `--file` flag lets you rebuild a single page while iterating instead of the whole site.

I built it to understand what frameworks like Next and Astro abstract away — how routing maps to files, how templates compose, and how a recursive directory walk turns into a publishable tree. It's not meant to compete with production SSGs; it's a clear reference for the core mechanics.

Nested paths work out of the box (`content/posts/hello.md` → `dist/posts/hello.html`), and `--clean` wipes the output folder before each full build so stale pages don't linger.

### Usage

```bash
cargo run -- build --clean
```

```bash
ssg build --input content --output dist --template templates/default.html --clean
```

## Development

```bash
cargo test
```

## Stack

- **Rust** (edition 2021)
- **pulldown-cmark** — markdown to HTML
- **serde + serde_yaml** — front-matter deserialization
- **walkdir** — recursive directory traversal
- **clap** — CLI
- **anyhow** — error handling
