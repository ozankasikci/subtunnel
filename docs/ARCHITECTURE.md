# SubTunnel — Production SaaS Architecture

*This document is a forward-looking design. The current implementation consists of the CLI (server and local commands) and the marketing site. Sections describing Postgres, Redis, JWT authentication, dashboards, and multi-region operation are future design, not shipped functionality.*

> Technical architecture document for evolving SubTunnel from a single-user self-hosted tunnel into a multi-tenant SaaS product.
>
> Last updated: 2026-02-12

---

## Table of Contents

1. [Current State Assessment](#1-current-state-assessment)
2. [Auth System Design](#2-auth-system-design)
3. [Multi-tenancy](#3-multi-tenancy)
4. [API Design](#4-api-design)
5. [Database Schema](#5-database-schema)
6. [Rate Limiting & Abuse Prevention](#6-rate-limiting--abuse-prevention)
7. [Monitoring & Observability](#7-monitoring--observability)
8. [Scaling Strategy](#8-scaling-strategy)
9. [Security](#9-security)
10. [CLI Auth Flow](#10-cli-auth-flow)
11. [Billing Integration](#11-billing-integration)
12. [Deployment](#12-deployment)

---

## 1. Current State Assessment

### What We Have

SubTunnel is a functional TCP/HTTP tunnel with a clean Rust architecture:

| Component | Status | Notes |
|-----------|--------|-------|
| **Control Protocol** | ✅ Complete | JSON + 4-byte length-prefix framing over yamux-multiplexed TLS streams |
| **Server** | ✅ Complete | Accepts agent connections, manages tunnel registry, routes HTTP by Host header |
| **Client** | ✅ Complete | Connects with TLS + yamux, auto-reconnect with exponential backoff |
| **TLS** | ✅ Dev-ready | Self-signed cert generation via `rcgen`; `NoVerifier` on client side |
| **Mux** | ✅ Complete | Yamux session driver with background polling, channel-based stream delivery |
| **Auth** | ⚠️ Minimal | Single shared token (`SUBTUNNEL_TOKEN`), constant-time comparison |
| **Subdomain routing** | ✅ Complete | Host header sniffing, wildcard domain matching, custom subdomain requests |
| **Heartbeat** | ✅ Complete | Bi-directional heartbeat every 30s |

### Architecture Diagram (Current)

```
┌──────────────┐         TLS + Yamux         ┌──────────────────┐
│  CLI Client  │ ◄──────────────────────────► │   SubTunnel      │
│  (subtunnel  │    Control Stream (auth,     │   Server         │
│   local)     │     heartbeat, tunnel req)   │                  │
│              │    Data Streams (proxied      │  ┌────────────┐ │    ┌─────────┐
│  localhost:N │     TCP connections)          │  │ HTTP       │ │◄───│ Internet │
└──────────────┘                              │  │ Listener   │ │    │ Traffic  │
                                              │  │ (:8080)    │ │    └─────────┘
                                              │  └────────────┘ │
                                              │  ┌────────────┐ │
                                              │  │ TunnelMgr  │ │
                                              │  │ (in-memory)│ │
                                              │  └────────────┘ │
                                              └──────────────────┘
```

### What's Missing for Production

| Gap | Priority | Effort |
|-----|----------|--------|
| **Per-user auth** (JWT/API keys, not shared token) | P0 | Medium |
| **Persistent state** (DB for users, tunnels, keys) | P0 | Medium |
| **Multi-tenancy** (user isolation, resource limits) | P0 | High |
| **Management API** (REST for dashboard) | P0 | High |
| **Proper TLS** (ACME/Let's Encrypt, no self-signed) | P0 | Medium |
| **Rate limiting** (per-user bandwidth/connection caps) | P1 | Medium |
| **Billing** (Stripe, plan enforcement) | P1 | High |
| **Observability** (Prometheus, structured logs, tracing) | P1 | Medium |
| **Horizontal scaling** (multi-node, tunnel state sync) | P2 | High |
| **Audit logging** | P2 | Low |
| **RBAC** (team accounts, roles) | P2 | Medium |
| **TCP tunnel port allocation** (not just HTTP subdomain) | P2 | Medium |

---

## 2. Auth System Design

### Authentication Methods

SubTunnel supports three auth mechanisms:

| Method | Use Case | Format |
|--------|----------|--------|
| **API Key** | CLI, CI/CD, programmatic access | `stk_live_<base62(32 bytes)>` |
| **JWT** | Dashboard sessions, short-lived | RS256, 15min access + 7d refresh |
| **OAuth2** | `subtunnel login` browser flow | GitHub/Google provider → JWT exchange |

### API Key Format

```
stk_live_a1B2c3D4e5F6g7H8i9J0k1L2m3N4o5P6    (live key)
stk_test_a1B2c3D4e5F6g7H8i9J0k1L2m3N4o5P6    (test/sandbox)
```

Keys are stored as SHA-256 hashes in the database. The prefix (`stk_live_`, `stk_test_`) is stored in plaintext for identification. Only the first 8 characters of the raw key are stored for display (`stk_live_a1B2c3D4...`).

### JWT Claims

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — user ID
    pub sub: String,
    /// Issuer
    pub iss: String,            // "subtunnel"
    /// Expiration (Unix timestamp)
    pub exp: u64,
    /// Issued at
    pub iat: u64,
    /// Token type
    pub typ: TokenType,         // "access" | "refresh"
    /// User's plan
    pub plan: Plan,
    /// Organization ID (if team account)
    pub org_id: Option<String>,
    /// Scopes
    pub scopes: Vec<String>,    // ["tunnels:create", "tunnels:read", "keys:manage"]
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}
```

### Auth Flow: CLI → Server

```
┌────────┐                    ┌─────────────┐                 ┌──────────┐
│  CLI   │                    │  SubTunnel   │                 │   DB     │
│        │                    │  Server      │                 │          │
│        │ ── Auth{token} ──► │              │                 │          │
│        │                    │  if starts   │                 │          │
│        │                    │  "stk_"  ──► │ lookup hash ──► │          │
│        │                    │              │ ◄── user_id ─── │          │
│        │                    │  if "ey"  ─► │ verify JWT sig  │          │
│        │                    │              │ extract claims   │          │
│        │ ◄─ AuthResp ────── │              │                 │          │
└────────┘                    └─────────────┘                 └──────────┘
```

### Updated Control Message

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlMessage {
    Auth {
        token: String,            // API key or JWT
        #[serde(skip_serializing_if = "Option::is_none")]
        client_version: Option<String>,
    },
    AuthResp {
        success: bool,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        plan: Option<Plan>,
    },
    RegisterReq {
        protocol: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        subdomain: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        labels: Option<HashMap<String, String>>,
    },
    RegisterResp {
        success: bool,
        tunnel_id: String,
        subdomain: String,
        message: String,
    },
    Heartbeat,
    HeartbeatAck,
    Error {
        code: String,
        message: String,
    },
}
```

### Rust Auth Middleware

```rust
use sha2::{Sha256, Digest};

pub struct AuthContext {
    pub user_id: String,
    pub plan: Plan,
    pub org_id: Option<String>,
    pub scopes: Vec<String>,
    pub auth_method: AuthMethod,
}

pub enum AuthMethod {
    ApiKey { key_id: String },
    Jwt,
}

pub async fn authenticate(token: &str, db: &DbPool, jwt_keys: &JwtKeys) -> Result<AuthContext> {
    if token.starts_with("stk_") {
        authenticate_api_key(token, db).await
    } else {
        authenticate_jwt(token, jwt_keys)
    }
}

async fn authenticate_api_key(key: &str, db: &DbPool) -> Result<AuthContext> {
    let hash = hex::encode(Sha256::digest(key.as_bytes()));
    let row = sqlx::query!(
        "SELECT ak.id, ak.user_id, ak.scopes, u.plan, u.org_id
         FROM api_keys ak JOIN users u ON ak.user_id = u.id
         WHERE ak.key_hash = $1 AND ak.revoked_at IS NULL AND (ak.expires_at IS NULL OR ak.expires_at > NOW())",
        hash
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| anyhow::anyhow!("invalid API key"))?;

    // Update last_used_at
    sqlx::query!("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1", row.id)
        .execute(db)
        .await?;

    Ok(AuthContext {
        user_id: row.user_id,
        plan: serde_json::from_str(&row.plan)?,
        org_id: row.org_id,
        scopes: serde_json::from_str(&row.scopes)?,
        auth_method: AuthMethod::ApiKey { key_id: row.id },
    })
}
```

---

## 3. Multi-tenancy

### Isolation Model

SubTunnel uses **logical multi-tenancy** — all users share the same server process but are isolated at the application layer.

```
┌───────────────────────────────────────────────────┐
│                SubTunnel Server                    │
│                                                   │
│  ┌─────────────────────────────────────────────┐  │
│  │            Connection Manager                │  │
│  │  ┌────────┐  ┌────────┐  ┌────────┐        │  │
│  │  │ User A │  │ User B │  │ User C │        │  │
│  │  │ 2 tun  │  │ 5 tun  │  │ 1 tun  │        │  │
│  │  │ Free   │  │ Pro    │  │ Ent    │        │  │
│  │  └────────┘  └────────┘  └────────┘        │  │
│  └─────────────────────────────────────────────┘  │
│                                                   │
│  ┌──────────────┐  ┌──────────────┐              │
│  │ Rate Limiter │  │ Bandwidth    │              │
│  │ (per-user)   │  │ Tracker      │              │
│  └──────────────┘  └──────────────┘              │
└───────────────────────────────────────────────────┘
```

### Resource Limits by Plan

```rust
#[derive(Debug, Clone)]
pub struct PlanLimits {
    pub max_tunnels: u32,
    pub max_connections_per_tunnel: u32,
    pub bandwidth_bytes_per_month: u64,
    pub max_subdomains: u32,
    pub custom_subdomains: bool,
    pub max_requests_per_minute: u32,
    pub tunnel_timeout_hours: Option<u32>,  // None = unlimited
    pub tcp_tunnels: bool,
    pub tls_tunnels: bool,
}

impl PlanLimits {
    pub fn for_plan(plan: &Plan) -> Self {
        match plan {
            Plan::Free => Self {
                max_tunnels: 3,
                max_connections_per_tunnel: 20,
                bandwidth_bytes_per_month: 1_073_741_824,    // 1 GB
                max_subdomains: 3,
                custom_subdomains: false,
                max_requests_per_minute: 60,
                tunnel_timeout_hours: Some(8),
                tcp_tunnels: false,
                tls_tunnels: false,
            },
            Plan::Pro => Self {
                max_tunnels: 20,
                max_connections_per_tunnel: 100,
                bandwidth_bytes_per_month: 107_374_182_400,  // 100 GB
                max_subdomains: 20,
                custom_subdomains: true,
                max_requests_per_minute: 600,
                tunnel_timeout_hours: None,
                tcp_tunnels: true,
                tls_tunnels: true,
            },
            Plan::Enterprise => Self {
                max_tunnels: 1000,
                max_connections_per_tunnel: 1000,
                bandwidth_bytes_per_month: 1_099_511_627_776, // 1 TB
                max_subdomains: 1000,
                custom_subdomains: true,
                max_requests_per_minute: 6000,
                tunnel_timeout_hours: None,
                tcp_tunnels: true,
                tls_tunnels: true,
            },
        }
    }
}
```

### Enforcement in TunnelManager

```rust
impl TunnelManager {
    pub async fn register(
        &self,
        auth: &AuthContext,
        protocol: &str,
        requested_subdomain: Option<&str>,
        db: &DbPool,
    ) -> Result<RegisteredTunnel> {
        let limits = PlanLimits::for_plan(&auth.plan);

        // Check tunnel count
        let current_count = self.user_tunnel_count(&auth.user_id).await;
        if current_count >= limits.max_tunnels {
            bail!("tunnel limit reached ({}/{}). Upgrade your plan.", current_count, limits.max_tunnels);
        }

        // Check custom subdomain permission
        if requested_subdomain.is_some() && !limits.custom_subdomains {
            bail!("custom subdomains require a Pro plan or higher");
        }

        // Check protocol permission
        if protocol == "tcp" && !limits.tcp_tunnels {
            bail!("TCP tunnels require a Pro plan or higher");
        }

        // Check reserved subdomains
        if let Some(sub) = requested_subdomain {
            let owner = sqlx::query_scalar!(
                "SELECT user_id FROM reserved_subdomains WHERE subdomain = $1",
                sub
            ).fetch_optional(db).await?;
            if let Some(owner_id) = owner {
                if owner_id != auth.user_id {
                    bail!("subdomain '{sub}' is reserved by another user");
                }
            }
        }

        // Check bandwidth
        let used = self.get_monthly_bandwidth(&auth.user_id, db).await?;
        if used >= limits.bandwidth_bytes_per_month {
            bail!("monthly bandwidth limit exceeded");
        }

        // Proceed with registration
        self.register_inner(auth, protocol, requested_subdomain, &limits).await
    }
}
```

### User Session Tracking

```rust
/// Tracks all resources for a connected user.
pub struct UserSession {
    pub user_id: String,
    pub plan: Plan,
    pub connected_at: Instant,
    pub tunnels: Vec<String>,           // tunnel IDs
    pub bytes_up: AtomicU64,
    pub bytes_down: AtomicU64,
    pub active_connections: AtomicU32,
}
```

---

## 4. API Design

### Base URL & Conventions

```
Base URL:     https://api.subtunnel.dev/v1
Auth:         Authorization: Bearer <api_key_or_jwt>
Content-Type: application/json
Versioning:   URL path (/v1/)
Pagination:   ?cursor=<id>&limit=<n>  (max 100, default 25)
```

### Error Format

```json
{
  "error": {
    "code": "ERR_TUNNEL_LIMIT",
    "message": "Tunnel limit reached (3/3). Upgrade your plan.",
    "status": 429,
    "request_id": "req_a1b2c3d4"
  }
}
```

### Endpoints

#### Authentication

```
POST   /v1/auth/login          # Email/password login → JWT pair
POST   /v1/auth/refresh         # Refresh token → new JWT pair
POST   /v1/auth/logout          # Revoke refresh token
POST   /v1/auth/oauth/github    # OAuth callback → JWT pair
POST   /v1/auth/oauth/google    # OAuth callback → JWT pair
POST   /v1/auth/device/begin    # Device auth flow (CLI login)
POST   /v1/auth/device/poll     # Poll for device auth completion
```

##### `POST /v1/auth/device/begin`

Used by `subtunnel login` to initiate the browser-based auth flow.

```json
// Request
{ "client_id": "subtunnel-cli" }

// Response 200
{
  "device_code": "dc_a1b2c3d4e5f6",
  "user_code": "ABCD-1234",
  "verification_url": "https://app.subtunnel.dev/activate",
  "expires_in": 900,
  "interval": 5
}
```

##### `POST /v1/auth/device/poll`

```json
// Request
{ "device_code": "dc_a1b2c3d4e5f6" }

// Response 200 (success)
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_type": "bearer",
  "expires_in": 900,
  "user": { "id": "usr_abc", "email": "user@example.com", "plan": "pro" }
}

// Response 400 (pending)
{ "error": { "code": "authorization_pending", "message": "waiting for user" } }
```

#### Users

```
GET    /v1/user                 # Current user profile
PATCH  /v1/user                 # Update profile
GET    /v1/user/usage           # Current billing period usage
```

##### `GET /v1/user`

```json
// Response 200
{
  "id": "usr_a1b2c3d4",
  "email": "user@example.com",
  "name": "Ozan",
  "plan": "pro",
  "org_id": null,
  "created_at": "2026-01-15T10:30:00Z",
  "usage": {
    "tunnels_active": 3,
    "tunnels_limit": 20,
    "bandwidth_used_bytes": 5368709120,
    "bandwidth_limit_bytes": 107374182400,
    "period_start": "2026-02-01T00:00:00Z",
    "period_end": "2026-03-01T00:00:00Z"
  }
}
```

#### Tunnels

```
GET    /v1/tunnels              # List active tunnels
GET    /v1/tunnels/:id          # Get tunnel details
DELETE /v1/tunnels/:id          # Force-close a tunnel
GET    /v1/tunnels/:id/requests # Recent requests log
```

##### `GET /v1/tunnels`

```json
// Response 200
{
  "tunnels": [
    {
      "id": "t_a1b2c3d4e5f6",
      "subdomain": "myapp",
      "public_url": "https://myapp.subtunnel.dev",
      "protocol": "http",
      "status": "online",
      "created_at": "2026-02-12T19:30:00Z",
      "last_activity_at": "2026-02-12T19:41:00Z",
      "metrics": {
        "connections_total": 1542,
        "connections_active": 3,
        "bytes_in": 2048576,
        "bytes_out": 10485760
      }
    }
  ],
  "cursor": "t_prev_id",
  "has_more": false
}
```

#### API Keys

```
GET    /v1/api-keys             # List API keys
POST   /v1/api-keys             # Create a new API key
DELETE /v1/api-keys/:id         # Revoke an API key
```

##### `POST /v1/api-keys`

```json
// Request
{
  "name": "CI/CD Pipeline",
  "scopes": ["tunnels:create", "tunnels:read"],
  "expires_in_days": 90
}

// Response 201
{
  "id": "key_a1b2c3d4",
  "name": "CI/CD Pipeline",
  "key": "stk_live_a1B2c3D4e5F6g7H8i9J0k1L2m3N4o5P6",
  "prefix": "stk_live_a1B2c3D4",
  "scopes": ["tunnels:create", "tunnels:read"],
  "expires_at": "2026-05-13T19:41:00Z",
  "created_at": "2026-02-12T19:41:00Z"
}
```

> ⚠️ The full `key` is returned **only once** at creation time.

#### Reserved Subdomains

```
GET    /v1/subdomains           # List reserved subdomains
POST   /v1/subdomains           # Reserve a subdomain
DELETE /v1/subdomains/:name     # Release a subdomain
```

##### `POST /v1/subdomains`

```json
// Request
{ "subdomain": "myapp" }

// Response 201
{
  "subdomain": "myapp",
  "public_url": "https://myapp.subtunnel.dev",
  "reserved_at": "2026-02-12T19:41:00Z"
}

// Response 409
{ "error": { "code": "ERR_SUBDOMAIN_TAKEN", "message": "subdomain 'myapp' is already reserved" } }
```

#### Billing

```
GET    /v1/billing              # Current plan & subscription info
POST   /v1/billing/checkout     # Create Stripe checkout session
POST   /v1/billing/portal       # Create Stripe billing portal session
GET    /v1/billing/invoices     # List invoices
```

#### Admin (internal)

```
GET    /v1/admin/users          # List all users (admin only)
GET    /v1/admin/tunnels        # List all active tunnels
GET    /v1/admin/stats          # Server stats
POST   /v1/admin/users/:id/ban  # Ban a user
```

### API Rust Types

```rust
use axum::{Router, routing::{get, post, delete, patch}};

pub fn api_router(state: AppState) -> Router {
    Router::new()
        // Auth
        .route("/v1/auth/device/begin", post(auth::device_begin))
        .route("/v1/auth/device/poll", post(auth::device_poll))
        .route("/v1/auth/login", post(auth::login))
        .route("/v1/auth/refresh", post(auth::refresh))
        .route("/v1/auth/logout", post(auth::logout))
        // User
        .route("/v1/user", get(users::get_current).patch(users::update))
        .route("/v1/user/usage", get(users::usage))
        // Tunnels
        .route("/v1/tunnels", get(tunnels::list))
        .route("/v1/tunnels/:id", get(tunnels::get).delete(tunnels::close))
        .route("/v1/tunnels/:id/requests", get(tunnels::requests))
        // API Keys
        .route("/v1/api-keys", get(keys::list).post(keys::create))
        .route("/v1/api-keys/:id", delete(keys::revoke))
        // Subdomains
        .route("/v1/subdomains", get(subdomains::list).post(subdomains::reserve))
        .route("/v1/subdomains/:name", delete(subdomains::release))
        // Billing
        .route("/v1/billing", get(billing::info))
        .route("/v1/billing/checkout", post(billing::checkout))
        .route("/v1/billing/portal", post(billing::portal))
        .route("/v1/billing/invoices", get(billing::invoices))
        // Webhooks
        .route("/v1/webhooks/stripe", post(billing::stripe_webhook))
        .with_state(state)
}
```

---

## 5. Database Schema

PostgreSQL is the primary data store. Redis is used for ephemeral state (rate limits, active sessions).

### DDL

```sql
-- Extensions
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================
-- Users
-- ============================================================
CREATE TABLE users (
    id              TEXT PRIMARY KEY DEFAULT 'usr_' || encode(gen_random_bytes(12), 'hex'),
    email           TEXT NOT NULL UNIQUE,
    name            TEXT,
    password_hash   TEXT,                          -- NULL if OAuth-only
    plan            TEXT NOT NULL DEFAULT 'free',   -- free | pro | enterprise
    stripe_customer_id TEXT UNIQUE,
    org_id          TEXT REFERENCES organizations(id),
    banned_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_stripe ON users(stripe_customer_id);

-- ============================================================
-- Organizations (team accounts)
-- ============================================================
CREATE TABLE organizations (
    id              TEXT PRIMARY KEY DEFAULT 'org_' || encode(gen_random_bytes(12), 'hex'),
    name            TEXT NOT NULL,
    plan            TEXT NOT NULL DEFAULT 'pro',
    stripe_customer_id TEXT UNIQUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE org_members (
    org_id          TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role            TEXT NOT NULL DEFAULT 'member',  -- owner | admin | member
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (org_id, user_id)
);

-- ============================================================
-- OAuth accounts (linked providers)
-- ============================================================
CREATE TABLE oauth_accounts (
    id              TEXT PRIMARY KEY DEFAULT 'oa_' || encode(gen_random_bytes(12), 'hex'),
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider        TEXT NOT NULL,                   -- github | google
    provider_id     TEXT NOT NULL,
    provider_email  TEXT,
    access_token    TEXT,                            -- encrypted at rest
    refresh_token   TEXT,                            -- encrypted at rest
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (provider, provider_id)
);

-- ============================================================
-- API Keys
-- ============================================================
CREATE TABLE api_keys (
    id              TEXT PRIMARY KEY DEFAULT 'key_' || encode(gen_random_bytes(12), 'hex'),
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    key_hash        TEXT NOT NULL UNIQUE,            -- SHA-256 of full key
    key_prefix      TEXT NOT NULL,                   -- first 16 chars for display
    scopes          JSONB NOT NULL DEFAULT '["tunnels:create","tunnels:read"]',
    expires_at      TIMESTAMPTZ,
    last_used_at    TIMESTAMPTZ,
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_api_keys_hash ON api_keys(key_hash) WHERE revoked_at IS NULL;
CREATE INDEX idx_api_keys_user ON api_keys(user_id);

-- ============================================================
-- Reserved Subdomains
-- ============================================================
CREATE TABLE reserved_subdomains (
    subdomain       TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_subdomains_user ON reserved_subdomains(user_id);

-- ============================================================
-- Tunnel History (persistent log, not just in-memory)
-- ============================================================
CREATE TABLE tunnel_sessions (
    id              TEXT PRIMARY KEY DEFAULT 't_' || encode(gen_random_bytes(12), 'hex'),
    user_id         TEXT NOT NULL REFERENCES users(id),
    subdomain       TEXT NOT NULL,
    protocol        TEXT NOT NULL DEFAULT 'http',
    server_node     TEXT,                            -- which server node handled this
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at        TIMESTAMPTZ,
    bytes_in        BIGINT NOT NULL DEFAULT 0,
    bytes_out       BIGINT NOT NULL DEFAULT 0,
    connections     INTEGER NOT NULL DEFAULT 0,
    close_reason    TEXT                             -- client_disconnect | server_shutdown | timeout | admin_close
);

CREATE INDEX idx_tunnel_sessions_user ON tunnel_sessions(user_id);
CREATE INDEX idx_tunnel_sessions_active ON tunnel_sessions(ended_at) WHERE ended_at IS NULL;
CREATE INDEX idx_tunnel_sessions_subdomain ON tunnel_sessions(subdomain) WHERE ended_at IS NULL;

-- ============================================================
-- Usage Metrics (hourly aggregates for billing)
-- ============================================================
CREATE TABLE usage_metrics (
    id              BIGSERIAL PRIMARY KEY,
    user_id         TEXT NOT NULL REFERENCES users(id),
    period_start    TIMESTAMPTZ NOT NULL,            -- hour boundary
    bytes_in        BIGINT NOT NULL DEFAULT 0,
    bytes_out       BIGINT NOT NULL DEFAULT 0,
    connections     INTEGER NOT NULL DEFAULT 0,
    requests        INTEGER NOT NULL DEFAULT 0,
    UNIQUE (user_id, period_start)
);

CREATE INDEX idx_usage_user_period ON usage_metrics(user_id, period_start);

-- ============================================================
-- Subscriptions & Billing
-- ============================================================
CREATE TABLE subscriptions (
    id              TEXT PRIMARY KEY DEFAULT 'sub_' || encode(gen_random_bytes(12), 'hex'),
    user_id         TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_subscription_id TEXT UNIQUE,
    stripe_price_id TEXT NOT NULL,
    plan            TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active',  -- active | past_due | canceled | trialing
    current_period_start TIMESTAMPTZ NOT NULL,
    current_period_end   TIMESTAMPTZ NOT NULL,
    cancel_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_subscriptions_user ON subscriptions(user_id);

CREATE TABLE invoices (
    id              TEXT PRIMARY KEY DEFAULT 'inv_' || encode(gen_random_bytes(12), 'hex'),
    user_id         TEXT NOT NULL REFERENCES users(id),
    stripe_invoice_id TEXT UNIQUE,
    amount_cents    INTEGER NOT NULL,
    currency        TEXT NOT NULL DEFAULT 'usd',
    status          TEXT NOT NULL,                   -- paid | open | void | uncollectible
    period_start    TIMESTAMPTZ NOT NULL,
    period_end      TIMESTAMPTZ NOT NULL,
    pdf_url         TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- Audit Log
-- ============================================================
CREATE TABLE audit_log (
    id              BIGSERIAL PRIMARY KEY,
    user_id         TEXT REFERENCES users(id),
    action          TEXT NOT NULL,                   -- tunnel.create | key.create | key.revoke | user.login | ...
    resource_type   TEXT,                            -- tunnel | api_key | user | subdomain
    resource_id     TEXT,
    ip_address      INET,
    user_agent      TEXT,
    metadata        JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_user ON audit_log(user_id, created_at DESC);
CREATE INDEX idx_audit_action ON audit_log(action, created_at DESC);

-- ============================================================
-- Device Auth Codes (for CLI login flow)
-- ============================================================
CREATE TABLE device_codes (
    device_code     TEXT PRIMARY KEY,
    user_code       TEXT NOT NULL UNIQUE,
    user_id         TEXT REFERENCES users(id),       -- NULL until authorized
    status          TEXT NOT NULL DEFAULT 'pending',  -- pending | authorized | expired
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Redis Keys

```
# Rate limiting (sliding window)
rate:{user_id}:api         → sorted set of timestamps
rate:{user_id}:tunnel_conn → sorted set of timestamps

# Active sessions (ephemeral)
active:tunnels:{tunnel_id}    → JSON { user_id, subdomain, server_node, connected_at }
active:users:{user_id}        → SET of tunnel_ids
active:subdomains:{subdomain} → tunnel_id

# Bandwidth counters (per billing period)
bw:{user_id}:2026-02         → integer (bytes, INCRBY)

# Device auth flow
device:{device_code}          → JSON { user_code, status, user_id }  TTL 900s
```

---

## 6. Rate Limiting & Abuse Prevention

### Layers

| Layer | Scope | Implementation |
|-------|-------|----------------|
| **Global** | Per IP | Nginx `limit_req` — 100 req/s burst 200 |
| **API** | Per user | Redis sliding window — plan-based limits |
| **Tunnel creation** | Per user | Max N concurrent tunnels (plan limit) |
| **Connections** | Per tunnel | Max concurrent connections (plan limit) |
| **Bandwidth** | Per user/month | Atomic counter in Redis, checked pre-proxy |

### Sliding Window Rate Limiter (Rust)

```rust
use redis::AsyncCommands;

pub struct RateLimiter {
    redis: redis::Client,
}

impl RateLimiter {
    /// Check and record a request. Returns Ok(remaining) or Err if limit exceeded.
    pub async fn check(
        &self,
        key: &str,
        max_requests: u32,
        window_secs: u64,
    ) -> Result<u32, RateLimitError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as f64;
        let window_start = now - (window_secs as f64 * 1000.0);

        // Atomic pipeline: remove old entries, add new, count, set expiry
        let (count,): (u32,) = redis::pipe()
            .atomic()
            .zrembyscore(key, "-inf", window_start)
            .ignore()
            .zadd(key, now, now.to_string())
            .ignore()
            .zcard(key)
            .expire(key, window_secs as i64 + 1)
            .ignore()
            .query_async(&mut conn)
            .await?;

        if count > max_requests {
            return Err(RateLimitError::Exceeded {
                limit: max_requests,
                retry_after_secs: window_secs,
            });
        }

        Ok(max_requests - count)
    }
}
```

### Bandwidth Tracking

Bandwidth is tracked by wrapping the proxy copy with a counting wrapper:

```rust
pub struct BandwidthTracker {
    user_id: String,
    redis: redis::Client,
    bytes_in: AtomicU64,
    bytes_out: AtomicU64,
}

impl BandwidthTracker {
    /// Flush accumulated bytes to Redis (called periodically or on connection close)
    pub async fn flush(&self) -> Result<()> {
        let in_bytes = self.bytes_in.swap(0, Ordering::Relaxed);
        let out_bytes = self.bytes_out.swap(0, Ordering::Relaxed);
        if in_bytes == 0 && out_bytes == 0 { return Ok(()) }

        let key = format!("bw:{}:{}", self.user_id, chrono::Utc::now().format("%Y-%m"));
        let total = in_bytes + out_bytes;
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        conn.incr::<_, _, u64>(&key, total).await?;
        // Expire key after 90 days (covers billing reconciliation window)
        conn.expire::<_, ()>(&key, 86400 * 90).await?;
        Ok(())
    }
}
```

### Abuse Prevention Checklist

- [ ] Subdomain squatting: Free plan gets random subdomains only
- [ ] Connection flooding: Per-tunnel connection limit + global connection limit per user
- [ ] Bandwidth abuse: Hard cutoff at plan limit, warning email at 80%
- [ ] Port scanning: No raw TCP tunnels on Free plan
- [ ] Crypto mining detection: Monitor for sustained high-bandwidth low-request-count patterns
- [ ] Blocked subdomains: Maintain denylist (`admin`, `www`, `api`, `app`, `mail`, `ftp`, etc.)
- [ ] TOS violation: Admin endpoint to ban users + force-close tunnels

---

## 7. Monitoring & Observability

### Metrics (Prometheus)

Expose a `/metrics` endpoint on internal port (9090).

```rust
use prometheus::{IntCounter, IntGauge, Histogram, register_int_counter, register_int_gauge, register_histogram};

lazy_static! {
    // Connections
    static ref ACTIVE_TUNNELS: IntGauge = register_int_gauge!(
        "subtunnel_active_tunnels", "Number of currently active tunnels"
    ).unwrap();
    static ref ACTIVE_CONNECTIONS: IntGauge = register_int_gauge!(
        "subtunnel_active_connections", "Number of currently active proxy connections"
    ).unwrap();
    static ref CONNECTIONS_TOTAL: IntCounter = register_int_counter!(
        "subtunnel_connections_total", "Total proxy connections handled"
    ).unwrap();

    // Bandwidth
    static ref BYTES_IN: IntCounter = register_int_counter!(
        "subtunnel_bytes_in_total", "Total bytes received from clients"
    ).unwrap();
    static ref BYTES_OUT: IntCounter = register_int_counter!(
        "subtunnel_bytes_out_total", "Total bytes sent to clients"
    ).unwrap();

    // Latency
    static ref PROXY_DURATION: Histogram = register_histogram!(
        "subtunnel_proxy_duration_seconds", "Duration of proxy connections",
        vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 30.0, 60.0, 300.0]
    ).unwrap();

    // Auth
    static ref AUTH_SUCCESS: IntCounter = register_int_counter!(
        "subtunnel_auth_success_total", "Successful authentications"
    ).unwrap();
    static ref AUTH_FAILURE: IntCounter = register_int_counter!(
        "subtunnel_auth_failure_total", "Failed authentications"
    ).unwrap();

    // API
    static ref API_REQUESTS: IntCounter = register_int_counter!(
        "subtunnel_api_requests_total", "Total API requests"
    ).unwrap();
}
```

### Health Check Endpoint

```
GET /healthz          → 200 { "status": "ok", "version": "0.2.0" }
GET /readyz           → 200 if DB + Redis reachable, 503 otherwise
```

```rust
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    let redis_ok = state.redis.get_multiplexed_async_connection().await.is_ok();

    if db_ok && redis_ok {
        (StatusCode::OK, Json(json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")})))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
            "status": "degraded",
            "db": db_ok,
            "redis": redis_ok,
        })))
    }
}
```

### Structured Logging

Continue using `tracing` with JSON output in production:

```rust
tracing_subscriber::fmt()
    .json()
    .with_env_filter(EnvFilter::from_default_env())
    .with_target(true)
    .with_span_events(FmtSpan::CLOSE)
    .init();
```

Key spans to instrument:
- `agent_connection{peer, user_id, agent_id}`
- `tunnel{tunnel_id, subdomain, user_id}`
- `proxy_stream{tunnel_id, client_addr}`
- `api_request{method, path, user_id, request_id}`

### Alerting Rules (Prometheus/Grafana)

```yaml
groups:
  - name: subtunnel
    rules:
      - alert: HighErrorRate
        expr: rate(subtunnel_auth_failure_total[5m]) > 10
        for: 5m
        labels: { severity: warning }
        annotations:
          summary: "High auth failure rate"

      - alert: TunnelServerDown
        expr: up{job="subtunnel"} == 0
        for: 1m
        labels: { severity: critical }

      - alert: HighMemoryUsage
        expr: process_resident_memory_bytes{job="subtunnel"} > 2e9
        for: 10m
        labels: { severity: warning }
```

---

## 8. Scaling Strategy

### Phase 1: Single Server (current → 1K users)

```
                    ┌─────────────────┐
    DNS *.sub.dev ──► Nginx (TLS term) │
                    │   ├─ /api → :3000│ (axum API server)
                    │   ├─ *.sub → :8080│ (tunnel HTTP listener)
                    │   └─ :7835       │ (control plane, TLS passthrough)
                    │                  │
                    │  ┌──────────┐   │
                    │  │ Postgres │   │
                    │  │ Redis    │   │
                    │  └──────────┘   │
                    └─────────────────┘
```

### Phase 2: Separated Services (1K → 10K users)

```
                    ┌──────────────────┐
    DNS ──────────► │   Load Balancer   │
                    └────┬────┬────┬───┘
                         │    │    │
              ┌──────────┘    │    └──────────┐
              ▼               ▼               ▼
         ┌─────────┐    ┌─────────┐    ┌─────────┐
         │ API     │    │ Tunnel  │    │ Tunnel  │
         │ Server  │    │ Node 1  │    │ Node 2  │
         └────┬────┘    └────┬────┘    └────┬────┘
              │              │              │
              └──────┬───────┴──────────────┘
                     ▼
              ┌─────────────┐    ┌─────────┐
              │  Postgres   │    │  Redis   │
              │  (RDS)      │    │  Cluster │
              └─────────────┘    └─────────┘
```

#### Tunnel Routing Between Nodes

When a tunnel is on Node 1 but HTTP traffic arrives at Node 2:

```rust
/// Look up which node owns a tunnel. If local, route directly.
/// If remote, proxy via inter-node gRPC.
pub async fn route_connection(
    subdomain: &str,
    stream: TcpStream,
    preread: Vec<u8>,
    local_mgr: &TunnelManager,
    cluster: &ClusterState,
) -> Result<()> {
    // Try local first
    if local_mgr.has_subdomain(subdomain).await {
        return local_mgr.route_with_preread(subdomain, stream, preread).await;
    }
    // Look up in Redis
    let node = cluster.find_tunnel_node(subdomain).await?;
    if let Some(node_addr) = node {
        cluster.proxy_to_node(&node_addr, subdomain, stream, preread).await
    } else {
        bail!("no tunnel for subdomain: {subdomain}");
    }
}
```

### Phase 3: Multi-Region (10K+ users)

```
         US-East                     EU-West
    ┌───────────────┐          ┌───────────────┐
    │  LB + Nodes   │◄── DNS ──►  LB + Nodes   │
    │  Postgres RDS │   GeoDNS │  Postgres RDS │
    │  Redis        │          │  Redis        │
    └───────────────┘          └───────────────┘
              │                        │
              └───── Postgres Logical ─┘
                     Replication
                     (users, keys, billing
                      read replicas)
```

- **Tunnel state is region-local** — tunnels exist on the region where the client connects
- **User data is replicated globally** — auth and billing work everywhere
- **GeoDNS routes users to nearest region** by default
- **CLI `--region` flag** for explicit region selection: `subtunnel local 8080 --to subtunnel.dev --region eu-west`

### Failover

1. **Node failure**: Agent reconnects (existing backoff logic) to another node via LB. Redis tunnel state is ephemeral; new tunnel registered on new node.
2. **Region failure**: DNS failover (Route53 health checks, 60s TTL). Agents reconnect to surviving region.
3. **DB failure**: RDS Multi-AZ automatic failover. API returns 503 during failover window (~30s). Tunnel proxying continues (doesn't need DB).

---

## 9. Security

### TLS Everywhere

| Connection | TLS | Certificate |
|-----------|-----|-------------|
| Client → Nginx | TLS 1.2+ | Let's Encrypt (wildcard `*.subtunnel.dev`) |
| Nginx → API Server | Plaintext (localhost) | N/A |
| Nginx → Tunnel HTTP | Plaintext (localhost) | N/A |
| CLI → Control Plane | TLS 1.3 | Let's Encrypt (via Nginx TLS passthrough) |
| Server → Postgres | TLS | RDS CA cert |
| Server → Redis | TLS | ElastiCache CA cert |
| Node ↔ Node | mTLS | Internal CA |

Remove `NoVerifier` from client — use system roots or pin server cert:

```rust
// Replace the NoVerifier with proper cert verification
fn make_tls_config() -> ClientConfig {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}
```

### Token Rotation

- **JWT access tokens**: 15-minute lifetime, non-revocable (short-lived enough)
- **JWT refresh tokens**: 7-day lifetime, stored in DB, revocable
- **API keys**: Optional expiry. Key rotation via create-new → delete-old. `last_used_at` tracking for stale key detection.
- **Admin forced rotation**: API to revoke all keys/sessions for a user

### RBAC

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Scope {
    #[serde(rename = "tunnels:create")]
    TunnelsCreate,
    #[serde(rename = "tunnels:read")]
    TunnelsRead,
    #[serde(rename = "tunnels:delete")]
    TunnelsDelete,
    #[serde(rename = "keys:manage")]
    KeysManage,
    #[serde(rename = "billing:read")]
    BillingRead,
    #[serde(rename = "billing:manage")]
    BillingManage,
    #[serde(rename = "admin:*")]
    Admin,
}

/// Check that the auth context has the required scope.
pub fn require_scope(auth: &AuthContext, scope: Scope) -> Result<()> {
    if auth.scopes.contains(&Scope::Admin) || auth.scopes.contains(&scope) {
        Ok(())
    } else {
        Err(ApiError::forbidden(format!("missing scope: {scope:?}")))
    }
}
```

### Audit Log

Every mutation is logged to the `audit_log` table (see DDL above). Example:

```rust
pub async fn audit(db: &DbPool, event: AuditEvent) {
    let _ = sqlx::query!(
        "INSERT INTO audit_log (user_id, action, resource_type, resource_id, ip_address, user_agent, metadata)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        event.user_id, event.action, event.resource_type, event.resource_id,
        event.ip_address, event.user_agent, event.metadata,
    ).execute(db).await;
}
```

---

## 10. CLI Auth Flow

### `subtunnel login`

```
$ subtunnel login

  Opening browser to authenticate...
  If the browser doesn't open, visit: https://app.subtunnel.dev/activate
  Enter code: ABCD-1234

  Waiting for authorization... ✓

  Authenticated as ozan@example.com (Pro plan)
  Token saved to ~/.subtunnel/config.toml
```

### Flow

```
┌────────┐          ┌───────────┐          ┌───────────┐          ┌─────────┐
│  CLI   │          │  API      │          │  Browser  │          │  OAuth  │
│        │          │  Server   │          │  (Dashboard)         │  Provider│
│        │─POST─────►           │          │           │          │         │
│        │ /device/ │           │          │           │          │         │
│        │ begin    │  creates  │          │           │          │         │
│        │◄─────────│  device   │          │           │          │         │
│        │ user_code│  code     │          │           │          │         │
│        │          │           │          │           │          │         │
│  opens browser ───────────────────────► │           │          │         │
│        │          │           │  /activate│          │          │         │
│        │          │           │  user enters         │          │         │
│        │          │           │  ABCD-1234│          │          │         │
│        │          │           │          │ ─OAuth──► │          │         │
│        │          │           │          │           │          │         │
│        │          │           │          │ ◄─token── │          │         │
│        │          │           │ ◄─POST───│           │          │         │
│        │          │  marks    │ /device/ │           │          │         │
│        │          │  authorized authorize│           │          │         │
│        │─POST─────►           │          │           │          │         │
│        │ /device/ │           │          │           │          │         │
│        │ poll     │           │          │           │          │         │
│        │◄─────────│ JWT pair  │          │           │          │         │
│        │          │           │          │           │          │         │
│  saves to config  │           │          │           │          │         │
└────────┘          └───────────┘          └───────────┘          └─────────┘
```

### Config File

`~/.subtunnel/config.toml`:

```toml
[auth]
# Populated by `subtunnel login`
access_token = "eyJ..."
refresh_token = "eyJ..."
expires_at = 2026-02-12T20:00:00Z

# Or use an API key directly
# api_key = "stk_live_..."

[defaults]
server = "subtunnel.dev"
region = "auto"

[tls]
# For self-hosted servers with custom CA
# ca_cert = "/path/to/ca.pem"
# skip_verify = false
```

### CLI Struct Updates

```rust
#[derive(Subcommand)]
enum Command {
    /// Authenticate with the SubTunnel service
    Login {
        /// Server to authenticate with
        #[arg(long, default_value = "https://api.subtunnel.dev")]
        server: String,
    },

    /// Show current auth status
    Status,

    /// Log out and remove saved credentials
    Logout,

    /// Run the tunnel server (self-hosted mode)
    Server { /* existing fields */ },

    /// Connect to a server and expose a local port
    Local {
        local_port: u16,

        /// Server address (default: reads from config)
        #[arg(long)]
        to: Option<String>,

        /// Auth token (default: reads from config)
        #[arg(long, env = "SUBTUNNEL_TOKEN")]
        token: Option<String>,

        #[arg(long)]
        subdomain: Option<String>,

        /// Region preference
        #[arg(long, default_value = "auto")]
        region: String,
    },
}
```

---

## 11. Billing Integration

### Stripe Products

| Plan | Price ID | Monthly | Annual | Metered Component |
|------|----------|---------|--------|-------------------|
| Free | — | $0 | — | — |
| Pro | `price_pro_monthly` | $15/mo | $144/yr | Bandwidth overage: $0.10/GB after 100GB |
| Enterprise | `price_ent_monthly` | $75/mo | $720/yr | Bandwidth overage: $0.05/GB after 1TB |

### Stripe Webhook Handler

```rust
async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let sig = headers.get("stripe-signature")
        .ok_or(ApiError::bad_request("missing stripe-signature"))?
        .to_str()?;

    let event = stripe::Webhook::construct_event(
        &String::from_utf8(body.to_vec())?,
        sig,
        &state.config.stripe_webhook_secret,
    )?;

    match event.type_ {
        EventType::CheckoutSessionCompleted => {
            let session: CheckoutSession = serde_json::from_value(event.data.object)?;
            handle_checkout_complete(&state, session).await?;
        }
        EventType::CustomerSubscriptionUpdated => {
            let sub: Subscription = serde_json::from_value(event.data.object)?;
            handle_subscription_update(&state, sub).await?;
        }
        EventType::CustomerSubscriptionDeleted => {
            let sub: Subscription = serde_json::from_value(event.data.object)?;
            handle_subscription_cancel(&state, sub).await?;
        }
        EventType::InvoicePaid => {
            let invoice: Invoice = serde_json::from_value(event.data.object)?;
            handle_invoice_paid(&state, invoice).await?;
        }
        _ => {}
    }

    Ok(StatusCode::OK)
}
```

### Plan Enforcement

When plan changes (upgrade/downgrade), enforce limits asynchronously:

```rust
async fn enforce_plan_limits(user_id: &str, new_plan: &Plan, state: &AppState) -> Result<()> {
    let limits = PlanLimits::for_plan(new_plan);

    // If downgrading, check if current usage exceeds new limits
    let active_tunnels = state.tunnel_mgr.user_tunnel_count(user_id).await;
    if active_tunnels > limits.max_tunnels {
        // Don't force-close existing tunnels — just prevent new ones
        // Log warning, send email notification
        tracing::warn!(user_id, active_tunnels, max = limits.max_tunnels, "user exceeds new plan tunnel limit");
    }

    // Update plan in DB
    sqlx::query!("UPDATE users SET plan = $1, updated_at = NOW() WHERE id = $2",
        serde_json::to_string(new_plan)?, user_id
    ).execute(&state.db).await?;

    Ok(())
}
```

### Usage-Based Billing (Bandwidth Overage)

At the end of each billing period, report metered usage to Stripe:

```rust
/// Called by a cron job at the end of each billing period
async fn report_bandwidth_overage(state: &AppState) -> Result<()> {
    let users = sqlx::query!(
        "SELECT u.id, u.plan, s.stripe_subscription_id
         FROM users u JOIN subscriptions s ON u.id = s.user_id
         WHERE s.status = 'active'"
    ).fetch_all(&state.db).await?;

    for user in users {
        let limits = PlanLimits::for_plan(&serde_json::from_str(&user.plan)?);
        let used = get_period_bandwidth(&state.db, &user.id).await?;

        if used > limits.bandwidth_bytes_per_month {
            let overage_gb = (used - limits.bandwidth_bytes_per_month) as f64 / 1_073_741_824.0;
            // Report to Stripe as metered usage
            stripe::SubscriptionItem::create_usage_record(
                &state.stripe,
                &user.stripe_subscription_id,
                stripe::CreateUsageRecord {
                    quantity: overage_gb.ceil() as i64,
                    timestamp: Some(chrono::Utc::now().timestamp()),
                    action: Some(stripe::UsageRecordAction::Set),
                },
            ).await?;
        }
    }
    Ok(())
}
```

---

## 12. Deployment

### Docker Compose (Self-Hosted)

```yaml
# docker-compose.yml
version: "3.9"

services:
  subtunnel:
    image: ghcr.io/ozankasikci/subtunnel:latest
    ports:
      - "7835:7835"   # Control plane
      - "8080:8080"   # HTTP tunnel traffic
      - "3000:3000"   # API server
      - "9090:9090"   # Metrics
    environment:
      SUBTUNNEL_MODE: all          # Run server + API in one process
      SUBTUNNEL_DOMAIN: tunnel.example.com
      SUBTUNNEL_DB_URL: postgres://subtunnel:secret@postgres:5432/subtunnel
      SUBTUNNEL_REDIS_URL: redis://redis:6379
      SUBTUNNEL_JWT_SECRET: ${JWT_SECRET}
      SUBTUNNEL_STRIPE_KEY: ${STRIPE_SECRET_KEY}
      SUBTUNNEL_STRIPE_WEBHOOK_SECRET: ${STRIPE_WEBHOOK_SECRET}
      RUST_LOG: info,subtunnel=debug
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started
    restart: unless-stopped

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: subtunnel
      POSTGRES_USER: subtunnel
      POSTGRES_PASSWORD: secret
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U subtunnel"]
      interval: 5s
      timeout: 3s
      retries: 5

  redis:
    image: redis:7-alpine
    volumes:
      - redisdata:/data
    command: redis-server --appendonly yes

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
    depends_on:
      - subtunnel

volumes:
  pgdata:
  redisdata:
```

### Nginx Config

```nginx
# nginx.conf
worker_processes auto;

events {
    worker_connections 4096;
}

stream {
    # TLS passthrough for control plane
    server {
        listen 7835;
        proxy_pass subtunnel:7835;
    }
}

http {
    upstream api {
        server subtunnel:3000;
    }
    upstream tunnels {
        server subtunnel:8080;
    }

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=global:10m rate=100r/s;

    # API
    server {
        listen 443 ssl;
        server_name api.tunnel.example.com app.tunnel.example.com;

        ssl_certificate     /etc/nginx/certs/fullchain.pem;
        ssl_certificate_key /etc/nginx/certs/privkey.pem;

        limit_req zone=global burst=200 nodelay;

        location / {
            proxy_pass http://api;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }

    # Tunnel traffic (wildcard)
    server {
        listen 443 ssl;
        server_name *.tunnel.example.com;

        ssl_certificate     /etc/nginx/certs/fullchain.pem;
        ssl_certificate_key /etc/nginx/certs/privkey.pem;

        location / {
            proxy_pass http://tunnels;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;

            # WebSocket support
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
    }

    # HTTP → HTTPS redirect
    server {
        listen 80;
        server_name *.tunnel.example.com api.tunnel.example.com app.tunnel.example.com;
        return 301 https://$host$request_uri;
    }
}
```

### Server Configuration

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Operating mode
    pub mode: Mode,                      // server | api | all
    /// Domain for tunnel subdomains
    pub domain: String,
    /// Extra domains to accept
    #[serde(default)]
    pub extra_domains: Vec<String>,
    /// Control plane port
    #[serde(default = "default_control_port")]
    pub control_port: u16,               // 7835
    /// HTTP listener port
    #[serde(default = "default_http_port")]
    pub http_port: u16,                  // 8080
    /// API server port
    #[serde(default = "default_api_port")]
    pub api_port: u16,                   // 3000
    /// Metrics port
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,               // 9090
    /// Database URL
    pub db_url: String,
    /// Redis URL
    pub redis_url: String,
    /// JWT signing secret (RS256 private key path or HMAC secret)
    pub jwt_secret: String,
    /// Stripe secret key
    pub stripe_key: Option<String>,
    /// Stripe webhook secret
    pub stripe_webhook_secret: Option<String>,
    /// Self-hosted mode (disables billing, relaxes auth)
    #[serde(default)]
    pub self_hosted: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Server,  // Only tunnel server (control + HTTP)
    Api,     // Only REST API
    All,     // Both in one process
}
```

### Dockerfile

```dockerfile
FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin subtunnel

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/subtunnel /usr/local/bin/subtunnel
EXPOSE 7835 8080 3000 9090
ENTRYPOINT ["subtunnel"]
CMD ["server"]
```

### Managed Cloud Offering

For the managed SaaS (`subtunnel.dev`):

- **Infra**: AWS (eu-west-1 primary, us-east-1 secondary)
- **Compute**: ECS Fargate or EC2 (Graviton4, ARM64) for cost efficiency
- **DB**: RDS PostgreSQL 16 Multi-AZ
- **Cache**: ElastiCache Redis Cluster
- **TLS**: ACM wildcard cert for `*.subtunnel.dev`
- **DNS**: Route53 with health-check-based failover
- **CDN**: CloudFront for dashboard static assets
- **CI/CD**: GitHub Actions → ECR → ECS rolling deploy
- **Monitoring**: Prometheus (self-hosted) + Grafana, or CloudWatch
- **Logs**: Structured JSON → CloudWatch Logs → optional export to S3

---

## Appendix: Crate Structure (Target)

```
subtunnel/
├── crates/
│   ├── cli/                    # CLI binary (login, local, server commands)
│   │   └── src/
│   │       ├── main.rs
│   │       ├── commands/
│   │       │   ├── login.rs
│   │       │   ├── local.rs
│   │       │   ├── server.rs
│   │       │   └── status.rs
│   │       └── config.rs       # Config file parser (~/.subtunnel/config.toml)
│   ├── server/                 # Tunnel server (control plane + HTTP listener)
│   │   └── src/
│   │       ├── handler.rs
│   │       ├── listener.rs
│   │       ├── tunnel_mgr.rs
│   │       ├── auth.rs         # Multi-method auth (API key, JWT)
│   │       ├── bandwidth.rs    # Bandwidth tracking
│   │       └── cluster.rs      # Multi-node coordination
│   ├── api/                    # REST API server (axum)
│   │   └── src/
│   │       ├── routes/
│   │       │   ├── auth.rs
│   │       │   ├── tunnels.rs
│   │       │   ├── keys.rs
│   │       │   ├── subdomains.rs
│   │       │   ├── billing.rs
│   │       │   └── admin.rs
│   │       ├── middleware/
│   │       │   ├── auth.rs
│   │       │   ├── rate_limit.rs
│   │       │   └── request_id.rs
│   │       └── lib.rs
│   ├── shared/                 # Shared types (existing)
│   │   └── src/
│   │       ├── models.rs
│   │       ├── limits.rs
│   │       └── errors.rs
│   ├── protocol/               # Wire protocol (extracted from cli)
│   │   └── src/
│   │       ├── codec.rs
│   │       └── messages.rs
│   └── transport/              # TLS + yamux (extracted from cli)
│       └── src/
│           ├── mux.rs
│           └── tls.rs
├── migrations/                 # SQL migrations (sqlx)
│   ├── 001_initial.sql
│   └── 002_billing.sql
├── docker-compose.yml
├── Dockerfile
└── nginx.conf
```

---

*This document is a living spec. Update it as implementation progresses.*
