//! Transport layer — multiplexing and TLS.
//!
//! Provides yamux-based stream multiplexing ([`mux`]) and TLS connection
//! setup helpers ([`tls`]).

pub mod mux;
pub mod tls;

use std::time::Duration;

use anyhow::{Context, Result};
use socket2::{SockRef, TcpKeepalive};
use tokio::net::TcpStream;
use tracing::warn;

pub(crate) const TCP_KEEPALIVE_IDLE: Duration = Duration::from_secs(30);

pub(crate) fn set_tcp_keepalive(stream: &TcpStream) -> Result<()> {
    let keepalive = TcpKeepalive::new().with_time(TCP_KEEPALIVE_IDLE);
    SockRef::from(stream)
        .set_tcp_keepalive(&keepalive)
        .context("failed to configure TCP keepalive")
}

/// Disable Nagle's algorithm so small writes leave immediately.
///
/// Tunneled traffic is mostly small interactive frames, and every leg that
/// buffers them adds up to hundreds of milliseconds of latency. Best effort:
/// a failure is logged and the connection proceeds without the option.
pub(crate) fn set_tcp_nodelay(stream: &TcpStream) {
    if let Err(e) = stream.set_nodelay(true) {
        warn!(error = %e, "failed to set TCP_NODELAY");
    }
}

pub use mux::{client_mux, server_mux, MuxSession, YamuxStreamCompatExt};
pub use tls::{
    client_config_with_cert, generate_self_signed_cert, server_config, tls_accept, tls_connect,
    SelfSignedCert,
};
