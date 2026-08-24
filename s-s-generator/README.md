---
title: Static Site Generator
subtext: A small static site generator written in Rust with markdown, YAML front-matter, and HTML templates.
stack: [Rust, pulldown-cmark, serde, clap]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# s-s-generator

A small static site generator written in Rust. Reads markdown files, parses YAML front-matter, renders HTML with a layout template, and walks a content directory to produce a `dist/` output.

## Stack

- **Rust** (edition 2021)
- **pulldown-cmark** — markdown to HTML
- **serde + serde_yaml** — front-matter deserialization
- **walkdir** — recursive directory traversal
- **clap** — CLI
- **anyhow** — error handling

## Usage

From this directory:

```bash
cargo run -- build --clean
```

Options:

```bash
ssg build \
  --input content \
  --output dist \
  --template templates/default.html \
  --clean
```

Debug a single file:

```bash
cargo run -- build --file content/index.md
```

## Content layout

```
content/
  index.md          → dist/index.html
  style.css         → dist/style.css
  posts/hello.md    → dist/posts/hello.html
```

Markdown files use YAML front-matter between `---` delimiters:

```markdown
---
title: My Page
date: 2026-08-24
---

# Body starts here
```

Templates use `{{title}}`, `{{date}}`, and `{{content}}` placeholders.

## Tests

```bash
cargo test
```
