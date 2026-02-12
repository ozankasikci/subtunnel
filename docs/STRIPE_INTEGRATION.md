# SubTunnel — Stripe Integration Guide

> Exact Stripe objects, webhook design, checkout flow, and customer portal configuration.
>
> Last updated: 2026-02-12

---

## 1. Stripe Product & Price Objects

### 1.1 Products to Create

```bash
# Create products via Stripe CLI or Dashboard

# Product 1: SubTunnel Pro
stripe products create \
  --name="SubTunnel Pro" \
  --description="25 tunnels, 10 custom domains, 10 team members, priority support" \
  --metadata[plan]="pro" \
  --metadata[tunnels]="25" \
  --metadata[domains]="10" \
  --metadata[members]="10"

# Product 2: SubTunnel Team
stripe products create \
  --name="SubTunnel Team" \
  --description="100 tunnels, 50 custom domains, 50 team members, SSO, audit logs" \
  --metadata[plan]="team" \
  --metadata[tunnels]="100" \
  --metadata[domains]="50" \
  --metadata[members]="50"
```

### 1.2 Prices to Create

```bash
# Pro Monthly — $10/mo
stripe prices create \
  --product=prod_Pro \
  --unit-amount=1000 \
  --currency=usd \
  --recurring[interval]=month \
  --lookup-key="pro_monthly" \
  --metadata[plan]="pro" \
  --metadata[billing]="monthly"

# Pro Annual — $100/yr ($8.33/mo effective)
stripe prices create \
  --product=prod_Pro \
  --unit-amount=10000 \
  --currency=usd \
  --recurring[interval]=year \
  --lookup-key="pro_annual" \
  --metadata[plan]="pro" \
  --metadata[billing]="annual"

# Team Monthly — $30/mo
stripe prices create \
  --product=prod_Team \
  --unit-amount=3000 \
  --currency=usd \
  --recurring[interval]=month \
  --lookup-key="team_monthly" \
  --metadata[plan]="team" \
  --metadata[billing]="monthly"

# Team Annual — $300/yr ($25/mo effective)
stripe prices create \
  --product=prod_Team \
  --unit-amount=30000 \
  --currency=usd \
  --recurring[interval]=year \
  --lookup-key="team_annual" \
  --metadata[plan]="team" \
  --metadata[billing]="annual"
```

### 1.3 Price ID Reference Table

| Lookup Key | Plan | Interval | Amount | Stripe Price ID |
|-----------|------|----------|--------|-----------------|
| `pro_monthly` | Pro | Monthly | $10.00 | `price_xxxx` (auto-generated) |
| `pro_annual` | Pro | Annual | $100.00 | `price_xxxx` |
| `team_monthly` | Team | Monthly | $30.00 | `price_xxxx` |
| `team_annual` | Team | Annual | $300.00 | `price_xxxx` |

> Store lookup keys in config, not raw price IDs. Use `stripe.prices.list(lookup_keys=["pro_monthly"])` to resolve.

---

## 2. Checkout Session Flow

### 2.1 Flow Diagram

```
User clicks "Upgrade to Pro"
        │
        ▼
POST /v1/billing/checkout
  { plan: "pro", interval: "monthly" }
        │
        ▼
Server creates Stripe Checkout Session
  - Finds/creates Stripe Customer for user
  - Sets price_id from lookup key
  - Sets success_url & cancel_url
  - Attaches trial_period_days: 14 (if eligible)
  - Sets metadata: { user_id, plan }
        │
        ▼
Returns { checkout_url: "https://checkout.stripe.com/..." }
        │
        ▼
Frontend redirects to Stripe Checkout
        │
        ▼
User enters payment info → Stripe processes
        │
        ▼
Stripe sends checkout.session.completed webhook
        │
        ▼
Server updates user plan in DB
        │
        ▼
User redirected to success_url (/dashboard?upgraded=true)
```

### 2.2 Checkout Session Creation (Rust)

