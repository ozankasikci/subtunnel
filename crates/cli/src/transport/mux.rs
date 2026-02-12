//! Yamux multiplexer wrapper for creating and managing mux sessions.
//!
//! This module provides [`MuxSession`] — a high-level async interface over
//! yamux's poll-based [`Connection`](yamux::Connection). A background driver
//! task is spawned to continuously poll the connection, while the caller
//! accepts or opens streams through async methods.
//!
//! Yamux uses `futures::io` traits internally, so tokio streams must be
//! wrapped with [`tokio_util::compat`] before being passed in. The returned
//! yamux [`Stream`](yamux::Stream) objects likewise implement `futures::io`
//! traits and need the compat adapter for use with tokio's IO utilities.

use std::collections::VecDeque;
use std::future::poll_fn;
use std::task::Poll;

use anyhow::{Context, Result};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tracing::{debug, trace};
use yamux::{Config, Connection, Mode, Stream};

/// A yamux multiplexed session.
///
/// Wraps a [`yamux::Connection`] and spawns a background driver task that
/// continuously polls the connection. Inbound streams are forwarded via a
/// channel, and outbound streams can be requested via [`open_stream`](Self::open_stream).
///
/// When working with tokio streams, wrap them with
/// [`TokioAsyncReadCompatExt::compat()`] before passing to [`new`](Self::new).
pub struct MuxSession {
    incoming_rx: mpsc::Receiver<Stream>,
    open_tx: mpsc::UnboundedSender<oneshot::Sender<yamux::Result<Stream>>>,
    // Dropping the JoinHandle detaches the driver (it keeps running).
    // The driver exits when channels close and the connection terminates.
    _driver: JoinHandle<()>,
}

impl MuxSession {
    /// Create a new mux session over the given transport.
    ///
    /// `mode` determines whether this side acts as a yamux client or server.
    /// The client opens odd-numbered streams; the server opens even-numbered.
    ///
    /// A background task is spawned to drive the yamux connection.
    pub fn new<T>(io: T, mode: Mode) -> Self
    where
        T: futures::AsyncRead + futures::AsyncWrite + Unpin + Send + 'static,
    {
        let mut config = Config::default();
        config.set_split_send_size(16 * 1024);
        debug!(?mode, "mux session created");

        let conn = Connection::new(io, config, mode);
        let (incoming_tx, incoming_rx) = mpsc::channel(64);
        let (open_tx, open_rx) = mpsc::unbounded_channel();
        let driver = tokio::spawn(drive_connection(conn, incoming_tx, open_rx));

        Self {
            incoming_rx,
            open_tx,
            _driver: driver,
        }
    }

    /// Accept the next inbound stream opened by the remote peer.
    ///
    /// Returns `None` when the connection is closed (EOF).
    pub async fn accept_stream(&mut self) -> Result<Option<Stream>> {
        let stream = self.incoming_rx.recv().await;
        if stream.is_some() {
            trace!("accepted inbound stream");
        } else {
            debug!("mux connection closed (no more inbound streams)");
        }
        Ok(stream)
    }

    /// Open a new outbound stream to the remote peer.
    pub async fn open_stream(&mut self) -> Result<Stream> {
        let (tx, rx) = oneshot::channel();
        self.open_tx
            .send(tx)
            .map_err(|_| anyhow::anyhow!("mux driver closed"))?;
        rx.await
            .context("mux driver dropped")?
            .map_err(|e| anyhow::anyhow!("yamux open stream error: {e}"))
    }
}

