---
title: m-search
subtext: Multi-threaded file name and content search for Rust.
order: 5
portfolioMode: summary-collapsible
detailsCollapsed: true
date: Aug 2026
stack: [Rust]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# m-search

## Portfolio

**m-search** is a multi-threaded file search tool and library for Rust. It searches file names with glob patterns and file contents with regex, walking directories in parallel while respecting `.gitignore` rules via the `ignore` crate — so you're not wading through `target/` or `node_modules/` unless you mean to.

From the terminal it behaves like a focused search utility: point it at a path, give it a pattern, and get matches back quickly on large trees. Flags cover the cases you actually hit — content search (`-c`), case-insensitive (`-i`), JSON output for scripting (`--json`), thread count (`-j`), and max depth (`--max-depth`). As a library, the same `SearchConfig` + `search()` API drops into other Rust tools when you need programmatic discovery.

The design goal was parallel I/O without pulling in an async runtime. Worker threads chew through the file list; results stream back as they're found. It's not ripgrep — and doesn't try to be — but it's a solid, understandable baseline for "find this string in my repo" or "list every `.rs` file under `src/`."

Good for learning how directory traversal, glob matching, and thread pools fit together in a real CLI, and for day-to-day grepping when you want something you own end-to-end.

### Usage

```bash
m-search "TODO" .
m-search "*.rs" ./src
m-search -c "fn main" .
m-search --json "pattern" .
```

### Library

```rust
use std::path::PathBuf;
use m_search::{search, SearchConfig};

let config = SearchConfig::from_pattern("*.rs", vec![PathBuf::from("./src")], true, false, false, false, None, None)?;
let results = search(&config)?;
```

## Development

```bash
cargo build --release
cargo test
```

## Usage

```bash
m-search -i "error" ./logs
m-search -j 8 --max-depth 3 "pattern" .
```
