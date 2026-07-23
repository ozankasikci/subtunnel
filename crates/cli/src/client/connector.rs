//! Server connection manager with TLS, yamux, auth, and auto-reconnect.

use std::time::Duration;

use anyhow::{bail, Context, Result};
use rustls::pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tracing::{debug, error, info, warn};

use crate::protocol::codec::{read_message, write_message_with_timeout};
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

/// Established connection to the subtunnel server, ready for proxying.
pub struct EstablishedConnection {
    /// The yamux session for multiplexing data streams.
    pub mux: MuxSession,
    /// Info about the negotiated tunnel.
    pub tunnel_info: TunnelInfo,
    /// Receiver that becomes `false` when the control channel detects a dead connection.
    pub alive: tokio::sync::watch::Receiver<bool>,
    /// Guard for the control stream keepalive task. Dropping this aborts the task.
    pub _control_handle: AbortOnDrop,
}

/// A Tokio task handle that aborts its task when dropped.
pub struct AbortOnDrop {
    handle: tokio::task::JoinHandle<()>,
}

impl AbortOnDrop {
    fn new(handle: tokio::task::JoinHandle<()>) -> Self {
        Self { handle }
    }

    fn abort(&self) {
        self.handle.abort();
    }
}

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl EstablishedConnection {
    fn from_parts(
        mux: MuxSession,
        tunnel_info: TunnelInfo,
        alive: tokio::sync::watch::Receiver<bool>,
        control_handle: tokio::task::JoinHandle<()>,
    ) -> Self {
        Self {
            mux,
            tunnel_info,
            alive,
            _control_handle: AbortOnDrop::new(control_handle),
        }
    }
}

/// Timing controls for the client control stream.
#[derive(Debug, Clone, Copy)]
pub struct ClientControlConfig {
    /// How often the client sends heartbeats to the server.
    pub heartbeat_interval: Duration,
    /// How long the client tolerates receiving no server control messages.
    pub heartbeat_timeout: Duration,
    /// Maximum time allowed for one control-channel write.
    pub write_timeout: Duration,
}

impl Default for ClientControlConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(15),
            heartbeat_timeout: Duration::from_secs(45),
            write_timeout: Duration::from_secs(10),
        }
    }
}

/// Extract the hostname from a "host:port" address string.
fn extract_hostname(server_addr: &str) -> &str {
    // Handle [ipv6]:port
    if let Some(bracket_end) = server_addr.find(']') {
        &server_addr[1..bracket_end]
    } else if let Some(colon) = server_addr.rfind(':') {
        &server_addr[..colon]
    } else {
        server_addr
    }
}

async fn read_setup_response<S>(
    control: &mut S,
    write_timeout: Duration,
    closed_context: &'static str,
) -> Result<ControlMessage>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    loop {
        let message = read_message(&mut *control).await?.context(closed_context)?;
        match message {
            ControlMessage::Heartbeat => {
                debug!("heartbeat received during connection setup");
                write_message_with_timeout(
                    &mut *control,
                    &ControlMessage::HeartbeatAck,
                    write_timeout,
                )
                .await
                .context("failed to acknowledge heartbeat during connection setup")?;
            }
            ControlMessage::HeartbeatAck => {
                debug!("heartbeat ack received during connection setup");
            }
            other => return Ok(other),
        }
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
    connect_with_config(
        server_addr,
        token,
        requested_subdomain,
        tls_opts,
        ClientControlConfig::default(),
    )
    .await
}

