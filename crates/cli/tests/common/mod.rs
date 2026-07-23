#![allow(dead_code)]

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use std::time::Duration;

use anyhow::{Context, Result};
use subtunnel::protocol::codec::{read_message, write_message};
use subtunnel::protocol::ControlMessage;
use subtunnel::transport::mux::{MuxSession, YamuxStreamCompatExt};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::compat::TokioAsyncReadCompatExt;

pub struct FakeAgent {
    pub mux: MuxSession,
    pub control: tokio_util::compat::Compat<yamux::Stream>,
}

pub struct SlowStream<S> {
    inner: S,
    inter_byte_delay: Duration,
    delay: Option<Pin<Box<tokio::time::Sleep>>>,
}

pub struct ReadGate<S> {
    inner: S,
    enabled: Arc<AtomicBool>,
}

impl<S> ReadGate<S> {
    pub fn new(inner: S) -> (Self, Arc<AtomicBool>) {
        let enabled = Arc::new(AtomicBool::new(true));
        (
            Self {
                inner,
                enabled: enabled.clone(),
            },
            enabled,
        )
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for ReadGate<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if !self.enabled.load(Ordering::SeqCst) {
            return Poll::Pending;
        }
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for ReadGate<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

impl<S> SlowStream<S> {
    pub fn new(inner: S, inter_byte_delay: Duration) -> Self {
        Self {
            inner,
            inter_byte_delay,
            delay: None,
        }
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for SlowStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }

        if self.delay.is_none() {
            self.delay = Some(Box::pin(tokio::time::sleep(self.inter_byte_delay)));
        }
        if self.delay.as_mut().unwrap().as_mut().poll(cx).is_pending() {
            return Poll::Pending;
        }
        self.delay = None;

        let mut byte = [0u8; 1];
        let mut one_byte = ReadBuf::new(&mut byte);
        match Pin::new(&mut self.inner).poll_read(cx, &mut one_byte) {
            Poll::Ready(Ok(())) => {
                buf.put_slice(one_byte.filled());
                Poll::Ready(Ok(()))
            }
            other => other,
        }
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for SlowStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

impl FakeAgent {
    pub async fn connect<T>(io: T, subdomain: &str) -> Result<Self>
    where
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let mut mux = MuxSession::new(io.compat(), yamux::Mode::Client);
        let mut control = mux.open_stream().await?.compat();

        write_message(
            &mut control,
            &ControlMessage::Auth {
                token: "test-token".into(),
            },
        )
        .await?;
        match read_message(&mut control)
            .await?
            .context("server closed during authentication")?
        {
            ControlMessage::AuthResp { success: true, .. } => {}
            other => anyhow::bail!("unexpected auth response: {other:?}"),
        }

        write_message(
            &mut control,
            &ControlMessage::RegisterReq {
                protocol: "tcp".into(),
                subdomain: Some(subdomain.into()),
            },
        )
        .await?;
        loop {
            match read_message(&mut control)
                .await?
                .context("server closed during tunnel registration")?
            {
                ControlMessage::RegisterResp { success: true, .. } => break,
                ControlMessage::Heartbeat => {
                    write_message(&mut control, &ControlMessage::HeartbeatAck).await?;
                }
                other => anyhow::bail!("unexpected tunnel response: {other:?}"),
            }
        }

        Ok(Self { mux, control })
    }
}
