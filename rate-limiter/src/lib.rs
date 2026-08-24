//! A small, std-only token-bucket rate limiter.
//!
//! Use [`TokenBucket`] in single-threaded code. For shared access across threads,
//! use [`RateLimiter`].
//!
//! # Example
//!
//! ```
//! use rate_limiter::TokenBucket;
//!
//! let mut limiter = TokenBucket::new(5, 2.0);
//!
//! assert!(limiter.try_acquire(1));
//! assert_eq!(limiter.available(), 4);
//! ```

#![deny(missing_docs)]

mod bucket;
mod limiter;

pub use bucket::TokenBucket;
pub use limiter::RateLimiter;
