//! Tunnel registry — tracks active tunnels and maps public ports to agent sessions.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Port range for dynamically allocated tunnel ports.
const PORT_RANGE_START: u16 = 20_000;
const PORT_RANGE_END: u16 = 30_000;

/// Information about a single active tunnel.
#[derive(Debug)]
pub struct TunnelInfo {
    /// Unique tunnel identifier (e.g. "t_abc123").
    pub tunnel_id: String,
    /// The agent that owns this tunnel.
    pub agent_id: String,
    /// The public port this tunnel is exposed on.
    pub remote_port: u16,
    /// Protocol (e.g. "tcp").
    pub protocol: String,
    /// TCP listener for the public-facing port. Held here so it stays bound
    /// for the lifetime of the tunnel.
    pub listener: TcpListener,
    /// Sender half of the channel used to deliver new public connections to the
    /// agent handler. Each value is a `TcpStream` from an internet client.
    pub conn_tx: tokio::sync::mpsc::Sender<tokio::net::TcpStream>,
}

/// Thread-safe tunnel registry.
///
/// Provides register/unregister/lookup operations for active tunnels.
/// All access is behind `Arc<RwLock<>>` so the manager can be shared
/// across tokio tasks.
#[derive(Debug, Clone)]
pub struct TunnelManager {
    inner: Arc<RwLock<TunnelManagerInner>>,
}

#[derive(Debug)]
struct TunnelManagerInner {
    /// Tunnels indexed by tunnel_id.
    tunnels: HashMap<String, TunnelEntry>,
    /// Reverse index: remote_port → tunnel_id for fast lookup.
    port_to_tunnel: HashMap<u16, String>,
}

/// Stored entry — we keep the metadata but hand off the listener and channel
/// to the caller at registration time.
#[derive(Debug)]
struct TunnelEntry {
    agent_id: String,
    remote_port: u16,
    #[allow(dead_code)] // stored for future HTTP/subdomain routing
    protocol: String,
}

/// Returned to the caller on successful registration so they can start
/// accepting connections on the public port.
pub struct RegisteredTunnel {
    pub tunnel_id: String,
    pub remote_port: u16,
    pub conn_rx: tokio::sync::mpsc::Receiver<tokio::net::TcpStream>,
}

