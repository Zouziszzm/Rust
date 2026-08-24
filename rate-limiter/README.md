---
title: rate-limiter
subtext: A small, std-only token-bucket rate limiter for Rust — no async runtime, no external dependencies.
order: 6
portfolioMode: summary-collapsible
detailsCollapsed: true
stack: [Rust]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# std-only-rate-limiter

## Portfolio

A small, **std-only** token-bucket rate limiter for Rust. Tokens refill continuously at a configured rate, burst up to a capacity, and callers either acquire tokens or get rejected.

## Development

```bash
cargo test
cargo doc --open --no-deps
```

## Quick start

```toml
[dependencies]
std-only-rate-limiter = "0.1"
```

```rust
use rate_limiter::TokenBucket;

let mut limiter = TokenBucket::new(10, 5.0);

for _ in 0..15 {
    if limiter.try_acquire(1) {
        println!("request allowed");
    } else {
        println!("rate limited");
    }
}
```

Published on [crates.io](https://crates.io/crates/std-only-rate-limiter).
