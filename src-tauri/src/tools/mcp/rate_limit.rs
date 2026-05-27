//! Token bucket rate limiter for tool execution

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

/// Token bucket rate limiter
pub struct RateLimiter {
    buckets: Mutex<HashMap<String, TokenBucket>>,
}

struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Configure a rate limit for a tool
    pub fn configure(&self, tool_id: &str, max_per_second: u32) {
        let mut buckets = self.buckets.lock().unwrap();
        buckets.insert(
            tool_id.to_string(),
            TokenBucket {
                tokens: max_per_second as f64,
                max_tokens: max_per_second as f64,
                refill_rate: max_per_second as f64,
                last_refill: Instant::now(),
            },
        );
    }

    /// Try to consume a token. Returns true if allowed, false if rate limited.
    pub fn try_acquire(&self, tool_id: &str) -> bool {
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets
            .entry(tool_id.to_string())
            .or_insert_with(|| TokenBucket {
                tokens: 10.0,
                max_tokens: 10.0,
                refill_rate: 10.0,
                last_refill: Instant::now(),
            });

        // Refill tokens based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * bucket.refill_rate).min(bucket.max_tokens);
        bucket.last_refill = now;

        // Try to consume
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn allows_up_to_limit() {
        let limiter = RateLimiter::new();
        limiter.configure("test.tool", 5);
        for _ in 0..5 {
            assert!(limiter.try_acquire("test.tool"));
        }
    }

    #[test]
    fn blocks_over_limit() {
        let limiter = RateLimiter::new();
        limiter.configure("test.tool", 2);
        assert!(limiter.try_acquire("test.tool"));
        assert!(limiter.try_acquire("test.tool"));
        assert!(!limiter.try_acquire("test.tool")); // third call blocked
    }

    #[test]
    fn refills_over_time() {
        let limiter = RateLimiter::new();
        limiter.configure("test.tool", 10);
        for _ in 0..10 {
            assert!(limiter.try_acquire("test.tool"));
        }
        assert!(!limiter.try_acquire("test.tool"));
        thread::sleep(Duration::from_millis(150));
        assert!(limiter.try_acquire("test.tool")); // refilled
    }

    #[test]
    fn unconfigured_tool_uses_default_bucket() {
        let limiter = RateLimiter::new();
        // Default bucket has 10 tokens
        for _ in 0..10 {
            assert!(limiter.try_acquire("unknown.tool"));
        }
        assert!(!limiter.try_acquire("unknown.tool"));
    }

    #[test]
    fn independent_buckets() {
        let limiter = RateLimiter::new();
        limiter.configure("tool.a", 2);
        limiter.configure("tool.b", 2);

        // Exhaust tool.a
        assert!(limiter.try_acquire("tool.a"));
        assert!(limiter.try_acquire("tool.a"));
        assert!(!limiter.try_acquire("tool.a"));

        // tool.b still has tokens
        assert!(limiter.try_acquire("tool.b"));
        assert!(limiter.try_acquire("tool.b"));
        assert!(!limiter.try_acquire("tool.b"));
    }
}
