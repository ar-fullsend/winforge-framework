use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::{CoreError, CoreResult};

/// Read one length-prefixed frame from `reader`.
/// Wire format: 4-byte LE u32 length, then that many UTF-8 bytes.
pub async fn read_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> CoreResult<String> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len == 0 || len > 64 * 1024 * 1024 {
        return Err(CoreError::Ipc(format!("invalid frame length: {len}")));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    String::from_utf8(buf).map_err(|e| CoreError::Ipc(format!("frame not valid UTF-8: {e}")))
}

/// Write one length-prefixed frame to `writer`.
pub async fn write_frame<W: AsyncWriteExt + Unpin>(writer: &mut W, data: &str) -> CoreResult<()> {
    let bytes = data.as_bytes();
    let len = bytes.len() as u32;
    writer.write_all(&len.to_le_bytes()).await?;
    writer.write_all(bytes).await?;
    writer.flush().await?;
    Ok(())
}