```rust
use stripe::{
    CheckoutSession, CreateCheckoutSession, CreateCheckoutSessionLineItems,
    CheckoutSessionMode, Customer, CreateCustomer,
};

pub async fn create_checkout(
    user: &User,
    plan: &str,        // "pro" | "team"
    interval: &str,    // "monthly" | "annual"
    stripe_client: &stripe::Client,
    config: &BillingConfig,
) -> Result<String> {
    // 1. Ensure Stripe customer exists
    let customer_id = match &user.stripe_customer_id {
        Some(id) => id.clone(),
        None => {
            let customer = Customer::create(stripe_client, CreateCustomer {
                email: Some(&user.email),
                name: user.name.as_deref(),
                metadata: Some([
                    ("user_id".into(), user.id.clone()),
                ].into()),
                ..Default::default()
            }).await?;
            // Save customer_id to DB
            save_stripe_customer_id(&user.id, &customer.id).await?;
            customer.id.to_string()
        }
    };

    // 2. Resolve price
    let lookup_key = format!("{}_{}", plan, interval);
    let prices = stripe::Price::list(stripe_client, &stripe::ListPrices {
        lookup_keys: Some(vec![lookup_key.clone()]),
        ..Default::default()
    }).await?;
    let price = prices.data.first()
        .ok_or_else(|| anyhow!("price not found for {lookup_key}"))?;

    // 3. Check trial eligibility (only if user has never had a paid plan)
    let trial_days = if user_never_had_paid_plan(&user.id).await? {
        Some(14)
    } else {
        None
    };

    // 4. Create checkout session
    let session = CheckoutSession::create(stripe_client, CreateCheckoutSession {
        customer: Some(customer_id.parse()?),
        mode: Some(CheckoutSessionMode::Subscription),
        line_items: Some(vec![CreateCheckoutSessionLineItems {
            price: Some(price.id.to_string()),
            quantity: Some(1),
            ..Default::default()
        }]),
        success_url: Some(&format!("{}/dashboard?checkout=success", config.app_url)),
        cancel_url: Some(&format!("{}/pricing?checkout=cancelled", config.app_url)),
        subscription_data: Some(stripe::CreateCheckoutSessionSubscriptionData {
            trial_period_days: trial_days.map(|d| d as u32),
            metadata: Some([
                ("user_id".into(), user.id.clone()),
                ("plan".into(), plan.into()),
            ].into()),
            ..Default::default()
        }),
        allow_promotion_codes: Some(true),
        ..Default::default()
    }).await?;

    Ok(session.url.ok_or_else(|| anyhow!("no checkout URL"))?)
}
```

### 2.3 API Endpoint

```rust
// POST /v1/billing/checkout
#[derive(Deserialize)]
struct CheckoutRequest {
    plan: String,       // "pro" | "team"
    interval: String,   // "monthly" | "annual"
}

#[derive(Serialize)]
struct CheckoutResponse {
    checkout_url: String,
}

async fn checkout(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(req): Json<CheckoutRequest>,
) -> Result<Json<CheckoutResponse>, ApiError> {
    // Validate plan
    if !["pro", "team"].contains(&req.plan.as_str()) {
        return Err(ApiError::bad_request("invalid plan"));
    }
    if !["monthly", "annual"].contains(&req.interval.as_str()) {
        return Err(ApiError::bad_request("invalid interval"));
    }

    let user = state.db.get_user(&auth.user_id).await?;
    let url = create_checkout(&user, &req.plan, &req.interval, &state.stripe, &state.config.billing).await?;

    Ok(Json(CheckoutResponse { checkout_url: url }))
}
```

---

## 3. Webhook Endpoint Design

### 3.1 Endpoint Configuration

**URL:** `https://api.subtunnel.dev/v1/webhooks/stripe`

**Events to subscribe to:**
```
checkout.session.completed
customer.subscription.created
customer.subscription.updated
customer.subscription.deleted
customer.subscription.trial_will_end
invoice.paid
invoice.payment_failed
customer.updated
```

