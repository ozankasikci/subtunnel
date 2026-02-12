use dashmap::DashMap;
use std::collections::VecDeque;
use std::time::Instant;

/// Sliding window request counter per user.
///
/// Tracks request timestamps in a deque and counts requests within the window.
pub struct SlidingWindowCounter {
    windows: DashMap<String, VecDeque<Instant>>,
}

impl SlidingWindowCounter {
    pub fn new() -> Self {
        Self {
            windows: DashMap::new(),
        }
    }

    /// Record a request and check if the limit is exceeded.
    /// `max_requests` of 0 means unlimited.
    /// `window` is the sliding window duration.
    /// Returns `Ok(count)` with current count or `Err(count)` if limit exceeded.
    pub fn record_and_check(
        &self,
        user_id: &str,
        max_requests: u64,
        window: std::time::Duration,
    ) -> Result<u64, u64> {
        if max_requests == 0 {
            return Ok(0); // unlimited
        }

        let now = Instant::now();
        let mut entry = self.windows.entry(user_id.to_string()).or_insert_with(VecDeque::new);
        let deque = entry.value_mut();

        // Evict expired entries
        let cutoff = now - window;
        while deque.front().map_or(false, |&t| t < cutoff) {
            deque.pop_front();
        }

        let count = deque.len() as u64;
        if count >= max_requests {
            return Err(count);
        }

        deque.push_back(now);
        Ok(count + 1)
    }

    /// Get current count without recording.
    pub fn count(&self, user_id: &str, window: std::time::Duration) -> u64 {
        let now = Instant::now();
        let cutoff = now - window;
        self.windows
            .get(user_id)
            .map(|entry| entry.iter().filter(|&&t| t >= cutoff).count() as u64)
            .unwrap_or(0)
    }

    /// Reset a user's counter.
    pub fn reset(&self, user_id: &str) {
        self.windows.remove(user_id);
    }
}
