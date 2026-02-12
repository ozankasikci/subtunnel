//! Server connection manager with TLS, yamux, auth, and auto-reconnect.

use std::time::Duration;

use anyhow::{bail, Context, Result};
use rustls::pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tracing::{debug, error, info, warn};

use crate::protocol::codec::{read_message, write_message};
use crate::protocol::ControlMessage;
use crate::transport::mux::{MuxSession, YamuxStreamCompatExt};
use crate::transport::tls::{client_config_from_options, ClientTlsOptions};

/// TLS options for the client connection.
#[derive(Debug, Clone)]
pub struct ConnectTlsOptions {
    /// Verify server certificate (default: true).
    pub verify: bool,
    /// Optional custom CA PEM file path.
    pub ca_path: Option<String>,
}

impl Default for ConnectTlsOptions {
    fn default() -> Self {
        Self {
            verify: true,
            ca_path: None,
        }
    }
}

/// Result of a successful tunnel negotiation.
pub struct TunnelInfo {
    /// Unique tunnel identifier assigned by the server.
    pub tunnel_id: String,
    /// Public URL for this tunnel.
    pub public_url: String,
    /// Assigned subdomain.
    pub subdomain: String,
}

/// Established connection to the tunnelr server, ready for proxying.
pub struct EstablishedConnection {
    /// The yamux session for multiplexing data streams.
    pub mux: MuxSession,
    /// Info about the negotiated tunnel.
    pub tunnel_info: TunnelInfo,
    /// Handle to the control stream keepalive task. Dropping this aborts the task.
    pub _control_handle: tokio::task::JoinHandle<()>,
}

/// Extract the hostname from a "host:port" address string.
fn extract_hostname(server_addr: &str) -> &str {
    // Handle [ipv6]:port
    if let Some(bracket_end) = server_addr.find(']') {
        &server_addr[..=bracket_end]
    } else if let Some(colon) = server_addr.rfind(':') {
        &server_addr[..colon]
    } else {
        server_addr
    }
}

/// Connect to the server, perform TLS + auth + tunnel handshake.
///
/// Returns an `EstablishedConnection` on success.
pub async fn connect(
    server_addr: &str,
    token: &str,
    requested_subdomain: Option<&str>,
    tls_opts: &ConnectTlsOptions,
) -> Result<EstablishedConnection> {
    // TCP connect
    let tcp = TcpStream::connect(server_addr)
        .await
        .with_context(|| format!("failed to connect to {server_addr}"))?;
    debug!("TCP connected to {server_addr}");

    // Determine hostname for SNI from the server address
    let hostname = extract_hostname(server_addr).to_string();

    // TLS handshake
    let tls_config = client_config_from_options(&ClientTlsOptions {
        verify: tls_opts.verify,
        ca_path: tls_opts.ca_path.clone(),
        hostname: hostname.clone(),
    })?;
    let connector = TlsConnector::from(tls_config);
    let server_name = ServerName::try_from(hostname.clone())
        .map_err(|e| anyhow::anyhow!("invalid server name '{hostname}': {e}"))?;
    let tls_stream = connector
        .connect(server_name, tcp)
        .await
        .context("TLS handshake failed")?;
    debug!("TLS handshake complete");

    // Set up yamux (client mode) — wrap TLS stream with compat for futures::io
    let mut mux = MuxSession::new(tls_stream.compat(), yamux::Mode::Client);

    // Open control stream and perform auth handshake
    let control = mux.open_stream().await?;
    // Wrap yamux stream with compat for tokio::io (needed by read_message/write_message)
    let mut control = control.compat();
    debug!("control stream opened");

    // Send Auth
    let auth_msg = ControlMessage::Auth {
        token: token.to_string(),
    };
    write_message(&mut control, &auth_msg).await?;
    debug!("sent auth");

    // Read AuthResp
    let resp = read_message(&mut control)
        .await?
        .context("server closed connection during auth")?;
    match resp {
        ControlMessage::AuthResp {
            success: true,
            message,
        } => {
            debug!("auth success: {message}");
        }
        ControlMessage::AuthResp {
            success: false,
            message,
        } => {
            bail!("authentication failed: {message}");
        }
        other => bail!("unexpected message during auth: {other:?}"),
    }

    // Send TunnelReq
    let tunnel_req = ControlMessage::TunnelReq {
        protocol: "tcp".into(),
        subdomain: requested_subdomain.map(|s| s.to_string()),
    };
    write_message(&mut control, &tunnel_req).await?;
    debug!("sent tunnel request");

    // Read TunnelResp
    let resp = read_message(&mut control)
        .await?
        .context("server closed connection during tunnel setup")?;
    let tunnel_info = match resp {
        ControlMessage::TunnelResp {
            success: true,
            tunnel_id,
            subdomain,
            message,
        } => TunnelInfo {
            tunnel_id,
            public_url: message,
            subdomain,
        },
        ControlMessage::TunnelResp {
            success: false,
            message,
            ..
        } => {
            bail!("tunnel request rejected: {message}");
        }
        other => bail!("unexpected message during tunnel setup: {other:?}"),
    };

    info!(
        tunnel_id = %tunnel_info.tunnel_id,
        public_url = %tunnel_info.public_url,
        "tunnel established"
    );

    // Spawn a task to keep the control stream alive and handle heartbeats.
    let control_handle = tokio::spawn(async move {
        loop {
            match read_message(&mut control).await {
                Ok(Some(ControlMessage::Heartbeat)) => {
                    debug!("heartbeat received from server");
                    if let Err(e) = write_message(&mut control, &ControlMessage::HeartbeatAck).await {
                        warn!("failed to send heartbeat ack: {e}");
                        break;
                    }
                }
                Ok(Some(ControlMessage::HeartbeatAck)) => {
                    debug!("heartbeat ack from server");
                }
                Ok(Some(other)) => {
                    debug!("control message: {other:?}");
                }
                Ok(None) => {
                    info!("control stream closed by server");
                    break;
                }
                Err(e) => {
                    warn!("control stream error: {e}");
                    break;
                }
            }
        }
    });

    Ok(EstablishedConnection { mux, tunnel_info, _control_handle: control_handle })
}