**Stripe CLI (development):**
```bash
stripe listen --forward-to localhost:3000/v1/webhooks/stripe \
  --events checkout.session.completed,customer.subscription.created,customer.subscription.updated,customer.subscription.deleted,invoice.paid,invoice.payment_failed
```

### 3.2 Webhook Handler Implementation

```rust
async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let sig = headers.get("stripe-signature")
        .ok_or(ApiError::bad_request("missing signature"))?
        .to_str().map_err(|_| ApiError::bad_request("invalid signature header"))?;

    let event = stripe::Webhook::construct_event(
        &String::from_utf8_lossy(&body),
        sig,
        &state.config.stripe_webhook_secret,
    ).map_err(|e| {
        tracing::warn!("webhook signature verification failed: {e}");
        ApiError::bad_request("invalid signature")
    })?;

    // Idempotency: skip if already processed
    let event_id = event.id.to_string();
    if sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM webhook_events WHERE event_id = $1)", &event_id)
        .fetch_one(&state.db)
        .await?
        .unwrap_or(false)
    {
        return Ok(StatusCode::OK);
    }

    let result = match event.type_.as_str() {
        "checkout.session.completed" => {
            let session: stripe::CheckoutSession = serde_json::from_value(event.data.object.clone())?;
            handle_checkout_complete(&state, session).await
        }
        "customer.subscription.updated" => {
            let sub: stripe::Subscription = serde_json::from_value(event.data.object.clone())?;
            handle_subscription_updated(&state, sub).await
        }
        "customer.subscription.deleted" => {
            let sub: stripe::Subscription = serde_json::from_value(event.data.object.clone())?;
            handle_subscription_deleted(&state, sub).await
        }
        "customer.subscription.trial_will_end" => {
            let sub: stripe::Subscription = serde_json::from_value(event.data.object.clone())?;
            handle_trial_ending(&state, sub).await
        }
        "invoice.paid" => {
            let invoice: stripe::Invoice = serde_json::from_value(event.data.object.clone())?;
            handle_invoice_paid(&state, invoice).await
        }
        "invoice.payment_failed" => {
            let invoice: stripe::Invoice = serde_json::from_value(event.data.object.clone())?;
            handle_payment_failed(&state, invoice).await
        }
        _ => {
            tracing::debug!("unhandled webhook event: {}", event.type_);
            Ok(())
        }
    };

    // Record event regardless of outcome (for idempotency)
    sqlx::query!(
        "INSERT INTO webhook_events (event_id, event_type, processed_at, success) VALUES ($1, $2, NOW(), $3)",
        &event_id, event.type_.as_str(), result.is_ok()
    ).execute(&state.db).await?;

    match result {
        Ok(()) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("webhook processing error: {e}");
            // Return 500 so Stripe retries
            Err(ApiError::internal(format!("processing error: {e}")))
        }
    }
}
```

### 3.3 Event Handlers

