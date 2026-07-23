//! Agent connection handler — manages a single agent's lifecycle.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use tokio::sync::{mpsc, watch, Mutex};
use tracing::{debug, error, info, warn};

use crate::protocol::codec::{read_message, write_message_with_timeout};
use crate::protocol::ControlMessage;
use crate::transport::mux::{MuxSession, YamuxStreamCompatExt};

use super::auth::Authenticator;
use super::listener::{proxy_tunnel_connections_with_config, ListenerConfig};
use super::tunnel_mgr::TunnelManager;

const AUTH_TIMEOUT: Duration = Duration::from_secs(10);

struct TaskAbortGuard(tokio::task::JoinHandle<()>);

impl TaskAbortGuard {
    fn new(handle: tokio::task::JoinHandle<()>) -> Self {
        Self(handle)
    }

    fn abort(&self) {
        self.0.abort();
    }
}

impl Drop for TaskAbortGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Timing controls for server-side agent heartbeats.
#[derive(Debug, Clone, Copy)]
pub struct HeartbeatConfig {
    /// How often the server sends a heartbeat.
    pub interval: Duration,
    /// Maximum consecutive probes that may receive no agent message.
    ///
    /// The first probe is sent after one full `interval`, and each sent probe
    /// is given another full `interval` for any agent message to reset the
    /// count. The server disconnects after `miss_limit` unanswered probes. A
    /// value of zero disconnects immediately without sending a probe.
    pub miss_limit: u32,
    /// Maximum time allowed for one control-channel write.
    pub write_timeout: Duration,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            miss_limit: 3,
            write_timeout: Duration::from_secs(10),
        }
    }
}

pub async fn handle_agent_connection<T>(
    io: T,
    tunnel_mgr: TunnelManager,
    auth: Authenticator,
    domain: String,
) -> Result<()>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    handle_agent_connection_with_config(io, tunnel_mgr, auth, domain, HeartbeatConfig::default())
        .await
}

/// Handle one agent connection with explicit heartbeat timing.
pub async fn handle_agent_connection_with_config<T>(
    io: T,
    tunnel_mgr: TunnelManager,
    auth: Authenticator,
    domain: String,
    heartbeat_config: HeartbeatConfig,
) -> Result<()>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    handle_agent_connection_with_configs(
        io,
        tunnel_mgr,
        auth,
        domain,
        heartbeat_config,
        ListenerConfig::default(),
    )
    .await
}

