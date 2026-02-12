//! Agent connection handler — manages a single agent's lifecycle.
//!
//! For each agent that connects:
//! 1. Accept TLS connection
//! 2. Upgrade to yamux (server mode)
//! 3. Accept the first yamux stream as the control channel
//! 4. Authenticate via control message
//! 5. Process tunnel requests
//! 6. Run heartbeat loop
//! 7. For each public connection, open a yamux stream and proxy

use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::protocol::codec::{read_message, write_message};
use crate::protocol::ControlMessage;
use crate::transport::mux::{MuxSession, YamuxStreamCompatExt};

use super::auth::Authenticator;
use super::listener::proxy_tunnel_connections;
use super::tunnel_mgr::TunnelManager;

/// How often we send heartbeats to the agent.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
/// How long to wait for the initial auth message.
const AUTH_TIMEOUT: Duration = Duration::from_secs(10);

/// Handle a single agent connection.
///
/// `io` is the raw (already TLS-terminated) transport. This function upgrades
/// it to yamux, authenticates, and then manages the agent session until
/// disconnect.
pub async fn handle_agent_connection<T>(
    io: T,
    tunnel_mgr: TunnelManager,
    auth: Authenticator,
    server_host: String,
) -> Result<()>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    use tokio_util::compat::TokioAsyncReadCompatExt;

    // Set up yamux in server mode — wrap tokio stream with compat for futures::io
    let mut mux = MuxSession::new(io.compat(), yamux::Mode::Server);

    // The first inbound stream is the control channel.
    let control_stream = match mux.accept_stream().await? {
        Some(stream) => stream,
        None => bail!("agent disconnected before opening control stream"),
    };

    // Wrap yamux stream with compat for tokio::io (needed by read_message/write_message)
    let control = control_stream.compat();
    let (ctrl_read, ctrl_write) = tokio::io::split(control);
    let ctrl_read = Arc::new(Mutex::new(ctrl_read));
    let ctrl_write = Arc::new(Mutex::new(ctrl_write));

    // --- Authentication ---
    let agent_id = {
        let mut reader = ctrl_read.lock().await;
        let msg = tokio::time::timeout(AUTH_TIMEOUT, read_message(&mut *reader))
            .await
            .context("auth timeout")?
            .context("failed to read auth message")?;

        match msg {
            Some(ControlMessage::Auth { token }) => {
                if !auth.validate(&token) {
                    let mut writer = ctrl_write.lock().await;
                    let _ = write_message(
                        &mut *writer,
                        &ControlMessage::AuthResp {
                            success: false,
                            message: "invalid token".into(),
                        },
                    )
                    .await;
                    bail!("agent provided invalid token");
                }
                let agent_id = format!("agent_{}", uuid::Uuid::new_v4().as_simple());
                let mut writer = ctrl_write.lock().await;
                write_message(
                    &mut *writer,
                    &ControlMessage::AuthResp {
                        success: true,
                        message: format!("welcome, {agent_id}"),
                    },
                )
                .await
                .context("failed to send auth response")?;
                info!(agent_id = %agent_id, "agent authenticated");
                agent_id
            }
            Some(other) => {
                bail!("expected Auth message, got: {other:?}");
            }
            None => {
                bail!("agent disconnected during authentication");
            }
        }
    };

    // Wrap the mux session in Arc<Mutex> so we can open outbound streams
    // from the proxy tasks while the main loop reads control messages.
    let mux = Arc::new(Mutex::new(mux));

    // Spawn heartbeat task.
    let heartbeat_handle = {
        let ctrl_write = ctrl_write.clone();
        let agent_id = agent_id.clone();
        tokio::spawn(async move {
            heartbeat_loop(agent_id, ctrl_write).await;
        })
    };

    // Run the control message loop; cleanup happens after it returns.
    let result = control_loop(
        &agent_id,
        &ctrl_read,
        &ctrl_write,
        &tunnel_mgr,
        &mux,
        &server_host,
    )
    .await;

    // --- Cleanup ---
    heartbeat_handle.abort();
    tunnel_mgr.unregister_agent(&agent_id).await;
    info!(agent_id = %agent_id, "agent handler exiting");

    result
}