```rust
async fn handle_checkout_complete(state: &AppState, session: stripe::CheckoutSession) -> Result<()> {
    let customer_id = session.customer.ok_or(anyhow!("no customer"))?;
    let subscription_id = session.subscription.ok_or(anyhow!("no subscription"))?;

    // Find user by stripe_customer_id
    let user = sqlx::query!("SELECT id FROM users WHERE stripe_customer_id = $1", customer_id.to_string())
        .fetch_one(&state.db).await?;

    // Fetch full subscription to get plan details
    let sub = stripe::Subscription::retrieve(&state.stripe, &subscription_id, &[]).await?;
    let price = &sub.items.data[0].price;
    let plan = price.metadata.get("plan").cloned().unwrap_or_else(|| "pro".into());

    // Update user plan
    sqlx::query!("UPDATE users SET plan = $1, updated_at = NOW() WHERE id = $2", &plan, &user.id)
        .execute(&state.db).await?;

    // Record subscription
    sqlx::query!(
        "INSERT INTO subscriptions (user_id, stripe_subscription_id, stripe_price_id, plan, status, current_period_start, current_period_end)
         VALUES ($1, $2, $3, $4, $5, to_timestamp($6), to_timestamp($7))
         ON CONFLICT (stripe_subscription_id) DO UPDATE SET
           plan = EXCLUDED.plan, status = EXCLUDED.status,
           current_period_start = EXCLUDED.current_period_start,
           current_period_end = EXCLUDED.current_period_end",
        &user.id,
        subscription_id.to_string(),
        price.id.to_string(),
        &plan,
        sub.status.map(|s| s.to_string()).unwrap_or_else(|| "active".into()),
        sub.current_period_start.map(|t| t as f64),
        sub.current_period_end.map(|t| t as f64),
    ).execute(&state.db).await?;

    // Send welcome email
    state.email.send_plan_activated(&user.id, &plan).await?;

    // Audit log
    audit(&state.db, AuditEvent::plan_change(&user.id, "free", &plan)).await;

    Ok(())
}

async fn handle_subscription_updated(state: &AppState, sub: stripe::Subscription) -> Result<()> {
    let customer_id = sub.customer.to_string();
    let user = sqlx::query!("SELECT id, plan as current_plan FROM users WHERE stripe_customer_id = $1", &customer_id)
        .fetch_one(&state.db).await?;

    let price = &sub.items.data[0].price;
    let new_plan = price.metadata.get("plan").cloned().unwrap_or_else(|| "pro".into());
    let status = sub.status.map(|s| s.to_string()).unwrap_or_else(|| "active".into());

    // Update subscription record
    sqlx::query!(
        "UPDATE subscriptions SET plan = $1, status = $2, stripe_price_id = $3,
         current_period_start = to_timestamp($4), current_period_end = to_timestamp($5)
         WHERE stripe_subscription_id = $6",
        &new_plan, &status, price.id.to_string(),
        sub.current_period_start.map(|t| t as f64),
        sub.current_period_end.map(|t| t as f64),
        sub.id.to_string(),
    ).execute(&state.db).await?;

    // Update user plan if status is active
    if status == "active" || status == "trialing" {
        sqlx::query!("UPDATE users SET plan = $1, updated_at = NOW() WHERE id = $2", &new_plan, &user.id)
            .execute(&state.db).await?;
    }

    // If past_due or canceled, downgrade
    if status == "past_due" || status == "canceled" || status == "unpaid" {
        sqlx::query!("UPDATE users SET plan = 'free', updated_at = NOW() WHERE id = $1", &user.id)
            .execute(&state.db).await?;
    }

    Ok(())
}

async fn handle_subscription_deleted(state: &AppState, sub: stripe::Subscription) -> Result<()> {
    let customer_id = sub.customer.to_string();

    sqlx::query!(
        "UPDATE users SET plan = 'free', updated_at = NOW() WHERE stripe_customer_id = $1",
        &customer_id
    ).execute(&state.db).await?;

    sqlx::query!(
        "UPDATE subscriptions SET status = 'canceled' WHERE stripe_subscription_id = $1",
        sub.id.to_string()
    ).execute(&state.db).await?;

    // Send cancellation confirmation email
    let user = sqlx::query!("SELECT id FROM users WHERE stripe_customer_id = $1", &customer_id)
        .fetch_one(&state.db).await?;
    state.email.send_plan_canceled(&user.id).await?;

    Ok(())
}

async fn handle_trial_ending(state: &AppState, sub: stripe::Subscription) -> Result<()> {
    let customer_id = sub.customer.to_string();
    let user = sqlx::query!("SELECT id, email FROM users WHERE stripe_customer_id = $1", &customer_id)
        .fetch_one(&state.db).await?;

    // Send "trial ending in 3 days" email
    state.email.send_trial_ending(&user.id, &user.email).await?;

    Ok(())
}

async fn handle_invoice_paid(state: &AppState, invoice: stripe::Invoice) -> Result<()> {
    let customer_id = invoice.customer.map(|c| c.to_string()).ok_or(anyhow!("no customer"))?;
    let user = sqlx::query!("SELECT id FROM users WHERE stripe_customer_id = $1", &customer_id)
        .fetch_one(&state.db).await?;

    sqlx::query!(
        "INSERT INTO invoices (user_id, stripe_invoice_id, amount_cents, currency, status, period_start, period_end, pdf_url)
         VALUES ($1, $2, $3, $4, 'paid', to_timestamp($5), to_timestamp($6), $7)
         ON CONFLICT (stripe_invoice_id) DO UPDATE SET status = 'paid', pdf_url = EXCLUDED.pdf_url",
        &user.id,
        invoice.id.map(|id| id.to_string()),
        invoice.amount_paid.map(|a| a as i32),
        invoice.currency.as_deref().unwrap_or("usd"),
        invoice.period_start.map(|t| t as f64),
        invoice.period_end.map(|t| t as f64),
        invoice.invoice_pdf.as_deref(),
    ).execute(&state.db).await?;

    Ok(())
}

async fn handle_payment_failed(state: &AppState, invoice: stripe::Invoice) -> Result<()> {
    let customer_id = invoice.customer.map(|c| c.to_string()).ok_or(anyhow!("no customer"))?;
    let user = sqlx::query!("SELECT id, email FROM users WHERE stripe_customer_id = $1", &customer_id)
        .fetch_one(&state.db).await?;

    // Send payment failed email with update link
    let portal_url = create_billing_portal_url(&state, &customer_id).await?;
    state.email.send_payment_failed(&user.email, &portal_url).await?;

    Ok(())
}
```

