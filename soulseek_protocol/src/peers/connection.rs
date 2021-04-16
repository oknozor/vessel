use std::io::Cursor;

use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use crate::frame::ToBytes;
use crate::message_common::ConnectionType;
use crate::peers::messages::connection::{PeerConnectionMessage, CONNECTION_MSG_HEADER_LEN};
use crate::peers::messages::{PeerRequestPacket, PeerResponsePacket};
use crate::peers::response::PeerResponse;
use std::net::{Ipv4Addr, SocketAddr};

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    pub(crate) connection_type: Option<ConnectionType>,
}

impl Connection {
    /// Attempt to read a message from the peer connection.
    /// First parse the message [`Header`], if there are at least as much bytes as the header content
    /// length, try to parse it, otherwise, try to read more bytes from the soulseek TcpStream buffer.
    /// **WARNING**  :
    /// If this function is called when the buffer is empty it will block trying to read the buffer,
    /// use [`read_response_with_timeout`] to avoid this
    ///
    /// [`Header`]: crate::peers::messages::Header
    /// [`read_response_with_timeout`]: SlskConnection::read_response_with_timeout
    pub async fn read_response(&mut self) -> crate::Result<PeerResponsePacket> {
        loop {
            match self.connection_type {
                Some(ConnectionType::PeerToPeer) => {
                    if let Some(message) = self.parse_peer_message()? {
                        return Ok(PeerResponsePacket::Message(message));
                    }
                }
                Some(ConnectionType::DistributedNetwork) => {
                    // TODO
                }
                Some(ConnectionType::FileTransfer) => {
                    // TODO
                }
                None => {
                    if let Some(message) = self.parse_connection_message()? {
                        return Ok(PeerResponsePacket::ConnectionMessage(message));
                    }
                }
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return if self.buffer.is_empty() {
                    Ok(PeerResponsePacket::None)
                } else {
                    Err(crate::SlskError::ConnectionResetByPeer)
                };
            }
        }
    }

    /// Send a [`PeerMessage`] the soulseek server, using `[ToBytes]` to write to the buffer.
    pub async fn write_request(&mut self, message: PeerRequestPacket) -> tokio::io::Result<()> {
        match message {
            PeerRequestPacket::Message(message) => {
                message.write_to_buf(&mut self.stream).await?;
                info!("Peer request sent to peer : {}", message.kind());
            }
            PeerRequestPacket::ConnectionMessage(message) => {
                message.write_to_buf(&mut self.stream).await?;
                info!("Connection request sent to peer : {}", message.kind());
            }
            _ => unreachable!(),
        }

        self.stream.flush().await
    }

    /// Advance the soulseek tcp connection buffer. The amount of byte consumed is `message_len`
    /// Parse the connection message with a 5 bytes prefix if we are trying to establish connection to a peer,
    /// otherwise consume the peer message plus the 8 bytes header prefix.
    fn consume(&mut self, message_len: usize) {
        match self.connection_type {
            Some(_) => self.buffer.advance(message_len),
            None => self.buffer.advance(message_len),
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
                buf.set_position(CONNECTION_MSG_HEADER_LEN as u64);
                let connection_message = PeerConnectionMessage::parse(&mut buf, &header)?;
                // consume the message bytes
                self.consume(header.message_len);
                Ok(Some(connection_message))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn parse_peer_message(&mut self) -> crate::Result<Option<PeerResponse>> {
        use crate::SlskError::Incomplete;
        let mut buf = Cursor::new(&self.buffer[..]);

        match PeerResponse::check(&mut buf) {
            Ok(header) => {
                let peer_message = PeerResponse::parse(&mut buf, &header)?;

                // consume the message bytes
                self.consume(header.message_len as usize);
                Ok(Some(peer_message))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_peer_address(&self) -> Ipv4Addr {
        match &self
            .stream
            .get_ref()
            .peer_addr()
            .expect("Unable to get peer address")
        {
            SocketAddr::V4(address) => address.ip().to_owned(),
            SocketAddr::V6(_) => {
                unreachable!()
            }
        }
    }
}
