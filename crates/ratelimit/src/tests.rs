use crate::*;
use std::time::Duration;

// ── Token Bucket ────────────────────────────────────────────

#[test]
fn token_bucket_allows_burst_up_to_capacity() {
    let tb = TokenBucket::new();
    for i in 0..100 {
        assert!(
            tb.try_acquire("user1", 100, 60.0).is_ok(),
            "request {i} should succeed"
        );
    }
    // 101st should fail
    assert!(tb.try_acquire("user1", 100, 60.0).is_err());
}

#[test]
fn token_bucket_refills_over_time() {
    let tb = TokenBucket::new();
    // Exhaust all tokens
    for _ in 0..10 {
        let _ = tb.try_acquire("user1", 10, 1.0);
    }
    assert!(tb.try_acquire("user1", 10, 1.0).is_err());

    // Wait for refill (tokens refill at 10/sec, need 1 token → ~0.1s)
    std::thread::sleep(Duration::from_millis(150));
    assert!(tb.try_acquire("user1", 10, 1.0).is_ok());
}

#[test]
fn token_bucket_unlimited_when_zero_capacity() {
    let tb = TokenBucket::new();
    for _ in 0..10_000 {
        assert!(tb.try_acquire("user1", 0, 60.0).is_ok());
    }
}

#[test]
fn token_bucket_isolates_users() {
    let tb = TokenBucket::new();
    // Exhaust user1
    for _ in 0..5 {
        let _ = tb.try_acquire("user1", 5, 60.0);
    }
    assert!(tb.try_acquire("user1", 5, 60.0).is_err());
    // user2 should be fine
    assert!(tb.try_acquire("user2", 5, 60.0).is_ok());
}

// ── Sliding Window ──────────────────────────────────────────

#[test]
fn sliding_window_enforces_limit() {
    let sw = SlidingWindowCounter::new();
    let window = Duration::from_secs(60);
    for _ in 0..10 {
        assert!(sw.record_and_check("user1", 10, window).is_ok());
    }
    assert!(sw.record_and_check("user1", 10, window).is_err());
}

#[test]
fn sliding_window_unlimited_when_zero() {
    let sw = SlidingWindowCounter::new();
    for _ in 0..1000 {
        assert!(sw.record_and_check("user1", 0, Duration::from_secs(60)).is_ok());
    }
}

#[test]
fn sliding_window_resets() {
    let sw = SlidingWindowCounter::new();
    let window = Duration::from_secs(60);
    for _ in 0..10 {
        let _ = sw.record_and_check("user1", 10, window);
    }
    assert!(sw.record_and_check("user1", 10, window).is_err());
    sw.reset("user1");
    assert!(sw.record_and_check("user1", 10, window).is_ok());
}

// ── Bandwidth ───────────────────────────────────────────────

#[test]
fn bandwidth_tracks_and_limits() {
    let bt = BandwidthTracker::new();
    let limit = 1000u64;
    assert!(bt.record("user1", 500, limit).is_ok());
    assert_eq!(bt.get_usage("user1"), 500);
    assert!(bt.record("user1", 400, limit).is_ok());
    assert_eq!(bt.get_usage("user1"), 900);
    // Exceeds limit
    assert!(bt.record("user1", 200, limit).is_err());
}

#[test]
fn bandwidth_unlimited_when_zero() {
    let bt = BandwidthTracker::new();
    assert!(bt.record("user1", 1_000_000_000, 0).is_ok());
}

#[test]
fn bandwidth_reset() {
    let bt = BandwidthTracker::new();
    bt.record("user1", 500, 1000).unwrap();
    bt.reset("user1");
    assert_eq!(bt.get_usage("user1"), 0);
}

// ── Connection Limiter ──────────────────────────────────────

#[test]
fn connection_limiter_enforces_max() {
    let cl = ConnectionLimiter::new();
    assert!(cl.try_acquire("user1", 3).is_ok());
    assert!(cl.try_acquire("user1", 3).is_ok());
    assert!(cl.try_acquire("user1", 3).is_ok());
    assert!(cl.try_acquire("user1", 3).is_err());
    // Release one
    cl.release("user1");
    assert!(cl.try_acquire("user1", 3).is_ok());
}

#[test]
fn connection_limiter_unlimited_when_zero() {
    let cl = ConnectionLimiter::new();
    for _ in 0..1000 {
        assert!(cl.try_acquire("user1", 0).is_ok());
    }
}

// ── Unified RateLimiter ─────────────────────────────────────

#[test]
fn rate_limiter_free_plan_tunnel_limit() {
    let rl = RateLimiter::new();
    for _ in 0..3 {
        assert!(rl.check("user1", Plan::Free, Action::OpenTunnel).is_ok());
    }
    assert!(matches!(
        rl.check("user1", Plan::Free, Action::OpenTunnel),
        Err(RateLimitError::TunnelLimit { limit: 3 })
    ));
}

#[test]
fn rate_limiter_enterprise_unlimited() {
    let rl = RateLimiter::new();
    // Enterprise can open many tunnels
    for _ in 0..100 {
        assert!(rl.check("user1", Plan::Enterprise, Action::OpenTunnel).is_ok());
    }
    // Enterprise can make many requests
    for _ in 0..10_000 {
        assert!(rl.check("user1", Plan::Enterprise, Action::Request).is_ok());
    }
    // Enterprise bandwidth unlimited
    assert!(rl.record_bandwidth("user1", Plan::Enterprise, 1_000_000_000_000).is_ok());
}

#[test]
fn rate_limiter_bandwidth_enforcement() {
    let rl = RateLimiter::new();
    let gb = 1_073_741_824u64;
    // Free plan: 1GB
    assert!(rl.record_bandwidth("user1", Plan::Free, gb - 100).is_ok());
    assert!(rl.record_bandwidth("user1", Plan::Free, 200).is_err());
}

#[test]
fn rate_limiter_reset_user() {
    let rl = RateLimiter::new();
    // Use some limits
    rl.check("user1", Plan::Free, Action::OpenTunnel).unwrap();
    rl.record_bandwidth("user1", Plan::Free, 500_000_000).unwrap();
    // Reset
    rl.reset_user("user1");
    assert_eq!(rl.bandwidth.get_usage("user1"), 0);
    // Connection still active (release is explicit), but bandwidth is reset
    rl.release_tunnel("user1");
    assert_eq!(rl.connections.count("user1"), 0);
}

#[test]
fn rate_limiter_pro_plan_higher_limits() {
    let rl = RateLimiter::new();
    // Pro can open 20 tunnels
    for _ in 0..20 {
        assert!(rl.check("user1", Plan::Pro, Action::OpenTunnel).is_ok());
    }
    assert!(rl.check("user1", Plan::Pro, Action::OpenTunnel).is_err());
}