/// Handle one agent connection with explicit heartbeat and proxy timing.
pub async fn handle_agent_connection_with_configs<T>(
    io: T,
    tunnel_mgr: TunnelManager,
    auth: Authenticator,
    domain: String,
    heartbeat_config: HeartbeatConfig,
    listener_config: ListenerConfig,
) -> Result<()>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    use tokio_util::compat::TokioAsyncReadCompatExt;

    let mut mux = MuxSession::new(io.compat(), yamux::Mode::Server);

    let control_stream = match mux.accept_stream().await? {
        Some(stream) => stream,
        None => bail!("agent disconnected before opening control stream"),
    };

    let control = control_stream.compat();
    let (mut ctrl_read, ctrl_write) = tokio::io::split(control);
    let ctrl_write = Arc::new(Mutex::new(ctrl_write));

    // --- Authentication ---
    let agent_id = {
        let msg = tokio::time::timeout(AUTH_TIMEOUT, read_message(&mut ctrl_read))
            .await
            .context("auth timeout")?
            .context("failed to read auth message")?;

        match msg {
            Some(ControlMessage::Auth { token }) => {
                if !auth.validate(&token) {
                    let mut writer = ctrl_write.lock().await;
                    let _ = write_message_with_timeout(
                        &mut *writer,
                        &ControlMessage::AuthResp {
                            success: false,
                            message: "invalid token".into(),
                        },
                        heartbeat_config.write_timeout,
                    )
                    .await;
                    bail!("agent provided invalid token");
                }
                let agent_id = format!("agent_{}", uuid::Uuid::new_v4().as_simple());
                let mut writer = ctrl_write.lock().await;
                write_message_with_timeout(
                    &mut *writer,
                    &ControlMessage::AuthResp {
                        success: true,
                        message: format!("welcome, {agent_id}"),
                    },
                    heartbeat_config.write_timeout,
                )
                .await
                .context("failed to send auth response")?;
                info!(agent_id = %agent_id, "agent authenticated");
                agent_id
            }
            Some(other) => bail!("expected Auth message, got: {other:?}"),
            None => bail!("agent disconnected during authentication"),
        }
    };

    let mux = Arc::new(Mutex::new(mux));

    let ack_counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let (disconnect_tx, disconnect_rx) = watch::channel(false);
    let heartbeat_handle = {
        let ctrl_write = ctrl_write.clone();
        let agent_id = agent_id.clone();
        let ack_counter = ack_counter.clone();
        TaskAbortGuard::new(tokio::spawn(async move {
            heartbeat_loop(agent_id, ctrl_write, ack_counter, heartbeat_config).await;
            let _ = disconnect_tx.send(true);
        }))
    };

    let (message_tx, message_rx) = mpsc::channel(16);
    let reader_handle = TaskAbortGuard::new(tokio::spawn(control_reader(ctrl_read, message_tx)));

    let context = ControlContext {
        agent_id: &agent_id,
        ctrl_write: &ctrl_write,
        tunnel_mgr: &tunnel_mgr,
        mux: &mux,
        domain: &domain,
        ack_counter: &ack_counter,
        write_timeout: heartbeat_config.write_timeout,
        listener_config,
    };
    let result = control_loop(context, message_rx, disconnect_rx).await;

    heartbeat_handle.abort();
    reader_handle.abort();
    tunnel_mgr.unregister_agent(&agent_id).await;
    info!(agent_id = %agent_id, "agent handler exiting");

    result
}

async fn control_reader<CR>(
    mut ctrl_read: CR,
    message_tx: mpsc::Sender<Result<Option<ControlMessage>>>,
) where
    CR: tokio::io::AsyncRead + Unpin,
{
    loop {
        let message = read_message(&mut ctrl_read).await;
        let finished = !matches!(message, Ok(Some(_)));
        if message_tx.send(message).await.is_err() || finished {
            return;
        }
    }
}

struct ControlContext<'a, CW> {
    agent_id: &'a str,
    ctrl_write: &'a Arc<Mutex<CW>>,
    tunnel_mgr: &'a TunnelManager,
    mux: &'a Arc<Mutex<MuxSession>>,
    domain: &'a str,
    ack_counter: &'a Arc<std::sync::atomic::AtomicU32>,
    write_timeout: Duration,
    listener_config: ListenerConfig,
}

async fn control_loop<CW>(
    context: ControlContext<'_, CW>,
    mut message_rx: mpsc::Receiver<Result<Option<ControlMessage>>>,
    mut disconnect_rx: watch::Receiver<bool>,
) -> Result<()>
where
    CW: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    loop {
        let msg = tokio::select! {
            changed = disconnect_rx.changed() => {
                if changed.is_err() || *disconnect_rx.borrow() {
                    info!(agent_id = %context.agent_id, "heartbeat monitor disconnected agent");
                    return Ok(());
                }
                continue;
            }
            message = message_rx.recv() => {
                match message {
                    Some(message) => message,
                    None => {
                        info!(agent_id = %context.agent_id, "control reader stopped");
                        return Ok(());
                    }
                }
            }
        };

        if let Ok(Some(_)) = &msg {
            context
                .ack_counter
                .store(0, std::sync::atomic::Ordering::SeqCst);
        }

        match msg {
            Ok(Some(ControlMessage::RegisterReq {
                protocol,
                subdomain,
            })) => {
                handle_tunnel_request(&context, &protocol, subdomain.as_deref()).await?;
            }
            Ok(Some(ControlMessage::Heartbeat)) => {
                let mut writer = context.ctrl_write.lock().await;
                write_message_with_timeout(
                    &mut *writer,
                    &ControlMessage::HeartbeatAck,
                    context.write_timeout,
                )
                .await
                .context("failed to send heartbeat ack")?;
            }
            Ok(Some(ControlMessage::HeartbeatAck)) => {
                debug!(agent_id = %context.agent_id, "heartbeat ack received");
            }
            Ok(Some(other)) => {
                warn!(agent_id = %context.agent_id, msg = ?other, "unexpected control message");
            }
            Ok(None) => {
                info!(agent_id = %context.agent_id, "agent disconnected (control stream EOF)");
                return Ok(());
            }
            Err(e) => {
                error!(agent_id = %context.agent_id, error = %e, "control stream read error");
                return Err(e);
            }
        }
    }
}

