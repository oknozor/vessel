use crate::frame::ToBytes;
use crate::peer_message::messages::PeerMessage;
use crate::peer_message::HEADER_LEN;
use crate::SlskError;
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::time::Duration;

#[derive(Debug)]
pub struct PeerConnection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl PeerConnection {
    /// Try to read a soulseek message and timeout after 10ms if nothing happened, this method is used
    /// to avoid blocking when the stream buffer is empty and soulseek is not sending message anymore.
    pub async fn read_response_with_timeout(&mut self) -> crate::Result<Option<PeerMessage>> {
        match timeout(Duration::from_millis(10), self.read_response()).await {
            Ok(read_result) => read_result,
            Err(e) => Err(SlskError::TimeOut(e)),
        }
    }

    /// Attempt to read a message from the peer connection.
    /// First parse the message [`Header`], if there are at least as much bytes as the header content
    /// length, try to parse it, otherwise, try to read more bytes from the soulseek TcpStream buffer.
    /// **WARNING**  :
    /// If this function is called when the buffer is empty it will block trying to read the buffer,
    /// use [`read_response_with_timeout`] to avoid this
    ///
    /// [`Header`]: crate::peer_message::Header
    /// [`read_response_with_timeout`]: SlskConnection::read_response_with_timeout
    pub async fn read_response(&mut self) -> crate::Result<Option<PeerMessage>> {
        loop {
            if let Some(server_message) = self.parse_response()? {
                return Ok(Some(server_message));
            };
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err("connection reset by peer".into())
                };
            }
        }
    }

    /// Send a [`PeerMessage`] the soulseek server, using `[ToBytes]` to write to the buffer.
    pub async fn write_request(&mut self, message: PeerMessage) -> tokio::io::Result<()> {
        message.write_to_buf(&mut self.stream).await?;
        info!("request sent to soulseek : {}", message.kind());
        self.stream.flush().await
    }

    /// Advance the soulseek tcp connection buffer. The amount of byte consumed is `message_len`
    /// four bytes for the u32 message length prefix and four bytes for the u32 message code prefix.
    fn consume(&mut self, message_len: usize) {
        self.buffer.advance(HEADER_LEN as usize + message_len)
    }

    fn new(socket: TcpStream) -> PeerConnection {
        PeerConnection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    fn parse_response(&mut self) -> crate::Result<Option<PeerMessage>> {
        use crate::SlskError::Incomplete;
        let mut buf = Cursor::new(&self.buffer[..]);

        match PeerMessage::check(&mut buf) {
            Ok(header) => {
                let server_response = PeerMessage::parse(&mut buf, &header)?;
                // consume the message bytes
                self.consume(header.message_len);
                Ok(Some(server_response))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
