//! Yamux multiplexer wrapper for creating and managing mux sessions.

use anyhow::{Context, Result};
use tokio::io::{AsyncRead, AsyncWrite};
use yamux::{Config, Connection, Mode, Stream};

/// A yamux multiplexed connection.
///
/// Wraps `yamux::Connection` to provide a simpler interface for opening and
/// accepting streams over a single underlying transport.
pub struct MuxSession<T> {
    connection: Connection<T>,
}

impl<T: AsyncRead + AsyncWrite + Unpin> MuxSession<T> {
    /// Create a new mux session over the given transport.
    ///
    /// `mode` determines whether this side acts as a client or server.
    pub fn new(io: T, mode: Mode) -> Self {
        let mut config = Config::default();
        config.set_split_send_size(16 * 1024);
        Self {
            connection: Connection::new(io, config, mode),
        }
    }

    /// Accept the next inbound stream opened by the remote peer.
    ///
    /// Returns `None` when the connection is closed.
    pub async fn accept_stream(&mut self) -> Result<Option<Stream>> {
        use futures_lite::StreamExt;
        match self.connection.next().await {
            Some(Ok(stream)) => Ok(Some(stream)),
            Some(Err(e)) => Err(e).context("yamux accept error"),
            None => Ok(None),
        }
    }

    /// Open a new outbound stream to the remote peer.
    pub async fn open_stream(&mut self) -> Result<Stream> {
        use yamux::ConnectionError;

        // yamux 0.13 uses poll-based stream opening via Connection::poll_new_outbound
        // We need to poll the connection to drive it and open streams.
        let mut control = self.connection.control();
        let stream = control
            .open_stream()
            .await
            .map_err(|e: ConnectionError| anyhow::anyhow!("yamux open stream error: {e}"))?;
        Ok(stream)
    }
}
