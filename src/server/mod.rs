//! Server-side components for tunnelr.
//!
//! The server listens for agent connections on a control port, authenticates
//! them, and for each registered tunnel, accepts public TCP connections and
//! proxies them through yamux streams to the agent.

pub mod auth;
pub mod handler;
pub mod listener;
pub mod tunnel_mgr;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tracing::{error, info};

use auth::Authenticator;
use handler::handle_agent_connection;
use tunnel_mgr::TunnelManager;

/// Configuration for the tunnelr server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to listen on for agent control connections.
    pub control_port: u16,
    /// Shared secret token that agents must present.
    /// If `None`, authentication is disabled.
    pub auth_token: Option<String>,
    /// Hostname that public tunnels are reachable on (for building public_addr).
    pub host: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            control_port: 7835,
            auth_token: None,
            host: "localhost".into(),
        }
    }
}

/// The tunnelr server.
///
/// Listens for agent connections, authenticates them, manages tunnel
/// registration, and proxies public traffic to agents via yamux.
pub struct Server {
    config: ServerConfig,
    tunnel_mgr: TunnelManager,
    auth: Authenticator,
}

impl Server {
    /// Create a new server with the given configuration.
    pub fn new(config: ServerConfig) -> Self {
        let auth = match &config.auth_token {
            Some(token) => Authenticator::new(token.clone()),
            None => Authenticator::allow_all(),
        };
        Self {
            config,
            tunnel_mgr: TunnelManager::new(),
            auth,
        }
    }

    /// Run the server, listening for agent connections on the control port.
    ///
    /// This function runs until the process is terminated.
    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(("0.0.0.0", self.config.control_port))
            .await
            .with_context(|| format!("failed to bind control port {}", self.config.control_port))?;

        info!(
            port = self.config.control_port,
            host = %self.config.host,
            "tunnelr server listening for agent connections"
        );

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!(peer = %addr, "new agent connection");

                    let tunnel_mgr = self.tunnel_mgr.clone();
                    let auth = self.auth.clone();
                    let host = self.config.host.clone();

                    tokio::spawn(async move {
                        // For MVP, accept raw TCP (no TLS). The handler is generic
                        // over AsyncRead+AsyncWrite, so TLS can be layered in later
                        // by wrapping `stream` with tokio_rustls::TlsAcceptor.
                        if let Err(e) =
                            handle_agent_connection(stream, tunnel_mgr, auth, host).await
                        {
                            error!(peer = %addr, error = %e, "agent handler error");
                        }
                    });
                }
                Err(e) => {
                    error!(error = %e, "failed to accept agent connection");
                }
            }
        }
    }

    /// Get a reference to the tunnel manager (for testing / introspection).
    pub fn tunnel_manager(&self) -> &TunnelManager {
        &self.tunnel_mgr
    }
}
