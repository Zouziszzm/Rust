---
title: m-search
subtext: Multi-threaded file name and content search for Rust.
order: 5
portfolioMode: summary-collapsible
detailsCollapsed: true
stack: [Rust]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# m-search

## Portfolio

Multi-threaded file name and content search for Rust. Searches file names (glob) and file contents (regex) in parallel, with a CLI and reusable library API.

## Development

```bash
cargo build --release
cargo test
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
