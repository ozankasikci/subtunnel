//! Message framing: 4-byte big-endian length prefix + JSON payload.
//!
//! Provides both standalone helper functions ([`encode_message`] /
//! [`decode_message`]) for working with raw bytes, and async
//! [`read_message`] / [`write_message`] helpers that operate on any
//! `AsyncRead + AsyncWrite` stream.

use anyhow::{Context, Result};
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::{instrument, trace};

use super::messages::ControlMessage;

/// Maximum message size (1 MB) to prevent abuse.
const MAX_MESSAGE_SIZE: u32 = 1_048_576;

/// Encode a [`ControlMessage`] into a length-prefixed byte buffer.
///
/// The format is `[4-byte BE length][JSON payload]`.
pub fn encode_message(msg: &ControlMessage) -> Result<Vec<u8>> {
    let payload = serde_json::to_vec(msg).context("failed to serialize message")?;
    let len = payload.len() as u32;
    let mut buf = Vec::with_capacity(4 + payload.len());
    buf.put_u32(len);
    buf.extend_from_slice(&payload);
    Ok(buf)
}

/// Decode a [`ControlMessage`] from a length-prefixed byte buffer.
///
/// Expects `buf` to contain exactly one framed message (length prefix +
/// payload). Returns the decoded message and the number of bytes consumed.
/// Returns `None` if the buffer does not yet contain a complete message.
pub fn decode_message(buf: &mut BytesMut) -> Result<Option<ControlMessage>> {
    if buf.len() < 4 {
        return Ok(None);
    }
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if len as u32 > MAX_MESSAGE_SIZE {
        anyhow::bail!("message too large: {len} bytes (max {MAX_MESSAGE_SIZE})");
    }
    if buf.len() < 4 + len {
        return Ok(None);
    }
    buf.advance(4);
    let payload = buf.split_to(len);
    let msg = serde_json::from_slice(&payload).context("failed to deserialize message")?;
    Ok(Some(msg))
}

/// Write a control message to a stream with length-prefix framing.
#[instrument(skip_all, fields(msg_type = ?std::mem::discriminant(msg)))]
pub async fn write_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    msg: &ControlMessage,
) -> Result<()> {
    let payload = serde_json::to_vec(msg).context("failed to serialize message")?;
    let len = payload.len() as u32;
    trace!(len, "writing control message");
    writer
        .write_all(&len.to_be_bytes())
        .await
        .context("failed to write length prefix")?;
    writer
        .write_all(&payload)
        .await
        .context("failed to write payload")?;
    writer.flush().await.context("failed to flush")?;
    Ok(())
}

/// Write a control message, failing if the complete framed write takes too long.
pub async fn write_message_with_timeout<W: AsyncWrite + Unpin>(
    writer: &mut W,
    msg: &ControlMessage,
    write_timeout: std::time::Duration,
) -> Result<()> {
    tokio::time::timeout(write_timeout, write_message(writer, msg))
        .await
        .with_context(|| format!("control write timed out after {write_timeout:?}"))??;
    Ok(())
}

/// Read a control message from a stream with length-prefix framing.
///
/// Returns `None` if the stream has closed cleanly (EOF on length read).
#[instrument(skip_all)]
pub async fn read_message<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Option<ControlMessage>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e).context("failed to read length prefix"),
    }
    let len = u32::from_be_bytes(len_buf);
    trace!(len, "reading control message");
    if len > MAX_MESSAGE_SIZE {
        anyhow::bail!("message too large: {len} bytes (max {MAX_MESSAGE_SIZE})");
    }
    let mut payload = vec![0u8; len as usize];
    reader
        .read_exact(&mut payload)
        .await
        .context("failed to read payload")?;
    let msg = serde_json::from_slice(&payload).context("failed to deserialize message")?;
    Ok(Some(msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let msg = ControlMessage::Auth {
            token: "hello".into(),
        };
        let encoded = encode_message(&msg).unwrap();
        let mut buf = BytesMut::from(&encoded[..]);
        let decoded = decode_message(&mut buf).unwrap().unwrap();
        assert_eq!(msg, decoded);
        assert!(buf.is_empty());
    }

    #[test]
    fn decode_incomplete_returns_none() {
        let mut buf = BytesMut::from(&[0u8, 0, 0, 10][..]);
        // Only 4 bytes of header, payload missing
        assert!(decode_message(&mut buf).unwrap().is_none());
    }

    #[test]
    fn decode_rejects_oversized() {
        let len = MAX_MESSAGE_SIZE + 1;
        let mut buf = BytesMut::new();
        buf.put_u32(len);
        buf.extend_from_slice(&[0u8; 8]);
        assert!(decode_message(&mut buf).is_err());
    }

    #[tokio::test]
    async fn async_roundtrip_framing() {
        let msg = ControlMessage::Auth {
            token: "test".into(),
        };
        let mut buf = Vec::new();
        write_message(&mut buf, &msg).await.unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let decoded = read_message(&mut cursor).await.unwrap().unwrap();
        assert_eq!(msg, decoded);
    }

    #[tokio::test]
    async fn async_eof_returns_none() {
        let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
        let result = read_message(&mut cursor).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn async_multiple_messages() {
        let msgs = vec![
            ControlMessage::Heartbeat,
            ControlMessage::HeartbeatAck,
            ControlMessage::Auth {
                token: "abc".into(),
            },
        ];
        let mut buf = Vec::new();
        for m in &msgs {
            write_message(&mut buf, m).await.unwrap();
        }

        let mut cursor = std::io::Cursor::new(buf);
        for expected in &msgs {
            let decoded = read_message(&mut cursor).await.unwrap().unwrap();
            assert_eq!(expected, &decoded);
        }
        // Next read should be EOF
        assert!(read_message(&mut cursor).await.unwrap().is_none());
    }
}
