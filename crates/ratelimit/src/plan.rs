use serde::{Deserialize, Serialize};

/// Subscription plan tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    Free,
    Pro,
    Enterprise,
}

/// Resource limits for a given plan.
#[derive(Debug, Clone)]
pub struct PlanLimits {
    /// Maximum concurrent tunnels.
    pub max_tunnels: u32,
    /// Maximum requests per calendar month (0 = unlimited).
    pub max_requests_per_month: u64,
    /// Maximum bandwidth bytes per calendar month (0 = unlimited).
    pub max_bandwidth_bytes_per_month: u64,
    /// Maximum requests per minute (token bucket capacity).
    pub max_requests_per_minute: u32,
}

impl PlanLimits {
    pub fn for_plan(plan: Plan) -> Self {
        match plan {
            Plan::Free => Self {
                max_tunnels: 3,
                max_requests_per_month: 100_000,
                max_bandwidth_bytes_per_month: 1_073_741_824, // 1 GB
                max_requests_per_minute: 1_000,
            },
            Plan::Pro => Self {
                max_tunnels: 20,
                max_requests_per_month: 0, // unlimited
                max_bandwidth_bytes_per_month: 107_374_182_400, // 100 GB
                max_requests_per_minute: 10_000,
            },
            Plan::Enterprise => Self {
                max_tunnels: 0, // unlimited
                max_requests_per_month: 0,
                max_bandwidth_bytes_per_month: 0,
                max_requests_per_minute: 0, // unlimited
            },
        }
    }

    pub fn is_unlimited_tunnels(&self) -> bool {
        self.max_tunnels == 0
    }

    pub fn is_unlimited_requests(&self) -> bool {
        self.max_requests_per_month == 0
    }

    pub fn is_unlimited_bandwidth(&self) -> bool {
        self.max_bandwidth_bytes_per_month == 0
    }

    pub fn is_unlimited_rate(&self) -> bool {
        self.max_requests_per_minute == 0
    }
}
