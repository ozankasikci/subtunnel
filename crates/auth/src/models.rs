use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use subtunnel_shared::models::Plan;

/// Stored user record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub id: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub plan: Plan,
    pub created_at: DateTime<Utc>,
}

/// Stored API key record (never contains the raw key).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: String,
    pub user_id: String,
    pub key_hash: String,
    pub prefix: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub revoked: bool,
}

/// Returned once when an API key is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedApiKey {
    pub raw_key: String,
    pub record: ApiKeyRecord,
}
