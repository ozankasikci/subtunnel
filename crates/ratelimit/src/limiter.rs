use crate::{
    bandwidth::{BandwidthTracker, LimitExceeded},
    connection::ConnectionLimiter,
    plan::{Plan, PlanLimits},
    sliding_window::SlidingWindowCounter,
    token_bucket::TokenBucket,
};
use std::fmt;
use std::time::Duration;

/// Actions that can be rate-limited.
#[derive(Debug, Clone, Copy)]
pub enum Action {
    /// An HTTP request through a tunnel.
    Request,
    /// Opening a new tunnel.
    OpenTunnel,
}

/// Rate limit error with context for HTTP response headers.
#[derive(Debug, Clone)]
pub enum RateLimitError {
    /// Token bucket (per-minute) rate limit exceeded.
    RateLimited {
        retry_after_secs: f64,
    },
    /// Monthly request quota exceeded.
    MonthlyRequestQuota {
        used: u64,
        limit: u64,
    },
    /// Tunnel connection limit exceeded.
    TunnelLimit {
        limit: u32,
    },
    /// Bandwidth limit exceeded.
    BandwidthExceeded(LimitExceeded),
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RateLimited { retry_after_secs } => {
                write!(f, "rate limit exceeded, retry after {retry_after_secs:.1}s")
            }
            Self::MonthlyRequestQuota { used, limit } => {
                write!(f, "monthly request quota exceeded ({used}/{limit})")
            }
            Self::TunnelLimit { limit } => {
                write!(f, "tunnel limit reached ({limit})")
            }
            Self::BandwidthExceeded(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for RateLimitError {}

/// Unified rate limiter combining all limiting strategies.
pub struct RateLimiter {
    token_bucket: TokenBucket,
    monthly_counter: SlidingWindowCounter,
    pub bandwidth: BandwidthTracker,
    pub connections: ConnectionLimiter,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            token_bucket: TokenBucket::new(),
            monthly_counter: SlidingWindowCounter::new(),
            bandwidth: BandwidthTracker::new(),
            connections: ConnectionLimiter::new(),
        }
    }

    /// Check whether a user can perform an action under their plan limits.
    pub fn check(&self, user_id: &str, plan: Plan, action: Action) -> Result<(), RateLimitError> {
        let limits = PlanLimits::for_plan(plan);

        match action {
            Action::Request => {
                // 1. Token bucket (per-minute burst)
                if let Err(retry_after) =
                    self.token_bucket.try_acquire(user_id, limits.max_requests_per_minute, 60.0)
                {
                    return Err(RateLimitError::RateLimited {
                        retry_after_secs: retry_after,
                    });
                }

                // 2. Monthly request counter (30-day sliding window)
                if let Err(count) = self.monthly_counter.record_and_check(
                    user_id,
                    limits.max_requests_per_month,
                    Duration::from_secs(30 * 24 * 3600),
                ) {
                    return Err(RateLimitError::MonthlyRequestQuota {
                        used: count,
                        limit: limits.max_requests_per_month,
                    });
                }

                Ok(())
            }
            Action::OpenTunnel => {
                self.connections
                    .try_acquire(user_id, limits.max_tunnels)
                    .map(|_| ())
                    .map_err(|limit| RateLimitError::TunnelLimit { limit })
            }
        }
    }

    /// Record bandwidth usage and check against plan limit.
    pub fn record_bandwidth(
        &self,
        user_id: &str,
        plan: Plan,
        bytes: u64,
    ) -> Result<u64, RateLimitError> {
        let limits = PlanLimits::for_plan(plan);
        self.bandwidth
            .record(user_id, bytes, limits.max_bandwidth_bytes_per_month)
            .map_err(RateLimitError::BandwidthExceeded)
    }

    /// Release a tunnel slot (call when tunnel disconnects).
    pub fn release_tunnel(&self, user_id: &str) {
        self.connections.release(user_id);
    }

    /// Reset all limits for a user (e.g., new billing period).
    pub fn reset_user(&self, user_id: &str) {
        self.token_bucket.remove(user_id);
        self.monthly_counter.reset(user_id);
        self.bandwidth.reset(user_id);
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
