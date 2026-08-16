//! HTTP listener — sniffs Host header from incoming connections and routes
//! them to the correct tunnel via subdomain lookup.

use std::time::Duration;

use anyhow::{Context, Result};
use tokio::io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, info, warn};

use super::tunnel_mgr::TunnelManager;

const HTTP_NOT_FOUND: &[u8] =
    b"HTTP/1.1 404 Not Found\r\nContent-Length: 10\r\nConnection: close\r\n\r\nNot Found\n";
const HTTP_BAD_GATEWAY: &[u8] =
    b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 12\r\nConnection: close\r\n\r\nBad Gateway\n";

/// Timing controls for public HTTP connection handling.
#[derive(Debug, Clone, Copy)]
pub struct ListenerConfig {
    /// Maximum time allowed for the first request bytes to arrive.
    pub initial_read_timeout: Duration,
    /// Maximum time allowed to open one agent-side yamux stream.
    pub open_stream_timeout: Duration,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        Self {
            initial_read_timeout: Duration::from_secs(10),
            open_stream_timeout: Duration::from_secs(10),
        }
    }
}

/// Run the HTTP listener that routes connections by Host header subdomain.
pub async fn run_http_listener(
    http_port: u16,
    domain: String,
    extra_domains: Vec<String>,
    tunnel_mgr: TunnelManager,
) -> Result<()> {
    run_http_listener_with_config(
        http_port,
        domain,
        extra_domains,
        tunnel_mgr,
        ListenerConfig::default(),
    )
    .await
}

/// Run the HTTP listener with explicit connection timing.
pub async fn run_http_listener_with_config(
    http_port: u16,
    domain: String,
    extra_domains: Vec<String>,
    tunnel_mgr: TunnelManager,
    config: ListenerConfig,
) -> Result<()> {
    let all_domains: Vec<String> = std::iter::once(domain.clone())
        .chain(extra_domains)
        .collect();
    let listener = TcpListener::bind(("0.0.0.0", http_port))
        .await
        .with_context(|| format!("failed to bind HTTP port {http_port}"))?;

    info!(port = http_port, "HTTP listener started");

    serve_http_listener(listener, all_domains, tunnel_mgr, config).await
}

/// Serve public HTTP connections from an already-bound listener.
#[doc(hidden)]
pub async fn serve_http_listener(
    listener: TcpListener,
    all_domains: Vec<String>,
    tunnel_mgr: TunnelManager,
    config: ListenerConfig,
) -> Result<()> {
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                crate::transport::set_tcp_nodelay(&stream);

                let domains = all_domains.clone();
                let tunnel_mgr = tunnel_mgr.clone();
                tokio::spawn(async move {
                    if let Err(e) =
                        handle_http_connection(stream, &domains, &tunnel_mgr, config).await
                    {
                        debug!(peer = %addr, error = %e, "HTTP connection error");
                    }
                });
            }
            Err(e) => {
                error!(error = %e, "failed to accept HTTP connection");
            }
        }
    }
}

/// Read enough of the TCP stream to extract the Host header,
/// then route the full connection (including already-read bytes) to the tunnel.
async fn handle_http_connection(
    mut stream: TcpStream,
    domains: &[String],
    tunnel_mgr: &TunnelManager,
    config: ListenerConfig,
) -> Result<()> {
    let mut buf = vec![0u8; 8192];
    let n = match tokio::time::timeout(config.initial_read_timeout, stream.read(&mut buf)).await {
        Ok(read) => read.context("failed to read from client")?,
        Err(_) => {
            debug!(timeout = ?config.initial_read_timeout, "initial HTTP read timed out");
            return Ok(());
        }
    };
    if n == 0 {
        anyhow::bail!("empty connection");
    }
    let initial = buf[..n].to_vec();

    debug!(
        "raw request first 200 bytes: {:?}",
        String::from_utf8_lossy(&initial[..initial.len().min(200)])
    );
    // Log hex of first 50 bytes for debugging
    let hex: String = initial[..initial.len().min(50)]
        .iter()
        .map(|b| format!("{:02x} ", b))
        .collect();
    debug!("raw hex: {}", hex);

    let subdomain = match extract_subdomain(&initial, domains) {
        Ok(subdomain) => subdomain,
        Err(e) => {
            debug!(error = %e, "HTTP request did not match a tunnel domain");
            stream
                .write_all(HTTP_NOT_FOUND)
                .await
                .context("failed to write HTTP 404 response")?;
            return Ok(());
        }
    };

    debug!(subdomain = %subdomain, "routing connection");

    let Some(sender) = tunnel_mgr.connection_sender(&subdomain).await else {
        stream
            .write_all(HTTP_NOT_FOUND)
            .await
            .context("failed to write HTTP 404 response")?;
        return Ok(());
    };

    if let Err(send_error) = sender.send((stream, initial)).await {
        let (mut stream, _) = send_error.0;
        stream
            .write_all(HTTP_BAD_GATEWAY)
            .await
            .context("failed to write HTTP 502 response")?;
    }
    Ok(())
}

