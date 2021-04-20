use std::{io::Cursor};

use bytes::{Buf, BytesMut};
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    time::timeout,
};

use crate::message_common::ConnectionType;
use crate::peers::messages::connection::{PeerConnectionMessage, CONNECTION_MSG_HEADER_LEN};
use crate::peers::messages::p2p::response::PeerResponse;
use crate::peers::messages::{PeerRequestPacket, PeerResponsePacket};
use crate::{frame::ToBytes, SlskError};
use std::net::{Ipv4Addr, SocketAddr};

use crate::peers::messages::distributed::DistributedMessage;
use crate::peers::messages::p2p::PEER_MSG_HEADER_LEN;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    pub(crate) connection_type: Option<ConnectionType>,
}

impl Drop for Connection {
    fn drop(&mut self) {
        debug!("Dropping peer connection")
    }
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
    #[instrument(level = "debug", skip(self))]
    pub async fn read_response(&mut self) -> crate::Result<Option<PeerResponsePacket>> {
        if 0 == self.stream.read_buf(&mut self.buffer).await? {
            // The remote closed the connection. For this to be a clean
            // shutdown, there should be no data in the read buffer. If
            // there is, this means that the peer closed the socket while
            // sending a frame.
            if self.buffer.is_empty() {
                return Ok(None);
            } else {
                return Err(SlskError::ConnectionResetByPeer);
            }
        }

        // Read incoming messages according to the connection type
        match self.connection_type {
            Some(ConnectionType::PeerToPeer) => match self.parse_peer_message() {
                Ok(Some(message)) => Ok(Some(PeerResponsePacket::Message(message))),
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            },
            Some(ConnectionType::DistributedNetwork) => match self.parse_distributed_message() {
                Ok(Some(message)) => Ok(Some(PeerResponsePacket::DistributedMessage(message))),
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            },
            Some(ConnectionType::FileTransfer) => {
                todo!("FT message");
            }
            None => match self.parse_connection_message() {
                Ok(Some(message)) => Ok(Some(PeerResponsePacket::ConnectionMessage(message))),
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            },
        }
    }

    /// Send a [`PeerMessage`] the soulseek server, using `[ToBytes]` to write to the buffer.
    #[instrument(level = "debug", skip(self))]
    pub async fn write_request(&mut self, message: PeerRequestPacket) -> tokio::io::Result<()> {
        match message {
            PeerRequestPacket::Message(message) => {
                message.write_to_buf(&mut self.stream).await?;
                info!("Peer request sent to peer");
            }
            PeerRequestPacket::ConnectionMessage(message) => {
                message.write_to_buf(&mut self.stream).await?;
                info!("Connection request sent to peer");
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
            Some(_) => self
                .buffer
                .advance(message_len + PEER_MSG_HEADER_LEN as usize),
            None => self
                .buffer
                .advance(message_len + CONNECTION_MSG_HEADER_LEN as usize),
        }
    }

    pub(crate) fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
            connection_type: None,
        }
    }

    #[instrument(level = "debug", skip(self))]
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

    #[instrument(level = "debug", skip(self))]
    fn parse_peer_message(&mut self) -> crate::Result<Option<PeerResponse>> {
        use crate::SlskError::Incomplete;
        let mut buf = Cursor::new(&self.buffer[..]);
        match PeerResponse::check(&mut buf) {
            Ok(header) => {
                let peer_message = PeerResponse::parse(&mut buf, &header)?;
                // consume the message bytes
                self.consume(header.message_len);
                Ok(Some(peer_message))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }

    #[instrument(level = "debug", skip(self))]
    fn parse_distributed_message(&mut self) -> crate::Result<Option<DistributedMessage>> {
        use crate::SlskError::Incomplete;
        let mut buf = Cursor::new(&self.buffer[..]);
        match DistributedMessage::check(&mut buf) {
            Ok(header) => {
                let distributed_message = DistributedMessage::parse(&mut buf, &header)?;
                // consume the message bytes
                self.consume(header.message_len);
                Ok(Some(distributed_message))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_peer_address(&self) -> Result<Ipv4Addr, std::io::Error> {
        match &self.stream.get_ref().peer_addr() {
            Ok(SocketAddr::V4(address)) => Ok(address.ip().to_owned()),
            Ok(SocketAddr::V6(_)) => {
                unreachable!()
            }
            Err(err) => Err(std::io::Error::from(err.kind())),
        }
    }
}
