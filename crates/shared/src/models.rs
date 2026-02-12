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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Free,
    Pro,
    Enterprise,
}
