//! Shared data models used across SubTunnel services.

use serde::{Deserialize, Serialize};

/// A tunnel's public-facing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelInfo {
    pub tunnel_id: String,
    pub subdomain: String,
    pub public_url: String,
    pub protocol: String,
}

/// User account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub plan: Plan,
}

/// Subscription plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Free,
    Pro,
    Enterprise,
}

impl std::fmt::Display for Plan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Plan::Free => write!(f, "free"),
            Plan::Pro => write!(f, "pro"),
            Plan::Enterprise => write!(f, "enterprise"),
        }
    }
}

/// Authentication method used for a request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    ApiKey { key_id: String },
    Jwt,
}

/// Authenticated user context, produced after successful auth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: String,
    pub plan: Plan,
    pub scopes: Vec<String>,
    pub auth_method: AuthMethod,
}
