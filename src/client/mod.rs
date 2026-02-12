//! Tunnelr client — connects to a server and proxies traffic to a local service.

pub mod connector;
pub mod local_proxy;

pub use connector::{connect, connect_with_retry, EstablishedConnection, TunnelInfo};
pub use local_proxy::run_proxy;

use anyhow::Result;
use tracing::info;

pub struct Client {
    server_addr: String,
    token: String,
    local_port: u16,
    subdomain: Option<String>,
}

impl Client {
    pub fn new(server_addr: String, token: String, local_port: u16, subdomain: Option<String>) -> Self {
        Self {
            server_addr,
            token,
            local_port,
            subdomain,
        }
    }

    pub async fn run(self, shutdown: tokio::sync::watch::Receiver<bool>) -> Result<()> {
        let local_addr = format!("localhost:{}", self.local_port);

        connect_with_retry(
            &self.server_addr,
            &self.token,
            self.subdomain.as_deref(),
            shutdown.clone(),
            |conn| {
                let local_addr = local_addr.clone();
                let shutdown = shutdown.clone();
                async move {
                    print_tunnel_status(&conn.tunnel_info, &local_addr);
                    run_proxy(conn.mux, &local_addr, shutdown).await
                }
            },
        )
        .await
    }
}

fn print_tunnel_status(info: &TunnelInfo, local_addr: &str) {
    info!(
        "\n\x1b[1;32m  tunnelr\x1b[0m v{}\n  \x1b[1mStatus:\x1b[0m     connected\n  \x1b[1mForwarding:\x1b[0m {} -> {}\n  \x1b[1mTunnel ID:\x1b[0m  {}\n",
        env!("CARGO_PKG_VERSION"),
        info.public_url,
        local_addr,
        info.tunnel_id,
    );
}
