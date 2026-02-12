//! Public TCP listener — accepts internet connections and proxies them
//! through yamux streams to the corresponding agent.

use anyhow::Result;
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

/// Accept public connections for a single tunnel and proxy each one through
/// a new yamux stream opened to the agent.
///
/// `S` is the stream type returned by the opener — typically a
/// `Compat<yamux::Stream>` that implements `tokio::io::AsyncRead + AsyncWrite`.
///
/// The `open_stream_fn` callback lets the caller inject how yamux streams are
/// opened (keeping this module decoupled from the yamux session details).
pub async fn proxy_tunnel_connections<F, Fut, S>(
    tunnel_id: String,
    mut conn_rx: tokio::sync::mpsc::Receiver<TcpStream>,
    open_stream_fn: F,
) where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<S>> + Send,
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    info!(tunnel_id = %tunnel_id, "tunnel proxy loop started");

    while let Some(mut client_stream) = conn_rx.recv().await {
        let peer_addr = client_stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".into());

        debug!(tunnel_id = %tunnel_id, client = %peer_addr, "opening yamux stream to agent");

        match (open_stream_fn)().await {
            Ok(mut yamux_stream) => {
                let tid = tunnel_id.clone();
                tokio::spawn(async move {
                    debug!(tunnel_id = %tid, client = %peer_addr, "proxying started");
                    match copy_bidirectional(&mut client_stream, &mut yamux_stream).await {
                        Ok((client_to_agent, agent_to_client)) => {
                            debug!(
                                tunnel_id = %tid,
                                client = %peer_addr,
                                client_to_agent,
                                agent_to_client,
                                "proxy session ended"
                            );
                        }
                        Err(e) => {
                            debug!(
                                tunnel_id = %tid,
                                client = %peer_addr,
                                error = %e,
                                "proxy session error"
                            );
                        }
                    }
                });
            }
            Err(e) => {
                warn!(
                    tunnel_id = %tunnel_id,
                    client = %peer_addr,
                    error = %e,
                    "failed to open yamux stream to agent, dropping connection"
                );
            }
        }
    }

    info!(tunnel_id = %tunnel_id, "tunnel proxy loop exiting (channel closed)");
}
