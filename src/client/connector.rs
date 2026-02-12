//! Server connection manager with TLS, yamux, auth, and auto-reconnect.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, Error as TlsError, SignatureScheme};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tracing::{debug, error, info, warn};

use crate::protocol::codec::{read_message, write_message};
use crate::protocol::ControlMessage;
use crate::transport::mux::{MuxSession, YamuxStreamCompatExt};

/// Result of a successful tunnel negotiation.
pub struct TunnelInfo {
    /// Unique tunnel identifier assigned by the server.
    pub tunnel_id: String,
    /// Public address that external clients connect to.
    pub public_addr: String,
    /// Assigned remote port on the server.
    pub remote_port: u16,
}

/// Established connection to the tunnelr server, ready for proxying.
pub struct EstablishedConnection {
    /// The yamux session for multiplexing data streams.
    pub mux: MuxSession,
    /// Info about the negotiated tunnel.
    pub tunnel_info: TunnelInfo,
}

/// Connect to the server, perform TLS + auth + tunnel handshake.
///
/// Returns an `EstablishedConnection` on success.
pub async fn connect(
    server_addr: &str,
    token: &str,
    remote_port: Option<u16>,
) -> Result<EstablishedConnection> {
    // TCP connect
    let tcp = TcpStream::connect(server_addr)
        .await
        .with_context(|| format!("failed to connect to {server_addr}"))?;
    debug!("TCP connected to {server_addr}");

    // TLS handshake (skip verification for self-hosted servers)
    let tls_config = make_tls_config();
    let connector = TlsConnector::from(Arc::new(tls_config));
    let server_name =
        ServerName::try_from("tunnelr").map_err(|e| anyhow::anyhow!("invalid server name: {e}"))?;
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
        remote_port,
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
            remote_port: port,
            ..
        } => TunnelInfo {
            tunnel_id,
            // Reconstruct public address from server_addr host + assigned port
            public_addr: format!(
                "{}:{}",
                server_addr.split(':').next().unwrap_or(server_addr),
                port
            ),
            remote_port: port,
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
        public_addr = %tunnel_info.public_addr,
        "tunnel established"
    );

    Ok(EstablishedConnection { mux, tunnel_info })
}

/// Connect with auto-reconnect and exponential backoff.
///
/// Calls `on_connected` each time a connection is established. The callback
/// should run the proxy loop and return when the connection drops.
pub async fn connect_with_retry<F, Fut>(
    server_addr: &str,
    token: &str,
    remote_port: Option<u16>,
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

        match connect(server_addr, token, remote_port).await {
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

/// Build a TLS client config that accepts any server certificate.
///
/// This is appropriate for self-hosted tunnelr servers using self-signed certs.
/// In production, you'd use proper CA verification.
fn make_tls_config() -> ClientConfig {
    let provider = rustls::crypto::ring::default_provider();
    ClientConfig::builder_with_provider(Arc::new(provider))
        .with_protocol_versions(&[&rustls::version::TLS13])
        .expect("TLS 1.3 config")
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier))
        .with_no_client_auth()
}

/// Certificate verifier that accepts any server certificate.
#[derive(Debug)]
struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
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
        // Add jitter: +/- 25%
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
