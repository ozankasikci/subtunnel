//! Local service forwarder — accepts yamux streams and proxies to a local service.

use anyhow::{Context, Result};
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use tracing::{debug, error, info, warn};

use crate::transport::mux::{MuxSession, YamuxStreamCompatExt};

/// Accept inbound yamux streams from the server and proxy each to the local service.
///
/// This drives the yamux `MuxSession`, accepting new streams opened by the server
/// (one per incoming client connection). Each stream is forwarded to
/// `local_addr` (e.g. `localhost:8080`) via bidirectional copy.
///
/// Returns when the yamux connection closes or the shutdown signal fires.
pub async fn run_proxy(
    mut mux: MuxSession,
    local_addr: &str,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> Result<()> {
    info!(local_addr, "proxy loop started, waiting for streams");

    loop {
        let stream = tokio::select! {
            result = mux.accept_stream() => {
                match result {
                    Ok(Some(stream)) => stream,
                    Ok(None) => {
                        info!("yamux connection closed by server");
                        return Ok(());
                    }
                    Err(e) => {
                        error!("yamux error: {e}");
                        return Err(e).context("yamux connection error");
                    }
                }
            }
            _ = shutdown_wait(&mut shutdown) => {
                info!("shutdown signal received, stopping proxy");
                return Ok(());
            }
        };

        let local_addr = local_addr.to_string();
        tokio::spawn(async move {
            if let Err(e) = proxy_stream(stream, &local_addr).await {
                debug!("stream proxy ended: {e:#}");
            }
        });
    }
}

/// Proxy a single yamux stream to the local service.
async fn proxy_stream(remote: yamux::Stream, local_addr: &str) -> Result<()> {
    debug!("new stream, connecting to {local_addr}");

    let mut local = match TcpStream::connect(local_addr).await {
        Ok(s) => s,
        Err(e) => {
            warn!("failed to connect to local service {local_addr}: {e}");
            return Err(e).context("local connect failed");
        }
    };

    // Bridge yamux stream (futures::io) to tokio via compat adapter
    let mut remote = remote.compat();

    let (up, down) = copy_bidirectional(&mut remote, &mut local)
        .await
        .context("bidirectional copy error")?;

    debug!("stream closed ({up} bytes up, {down} bytes down)");
    Ok(())
}

/// Wait until the shutdown signal fires.
async fn shutdown_wait(rx: &mut tokio::sync::watch::Receiver<bool>) {
    while !*rx.borrow() {
        if rx.changed().await.is_err() {
            return;
        }
    }
}
