//! Tunnel registry — tracks active tunnels and maps subdomains to agent sessions.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Result};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Debug)]
struct TunnelEntry {
    agent_id: String,
    subdomain: String,
    #[allow(dead_code)]
    protocol: String,
}

#[derive(Debug, Clone)]
pub struct TunnelManager {
    inner: Arc<RwLock<TunnelManagerInner>>,
}

#[derive(Debug)]
struct TunnelManagerInner {
    tunnels: HashMap<String, TunnelEntry>,
    subdomain_to_tunnel: HashMap<String, String>,
    subdomain_to_tx: HashMap<String, tokio::sync::mpsc::Sender<(TcpStream, Vec<u8>)>>,
}

pub struct RegisteredTunnel {
    pub tunnel_id: String,
    pub subdomain: String,
    pub conn_rx: tokio::sync::mpsc::Receiver<(TcpStream, Vec<u8>)>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(TunnelManagerInner {
                tunnels: HashMap::new(),
                subdomain_to_tunnel: HashMap::new(),
                subdomain_to_tx: HashMap::new(),
            })),
        }
    }

    pub async fn register(
        &self,
        agent_id: &str,
        protocol: &str,
        requested_subdomain: Option<&str>,
    ) -> Result<RegisteredTunnel> {
        let subdomain = if let Some(req) = requested_subdomain {
            validate_subdomain(req)?;
            req.to_lowercase()
        } else {
            generate_subdomain()
        };
        let tunnel_id = format!("t_{}", uuid::Uuid::new_v4().as_simple());
        let (conn_tx, conn_rx) = tokio::sync::mpsc::channel(64);

        {
            let mut inner = self.inner.write().await;
            if inner.subdomain_to_tunnel.contains_key(&subdomain) {
                bail!("subdomain collision, try again");
            }
            inner.tunnels.insert(
                tunnel_id.clone(),
                TunnelEntry {
                    agent_id: agent_id.to_string(),
                    subdomain: subdomain.clone(),
                    protocol: protocol.to_string(),
                },
            );
            inner.subdomain_to_tunnel.insert(subdomain.clone(), tunnel_id.clone());
            inner.subdomain_to_tx.insert(subdomain.clone(), conn_tx);
        }

        info!(
            tunnel_id = %tunnel_id,
            agent_id = %agent_id,
            subdomain = %subdomain,
            protocol = %protocol,
            "tunnel registered"
        );

        Ok(RegisteredTunnel { tunnel_id, subdomain, conn_rx })
    }

    /// Route an incoming connection (with pre-read bytes) to the matching tunnel.
    pub async fn route_with_preread(&self, subdomain: &str, stream: TcpStream, preread: Vec<u8>) -> Result<()> {
        let tx = {
            let inner = self.inner.read().await;
            inner.subdomain_to_tx.get(subdomain).cloned()
        };
        match tx {
            Some(tx) => {
                tx.send((stream, preread)).await.map_err(|_| anyhow::anyhow!("tunnel channel closed"))?;
                Ok(())
            }
            None => bail!("no tunnel for subdomain: {subdomain}"),
        }
    }

    pub async fn unregister(&self, tunnel_id: &str) {
        let mut inner = self.inner.write().await;
        if let Some(entry) = inner.tunnels.remove(tunnel_id) {
            inner.subdomain_to_tunnel.remove(&entry.subdomain);
            inner.subdomain_to_tx.remove(&entry.subdomain);
            info!(tunnel_id = %tunnel_id, subdomain = %entry.subdomain, "tunnel unregistered");
        }
    }

    pub async fn unregister_agent(&self, agent_id: &str) {
        let mut inner = self.inner.write().await;
        let tunnel_ids: Vec<String> = inner
            .tunnels
            .iter()
            .filter(|(_, entry)| entry.agent_id == agent_id)
            .map(|(id, _)| id.clone())
            .collect();
        for tid in &tunnel_ids {
            if let Some(entry) = inner.tunnels.remove(tid) {
                inner.subdomain_to_tunnel.remove(&entry.subdomain);
                inner.subdomain_to_tx.remove(&entry.subdomain);
                info!(tunnel_id = %tid, subdomain = %entry.subdomain, "tunnel unregistered (agent disconnect)");
            }
        }
    }

    pub async fn tunnel_count(&self) -> usize {
        let inner = self.inner.read().await;
        inner.tunnels.len()
    }
}

impl Default for TunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_subdomain(s: &str) -> Result<()> {
    if s.is_empty() || s.len() > 63 {
        bail!("subdomain must be 1-63 characters");
    }
    if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        bail!("subdomain may only contain alphanumeric characters and hyphens");
    }
    if s.starts_with('-') || s.ends_with('-') {
        bail!("subdomain must not start or end with a hyphen");
    }
    Ok(())
}

fn generate_subdomain() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 4] = rng.gen();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_unregister() {
        let mgr = TunnelManager::new();
        let reg = mgr.register("agent-1", "tcp", None).await.unwrap();
        assert_eq!(reg.subdomain.len(), 8);
        assert_eq!(mgr.tunnel_count().await, 1);
        mgr.unregister(&reg.tunnel_id).await;
        assert_eq!(mgr.tunnel_count().await, 0);
    }

    #[tokio::test]
    async fn register_custom_subdomain() {
        let mgr = TunnelManager::new();
        let reg = mgr.register("agent-1", "tcp", Some("myapp")).await.unwrap();
        assert_eq!(reg.subdomain, "myapp");

        // Duplicate should fail
        let err = mgr.register("agent-2", "tcp", Some("myapp")).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn unregister_agent() {
        let mgr = TunnelManager::new();
        let _r1 = mgr.register("agent-1", "tcp", None).await.unwrap();
        let _r2 = mgr.register("agent-1", "tcp", None).await.unwrap();
        let _r3 = mgr.register("agent-2", "tcp", None).await.unwrap();
        assert_eq!(mgr.tunnel_count().await, 3);
        mgr.unregister_agent("agent-1").await;
        assert_eq!(mgr.tunnel_count().await, 1);
    }
}