/// Extract subdomain from HTTP Host header, trying each domain.
fn extract_subdomain(data: &[u8], domains: &[String]) -> Result<String> {
    let header_end = data
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map_or(data.len(), |position| position + 4);
    let text = String::from_utf8_lossy(&data[..header_end]);

    let host = text
        .lines()
        .find_map(|line| {
            let lower = line.to_lowercase();
            if lower.starts_with("host:") {
                Some(line[5..].trim().to_string())
            } else {
                None
            }
        })
        .context("no Host header found")?;

    let host = host.split(':').next().unwrap_or(&host);

    for domain in domains {
        let suffix = format!(".{domain}");
        if host.ends_with(&suffix) {
            let subdomain = &host[..host.len() - suffix.len()];
            if !subdomain.is_empty() {
                return Ok(subdomain.to_string());
            }
        }
    }

    anyhow::bail!("host {host} does not match any configured domain: {domains:?}");
}

/// Accept connections from the tunnel manager and proxy them through yamux.
pub async fn proxy_tunnel_connections<F, Fut, S>(
    tunnel_id: String,
    conn_rx: tokio::sync::mpsc::Receiver<(TcpStream, Vec<u8>)>,
    open_stream_fn: F,
) where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<S>> + Send + 'static,
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    proxy_tunnel_connections_with_config(
        tunnel_id,
        conn_rx,
        open_stream_fn,
        ListenerConfig::default(),
    )
    .await;
}

/// Proxy tunnel connections with explicit stream-open timing.
#[doc(hidden)]
pub async fn proxy_tunnel_connections_with_config<F, Fut, S>(
    tunnel_id: String,
    conn_rx: tokio::sync::mpsc::Receiver<(TcpStream, Vec<u8>)>,
    open_stream_fn: F,
    config: ListenerConfig,
) where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<S>> + Send + 'static,
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    proxy_connections_inner(
        tunnel_id,
        conn_rx,
        open_stream_fn,
        config,
        |stream: &TcpStream| {
            stream
                .peer_addr()
                .map(|address| address.to_string())
                .unwrap_or_else(|_| "unknown".into())
        },
    )
    .await;
}

/// Proxy arbitrary async connections with explicit stream-open timing.
#[doc(hidden)]
pub async fn proxy_connections_with_config<F, Fut, S, C>(
    tunnel_id: String,
    conn_rx: tokio::sync::mpsc::Receiver<(C, Vec<u8>)>,
    open_stream_fn: F,
    config: ListenerConfig,
) where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<S>> + Send + 'static,
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    C: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    proxy_connections_inner(tunnel_id, conn_rx, open_stream_fn, config, |_| {
        "visitor".into()
    })
    .await;
}

