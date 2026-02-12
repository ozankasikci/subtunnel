//! Transport layer — multiplexing and TLS.
//!
//! Provides yamux-based stream multiplexing ([`mux`]) and TLS connection
//! setup helpers ([`tls`]).

pub mod mux;
pub mod tls;

pub use mux::{client_mux, server_mux, MuxSession, YamuxStreamCompatExt};
pub use tls::{
    client_config_with_cert, generate_self_signed_cert, server_config, tls_accept, tls_connect,
    SelfSignedCert,
};