### 3.4 Webhook Events Table (for idempotency)

```sql
CREATE TABLE webhook_events (
    event_id    TEXT PRIMARY KEY,
    event_type  TEXT NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    success     BOOLEAN NOT NULL DEFAULT true
);

-- Auto-cleanup after 90 days
CREATE INDEX idx_webhook_events_created ON webhook_events(processed_at);
```

---

## 4. Customer Portal Configuration

### 4.1 Portal Setup

The Stripe Billing Portal lets customers:
- Update payment methods
- View and download invoices
- Cancel or resume subscriptions
- Switch between monthly/annual billing

**Configuration (via Stripe Dashboard → Settings → Billing → Customer portal):**

| Setting | Value |
|---------|-------|
| Invoice history | ✅ Enabled |
| Payment method update | ✅ Enabled |
| Subscription cancellation | ✅ Enabled (cancel at period end) |
| Subscription pause | ❌ Disabled |
| Plan switching | ✅ Enabled (Pro ↔ Team, monthly ↔ annual) |
| Proration | ✅ Enabled (always prorate on upgrade) |
| Promotion codes | ✅ Enabled |

### 4.2 Portal Session Creation

```rust
// POST /v1/billing/portal
async fn billing_portal(
    State(state): State<AppState>,
    auth: AuthContext,
) -> Result<Json<PortalResponse>, ApiError> {
    let user = state.db.get_user(&auth.user_id).await?;
    let customer_id = user.stripe_customer_id
        .ok_or(ApiError::bad_request("no billing account — upgrade first"))?;

    let session = stripe::BillingPortalSession::create(&state.stripe, stripe::CreateBillingPortalSession {
        customer: customer_id.parse()?,
        return_url: Some(&format!("{}/dashboard/billing", state.config.app_url)),
        ..Default::default()
    }).await?;

    Ok(Json(PortalResponse { portal_url: session.url }))
}

#[derive(Serialize)]
struct PortalResponse {
    portal_url: String,
}
```

### 4.3 Portal Allowed Plan Switches

Configure in Stripe Dashboard under Customer Portal → Subscriptions:

