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

pub(crate) const TCP_KEEPALIVE_IDLE: Duration = Duration::from_secs(30);

pub(crate) fn set_tcp_keepalive(stream: &TcpStream) -> Result<()> {
    let keepalive = TcpKeepalive::new().with_time(TCP_KEEPALIVE_IDLE);
    SockRef::from(stream)
        .set_tcp_keepalive(&keepalive)
        .context("failed to configure TCP keepalive")
}

pub use mux::{client_mux, server_mux, MuxSession, YamuxStreamCompatExt};
pub use tls::{
    client_config_with_cert, generate_self_signed_cert, server_config, tls_accept, tls_connect,
    SelfSignedCert,
};
