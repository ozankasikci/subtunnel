# SubTunnel — Pricing & Billing Plan

> Comprehensive pricing strategy, competitive analysis, and billing implementation plan.
>
> Last updated: 2026-02-12

---

## Table of Contents

1. [Market Analysis](#1-market-analysis)
2. [SubTunnel Pricing Strategy](#2-subtunnel-pricing-strategy)
3. [Billing Implementation Plan](#3-billing-implementation-plan)
4. [Revenue Projections](#4-revenue-projections)
5. [Competitive Comparison](#5-competitive-comparison)

---

## 1. Market Analysis

### 1.1 ngrok Pricing Breakdown (as of early 2026)

ngrok has evolved into a confusing usage-based model. Here's what they actually charge:

| Resource | Free | Hobbyist ($8/mo) | Pay-as-you-go ($20/mo base) |
|----------|------|-------------------|------------------------------|
| Online endpoints | 3 | 3 | Unlimited |
| Custom domains | 0 (1 dev domain) | 0 (1 dev domain) | $0.01/hr per custom domain (~$7.30/mo) |
| TCP addresses | Random (w/ credit card) | 1 | 100 |
| HTTP requests | 20K/mo | 100K included | 100K included, then **$1/100K** |
| TCP/TLS connections | 5K/mo | 5K included | 5K included, then **$2/100K** |
| Data transfer out | 1 GB | 5 GB | 5 GB included, then **$0.10/GB** |
| Concurrent agents | 3 | 3 | Unlimited |
| Users | 1 | 1 | 3 included, then **$5/user/mo** |
| Interstitial page | Yes | No | No |

**Add-ons on Pay-as-you-go:**
- Extra endpoints: **$15/mo each**
- Traffic policy: **$49/mo**
- Identity (OAuth/OIDC): **$5/MAU**
- Observability: **$99/mo**

### 1.2 Real-World ngrok Cost Scenarios

#### Scenario A: Indie Developer (webhook testing, 1 project)

| Usage | Amount | ngrok Cost |
|-------|--------|------------|
| Tunnels | 1-2 active | Free (within 3 endpoint limit) |
| Custom domain | 1 | $7.30/mo (endpoint hours) |
| Bandwidth | 3 GB/mo | $0 on Hobbyist (5GB included) |
| HTTP requests | 50K/mo | $0 on Hobbyist |

**ngrok total: $8/mo (Hobbyist)** — but no custom domain on Hobbyist. To get custom domains, you need Pay-as-you-go at **$20/mo + $7.30/mo = ~$27/mo**.

**SubTunnel: $0** (self-hosted on a $5 VPS, custom domains free)

#### Scenario B: Startup Team (5 devs, webhook + demo + staging)

| Usage | Amount | ngrok Cost |
|-------|--------|------------|
| Base plan | Pay-as-you-go | $20/mo |
| Users | 5 (3 incl, +2) | +$10/mo |
| Custom domains | 3 | +$21.90/mo |
| Bandwidth | 25 GB/mo | +$2.00/mo (20GB overage × $0.10) |
| HTTP requests | 500K/mo | +$4.00/mo (400K overage) |
| Tunnels | 8 concurrent | $0 (unlimited on PAYG) |

**ngrok total: ~$58/mo** ($696/yr)

**SubTunnel: $0** (self-hosted) or **$10/mo Pro** for priority support = **$120/yr**

#### Scenario C: Mid-size Company (20 devs, heavy usage)

| Usage | Amount | ngrok Cost |
|-------|--------|------------|
| Base plan | Pay-as-you-go | $20/mo |
| Users | 20 (3 incl, +17) | +$85/mo |
| Custom domains | 10 | +$73/mo |
| Bandwidth | 100 GB/mo | +$9.50/mo |
| HTTP requests | 2M/mo | +$19/mo |
| Traffic policy | 1 add-on | +$49/mo |
| Observability | 1 add-on | +$99/mo |

**ngrok total: ~$355/mo** ($4,260/yr)

**SubTunnel: $0** (self-hosted) or **$10/mo Pro** = **$120/yr**. Savings: **$4,140/yr**.

#### Scenario D: Enterprise (50+ devs, compliance needs)

ngrok Enterprise pricing: **$39-47/seat/mo**. At 50 seats = **$1,950-$2,350/mo** ($23,400-$28,200/yr).

SubTunnel self-hosted: **$0 software** + ~$50-100/mo server costs = **$600-$1,200/yr**. Savings: **$22,000+/yr**.

### 1.3 Where ngrok's Pricing Hurts

1. **Custom domains are expensive** — $0.01/hr per domain sounds cheap until you realize that's $7.30/mo per domain, always-on. A startup with 5 custom domains pays $36.50/mo just for domains.

2. **Per-user pricing punishes teams** — $5/user/mo after the first 3 means a 10-person team pays $35/mo just for seats, on top of everything else.

3. **Unpredictable bills** — Usage-based means you can't know your bill until month-end. A traffic spike during a demo could surprise you.

4. **Add-on stacking** — Traffic policy ($49) + observability ($99) + identity ($5/MAU) adds up fast. Basic features that should be included cost extra.

5. **Free tier is unusable** — 1 GB bandwidth, 20K requests, interstitial warning page, no custom domains. You hit limits in a single afternoon of webhook testing.

6. **Hobbyist is a dead end** — $8/mo but still only 3 endpoints, no custom domains, 5 GB bandwidth. You outgrow it immediately.

7. **No self-hosted escape hatch** — Once you're on ngrok, there's no way to self-host. You're locked into their pricing forever.

---

## 2. SubTunnel Pricing Strategy

### 2.1 Philosophy: Flat-Rate Beats Usage-Based

**Why developers hate usage-based pricing:**
- Can't predict monthly costs
- Feels like a taxi meter running
- Penalizes success (more users = more cost)
- Creates anxiety about experimentation
- Bill shock destroys trust

**Why flat-rate works for SubTunnel:**
- Developers can budget with certainty
- No punishment for growth within a tier
- Encourages experimentation and adoption
- Builds trust — "what you see is what you pay"
- Simpler billing = fewer support tickets
- Follows the Tailscale model (simple tiers, generous free)

**The Tailscale inspiration:** Tailscale's pricing is beautifully simple — free for personal use with generous limits, then clear paid tiers for teams. No per-GB charges, no metered anything. Developers love it. We follow the same philosophy.

### 2.2 Tier Definitions

#### Free — $0/mo (Self-Hosted)
> Deploy on your own server. No limits on what matters.

| Resource | Limit |
|----------|-------|
| Tunnels (concurrent) | 5 |
| Bandwidth | Unlimited (your server) |
| HTTP requests | Unlimited |
| Custom domains | 3 |
| TCP tunnels | ✅ |
| WebSocket tunnels | ✅ |
| Team members | 3 |
| Request inspector | ✅ |
| API access | ✅ |
| Tunnel timeout | 8 hours (reconnect to refresh) |
| Reserved subdomains | 3 |
| Support | Community (GitHub Issues, Discord) |

**Why it's generous:** 5 concurrent tunnels, 3 custom domains, and unlimited bandwidth covers 80% of solo developers forever. No interstitial page. No bandwidth anxiety. This is what ngrok's free tier *should* be.

#### Pro — $10/mo ($100/yr with annual billing)
> Everything you need for professional use. One price, no surprises.

| Resource | Limit |
|----------|-------|
| Tunnels (concurrent) | 25 |
| Bandwidth | Unlimited |
| HTTP requests | Unlimited |
| Custom domains | 10 |
| TCP tunnels | ✅ |
| TLS tunnels | ✅ |
| WebSocket tunnels | ✅ |
| Team members | 10 |
| Request inspector | ✅ (90-day history) |
| API access | ✅ |
| Tunnel timeout | None (persistent) |
| Reserved subdomains | 25 |
| Alerting (Slack, email, webhook) | ✅ |
| Tunnel templates | ✅ |
| Priority email support | ✅ (24h SLA) |
| Analytics dashboard | ✅ (90-day retention) |

**Why $10/mo:** It's the magic price point for developer tools. Low enough to expense without approval. Lower than ngrok's Hobbyist ($8) while offering 10x more. Covers 95% of individual and small team use cases.

**Annual discount:** $100/yr (2 months free). Annual billing reduces churn and improves cash flow.

#### Team — $30/mo ($300/yr with annual billing)
> For growing teams that need collaboration and control.

| Resource | Limit |
|----------|-------|
| Tunnels (concurrent) | 100 |
| Bandwidth | Unlimited |
| HTTP requests | Unlimited |
| Custom domains | 50 |
| TCP/TLS/WS tunnels | ✅ |
| Team members | 50 |
| Request inspector | ✅ (1-year history) |
| SSO/SAML | ✅ |
| Audit logging | ✅ |
| RBAC (per-tunnel permissions) | ✅ |
| Tunnel templates | ✅ |
| Alerting integrations | ✅ |
| Analytics (1-year retention) | ✅ |
| Priority support | ✅ (12h SLA) |
| Organizations | ✅ |

**Why $30/mo:** Positioned well below ngrok's cost for equivalent usage (~$58-355/mo for teams). SSO and audit logging justify the jump from Pro.

#### Enterprise — Custom Pricing
> For organizations with compliance, scale, or deployment needs.

| Resource | Limit |
|----------|-------|
| Everything in Team | ✅ |
| Tunnels | Unlimited |
| Team members | Unlimited |
| Custom domains | Unlimited |
| Deployment assistance | ✅ |
| Custom SLA (99.9%+) | ✅ |
| BAA for HIPAA | ✅ |
| Dedicated support engineer | ✅ |
| On-premises deployment help | ✅ |
| Custom integrations | ✅ |
| Volume licensing | ✅ |
| Invoice billing (NET 30) | ✅ |

**Target price range:** $200-$1,000/mo depending on organization size and support needs.

### 2.3 Pricing Summary

| | Free | Pro | Team | Enterprise |
|---|------|-----|------|------------|
| **Price (monthly)** | $0 | $10/mo | $30/mo | Custom |
| **Price (annual)** | $0 | $100/yr | $300/yr | Custom |
| **Tunnels** | 5 | 25 | 100 | Unlimited |
| **Team members** | 3 | 10 | 50 | Unlimited |
| **Custom domains** | 3 | 10 | 50 | Unlimited |
| **Bandwidth** | Unlimited | Unlimited | Unlimited | Unlimited |
| **SSO/SAML** | ❌ | ❌ | ✅ | ✅ |
| **Audit log** | ❌ | ❌ | ✅ | ✅ |
| **Support** | Community | Email (24h) | Priority (12h) | Dedicated |

### 2.4 SubTunnel Cloud (Future — v2.0+)

For users who don't want to self-host, we'll offer a managed cloud service:

| | Cloud Starter | Cloud Pro |
|---|---------------|-----------|
| **Price** | $15/mo | $40/mo |
| **Tunnels** | 10 | 50 |
| **Bandwidth** | 50 GB/mo | 200 GB/mo |
| **Custom domains** | 5 | 25 |
| **Team members** | 5 | 25 |
| **Regions** | 1 | 3 (US, EU, Asia) |
| **Uptime SLA** | 99.5% | 99.9% |

Cloud pricing is higher because we bear the infrastructure costs. Overage: $0.10/GB bandwidth after included amount. No per-request charges.

---

## 3. Billing Implementation Plan

### 3.1 Stripe Integration Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│  Dashboard  │────►│  SubTunnel   │────►│   Stripe     │
│  (React)    │     │  API Server  │     │   API        │
│             │     │              │◄────│   Webhooks   │
└─────────────┘     └──────┬───────┘     └─────────────┘
                           │
                    ┌──────▼───────┐
                    │  PostgreSQL   │
                    │  (users,      │
                    │   subscriptions│
                    │   invoices)   │
                    └──────────────┘
```

**Stripe objects we create:**

| Stripe Object | SubTunnel Mapping |
|---------------|-------------------|
| Product: "SubTunnel Pro" | Pro plan |
| Product: "SubTunnel Team" | Team plan |
| Price: pro_monthly ($10/mo) | Monthly Pro billing |
| Price: pro_annual ($100/yr) | Annual Pro billing |
| Price: team_monthly ($30/mo) | Monthly Team billing |
| Price: team_annual ($300/yr) | Annual Team billing |
| Customer | 1:1 with SubTunnel user |
| Subscription | Active plan for a user |
| Checkout Session | Payment flow |
| Billing Portal | Self-service management |

### 3.2 Stripe Webhook Events to Handle

| Event | Action |
|-------|--------|
| `checkout.session.completed` | Create/upgrade subscription in DB, update user plan, send welcome email |
| `customer.subscription.created` | Record subscription, set plan limits |
| `customer.subscription.updated` | Handle plan changes (upgrade/downgrade), update limits |
| `customer.subscription.deleted` | Downgrade to Free, enforce Free limits on next tunnel creation |
| `customer.subscription.trial_will_end` | Send "trial ending in 3 days" email |
| `invoice.paid` | Record invoice, update `current_period_end`, send receipt |
| `invoice.payment_failed` | Mark subscription `past_due`, send "payment failed" email, retry logic |
| `invoice.upcoming` | Optional: send "upcoming charge" notification |
| `customer.updated` | Sync customer email/name changes |
| `payment_method.attached` | Log for audit |

### 3.3 Webhook Handler Design

```rust
// POST /v1/webhooks/stripe
async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    // 1. Verify signature
    let sig = headers.get("stripe-signature")
        .ok_or(ApiError::bad_request("missing stripe-signature"))?;
    let event = stripe::Webhook::construct_event(
        &String::from_utf8_lossy(&body),
        sig.to_str()?,
        &state.config.stripe_webhook_secret,
    ).map_err(|_| ApiError::bad_request("invalid webhook signature"))?;

    // 2. Idempotency check — store event.id, skip if already processed
    if state.db.webhook_event_exists(&event.id).await? {
        return Ok(StatusCode::OK);
    }

    // 3. Process event
    match event.type_.as_str() {
        "checkout.session.completed" => handle_checkout(&state, &event).await?,
        "customer.subscription.updated" => handle_sub_update(&state, &event).await?,
        "customer.subscription.deleted" => handle_sub_cancel(&state, &event).await?,
        "invoice.paid" => handle_invoice_paid(&state, &event).await?,
        "invoice.payment_failed" => handle_payment_failed(&state, &event).await?,
        _ => { /* log and ignore */ }
    }

    // 4. Record processed event
    state.db.record_webhook_event(&event.id, event.type_.as_str()).await?;

    Ok(StatusCode::OK)
}
```

### 3.4 Usage Tracking (for future metered billing)

Even though our pricing is flat-rate, we track usage for:
- Dashboard analytics (show users their usage)
- Abuse detection (flag outliers)
- Future cloud tier metered billing
- Capacity planning

**What we track:**

| Metric | Storage | Granularity |
|--------|---------|-------------|
| Bandwidth (bytes in/out) | Redis (live) + PostgreSQL (hourly) | Per-user, per-tunnel |
| HTTP requests | Redis (live) + PostgreSQL (hourly) | Per-user, per-tunnel |
| TCP connections | Redis (live) + PostgreSQL (hourly) | Per-user, per-tunnel |
| Active tunnels | Redis (real-time) | Per-user |
| Tunnel duration | PostgreSQL (tunnel_sessions) | Per-tunnel |

**Aggregation pipeline:**

```
Real-time (Redis)  →  Hourly rollup (cron job)  →  usage_metrics table
                                                  →  Monthly aggregates for billing dashboard
```

### 3.5 Trial Period Strategy

- **No trial for Free tier** — it's already free
- **14-day free trial for Pro** — no credit card required
  - During trial: full Pro limits
  - 3 days before expiry: email reminder
  - On expiry: auto-downgrade to Free (no charge)
  - If they add payment during trial: seamless conversion
- **14-day free trial for Team** — credit card required (higher tier)
  - Prevents abuse of SSO/audit features
  - Cancel anytime during trial, no charge
- **Enterprise** — 30-day POC with dedicated support

**Stripe trial config:**
```
Subscription.create(
  customer: cus_xxx,
  items: [{ price: price_pro_monthly }],
  trial_period_days: 14,
  trial_settings: {
    end_behavior: { missing_payment_method: "cancel" }  // Pro
    // or "create_invoice" for Team (has card on file)
  }
)
```

### 3.6 Upgrade / Downgrade Flow

**Upgrade (Free → Pro, Pro → Team):**
1. User clicks "Upgrade" in dashboard
2. Create Stripe Checkout Session with the new plan's Price
3. On `checkout.session.completed`: update user plan immediately
4. Prorated billing handled by Stripe (`proration_behavior: "create_prorations"`)
5. New limits take effect immediately

**Downgrade (Team → Pro, Pro → Free):**
1. User clicks "Downgrade" in dashboard
2. Update Stripe subscription to new Price with `proration_behavior: "none"`
3. Downgrade takes effect at end of current billing period
4. User keeps current tier limits until period ends
5. On period end, `customer.subscription.updated` fires → enforce new limits
6. If usage exceeds new limits: don't force-close tunnels, just prevent new ones

**Plan change matrix:**

| From → To | Billing | Limits | When |
|-----------|---------|--------|------|
| Free → Pro | Charge immediately (prorated) | Upgrade immediately | Instant |
| Free → Team | Charge immediately | Upgrade immediately | Instant |
| Pro → Team | Prorate (credit remaining Pro days) | Upgrade immediately | Instant |
| Team → Pro | No charge until next period | Downgrade at period end | End of period |
| Pro → Free | No charge until next period | Downgrade at period end | End of period |
| Team → Free | No charge until next period | Downgrade at period end | End of period |

### 3.7 Cancellation & Refund Policy

**Cancellation:**
- Users can cancel anytime from dashboard (Stripe Billing Portal)
- Subscription remains active until end of billing period
- No immediate downgrade — they paid for the full period
- After period ends: auto-downgrade to Free
- Existing tunnels continue to work until they disconnect

**Refund policy:**
- **Within 7 days of first payment:** Full refund, no questions asked
- **After 7 days:** No refund (subscription remains until period end)
- **Annual plans within 30 days:** Full refund
- **Annual plans after 30 days:** Prorated refund for remaining months (rounded down)
- **Billing errors:** Always refund immediately

**Implementation:**
```rust
// Cancel at period end (default)
stripe::Subscription::update(sub_id, UpdateSubscription {
    cancel_at_period_end: Some(true),
    ..Default::default()
});

// Immediate cancel with refund (within refund window)
stripe::Subscription::cancel(sub_id, CancelSubscription {
    prorate: Some(true),
    ..Default::default()
});
// Then issue refund on latest invoice
stripe::Refund::create(CreateRefund {
    payment_intent: Some(invoice.payment_intent),
    ..Default::default()
});
```

### 3.8 Invoice & Receipt Handling

- **Stripe handles invoice generation** — we don't build our own
- **Stripe Billing Portal** — users manage payment methods, view invoices, download receipts
- **We store invoice metadata** in our `invoices` table for quick dashboard access
- **Receipt emails** sent by Stripe automatically on successful payment
- **Failed payment emails** sent by our system (more actionable than Stripe's defaults)

**Invoice flow:**
1. Stripe generates invoice at period start
2. Stripe attempts charge
3. On success: `invoice.paid` webhook → record in DB, update period
4. On failure: `invoice.payment_failed` → email user, Stripe retries (Smart Retries)
5. After 3 failures: `customer.subscription.updated` (status: `past_due`)
6. After 7 days past_due: cancel subscription, downgrade to Free

---

## 4. Revenue Projections

### 4.1 Infrastructure Costs

**Single EC2 server (Phase 1: 0-1,000 users):**

| Component | Instance | Monthly Cost |
|-----------|----------|-------------|
| App server | t4g.medium (2 vCPU, 4GB) | $30 |
| PostgreSQL | db.t4g.micro (RDS) | $15 |
| Redis | cache.t4g.micro (ElastiCache) | $12 |
| Bandwidth | ~500 GB/mo (EC2 egress) | $45 |
| Domain + TLS | Route53 + ACM | $2 |
| **Total** | | **~$104/mo** |

**Scaled (Phase 2: 1,000-10,000 users):**

| Component | Instance | Monthly Cost |
|-----------|----------|-------------|
| App servers (2x) | t4g.large (2 vCPU, 8GB) × 2 | $120 |
| PostgreSQL | db.t4g.small (RDS Multi-AZ) | $50 |
| Redis | cache.t4g.small (ElastiCache) | $40 |
| Load balancer | ALB | $25 |
| Bandwidth | ~2 TB/mo | $180 |
| Monitoring | Grafana Cloud (free tier) | $0 |
| **Total** | | **~$415/mo** |

**Multi-region (Phase 3: 10,000+ users):**

| Component | Monthly Cost |
|-----------|-------------|
| 3 regions × 2 servers | $360 |
| 3 × RDS + Redis | $270 |
| 3 × ALB | $75 |
| Bandwidth (~10 TB) | $900 |
| **Total** | **~$1,605/mo** |

### 4.2 Break-Even Analysis

**Fixed costs at Phase 1: ~$104/mo**

| Scenario | Paying Users Needed | At $10/mo Pro |
|----------|--------------------|----|
| Cover infra | 11 | 11 Pro users |
| Cover infra + 1 FTE ($5K/mo) | 510 | 510 Pro users |
| Cover infra + 2 FTE ($10K/mo) | 1,010 | 1,010 Pro users |

**With tier mix (realistic: 70% Pro @ $10, 20% Team @ $30, 10% Enterprise @ $500 avg):**

Average revenue per paying user = (0.7 × $10) + (0.2 × $30) + (0.1 × $500) = **$63/mo**

To cover infra + 1 FTE: $5,104 / $63 = **~81 paying users**

### 4.3 Year 1 Revenue Projections

**Assumptions:**
- Free-to-paid conversion: 3% (conservative for dev tools)
- Monthly user growth: 20% months 1-6, 10% months 7-12
- Churn: 5% monthly on paid users
- Tier split of paying users: 75% Pro, 20% Team, 5% Enterprise

#### Conservative Scenario (slow start)

| Month | Total Users | Paying Users | MRR |
|-------|------------|-------------|-----|
| 1 | 100 | 3 | $30 |
| 2 | 120 | 4 | $40 |
| 3 | 200 | 6 | $78 |
| 4 | 350 | 11 | $143 |
| 5 | 500 | 15 | $248 |
| 6 | 800 | 24 | $396 |
| 7 | 1,000 | 30 | $495 |
| 8 | 1,200 | 36 | $594 |
| 9 | 1,500 | 45 | $743 |
| 10 | 1,800 | 54 | $891 |
| 11 | 2,200 | 66 | $1,089 |
| 12 | 2,500 | 75 | $1,238 |

**Year 1 total revenue: ~$5,985**
**Year 1 infra cost: ~$1,248** (Phase 1 for most of year)
**Net (before labor): ~$4,737**

#### Optimistic Scenario (HN front page, viral launch)

| Month | Total Users | Paying Users | MRR |
|-------|------------|-------------|-----|
| 1 | 500 | 15 | $248 |
| 3 | 2,000 | 60 | $990 |
| 6 | 5,000 | 150 | $2,475 |
| 9 | 8,000 | 240 | $3,960 |
| 12 | 12,000 | 360 | $5,940 |

**Year 1 total revenue: ~$28,000**
**Year 1 infra cost: ~$3,000** (scale to Phase 2 mid-year)
**Net (before labor): ~$25,000**

#### Moonshot Scenario (product-market fit + enterprise deals)

| Month | Total Users | Paying Users | MRR |
|-------|------------|-------------|-----|
| 6 | 10,000 | 300 | $4,950 |
| 12 | 30,000 | 900 | $14,850 |

**Year 1 total revenue: ~$80,000**

**MRR calculation basis:** Weighted average $16.50/paying user (75% × $10 + 20% × $30 + 5% × $500)

### 4.4 Key Metrics to Track

| Metric | Target (Month 12) |
|--------|-------------------|
| Monthly Active Users (MAU) | 2,500+ |
| Free-to-paid conversion | 3%+ |
| Monthly churn (paid) | <5% |
| Average Revenue Per User (ARPU) | $16.50 |
| Annual Run Rate (ARR) | $15K+ |
| Net Revenue Retention | 110%+ |
| LTV:CAC ratio | 3:1+ |

---

## 5. Competitive Comparison

### 5.1 Feature-by-Feature Comparison

| Feature | SubTunnel (Free) | SubTunnel (Pro $10) | ngrok (Free) | ngrok (PAYG $20+) | LocalXpose (Free) | LocalXpose (Pro $8) | zrok (Free) | bore (Free) |
|---------|-----------------|--------------------|--------------|--------------------|-------------------|--------------------|-----------|----|
| **Self-hosted** | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| **Open source** | ✅ MIT | ✅ MIT | ❌ | ❌ | ❌ | ❌ | ✅ Apache 2 | ✅ MIT |
| **Concurrent tunnels** | 5 | 25 | 3 | Unlimited | 2 | 10 | Unlimited | Unlimited |
| **Custom domains** | 3 | 10 | ❌ | ✅ ($7.30/mo each) | ❌ | ✅ | ❌ | ❌ |
| **Bandwidth** | Unlimited | Unlimited | 1 GB/mo | 5 GB + $0.10/GB | Limited | "Unlimited"* | Unlimited | Unlimited |
| **HTTP requests** | Unlimited | Unlimited | 20K/mo | 100K + $1/100K | Limited | Unlimited | Unlimited | N/A |
| **TCP tunnels** | ✅ | ✅ | Random port | ✅ | ❌ | ✅ | ✅ | ✅ |
| **TLS tunnels** | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ✅ | ❌ |
| **WebSocket** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |
| **Team members** | 3 | 10 | 1 | 3 + $5/each | 1 | 1 | N/A | N/A |
| **Request inspector** | ✅ | ✅ (90d) | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ |
| **Dashboard** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ |
| **Interstitial page** | Never | Never | Yes | No | Yes | No | No | N/A |
| **SSO/SAML** | ❌ | ❌ | ❌ | Add-on | ❌ | ❌ | ❌ | ❌ |
| **Data sovereignty** | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| **Persistent URLs** | ✅ | ✅ | ❌ | ✅ | ❌ | ✅ | ❌ | ❌ |
| **Auto-reconnect** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ |

*LocalXpose "unlimited" is subject to acceptable use policy.

### 5.2 Price Comparison for Common Use Cases

#### Solo Developer (1 tunnel, custom domain, 5 GB/mo)

| Service | Monthly Cost |
|---------|-------------|
| **SubTunnel Free** | **$0** (+$5 VPS) |
| SubTunnel Pro | $10 |
| ngrok Hobbyist | $8 (no custom domain) |
| ngrok PAYG | $27+ (with custom domain) |
| LocalXpose Pro | $8 |
| zrok | $0 (self-hosted) |
| bore | $0 (no features) |

#### Small Team (5 devs, 5 tunnels, 3 custom domains, 25 GB/mo)

| Service | Monthly Cost |
|---------|-------------|
| **SubTunnel Pro** | **$10** (+$5 VPS) |
| ngrok PAYG | ~$58 |
| LocalXpose Pro | $8 × 5 = $40 (no team plan) |
| zrok | $0 (no team features) |

#### Growing Company (20 devs, 15 tunnels, 10 domains, 100 GB/mo)

| Service | Monthly Cost |
|---------|-------------|
| **SubTunnel Team** | **$30** (+$20 VPS) |
| ngrok PAYG | ~$355 |
| LocalXpose | Enterprise (custom) |

### 5.3 Positioning Summary

| Competitor | SubTunnel's Advantage |
|------------|----------------------|
| **ngrok** | Self-hosted, no vendor lock-in, 10x cheaper, no usage-based surprises, data sovereignty |
| **LocalXpose** | Open source, self-hosted, team features, no per-seat multiplication |
| **zrok** | Better UX, web dashboard, team management, easier setup, better docs |
| **bore** | Full-featured dashboard, custom domains, auth, team management, request inspector |
| **Cloudflare Tunnel** | No ecosystem lock-in, works with any DNS provider, TCP tunnels, open source |
| **frp** | Modern dashboard, team management, better DX, built-in auth, request inspector |

---

## Appendix: Pricing Page Copy

```
## Simple, predictable pricing.
### No per-GB charges. No per-request fees. No surprises.

Free           Pro              Team             Enterprise
$0/mo          $10/mo           $30/mo           Custom
               $100/yr          $300/yr

Self-hosted,   Everything you   SSO, audit logs, Compliance,
open source,   need. One price. and team control  custom SLA,
no limits.     No surprises.    for growing orgs. dedicated support.

[Deploy Free]  [Start Trial]    [Start Trial]    [Contact Sales]
```

---

*Last updated: 2026-02-12*
