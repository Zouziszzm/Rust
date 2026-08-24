use std::sync::Mutex;
use std::time::Duration;

use crate::TokenBucket;

/// A thread-safe token-bucket rate limiter.
///
/// Wraps [`TokenBucket`] in a [`Mutex`] for shared use across threads.
#[derive(Debug)]
pub struct RateLimiter {
    inner: Mutex<TokenBucket>,
}

impl RateLimiter {
    /// Creates a limiter with `capacity` tokens that refills at `refill_per_second`.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is zero or `refill_per_second` is not positive.
    ///
    /// # Example
    ///
    /// ```
    /// use rate_limiter::RateLimiter;
    ///
    /// let limiter = RateLimiter::new(10, 5.0);
    /// assert_eq!(limiter.available(), 10);
    /// ```
    pub fn new(capacity: u64, refill_per_second: f64) -> Self {
        Self {
            inner: Mutex::new(TokenBucket::new(capacity, refill_per_second)),
        }
    }

    /// Creates a limiter that adds `refill_amount` tokens every `interval`.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` or `refill_amount` is zero, or if `interval` is zero.
    pub fn with_refill_interval(capacity: u64, refill_amount: u64, interval: Duration) -> Self {
        Self {
            inner: Mutex::new(TokenBucket::with_refill_interval(
                capacity,
                refill_amount,
                interval,
            )),
        }
    }

    /// Returns the maximum number of tokens the bucket can hold.
    pub fn capacity(&self) -> u64 {
        self.inner.lock().expect("rate limiter mutex poisoned").capacity()
    }

    /// Returns the configured refill rate in tokens per second.
    pub fn refill_per_second(&self) -> f64 {
        self.inner
            .lock()
            .expect("rate limiter mutex poisoned")
            .refill_per_second()
    }

    /// Returns the current number of available tokens after applying refill.
    pub fn available(&self) -> u64 {
        self.inner
            .lock()
            .expect("rate limiter mutex poisoned")
            .available()
    }

    /// Tries to consume `n` tokens without blocking.
    ///
    /// Returns `true` if `n` tokens were consumed, `false` otherwise.
    pub fn try_acquire(&self, n: u64) -> bool {
        self.inner
            .lock()
            .expect("rate limiter mutex poisoned")
            .try_acquire(n)
    }

    /// Returns how long to wait before `n` tokens are available.
    ///
    /// If enough tokens are available now, consumes them and returns [`Duration::ZERO`].
    pub fn acquire(&self, n: u64) -> Duration {
        self.inner
            .lock()
            .expect("rate limiter mutex poisoned")
            .acquire(n)
    }

    /// Refills the bucket to full capacity.
    pub fn reset(&self) {
        self.inner
            .lock()
            .expect("rate limiter mutex poisoned")
            .reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn rate_limiter_delegates_to_bucket() {
        let limiter = RateLimiter::new(5, 2.0);
        assert_eq!(limiter.available(), 5);
        assert!(limiter.try_acquire(2));
        assert_eq!(limiter.available(), 3);
    }

    #[test]
    fn rate_limiter_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RateLimiter>();
    }

    #[test]
    fn rate_limiter_refills_over_time() {
        let limiter = RateLimiter::new(2, 10.0);
        assert!(limiter.try_acquire(2));
        assert!(!limiter.try_acquire(1));

        thread::sleep(Duration::from_millis(150));
        assert!(limiter.try_acquire(1));
    }
}