impl TunnelManager {
    /// Create a new empty tunnel manager.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(TunnelManagerInner {
                tunnels: HashMap::new(),
                port_to_tunnel: HashMap::new(),
            })),
        }
    }

    /// Register a new tunnel.
    ///
    /// If `requested_port` is `Some`, tries to bind that specific port.
    /// Otherwise, allocates a random available port from the dynamic range.
    ///
    /// Returns a [`RegisteredTunnel`] containing the listener and a channel
    /// receiver for incoming connections.
    pub async fn register(
        &self,
        agent_id: &str,
        protocol: &str,
        requested_port: Option<u16>,
    ) -> Result<RegisteredTunnel> {
        let (listener, port) = match requested_port {
            Some(port) => {
                // Check if port is already taken.
                {
                    let inner = self.inner.read().await;
                    if inner.port_to_tunnel.contains_key(&port) {
                        bail!("port {port} is already in use by another tunnel");
                    }
                }
                let listener = TcpListener::bind(("0.0.0.0", port))
                    .await
                    .with_context(|| format!("failed to bind port {port}"))?;
                (listener, port)
            }
            None => self.allocate_port().await?,
        };

        let tunnel_id = format!("t_{}", uuid::Uuid::new_v4().as_simple());
        let (conn_tx, conn_rx) = tokio::sync::mpsc::channel(64);

        {
            let mut inner = self.inner.write().await;
            inner.tunnels.insert(
                tunnel_id.clone(),
                TunnelEntry {
                    agent_id: agent_id.to_string(),
                    remote_port: port,
                    protocol: protocol.to_string(),
                },
            );
            inner.port_to_tunnel.insert(port, tunnel_id.clone());
        }

        info!(
            tunnel_id = %tunnel_id,
            agent_id = %agent_id,
            port = port,
            protocol = %protocol,
            "tunnel registered"
        );

        // Spawn a task that accepts connections and forwards them via the channel.
        // The listener is moved into this task; when the tunnel is unregistered,
        // dropping the conn_tx will cause conn_rx to close, signaling the handler.
        let tx = conn_tx.clone();
        let tid = tunnel_id.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        tracing::debug!(tunnel_id = %tid, client = %addr, "public connection accepted");
                        if tx.send(stream).await.is_err() {
                            // Receiver dropped — tunnel was unregistered.
                            break;
                        }
                    }
                    Err(e) => {
                        warn!(tunnel_id = %tid, error = %e, "accept failed on public port");
                    }
                }
            }
            tracing::debug!(tunnel_id = %tid, "public listener task exiting");
        });

        Ok(RegisteredTunnel {
            tunnel_id,
            remote_port: port,
            conn_rx,
        })
    }

    /// Unregister a tunnel and free its port.
    pub async fn unregister(&self, tunnel_id: &str) {
        let mut inner = self.inner.write().await;
        if let Some(entry) = inner.tunnels.remove(tunnel_id) {
            inner.port_to_tunnel.remove(&entry.remote_port);
            info!(tunnel_id = %tunnel_id, port = entry.remote_port, "tunnel unregistered");
        }
    }

    /// Unregister all tunnels for a given agent.
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
                inner.port_to_tunnel.remove(&entry.remote_port);
                info!(tunnel_id = %tid, port = entry.remote_port, "tunnel unregistered (agent disconnect)");
            }
        }
    }

    /// Look up which tunnel is assigned to a given remote port.
    pub async fn lookup_by_port(&self, port: u16) -> Option<String> {
        let inner = self.inner.read().await;
        inner.port_to_tunnel.get(&port).cloned()
    }

    /// Get the number of active tunnels.
    pub async fn tunnel_count(&self) -> usize {
        let inner = self.inner.read().await;
        inner.tunnels.len()
    }

    /// Allocate a random available port from the dynamic range.
    async fn allocate_port(&self) -> Result<(TcpListener, u16)> {
        // Generate all random candidates upfront so the RNG doesn't live across await.
        let candidates: Vec<u16> = {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            (0..50)
                .map(|_| rng.gen_range(PORT_RANGE_START..PORT_RANGE_END))
                .collect()
        };

        // Try random ports first.
        for port in candidates {
            {
                let inner = self.inner.read().await;
                if inner.port_to_tunnel.contains_key(&port) {
                    continue;
                }
            }
            if let Ok(listener) = TcpListener::bind(("0.0.0.0", port)).await {
                return Ok((listener, port));
            }
        }

        // Fallback: sequential scan.
        for port in PORT_RANGE_START..PORT_RANGE_END {
            {
                let inner = self.inner.read().await;
                if inner.port_to_tunnel.contains_key(&port) {
                    continue;
                }
            }
            if let Ok(listener) = TcpListener::bind(("0.0.0.0", port)).await {
                return Ok((listener, port));
            }
        }

        bail!("no available ports in range {PORT_RANGE_START}-{PORT_RANGE_END}");
    }
}

impl Default for TunnelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_unregister() {
        let mgr = TunnelManager::new();

        let reg = mgr.register("agent-1", "tcp", None).await.unwrap();
        assert!(reg.remote_port >= PORT_RANGE_START);
        assert!(reg.remote_port < PORT_RANGE_END);
        assert_eq!(mgr.tunnel_count().await, 1);

        mgr.unregister(&reg.tunnel_id).await;
        assert_eq!(mgr.tunnel_count().await, 0);
    }

    #[tokio::test]
    async fn specific_port() {
        let mgr = TunnelManager::new();
        let reg = mgr.register("agent-1", "tcp", Some(19999)).await.unwrap();
        assert_eq!(reg.remote_port, 19999);
        mgr.unregister(&reg.tunnel_id).await;
    }

    #[tokio::test]
    async fn duplicate_port_rejected() {
        let mgr = TunnelManager::new();
        let reg = mgr.register("agent-1", "tcp", None).await.unwrap();
        let port = reg.remote_port;

        let result = mgr.register("agent-2", "tcp", Some(port)).await;
        assert!(result.is_err());

        mgr.unregister(&reg.tunnel_id).await;
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
