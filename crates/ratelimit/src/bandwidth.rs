use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::fmt;

/// Tracks bandwidth usage per user per billing period (in-memory).
///
/// For production, this should be periodically flushed to persistent storage.
pub struct BandwidthTracker {
    usage: DashMap<String, AtomicU64>,
}

/// Error returned when bandwidth limit is exceeded.
#[derive(Debug, Clone)]
pub struct LimitExceeded {
    pub used: u64,
    pub limit: u64,
}

impl fmt::Display for LimitExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bandwidth limit exceeded: used {} of {} bytes",
            self.used, self.limit
        )
    }
}

impl std::error::Error for LimitExceeded {}

impl BandwidthTracker {
    pub fn new() -> Self {
        Self {
            usage: DashMap::new(),
        }
    }

    /// Record bytes used by a user. Returns error if limit would be exceeded.
    /// `limit` of 0 means unlimited.
    pub fn record(&self, user_id: &str, bytes: u64, limit: u64) -> Result<u64, LimitExceeded> {
        if limit == 0 {
            // Unlimited — still track
            let entry = self.usage.entry(user_id.to_string()).or_insert_with(|| AtomicU64::new(0));
            let new_val = entry.value().fetch_add(bytes, Ordering::Relaxed) + bytes;
            return Ok(new_val);
        }

        let entry = self.usage.entry(user_id.to_string()).or_insert_with(|| AtomicU64::new(0));
        // Optimistic check — not perfectly atomic but good enough for rate limiting
        let current = entry.value().load(Ordering::Relaxed);
        if current + bytes > limit {
            return Err(LimitExceeded {
                used: current,
                limit,
            });
        }
        let new_val = entry.value().fetch_add(bytes, Ordering::Relaxed) + bytes;
        Ok(new_val)
    }

    /// Get current usage for a user.
    pub fn get_usage(&self, user_id: &str) -> u64 {
        self.usage
            .get(user_id)
            .map(|e| e.value().load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Reset usage for a user (e.g., new billing period).
    pub fn reset(&self, user_id: &str) {
        self.usage.remove(user_id);
    }

    /// Reset all users (e.g., global billing period reset).
    pub fn reset_all(&self) {
        self.usage.clear();
    }
}