/// Background driver that continuously polls the yamux connection.
///
/// Handles both inbound stream acceptance and outbound stream creation
/// requests, forwarding results through channels.
async fn drive_connection<T>(
    mut conn: Connection<T>,
    incoming_tx: mpsc::Sender<Stream>,
    mut open_rx: mpsc::UnboundedReceiver<oneshot::Sender<yamux::Result<Stream>>>,
) where
    T: futures::AsyncRead + futures::AsyncWrite + Unpin,
{
    let mut pending_opens: VecDeque<oneshot::Sender<yamux::Result<Stream>>> = VecDeque::new();
    let mut closing = false;

    loop {
        let result = poll_fn(|cx| {
            // If we're closing, just drive the close to completion.
            if closing {
                return match conn.poll_close(cx) {
                    Poll::Ready(_) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                };
            }

            // Collect new open requests
            while let Poll::Ready(msg) = open_rx.poll_recv(cx) {
                match msg {
                    Some(reply) => pending_opens.push_back(reply),
                    None => {
                        // MuxSession was dropped — gracefully close the connection.
                        // poll_close flushes pending stream data before closing.
                        closing = true;
                        return match conn.poll_close(cx) {
                            Poll::Ready(_) => Poll::Ready(None),
                            Poll::Pending => Poll::Pending,
                        };
                    }
                }
            }

            // Process pending opens
            while !pending_opens.is_empty() {
                match conn.poll_new_outbound(cx) {
                    Poll::Ready(Ok(stream)) => {
                        if let Some(reply) = pending_opens.pop_front() {
                            let _ = reply.send(Ok(stream));
                        }
                    }
                    Poll::Ready(Err(e)) => {
                        if let Some(reply) = pending_opens.pop_front() {
                            let _ = reply.send(Err(e));
                        }
                        return Poll::Ready(None);
                    }
                    Poll::Pending => break,
                }
            }

            // Drive connection and accept inbound streams
            conn.poll_next_inbound(cx)
        })
        .await;

        match result {
            Some(Ok(stream)) => {
                if incoming_tx.send(stream).await.is_err() {
                    debug!("inbound stream receiver dropped");
                    break;
                }
            }
            Some(Err(e)) => {
                debug!("mux connection error: {e}");
                break;
            }
            None => {
                debug!("mux connection closed");
                break;
            }
        }
    }
}

/// Create a client-side [`MuxSession`] from a tokio async stream.
///
/// Wraps the tokio stream with a compat adapter so yamux (which uses
/// `futures::io` traits) can drive it.
pub fn client_mux<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static>(
    io: T,
) -> MuxSession {
    MuxSession::new(io.compat(), Mode::Client)
}

/// Create a server-side [`MuxSession`] from a tokio async stream.
///
/// Wraps the tokio stream with a compat adapter so yamux (which uses
/// `futures::io` traits) can drive it.
pub fn server_mux<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static>(
    io: T,
) -> MuxSession {
    MuxSession::new(io.compat(), Mode::Server)
}

/// Extension trait providing `.compat()` on yamux streams for tokio interop.
///
/// Yamux streams implement `futures::io::AsyncRead/Write`. This re-exports
/// the compat adapter so callers can convert to tokio-compatible streams.
pub use tokio_util::compat::FuturesAsyncReadCompatExt as YamuxStreamCompatExt;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    #[tokio::test]
    async fn open_and_accept_stream() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut mux = server_mux(stream);
            let yamux_stream = mux.accept_stream().await.unwrap().unwrap();
            let mut compat = yamux_stream.compat();
            let mut buf = vec![0u8; 64];
            let n = AsyncReadExt::read(&mut compat, &mut buf).await.unwrap();
            assert_eq!(&buf[..n], b"hello mux");
        });

        let client = tokio::spawn(async move {
            let stream = TcpStream::connect(addr).await.unwrap();
            let mut mux = client_mux(stream);
            let yamux_stream = mux.open_stream().await.unwrap();
            let mut compat = yamux_stream.compat();
            AsyncWriteExt::write_all(&mut compat, b"hello mux")
                .await
                .unwrap();
            AsyncWriteExt::shutdown(&mut compat).await.unwrap();
        });

        let (s, c) = tokio::join!(server, client);
        s.unwrap();
        c.unwrap();
    }

    #[tokio::test]
    async fn multiple_streams() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut mux = server_mux(stream);

            for expected in [b"stream-0" as &[u8], b"stream-1", b"stream-2"] {
                let yamux_stream = mux.accept_stream().await.unwrap().unwrap();
                let mut compat = yamux_stream.compat();
                let mut buf = vec![0u8; 64];
                let n = AsyncReadExt::read(&mut compat, &mut buf).await.unwrap();
                assert_eq!(&buf[..n], expected);
            }
        });

        let client = tokio::spawn(async move {
            let stream = TcpStream::connect(addr).await.unwrap();
            let mut mux = client_mux(stream);

            for i in 0..3u8 {
                let yamux_stream = mux.open_stream().await.unwrap();
                let mut compat = yamux_stream.compat();
                let msg = format!("stream-{i}");
                AsyncWriteExt::write_all(&mut compat, msg.as_bytes())
                    .await
                    .unwrap();
                AsyncWriteExt::shutdown(&mut compat).await.unwrap();
            }
        });

        let (s, c) = tokio::join!(server, client);
        s.unwrap();
        c.unwrap();
    }
}
