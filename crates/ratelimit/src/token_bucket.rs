use dashmap::DashMap;
use std::time::Instant;

/// Per-user token bucket for request rate limiting.
///
/// Each user gets `capacity` tokens, refilled at `capacity` tokens per `window` (e.g., per minute).
/// Tokens are refilled continuously based on elapsed time.
pub struct TokenBucket {
    buckets: DashMap<String, Bucket>,
}

struct Bucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new() -> Self {
        Self {
            buckets: DashMap::new(),
        }
    }

    /// Try to consume one token for the given user.
    /// `capacity` is max tokens (burst size), `per_seconds` is the refill window.
    /// Returns `Ok(remaining)` or `Err(retry_after_secs)`.
    pub fn try_acquire(&self, user_id: &str, capacity: u32, per_seconds: f64) -> Result<u32, f64> {
        if capacity == 0 {
            return Ok(u32::MAX); // unlimited
        }
        let cap = capacity as f64;
        let refill_rate = cap / per_seconds;
        let now = Instant::now();

        let mut entry = self.buckets.entry(user_id.to_string()).or_insert_with(|| Bucket {
            tokens: cap,
            capacity: cap,
            refill_rate,
            last_refill: now,
        });

        let bucket = entry.value_mut();
        // Refill tokens
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * bucket.refill_rate).min(bucket.capacity);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(bucket.tokens as u32)
        } else {
            let wait = (1.0 - bucket.tokens) / bucket.refill_rate;
            Err(wait)
        }
    }

    /// Remove a user's bucket (e.g., on disconnect).
    pub fn remove(&self, user_id: &str) {
        self.buckets.remove(user_id);
    }
}
