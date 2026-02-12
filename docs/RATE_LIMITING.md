# Rate Limiting & Abuse Prevention

> Implementation guide for SubTunnel's `subtunnel-ratelimit` crate.

## Plan Limits

| Resource | Free | Pro | Enterprise |
|----------|------|-----|------------|
| Concurrent tunnels | 3 | 20 | Unlimited |
| Requests/month | 100K | Unlimited | Unlimited |
| Bandwidth/month | 1 GB | 100 GB | Unlimited |
| Requests/minute (burst) | 1,000 | 10,000 | Unlimited |

## Enforcement Layers

### 1. Connection Layer (Tunnel Creation)

**When**: Client sends `TunnelReq` control message.
**Check**: `RateLimiter::check(user_id, Plan, Action::OpenTunnel)`
**Enforces**: Max concurrent tunnels per plan.
**On rejection**: Server responds with `TunnelResp { success: false, message: "tunnel limit reached (3/3)" }`.
**Release**: Call `RateLimiter::release_tunnel(user_id)` when tunnel disconnects.

### 2. Request Layer (HTTP Requests)

**When**: HTTP request arrives at the tunnel listener, after subdomain lookup.
**Check**: `RateLimiter::check(user_id, Plan, Action::Request)`
**Enforces**:
- **Token bucket** — per-minute burst rate (e.g., 1000 req/min for Free). Tokens refill continuously.
- **Sliding window** — monthly request quota (e.g., 100K/month for Free). 30-day rolling window.
**On rejection**: HTTP `429 Too Many Requests` with `Retry-After` header.

### 3. Bandwidth Layer (Data Transfer)

**When**: After proxying data between client and tunnel endpoint.
**Check**: `RateLimiter::record_bandwidth(user_id, Plan, bytes)`
**Enforces**: Monthly bandwidth cap.
**On rejection**: Returns `BandwidthExceeded` error. Server should close the proxy stream and return HTTP `429` for subsequent requests.

## How Limits Reset

- **Token bucket**: Continuously refills. Capacity tokens per minute window.
- **Sliding window (monthly)**: Rolling 30-day window — old entries expire automatically.
- **Bandwidth**: Call `BandwidthTracker::reset(user_id)` at billing period boundary. In production, a cron job or billing webhook triggers this.
- **Connections**: Released individually as tunnels disconnect. No time-based reset.
- **Full reset**: `RateLimiter::reset_user(user_id)` clears token bucket, sliding window, and bandwidth for a user.

## Integration Points

The `subtunnel-ratelimit` crate is a pure in-memory library. For production:

1. **`handler.rs`** (agent handler): Check `Action::OpenTunnel` before registering tunnel. Call `release_tunnel` on disconnect.
2. **`listener.rs`** (HTTP listener): Check `Action::Request` before proxying. Return 429 on failure.
3. **Proxy loop**: Call `record_bandwidth` with bytes transferred after each proxy copy.
4. **Billing webhooks**: Call `reset_user` on plan change or new billing period.

## Monitoring & Alerting

### Metrics to Track
- `subtunnel_ratelimit_rejected_total{reason="rate|quota|bandwidth|tunnels"}` — counter of rejections by type
- `subtunnel_ratelimit_bandwidth_usage_bytes{user_id}` — gauge of current usage
- `subtunnel_ratelimit_active_tunnels{user_id}` — gauge of active tunnels

### Alert Conditions
- **Abuse pattern**: Single user hitting rate limit >100 times/minute → potential attack
- **Bandwidth spike**: User consuming >10% of plan bandwidth in <1 hour
- **Connection churn**: User opening/closing tunnels rapidly (>10/minute)
- **Quota exhaustion**: User at >80% monthly quota → send warning email

### Response Playbook
1. **Automated**: Rate limiter blocks excess traffic (HTTP 429, tunnel rejection)
2. **Warning**: Email user at 80% bandwidth/quota usage
3. **Hard cutoff**: Block at 100% of plan limit
4. **Admin action**: Ban user via admin API → force-close all tunnels
