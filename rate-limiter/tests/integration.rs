use rate_limiter::{RateLimiter, TokenBucket};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn burst_then_reject_at_ten_per_second() {
    let mut limiter = TokenBucket::new(10, 10.0);

    for _ in 0..10 {
        assert!(limiter.try_acquire(1), "burst requests should be allowed");
    }

    assert!(!limiter.try_acquire(1), "11th request should be rejected");
}

#[test]
fn refill_allows_requests_after_wait() {
    let mut limiter = TokenBucket::new(1, 5.0);
    assert!(limiter.try_acquire(1));
    assert!(!limiter.try_acquire(1));

    thread::sleep(Duration::from_millis(250));
    assert!(limiter.try_acquire(1));
}

#[test]
fn concurrent_acquires_stay_within_bounds() {
    let limiter = Arc::new(RateLimiter::new(20, 20.0));
    let start = Instant::now();
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let limiter = Arc::clone(&limiter);
            thread::spawn(move || {
                let mut successes = 0u64;
                while start.elapsed() < Duration::from_millis(200) {
                    if limiter.try_acquire(1) {
                        successes += 1;
                    }
                    thread::yield_now();
                }
                successes
            })
        })
        .collect();

    let total: u64 = handles.into_iter().map(|h| h.join().unwrap()).sum();
    let elapsed_secs = start.elapsed().as_secs_f64();
    let max_allowed = 20.0 + 20.0 * elapsed_secs;

    assert!(
        total as f64 <= max_allowed.ceil() + 1.0,
        "total successes {total} exceeded bound ~{max_allowed}"
    );
}
