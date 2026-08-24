---
title: m-search
subtext: Multi-threaded file name and content search for Rust.
stack: [Rust]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# m-search

Multi-threaded file name and content search for Rust.

## Features

- Search file names/paths with glob patterns
- Search file contents with regex
- Parallel directory traversal via the `ignore` crate (respects `.gitignore`)
- Library API and CLI

## Build

```bash
cargo build --release
```

## Usage

```bash
m-search "TODO" .
m-search "*.rs" ./src
m-search -c "fn main" .
m-search -i "error" ./logs
m-search --json "pattern" .
m-search -j 8 --max-depth 3 "pattern" .
```

## Library

```rust
use std::path::PathBuf;
use m_search::{search, SearchConfig};

let config = SearchConfig::from_pattern(
    "*.rs",
    vec![PathBuf::from("./src")],
    true,
    false,
    false,
    false,
    None,
    None,
)?;

let results = search(&config)?;
```

## Test

```bash
cargo test
```
