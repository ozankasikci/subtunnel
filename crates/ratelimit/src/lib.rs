//! Rate limiting and abuse prevention for SubTunnel.
//!
//! Provides token bucket rate limiting, sliding window counters,
//! bandwidth tracking, and connection limiting — all per-user and plan-aware.

mod plan;
mod token_bucket;
mod sliding_window;
mod bandwidth;
mod connection;
mod limiter;

pub use plan::{Plan, PlanLimits};
pub use token_bucket::TokenBucket;
pub use sliding_window::SlidingWindowCounter;
pub use bandwidth::BandwidthTracker;
pub use connection::ConnectionLimiter;
pub use limiter::{RateLimiter, RateLimitError, Action};

#[cfg(test)]
mod tests;