/// Main control message loop — reads messages from the agent and processes them.
async fn control_loop<CR, CW>(
    agent_id: &str,
    ctrl_read: &Arc<Mutex<CR>>,
    ctrl_write: &Arc<Mutex<CW>>,
    tunnel_mgr: &TunnelManager,
    mux: &Arc<Mutex<MuxSession>>,
    server_host: &str,
) -> Result<()>
where
    CR: tokio::io::AsyncRead + Unpin + Send,
    CW: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    loop {
        let msg = {
            let mut reader = ctrl_read.lock().await;
            read_message(&mut *reader).await
        };

        match msg {
            Ok(Some(ControlMessage::TunnelReq {
                protocol,
                remote_port,
            })) => {
                handle_tunnel_request(
                    agent_id,
                    &protocol,
                    remote_port,
                    ctrl_write,
                    tunnel_mgr,
                    mux,
                    server_host,
                )
                .await?;
            }
            Ok(Some(ControlMessage::Heartbeat)) => {
                let mut writer = ctrl_write.lock().await;
                write_message(&mut *writer, &ControlMessage::HeartbeatAck)
                    .await
                    .context("failed to send heartbeat ack")?;
            }
            Ok(Some(ControlMessage::HeartbeatAck)) => {
                debug!(agent_id = %agent_id, "heartbeat ack received");
            }
            Ok(Some(other)) => {
                warn!(agent_id = %agent_id, msg = ?other, "unexpected control message");
            }
            Ok(None) => {
                info!(agent_id = %agent_id, "agent disconnected (control stream EOF)");
                return Ok(());
            }
            Err(e) => {
                error!(agent_id = %agent_id, error = %e, "control stream read error");
                return Err(e);
            }
        }
    }
}

/// Process a tunnel request from the agent.
async fn handle_tunnel_request<CW>(
    agent_id: &str,
    protocol: &str,
    remote_port: Option<u16>,
    ctrl_write: &Arc<Mutex<CW>>,
    tunnel_mgr: &TunnelManager,
    mux: &Arc<Mutex<MuxSession>>,
    server_host: &str,
) -> Result<()>
where
    CW: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    debug!(agent_id = %agent_id, protocol = %protocol, "tunnel request");

    match tunnel_mgr.register(agent_id, protocol, remote_port).await {
        Ok(registered) => {
            let tid = registered.tunnel_id.clone();
            let port = registered.remote_port;

            // Send success response.
            {
                let mut writer = ctrl_write.lock().await;
                write_message(
                    &mut *writer,
                    &ControlMessage::TunnelResp {
                        success: true,
                        tunnel_id: tid.clone(),
                        remote_port: port,
                        message: format!("tunnel {tid} on {server_host}:{port}"),
                    },
                )
                .await
                .context("failed to send tunnel response")?;
            }

            // Spawn proxy loop for this tunnel.
            let mux = mux.clone();
            tokio::spawn(proxy_tunnel_connections(
                tid,
                registered.conn_rx,
                move || {
                    let mux = mux.clone();
                    async move {
                        let mut session = mux.lock().await;
                        let stream = session.open_stream().await?;
                        Ok(stream.compat())
                    }
                },
            ));
        }
        Err(e) => {
            warn!(agent_id = %agent_id, error = %e, "tunnel registration failed");
            let mut writer = ctrl_write.lock().await;
            write_message(
                &mut *writer,
                &ControlMessage::TunnelResp {
                    success: false,
                    tunnel_id: String::new(),
                    remote_port: 0,
                    message: e.to_string(),
                },
            )
            .await
            .context("failed to send tunnel error response")?;
        }
    }

    Ok(())
}

/// Periodically send heartbeats to the agent.
async fn heartbeat_loop<CW: tokio::io::AsyncWrite + Unpin + Send>(
    agent_id: String,
    ctrl_write: Arc<Mutex<CW>>,
) {
    loop {
        tokio::time::sleep(HEARTBEAT_INTERVAL).await;

        let mut writer = ctrl_write.lock().await;
        if let Err(e) = write_message(&mut *writer, &ControlMessage::Heartbeat).await {
            warn!(agent_id = %agent_id, error = %e, "failed to send heartbeat, stopping");
            break;
        }
        debug!(agent_id = %agent_id, "heartbeat sent");
    }
}
