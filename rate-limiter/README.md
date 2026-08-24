---
title: rate-limiter
subtext: A small, std-only token-bucket rate limiter for Rust — no async runtime, no external dependencies.
stack: [Rust]
extent: [Develop]
contribution: Solo Developer
category: Personal
---

# std-only-rate-limiter

A small, **std-only** token-bucket rate limiter for Rust. Tokens refill continuously at a configured rate, burst up to a capacity, and callers either acquire tokens or get rejected — no async runtime, no external dependencies.

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
std-only-rate-limiter = "0.1"
```

```rust
use rate_limiter::TokenBucket;

let mut limiter = TokenBucket::new(10, 5.0); // capacity 10, 5 tokens/sec

for _ in 0..15 {
    if limiter.try_acquire(1) {
        println!("request allowed");
    } else {
        println!("rate limited");
    }
}
```

For multi-threaded use, wrap with [`RateLimiter`](https://docs.rs/std-only-rate-limiter/latest/rate_limiter/struct.RateLimiter.html):

```rust
use rate_limiter::RateLimiter;
use std::sync::Arc;
use std::thread;

let limiter = Arc::new(RateLimiter::new(10, 10.0));

for _ in 0..4 {
    let limiter = Arc::clone(&limiter);
    thread::spawn(move || {
        if limiter.try_acquire(1) {
            println!("allowed");
        }
    });
}
```

## API

| Type | Use when |
|------|----------|
| [`TokenBucket`](https://docs.rs/std-only-rate-limiter/latest/rate_limiter/struct.TokenBucket.html) | Single-threaded code |
| [`RateLimiter`](https://docs.rs/std-only-rate-limiter/latest/rate_limiter/struct.RateLimiter.html) | Shared across threads (`Send + Sync`) |

Both support:

- `try_acquire(n)` — non-blocking; returns `true` if `n` tokens were consumed
- `acquire(n)` — returns `Duration::ZERO` and consumes tokens if ready; otherwise returns how long to wait
- `available()` — current token count (after refill)
- `reset()` — refill to full capacity

Construct with `TokenBucket::new(capacity, refill_per_second)` or `TokenBucket::with_refill_interval(capacity, refill_amount, interval)`.

## Development

```bash
cargo test
cargo doc --open --no-deps
```

## Publishing

This crate is ready to publish on [crates.io](https://crates.io/crates/std-only-rate-limiter).

## Non-goals (v0.1)

- Per-key limiters (`HashMap<String, RateLimiter>`)
- Async / Tokio integration
- Distributed rate limiting (Redis, etc.)

## License

MIT