async fn proxy_connections_inner<F, Fut, S, C, P>(
    tunnel_id: String,
    mut conn_rx: tokio::sync::mpsc::Receiver<(C, Vec<u8>)>,
    open_stream_fn: F,
    config: ListenerConfig,
    peer_label: P,
) where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<S>> + Send + 'static,
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    C: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    P: Fn(&C) -> String,
{
    info!(tunnel_id = %tunnel_id, "tunnel proxy loop started");
    let open_stream_fn = std::sync::Arc::new(open_stream_fn);

    while let Some((mut client_stream, preread)) = conn_rx.recv().await {
        let peer_addr = peer_label(&client_stream);

        debug!(tunnel_id = %tunnel_id, client = %peer_addr, "opening yamux stream to agent");

        let tid = tunnel_id.clone();
        let open_stream_fn = open_stream_fn.clone();
        tokio::spawn(async move {
            match tokio::time::timeout(config.open_stream_timeout, (open_stream_fn)()).await {
                Ok(Ok(mut yamux_stream)) => {
                    if !preread.is_empty() {
                        if let Err(e) = yamux_stream.write_all(&preread).await {
                            debug!(tunnel_id = %tid, error = %e, "failed to write preread");
                            return;
                        }
                    }
                    match copy_bidirectional(&mut client_stream, &mut yamux_stream).await {
                        Ok((up, down)) => {
                            debug!(tunnel_id = %tid, client = %peer_addr, up, down, "proxy ended");
                        }
                        Err(e) => {
                            debug!(tunnel_id = %tid, client = %peer_addr, error = %e, "proxy error");
                        }
                    }
                }
                Ok(Err(e)) => {
                    warn!(tunnel_id = %tid, client = %peer_addr, error = %e, "failed to open yamux stream");
                    let _ = client_stream.write_all(HTTP_BAD_GATEWAY).await;
                }
                Err(_) => {
                    warn!(
                        tunnel_id = %tid,
                        client = %peer_addr,
                        timeout = ?config.open_stream_timeout,
                        "timed out opening yamux stream"
                    );
                    let _ = client_stream.write_all(HTTP_BAD_GATEWAY).await;
                }
            }
        });
    }

    info!(tunnel_id = %tunnel_id, "tunnel proxy loop exiting");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn domains(d: &[&str]) -> Vec<String> {
        d.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn extract_subdomain_basic() {
        let req = b"GET / HTTP/1.1\r\nHost: abc123.tunnel.example.com\r\n\r\n";
        let sub = extract_subdomain(req, &domains(&["tunnel.example.com"])).unwrap();
        assert_eq!(sub, "abc123");
    }

    #[test]
    fn extract_subdomain_with_port() {
        let req = b"GET / HTTP/1.1\r\nHost: abc123.tunnel.example.com:443\r\n\r\n";
        let sub = extract_subdomain(req, &domains(&["tunnel.example.com"])).unwrap();
        assert_eq!(sub, "abc123");
    }

    #[test]
    fn extract_subdomain_multiple_domains() {
        let req = b"GET / HTTP/1.1\r\nHost: myapp.subtunnel.dev\r\n\r\n";
        let sub =
            extract_subdomain(req, &domains(&["tunnel.example.com", "subtunnel.dev"])).unwrap();
        assert_eq!(sub, "myapp");
    }

    #[test]
    fn extract_subdomain_wrong_domain() {
        let req = b"GET / HTTP/1.1\r\nHost: abc123.other.com\r\n\r\n";
        assert!(extract_subdomain(req, &domains(&["tunnel.example.com"])).is_err());
    }

    #[test]
    fn extract_subdomain_no_host() {
        let req = b"GET / HTTP/1.1\r\n\r\n";
        assert!(extract_subdomain(req, &domains(&["tunnel.example.com"])).is_err());
    }

    #[test]
    fn extract_subdomain_with_binary_body() {
        let req = b"POST / HTTP/1.1\r\nHost: sub.tunnel.example.com\r\nContent-Length: 3\r\n\r\n\x00\xff\x80";
        let subdomain = extract_subdomain(req, &domains(&["tunnel.example.com"])).unwrap();
        assert_eq!(subdomain, "sub");
    }

    #[test]
    fn host_in_body_is_ignored() {
        let req = b"POST / HTTP/1.1\r\nContent-Length: 32\r\n\r\nHost: evil.tunnel.example.com\r\n";
        assert!(extract_subdomain(req, &domains(&["tunnel.example.com"])).is_err());
    }
}
