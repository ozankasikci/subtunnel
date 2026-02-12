//! Message framing: 4-byte big-endian length prefix + JSON payload.

use anyhow::{Context, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::messages::ControlMessage;

/// Maximum message size (1 MB) to prevent abuse.
const MAX_MESSAGE_SIZE: u32 = 1_048_576;

/// Write a control message to a stream with length-prefix framing.
pub async fn write_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    msg: &ControlMessage,
) -> Result<()> {
    let payload = serde_json::to_vec(msg).context("failed to serialize message")?;
    let len = payload.len() as u32;
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

/// Read a control message from a stream with length-prefix framing.
///
/// Returns `None` if the stream has closed cleanly (EOF on length read).
pub async fn read_message<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Option<ControlMessage>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e).context("failed to read length prefix"),
    }
    let len = u32::from_be_bytes(len_buf);
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

    #[tokio::test]
    async fn roundtrip_framing() {
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
    async fn eof_returns_none() {
        let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
        let result = read_message(&mut cursor).await.unwrap();
        assert!(result.is_none());
    }
}
