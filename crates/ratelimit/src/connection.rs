use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, Ordering};

/// Limits concurrent tunnels per user.
pub struct ConnectionLimiter {
    connections: DashMap<String, AtomicU32>,
}

impl ConnectionLimiter {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
        }
    }

    /// Try to acquire a tunnel slot. Returns Ok(current_count) or Err(limit).
    /// `max_tunnels` of 0 means unlimited.
    pub fn try_acquire(&self, user_id: &str, max_tunnels: u32) -> Result<u32, u32> {
        let entry = self.connections.entry(user_id.to_string()).or_insert_with(|| AtomicU32::new(0));
        let current = entry.value().load(Ordering::Relaxed);
        if max_tunnels > 0 && current >= max_tunnels {
            return Err(max_tunnels);
        }
        let new_count = entry.value().fetch_add(1, Ordering::Relaxed) + 1;
        // Double-check after increment (race-tolerant)
        if max_tunnels > 0 && new_count > max_tunnels {
            entry.value().fetch_sub(1, Ordering::Relaxed);
            return Err(max_tunnels);
        }
        Ok(new_count)
    }

    /// Release a tunnel slot when a tunnel disconnects.
    pub fn release(&self, user_id: &str) {
        if let Some(entry) = self.connections.get(user_id) {
            let prev = entry.value().fetch_sub(1, Ordering::Relaxed);
            if prev <= 1 {
                drop(entry);
                self.connections.remove(user_id);
            }
        }
    }

    /// Get current connection count for a user.
    pub fn count(&self, user_id: &str) -> u32 {
        self.connections
            .get(user_id)
            .map(|e| e.value().load(Ordering::Relaxed))
            .unwrap_or(0)
    }
}
