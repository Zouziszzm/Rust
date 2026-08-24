use std::time::{Duration, Instant};

/// A single-threaded token-bucket rate limiter.
///
/// Tokens refill continuously at a configured rate, up to a maximum capacity.
/// Use [`crate::RateLimiter`] when the limiter is shared across threads.
#[derive(Debug)]
pub struct TokenBucket {
    capacity: u64,
    refill_per_second: f64,
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    /// Creates a bucket with `capacity` tokens that refills at `refill_per_second`.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is zero or `refill_per_second` is not positive.
    ///
    /// # Example
    ///
    /// ```
    /// use rate_limiter::TokenBucket;
    ///
    /// let limiter = TokenBucket::new(10, 5.0);
    /// assert_eq!(limiter.available(), 10);
    /// ```
    pub fn new(capacity: u64, refill_per_second: f64) -> Self {
        assert!(capacity > 0, "capacity must be greater than zero");
        assert!(
            refill_per_second > 0.0,
            "refill_per_second must be greater than zero"
        );

        Self {
            capacity,
            refill_per_second,
            tokens: capacity as f64,
            last_refill: Instant::now(),
        }
    }

    /// Creates a bucket that adds `refill_amount` tokens every `interval`.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` or `refill_amount` is zero, or if `interval` is zero.
    ///
    /// # Example
    ///
    /// ```
    /// use rate_limiter::TokenBucket;
    /// use std::time::Duration;
    ///
    /// let limiter = TokenBucket::with_refill_interval(10, 1, Duration::from_millis(200));
    /// assert_eq!(limiter.available(), 10);
    /// ```
    pub fn with_refill_interval(capacity: u64, refill_amount: u64, interval: Duration) -> Self {
        assert!(refill_amount > 0, "refill_amount must be greater than zero");
        assert!(!interval.is_zero(), "interval must be greater than zero");

        let refill_per_second = refill_amount as f64 / interval.as_secs_f64();
        Self::new(capacity, refill_per_second)
    }

    /// Returns the maximum number of tokens the bucket can hold.
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Returns the configured refill rate in tokens per second.
    pub fn refill_per_second(&self) -> f64 {
        self.refill_per_second
    }

    /// Returns the current number of available tokens after applying refill.
    pub fn available(&self) -> u64 {
        let tokens = self.tokens_after_refill(Instant::now());
        tokens.floor() as u64
    }

    /// Tries to consume `n` tokens without blocking.
    ///
    /// Returns `true` if `n` tokens were consumed, `false` otherwise.
    pub fn try_acquire(&mut self, n: u64) -> bool {
        let now = Instant::now();
        self.refill(now);

        let required = n as f64;
        if self.tokens >= required {
            self.tokens -= required;
            true
        } else {
            false
        }
    }

    /// Returns how long to wait before `n` tokens are available.
    ///
    /// If enough tokens are available now, consumes them and returns [`Duration::ZERO`].
    pub fn acquire(&mut self, n: u64) -> Duration {
        let now = Instant::now();
        self.refill(now);

        let required = n as f64;
        if self.tokens >= required {
            self.tokens -= required;
            return Duration::ZERO;
        }

        let deficit = required - self.tokens;
        let wait_secs = deficit / self.refill_per_second;
        Duration::from_secs_f64(wait_secs)
    }

    /// Refills the bucket to full capacity.
    pub fn reset(&mut self) {
        self.tokens = self.capacity as f64;
        self.last_refill = Instant::now();
    }

    fn refill(&mut self, now: Instant) {
        self.tokens = self.tokens_after_refill(now);
        self.last_refill = now;
    }

    fn tokens_after_refill(&self, now: Instant) -> f64 {
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let refilled = self.tokens + elapsed * self.refill_per_second;
        refilled.min(self.capacity as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn new_bucket_starts_at_capacity() {
        let bucket = TokenBucket::new(10, 5.0);
        assert_eq!(bucket.available(), 10);
    }

    #[test]
    fn try_acquire_succeeds_then_fails_when_drained() {
        let mut bucket = TokenBucket::new(3, 5.0);

        assert!(bucket.try_acquire(1));
        assert!(bucket.try_acquire(1));
        assert!(bucket.try_acquire(1));
        assert!(!bucket.try_acquire(1));
        assert_eq!(bucket.available(), 0);
    }

    #[test]
    fn tokens_refill_over_time() {
        let mut bucket = TokenBucket::new(10, 10.0);
        assert!(bucket.try_acquire(10));
        assert!(!bucket.try_acquire(1));

        thread::sleep(Duration::from_millis(250));
        assert!(bucket.try_acquire(1));
    }

    #[test]
    fn capacity_is_never_exceeded_after_idle() {
        let mut bucket = TokenBucket::new(5, 100.0);
        thread::sleep(Duration::from_millis(100));
        assert_eq!(bucket.available(), 5);
        assert!(!bucket.try_acquire(6));
    }

    #[test]
    fn acquire_returns_wait_when_empty() {
        let mut bucket = TokenBucket::new(1, 2.0);
        assert_eq!(bucket.acquire(1), Duration::ZERO);
        let wait = bucket.acquire(1);
        assert!(wait > Duration::ZERO);
        assert!(wait <= Duration::from_millis(600));
    }

    #[test]
    fn acquire_consumes_tokens_when_ready() {
        let mut bucket = TokenBucket::new(2, 10.0);
        assert_eq!(bucket.acquire(1), Duration::ZERO);
        assert_eq!(bucket.available(), 1);
    }

    #[test]
    fn reset_refills_to_capacity() {
        let mut bucket = TokenBucket::new(5, 1.0);
        assert!(bucket.try_acquire(5));
        bucket.reset();
        assert_eq!(bucket.available(), 5);
    }

    #[test]
    fn with_refill_interval_sets_rate() {
        let bucket = TokenBucket::with_refill_interval(10, 1, Duration::from_secs(1));
        assert_eq!(bucket.capacity(), 10);
        assert_eq!(bucket.refill_per_second(), 1.0);
    }
}
