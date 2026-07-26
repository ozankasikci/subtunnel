use anyhow::{bail, Result};
use tokio::task::JoinSet;
use tracing::error;

use crate::client::{Client, ConnectTlsOptions};
use crate::config::{Config, TunnelConfig};

pub async fn run_tunnels(
    config: &Config,
    tunnels: Vec<(String, TunnelConfig)>,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    let mut tasks = JoinSet::new();

    for (name, tunnel) in tunnels {
        let client = Client::new(
            config.server.clone(),
            config.token.clone(),
            tunnel.local_port,
            tunnel.subdomain,
            ConnectTlsOptions {
                verify: config.tls_verify,
                ca_path: config.tls_ca.clone(),
            },
        );
        let tunnel_shutdown = shutdown.clone();
        tasks.spawn(async move {
            let result = client.run_until_hard_error(tunnel_shutdown).await;
            (name, result)
        });
    }

    let mut failures = 0usize;
    while let Some(joined) = tasks.join_next().await {
        match joined {
            Ok((name, Ok(()))) => {
                if !*shutdown.borrow() {
                    error!(tunnel = %name, "tunnel task ended unexpectedly");
                    failures += 1;
                }
            }
            Ok((name, Err(error))) => {
                error!(tunnel = %name, "tunnel stopped with a hard error: {error:#}");
                failures += 1;
            }
            Err(error) => {
                error!("tunnel task failed to join: {error}");
                failures += 1;
            }
        }
    }

    if failures > 0 {
        bail!("{failures} tunnel task(s) failed");
    }

    Ok(())
}