| From | To | Proration |
|------|----|-----------|
| Pro Monthly ($10/mo) | Pro Annual ($100/yr) | Credit remaining days |
| Pro Monthly ($10/mo) | Team Monthly ($30/mo) | Charge prorated difference |
| Pro Monthly ($10/mo) | Team Annual ($300/yr) | Charge prorated difference |
| Pro Annual ($100/yr) | Team Annual ($300/yr) | Charge prorated difference |
| Team Monthly ($30/mo) | Team Annual ($300/yr) | Credit remaining days |
| Team Monthly ($30/mo) | Pro Monthly ($10/mo) | Credit, apply at renewal |
| Team Annual ($300/yr) | Pro Annual ($100/yr) | Credit, apply at renewal |

---

## 5. Billing API Endpoints Summary

```rust
// GET /v1/billing — Current billing status
async fn billing_info(auth: AuthContext, State(state): State<AppState>) -> Result<Json<BillingInfo>> {
    let user = state.db.get_user(&auth.user_id).await?;
    let subscription = state.db.get_active_subscription(&auth.user_id).await?;
    let invoices = state.db.get_recent_invoices(&auth.user_id, 5).await?;

    Ok(Json(BillingInfo {
        plan: user.plan,
        subscription: subscription.map(|s| SubscriptionInfo {
            status: s.status,
            plan: s.plan,
            current_period_end: s.current_period_end,
            cancel_at: s.cancel_at,
        }),
        invoices: invoices.into_iter().map(|i| InvoiceInfo {
            amount_cents: i.amount_cents,
            status: i.status,
            period_start: i.period_start,
            period_end: i.period_end,
            pdf_url: i.pdf_url,
        }).collect(),
    }))
}

// Response shape:
#[derive(Serialize)]
struct BillingInfo {
    plan: String,
    subscription: Option<SubscriptionInfo>,
    invoices: Vec<InvoiceInfo>,
}

#[derive(Serialize)]
struct SubscriptionInfo {
    status: String,          // "active" | "trialing" | "past_due" | "canceled"
    plan: String,            // "pro" | "team"
    current_period_end: DateTime<Utc>,
    cancel_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
struct InvoiceInfo {
    amount_cents: i32,
    status: String,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    pdf_url: Option<String>,
}
```

---

## 6. Environment Variables

```bash
# .env (production)
STRIPE_SECRET_KEY=sk_live_...
STRIPE_PUBLISHABLE_KEY=pk_live_...
STRIPE_WEBHOOK_SECRET=whsec_...

# Price lookup keys (resolved at runtime)
STRIPE_PRO_MONTHLY_LOOKUP=pro_monthly
STRIPE_PRO_ANNUAL_LOOKUP=pro_annual
STRIPE_TEAM_MONTHLY_LOOKUP=team_monthly
STRIPE_TEAM_ANNUAL_LOOKUP=team_annual

# App URLs
APP_URL=https://app.subtunnel.dev
API_URL=https://api.subtunnel.dev
```

---

## 7. Testing Checklist

### Stripe Test Mode

| Scenario | Test Card | Expected |
|----------|-----------|----------|
| Successful payment | `4242 4242 4242 4242` | checkout.session.completed → plan upgraded |
| Card declined | `4000 0000 0000 0002` | Checkout fails, user stays on current plan |
| Requires 3D Secure | `4000 0025 0000 3155` | 3DS challenge, then success |
| Trial → convert | `4242...` + 14-day trial | trial_will_end email, then invoice.paid |
| Trial → expire (no card) | No card on Pro trial | subscription.deleted → downgrade to Free |
| Payment fails → retry | `4000 0000 0000 0341` | invoice.payment_failed, Stripe retries |
| Cancel subscription | Via portal | cancel_at_period_end = true, downgrade at end |
| Upgrade Pro → Team | Via portal | Prorated charge, immediate plan change |
| Downgrade Team → Pro | Via portal | Change at period end, no immediate charge |

### Integration Tests

```bash
# Run Stripe CLI in test mode
stripe listen --forward-to localhost:3000/v1/webhooks/stripe

# Trigger test events
stripe trigger checkout.session.completed
stripe trigger customer.subscription.updated
stripe trigger customer.subscription.deleted
stripe trigger invoice.paid
stripe trigger invoice.payment_failed
```

---

*Last updated: 2026-02-12*
