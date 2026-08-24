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

# rate-limiter

## Portfolio

**std-only-rate-limiter** is a token-bucket rate limiter for Rust built entirely on `std` — no Tokio, no external crates, no async. Tokens refill continuously at a configured rate, burst up to a capacity, and callers either acquire them or get turned away. That makes it easy to drop into CLIs, sync HTTP handlers, or worker loops where you just need "don't exceed N requests per second."

The API has two shapes: **`TokenBucket`** for single-threaded code, and **`RateLimiter`** (a mutex-wrapped bucket) when multiple threads share one limit. Both support `try_acquire` for non-blocking checks, `acquire` when you're willing to wait, `available` to peek at the current level, and `reset` to refill instantly. You can construct by tokens-per-second or by custom refill intervals.

I wrote this because most rate-limiting crates either assume async or pull in a heavier dependency tree. Sometimes you only need a few lines of backpressure — cap API calls, throttle log writes, or protect a fragile downstream service — and want something you can audit in one sitting.

The crate is published on [crates.io](https://crates.io/crates/std-only-rate-limiter) and ready to use as a dependency today.

### Quick start

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

## Development

```bash
cargo test
cargo doc --open --no-deps
```

## Quick start

```rust
use rate_limiter::RateLimiter;
use std::sync::Arc;

let limiter = Arc::new(RateLimiter::new(10, 10.0));
// thread-safe variant — see repo for full API
```

## API

| Type | Use when |
|------|----------|
| `TokenBucket` | Single-threaded code |
| `RateLimiter` | Shared across threads |

## Publishing

Ready to publish on [crates.io](https://crates.io/crates/std-only-rate-limiter).

## Non-goals (v0.1)

- Per-key limiters, async/Tokio, distributed limiting

## License

MIT
