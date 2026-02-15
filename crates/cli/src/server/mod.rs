//! Server-side components for subtunnel.

pub mod auth;
pub mod handler;
pub mod listener;
pub mod tunnel_mgr;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use auth::Authenticator;
use handler::handle_agent_connection;
use listener::run_http_listener;
use tunnel_mgr::TunnelManager;

use crate::transport::tls;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub control_port: u16,
    pub http_port: u16,
    pub auth_token: Option<String>,
    pub host: String,
    pub domain: String,
    pub extra_domains: Vec<String>,
    /// Path to PEM certificate file (e.g. Let's Encrypt fullchain.pem).
    pub tls_cert: Option<String>,
    /// Path to PEM private key file.
    pub tls_key: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            control_port: 7835,
            http_port: 8080,
            auth_token: None,
            host: "localhost".into(),
            domain: "tunnel.localhost".into(),
            extra_domains: vec![],
            tls_cert: None,
            tls_key: None,
        }
    }
}

pub struct Server {
    config: ServerConfig,
    tunnel_mgr: TunnelManager,
    auth: Authenticator,
}

impl Server {
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

    pub async fn run(&self) -> Result<()> {
        let tls_config = match (&self.config.tls_cert, &self.config.tls_key) {
            (Some(cert_path), Some(key_path)) => {
                let (certs, key) = tls::load_certs_from_pem(cert_path, key_path)
                    .context("failed to load TLS certificate/key from PEM files")?;
                info!("using TLS certificate from {cert_path}");
                tls::server_config(certs, key)
                    .context("failed to build TLS server config")?
            }
            _ => {
                warn!("no --tls-cert/--tls-key provided; using self-signed certificate (development only)");
                let cert = tls::generate_self_signed_cert()
                    .context("failed to generate self-signed TLS certificate")?;
                tls::server_config(vec![cert.cert_der], cert.key_der)
                    .context("failed to build TLS server config")?
            }
        };

        // Spawn HTTP listener for public traffic
        let http_tunnel_mgr = self.tunnel_mgr.clone();
        let http_port = self.config.http_port;
        let domain = self.config.domain.clone();
        let extra_domains = self.config.extra_domains.clone();
        tokio::spawn(async move {
            if let Err(e) = run_http_listener(http_port, domain, extra_domains, http_tunnel_mgr).await {
                error!(error = %e, "HTTP listener failed");
            }
        });

        // Control plane listener (TLS + yamux)
        let listener = TcpListener::bind(("0.0.0.0", self.config.control_port))
            .await
            .with_context(|| format!("failed to bind control port {}", self.config.control_port))?;

        info!(
            control_port = self.config.control_port,
            http_port = self.config.http_port,
            domain = %self.config.domain,
            host = %self.config.host,
            "subtunnel server listening"
        );

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!(peer = %addr, "new agent connection");

                    let tunnel_mgr = self.tunnel_mgr.clone();
                    let auth = self.auth.clone();
                    let domain = self.config.domain.clone();
                    let tls_cfg = tls_config.clone();

                    tokio::spawn(async move {
                        let tls_stream = match tls::tls_accept(tls_cfg, stream).await {
                            Ok(s) => s,
                            Err(e) => {
                                warn!(peer = %addr, error = %e, "TLS handshake failed");
                                return;
                            }
                        };
                        if let Err(e) =
                            handle_agent_connection(tls_stream, tunnel_mgr, auth, domain).await
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

    pub fn tunnel_manager(&self) -> &TunnelManager {
        &self.tunnel_mgr
    }
}