/// Connect using explicit client control-stream timing.
pub async fn connect_with_config(
    server_addr: &str,
    token: &str,
    requested_subdomain: Option<&str>,
    tls_opts: &ConnectTlsOptions,
    control_config: ClientControlConfig,
) -> Result<EstablishedConnection> {
    // TCP connect
    let tcp = TcpStream::connect(server_addr)
        .await
        .with_context(|| format!("failed to connect to {server_addr}"))?;
    crate::transport::set_tcp_keepalive(&tcp)?;
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
    write_message_with_timeout(&mut control, &auth_msg, control_config.write_timeout).await?;
    debug!("sent auth");

    // Read AuthResp
    let resp = read_setup_response(
        &mut control,
        control_config.write_timeout,
        "server closed connection during auth",
    )
    .await?;
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

    // Send RegisterReq
    let register_req = ControlMessage::RegisterReq {
        protocol: "tcp".into(),
        subdomain: requested_subdomain.map(|s| s.to_string()),
    };
    write_message_with_timeout(&mut control, &register_req, control_config.write_timeout).await?;
    debug!("sent tunnel request");

    // Read RegisterResp
    let resp = read_setup_response(
        &mut control,
        control_config.write_timeout,
        "server closed connection during tunnel setup",
    )
    .await?;
    let tunnel_info = match resp {
        ControlMessage::RegisterResp {
            success: true,
            tunnel_id,
            subdomain,
            message,
        } => TunnelInfo {
            tunnel_id,
            public_url: message,
            subdomain,
        },
        ControlMessage::RegisterResp {
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

    // Spawn a task that sends heartbeats and detects dead connections.
    let (alive_rx, control_handle) = spawn_control_task(control, control_config, None);

    Ok(EstablishedConnection::from_parts(
        mux,
        tunnel_info,
        alive_rx,
        control_handle,
    ))
}

/// Start the heartbeat and control-message task on any Tokio async stream.
///
/// The optional observer receives a clone of every decoded server message.
#[doc(hidden)]
pub fn spawn_control_task<S>(
    control: S,
    config: ClientControlConfig,
    observer: Option<tokio::sync::mpsc::UnboundedSender<ControlMessage>>,
) -> (
    tokio::sync::watch::Receiver<bool>,
    tokio::task::JoinHandle<()>,
)
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (alive_tx, alive_rx) = tokio::sync::watch::channel(true);
    let control_handle = tokio::spawn(async move {
        let (mut ctrl_read, mut ctrl_write) = tokio::io::split(control);
        let (message_tx, mut message_rx) = tokio::sync::mpsc::channel(16);
        let reader_handle = AbortOnDrop::new(tokio::spawn(async move {
            loop {
                let message = read_message(&mut ctrl_read).await;
                let finished = !matches!(message, Ok(Some(_)));
                if message_tx.send(message).await.is_err() || finished {
                    return;
                }
            }
        }));
        let mut last_received = tokio::time::Instant::now();
        let mut heartbeat_interval = tokio::time::interval(config.heartbeat_interval);
        heartbeat_interval.tick().await; // consume the immediate first tick

        loop {
            tokio::select! {
                _ = heartbeat_interval.tick() => {
                    // Check if we've timed out waiting for any server message
                    if last_received.elapsed() > config.heartbeat_timeout {
                        warn!("no heartbeat from server in {:?}, connection presumed dead", config.heartbeat_timeout);
                        break;
                    }
                    // Send our own heartbeat
                    if let Err(e) = write_message_with_timeout(
                        &mut ctrl_write,
                        &ControlMessage::Heartbeat,
                        config.write_timeout,
                    ).await {
                        warn!("failed to send heartbeat: {e}");
                        break;
                    }
                    debug!("client heartbeat sent");
                }
                msg = message_rx.recv() => {
                    let Some(msg) = msg else {
                        info!("control reader stopped");
                        break;
                    };
                    if let Ok(Some(message)) = &msg {
                        if let Some(observer) = &observer {
                            let _ = observer.send(message.clone());
                        }
                    }
                    match msg {
                        Ok(Some(ControlMessage::Heartbeat)) => {
                            debug!("heartbeat received from server");
                            last_received = tokio::time::Instant::now();
                            if let Err(e) = write_message_with_timeout(
                                &mut ctrl_write,
                                &ControlMessage::HeartbeatAck,
                                config.write_timeout,
                            ).await {
                                warn!("failed to send heartbeat ack: {e}");
                                break;
                            }
                        }
                        Ok(Some(ControlMessage::HeartbeatAck)) => {
                            debug!("heartbeat ack from server");
                            last_received = tokio::time::Instant::now();
                        }
                        Ok(Some(other)) => {
                            debug!("control message: {other:?}");
                            last_received = tokio::time::Instant::now();
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
            }
        }
        reader_handle.abort();
        // Signal that the connection is dead
        let _ = alive_tx.send(false);
    });

    (alive_rx, control_handle)
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

#[cfg(test)]
mod tests {
    use super::*;

    struct DropSignal(Option<tokio::sync::oneshot::Sender<()>>);

    impl Drop for DropSignal {
        fn drop(&mut self) {
            if let Some(sender) = self.0.take() {
                let _ = sender.send(());
            }
        }
    }

    #[test]
    fn extract_hostname_strips_ipv6_brackets() {
        assert_eq!(extract_hostname("[::1]:7835"), "::1");
    }

    #[tokio::test]
    async fn setup_response_skips_and_acknowledges_heartbeats() {
        let (mut client, mut server) = tokio::io::duplex(1024);
        let server_task = tokio::spawn(async move {
            let responses = [
                ControlMessage::AuthResp {
                    success: true,
                    message: "welcome".into(),
                },
                ControlMessage::RegisterResp {
                    success: true,
                    tunnel_id: "tunnel-id".into(),
                    subdomain: "setup".into(),
                    message: "https://setup.example.test".into(),
                },
            ];

            for response in responses {
                write_message_with_timeout(
                    &mut server,
                    &ControlMessage::Heartbeat,
                    Duration::from_millis(100),
                )
                .await
                .unwrap();
                write_message_with_timeout(
                    &mut server,
                    &ControlMessage::HeartbeatAck,
                    Duration::from_millis(100),
                )
                .await
                .unwrap();
                write_message_with_timeout(&mut server, &response, Duration::from_millis(100))
                    .await
                    .unwrap();

                assert_eq!(
                    read_message(&mut server).await.unwrap(),
                    Some(ControlMessage::HeartbeatAck)
                );
            }
        });

        let auth_response = read_setup_response(
            &mut client,
            Duration::from_millis(100),
            "server closed during auth test",
        )
        .await
        .unwrap();
        assert!(matches!(
            auth_response,
            ControlMessage::AuthResp { success: true, .. }
        ));

        let tunnel_response = read_setup_response(
            &mut client,
            Duration::from_millis(100),
            "server closed during tunnel test",
        )
        .await
        .unwrap();
        assert!(matches!(
            tunnel_response,
            ControlMessage::RegisterResp { success: true, .. }
        ));

        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn dropping_established_connection_aborts_control_task() {
        let (io, _peer) = tokio::io::duplex(64);
        let mux = MuxSession::new(io.compat(), yamux::Mode::Client);
        let (_alive_tx, alive) = tokio::sync::watch::channel(true);
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (dropped_tx, dropped_rx) = tokio::sync::oneshot::channel();
        let control_handle = tokio::spawn(async move {
            let _drop_signal = DropSignal(Some(dropped_tx));
            let _ = started_tx.send(());
            std::future::pending::<()>().await;
        });
        started_rx.await.unwrap();

        let connection = EstablishedConnection::from_parts(
            mux,
            TunnelInfo {
                tunnel_id: "test".into(),
                public_url: "https://test.example".into(),
                subdomain: "test".into(),
            },
            alive,
            control_handle,
        );
        drop(connection);

        tokio::time::timeout(Duration::from_millis(200), dropped_rx)
            .await
            .expect("control task was detached instead of aborted")
            .unwrap();
    }
}
