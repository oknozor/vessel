use std::io::Cursor;
use tokio::io::{AsyncWrite, BufWriter};

/// A utility trait to parse incoming message according to soulseek protocol message definition
/// **NOTE : ** Since message headers are different depending on the message family, implementor of
/// this trait shall not care about message headers, these are meant to be handled in a top level
/// structure, typically an enum matching against a pre parsed message code and checking the buffer
/// length against the message length header.
/// For instance [`ServerResponse`] header length is 8 bytes while [`PeerMessage`]'s header is 4.
///
/// [`ServerResponse`]: crate::server_message::response::ServerResponse
/// [`PeerMessage`]: crate::peer_message::message::PeerMessage
pub trait ParseBytes {
    type Output;
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output>;
}

/// A utility trait to write soulseek server messages, peer messages and distributed messages
/// to a TCP stream buffer.
#[async_trait]
pub trait ToBytes {
    /// Write the request to the underlying buffer via [`BufWriter`].
    ///
    /// ## Example :
    /// ```
    /// use soulseek_protocol::server_message::login::LoginRequest;
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
