use bytes::Buf;
use std::io::{Cursor, Read};
use std::net::Ipv4Addr;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

/// A utility trait to parse incoming message according to soulseek protocol message definition
///
/// **NOTE : ** Since message headers are different depending on the message family, implementor of
/// this trait shall not care about message headers, these are meant to be handled in a top level
/// structure, typically an enum matching against a pre parsed message code and checking the buffer
/// length against the message length header.
/// For instance [`ServerResponse`] header length is 8 bytes while [`PeerMessage`]'s header is 5.
///
/// [`ServerResponse`]: crate::server::response::ServerResponse
/// [`PeerMessage`]: crate::peers::messages::PeerMessage
pub(crate) trait ParseBytes<Output = Self> {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Output>;
}

/// A utility trait to write soulseek server messages, peer messages and distributed messages
/// to a TCP stream buffer.
#[async_trait]
pub trait ToBytes {
    /// Write the request to the underlying buffer via [`BufWriter`].
    ///
    /// ## Example :
    /// ```
    /// use soulseek_protocol::server::messages::login::LoginRequest;
    /// use tokio::io::BufWriter;
    ///
    /// let request = LoginRequest::new("username", "password");
    /// let mut buff = BufWriter::new(&mut &[0u8, 1024]);
    ///
    /// request.write_to_buf(&mut buff).await.expect("Failed to write to buffer");
    /// ```
    ///
    /// [`BufWriter`]: tokio::io::BufWriter
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()>;
}

pub(crate) async fn write_string(
    src: &str,
    buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
) -> tokio::io::Result<()> {
    let bytes = src.as_bytes();
    buffer.write_u32_le(bytes.len() as u32).await?;
    buffer.write(bytes).await?;
    Ok(())
}

pub(crate) fn read_string(src: &mut Cursor<&[u8]>) -> std::io::Result<String> {
    let string_len = src.get_u32_le();
    if string_len > 0 {
        let mut string = vec![0u8; string_len as usize];
        src.read_exact(&mut string)?;
        Ok(String::from_utf8_lossy(&string).to_string())
    } else {
        Ok("".to_string())
    }
}

pub(crate) fn read_bool(src: &mut Cursor<&[u8]>) -> bool {
    src.get_u8() == 1
}

pub(crate) fn read_ipv4(src: &mut Cursor<&[u8]>) -> Ipv4Addr {
    let ip = src.get_u32_le();
    Ipv4Addr::from(ip)
}

pub(crate) const STR_LENGTH_PREFIX: u32 = 4;