/// Connect with auto-reconnect and exponential backoff.
pub async fn connect_with_retry<F, Fut>(
    server_addr: &str,
    token: &str,
    requested_subdomain: Option<&str>,
    tls_opts: &ConnectTlsOptions,
    shutdown: tokio::sync::watch::Receiver<bool>,
    mut on_connected: F,
) -> Result<()>
where
    F: FnMut(EstablishedConnection) -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let mut backoff = ExponentialBackoff::new();

    loop {
        if *shutdown.borrow() {
            info!("shutdown requested, stopping reconnect loop");
            return Ok(());
        }

        match connect(server_addr, token, requested_subdomain, tls_opts).await {
            Ok(conn) => {
                backoff.reset();
                if let Err(e) = on_connected(conn).await {
                    warn!("connection session ended: {e:#}");
                }
            }
            Err(e) => {
                error!("connection failed: {e:#}");
            }
        }

        if *shutdown.borrow() {
            return Ok(());
        }

        let delay = backoff.next_delay();
        info!("reconnecting in {delay:.1?}");
        tokio::select! {
            _ = tokio::time::sleep(delay) => {}
            _ = shutdown_wait(shutdown.clone()) => {
                return Ok(());
            }
        }
    }
}

/// Wait until the shutdown signal fires.
async fn shutdown_wait(mut rx: tokio::sync::watch::Receiver<bool>) {
    while !*rx.borrow() {
        if rx.changed().await.is_err() {
            return;
        }
    }
}

/// Exponential backoff with jitter for reconnection.
struct ExponentialBackoff {
    current: Duration,
    max: Duration,
}

impl ExponentialBackoff {
    fn new() -> Self {
        Self {
            current: Duration::from_secs(1),
            max: Duration::from_secs(60),
        }
    }

    fn reset(&mut self) {
        self.current = Duration::from_secs(1);
    }

    fn next_delay(&mut self) -> Duration {
        let delay = self.current;
        self.current = (self.current * 2).min(self.max);
        let jitter_range = delay.as_millis() as u64 / 4;
        if jitter_range > 0 {
            let jitter = rand::random::<u64>() % (jitter_range * 2);
            let base_ms = delay.as_millis() as u64;
            let jittered = base_ms - jitter_range + jitter;
            Duration::from_millis(jittered)
        } else {
            delay
        }
    }
}