async fn handle_tunnel_request<CW>(
    context: &ControlContext<'_, CW>,
    protocol: &str,
    requested_subdomain: Option<&str>,
) -> Result<()>
where
    CW: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    debug!(agent_id = %context.agent_id, protocol = %protocol, subdomain = ?requested_subdomain, "tunnel request");

    match context
        .tunnel_mgr
        .register(context.agent_id, protocol, requested_subdomain)
        .await
    {
        Ok(registered) => {
            let tid = registered.tunnel_id.clone();
            let subdomain = registered.subdomain.clone();

            {
                let mut writer = context.ctrl_write.lock().await;
                write_message_with_timeout(
                    &mut *writer,
                    &ControlMessage::RegisterResp {
                        success: true,
                        tunnel_id: tid.clone(),
                        subdomain: subdomain.clone(),
                        message: format!("https://{subdomain}.{}", context.domain),
                    },
                    context.write_timeout,
                )
                .await
                .context("failed to send tunnel response")?;
            }

            let mux = Arc::downgrade(context.mux);
            tokio::spawn(proxy_tunnel_connections_with_config(
                tid,
                registered.conn_rx,
                move || {
                    let mux = mux.clone();
                    async move {
                        let mux = mux.upgrade().context("agent yamux session closed")?;
                        let mut session = mux.lock().await;
                        let stream = session.open_stream().await?;
                        Ok(stream.compat())
                    }
                },
                context.listener_config,
            ));
        }
        Err(e) => {
            warn!(agent_id = %context.agent_id, error = %e, "tunnel registration failed");
            let mut writer = context.ctrl_write.lock().await;
            write_message_with_timeout(
                &mut *writer,
                &ControlMessage::RegisterResp {
                    success: false,
                    tunnel_id: String::new(),
                    subdomain: String::new(),
                    message: e.to_string(),
                },
                context.write_timeout,
            )
            .await
            .context("failed to send tunnel error response")?;
        }
    }

    Ok(())
}

async fn heartbeat_loop<CW: tokio::io::AsyncWrite + Unpin + Send>(
    agent_id: String,
    ctrl_write: Arc<Mutex<CW>>,
    ack_counter: Arc<std::sync::atomic::AtomicU32>,
    config: HeartbeatConfig,
) {
    if config.miss_limit == 0 {
        warn!(agent_id = %agent_id, "heartbeat miss limit is zero, disconnecting");
        return;
    }

    loop {
        tokio::time::sleep(config.interval).await;
        let missed = ack_counter.load(std::sync::atomic::Ordering::SeqCst);
        if missed >= config.miss_limit {
            warn!(agent_id = %agent_id, missed, "agent unresponsive, disconnecting");
            break;
        }

        let mut writer = ctrl_write.lock().await;
        ack_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if let Err(e) = write_message_with_timeout(
            &mut *writer,
            &ControlMessage::Heartbeat,
            config.write_timeout,
        )
        .await
        {
            warn!(agent_id = %agent_id, error = %e, "failed to send heartbeat, stopping");
            break;
        }
        drop(writer);
        debug!(agent_id = %agent_id, "heartbeat sent");
    }
}
