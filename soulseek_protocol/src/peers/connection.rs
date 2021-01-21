use crate::frame::ToBytes;
use crate::message_common::ConnectionType;
use crate::peers::messages::PEER_MSG_HEADER_LEN;
use crate::peers::messages::{
    PeerConnectionMessage, PeerMessage, PeerPacket, CONNECTION_MSG_HEADER_LEN,
};
use crate::SlskError;
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::time::Duration;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    pub(crate) connection_type: Option<ConnectionType>,
}

impl Connection {
    /// Try to read a soulseek message and timeout after 10ms if nothing happened, this method is used
    /// to avoid blocking when the stream buffer is empty and soulseek is not sending message anymore.
    pub async fn read_response_with_timeout(&mut self) -> crate::Result<PeerPacket> {
        match timeout(Duration::from_millis(100), self.read_response()).await {
            Ok(message) => message,
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
    /// [`Header`]: crate::peers::messages::Header
    /// [`read_response_with_timeout`]: SlskConnection::read_response_with_timeout
    pub async fn read_response(&mut self) -> crate::Result<PeerPacket> {
        loop {
            match self.connection_type {
                Some(_) => {
                    if let Some(message) = self.parse_peer_message()? {
                        return Ok(PeerPacket::Message(message));
                    }
                }
                None => {
                    if let Some(message) = self.parse_connection_message()? {
                        return Ok(PeerPacket::ConnectionMessage(message));
                    }
                }
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return if self.buffer.is_empty() {
                    Ok(PeerPacket::None)
                } else {
                    Err("connection reset by peer".into())
                };
            }
        }
    }

    /// Send a [`PeerMessage`] the soulseek server, using `[ToBytes]` to write to the buffer.
    pub async fn write_request(&mut self, message: PeerPacket) -> tokio::io::Result<()> {
        info!("writing request to peer connection");
        match message {
            PeerPacket::Message(message) => {
                message.write_to_buf(&mut self.stream).await?;
                info!("request sent to peer : {}", message.kind());
            },
            PeerPacket::ConnectionMessage(message) => {
                message.write_to_buf(&mut self.stream).await?;
                info!("request sent to peer : {}", message.kind());

            },
            _ => unreachable!()
        }

        self.stream.flush().await
    }

    /// Advance the soulseek tcp connection buffer. The amount of byte consumed is `message_len`
    /// Parse the connection message with a 5 bytes prefix if we are trying to establish connection to a peer,
    /// otherwise consume the peer message plus the 8 bytes header prefix.
    fn consume(&mut self, message_len: usize) {
        match self.connection_type {
            Some(_) => self
                .buffer
                .advance(CONNECTION_MSG_HEADER_LEN as usize + message_len),
            None => self
                .buffer
                .advance(PEER_MSG_HEADER_LEN as usize + message_len),
        }
    }

    pub(crate) fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
            connection_type: None,
        }
    }

    fn parse_connection_message(&mut self) -> crate::Result<Option<PeerConnectionMessage>> {
        use crate::SlskError::Incomplete;
        let mut buf = Cursor::new(&self.buffer[..]);

        match PeerConnectionMessage::check(&mut buf) {
            Ok(header) => {
                let connection_message = PeerConnectionMessage::parse(&mut buf, &header)?;
                // consume the message bytes
                self.consume(header.message_len);
                Ok(Some(connection_message))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn parse_peer_message(&mut self) -> crate::Result<Option<PeerMessage>> {
        use crate::SlskError::Incomplete;
        let mut buf = Cursor::new(&self.buffer[..]);

        match PeerMessage::check(&mut buf) {
            Ok(header) => {
                debug!("got peer message header");
                let peer_message = PeerMessage::parse(&mut buf, &header)?;
                // consume the message bytes
                self.consume(header.message_len as usize);
                Ok(Some(peer_message))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
