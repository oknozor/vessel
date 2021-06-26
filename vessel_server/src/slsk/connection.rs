use bytes::{Buf, BytesMut};
use socket2::{Domain, Protocol, Type};
use soulseek_protocol::frame::ToBytes;
use soulseek_protocol::server::request::ServerRequest;
use soulseek_protocol::server::response::ServerResponse;
use soulseek_protocol::server::HEADER_LEN;
use soulseek_protocol::SlskError;
use std::io::Cursor;
use std::net::ToSocketAddrs;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::{TcpSocket, TcpStream};

const DEFAULT_ADDRESS: &str = "server.slsknet.org:2242";

#[derive(Debug)]
pub struct SlskConnection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

/// Connect to the official Soulseek server at `server.slsknet.org:2242`.
pub async fn connect() -> SlskConnection {
    // Unfortunately tokio 1.0 does not allow to set KEEP_ALIVE.
    // We use socket2::Socket to get the keep alive option and convert back to a TcpStream
    let socket = socket2::Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();

    socket.set_keepalive(true).unwrap();
    socket.set_nonblocking(true).unwrap();

    let mut addr = DEFAULT_ADDRESS.to_socket_addrs().unwrap();
    let addr = addr.next().unwrap();
    let fd = socket.into_raw_fd();
    let socket: TcpSocket = unsafe { TcpSocket::from_raw_fd(fd) };
    let stream = socket.connect(addr).await.unwrap();

    info!("connected to Soulseek server");

    SlskConnection {
        stream: BufWriter::new(stream),
        buffer: BytesMut::with_capacity(4 * 1024),
    }
}

impl SlskConnection {
    /// Attempt to read a message from the soulseek server.
    /// First parse the message [`Header`], if there are at least as much bytes as the header content
    /// length, try to parse it, otherwise, try to read more bytes from the soulseek TcpStream buffer.
    /// **WARNING**  :
    /// [`Header`]: crate::server.messages::Header
    #[instrument(level = "trace")]
    pub async fn read_response(&mut self) -> soulseek_protocol::Result<Option<ServerResponse>> {
        loop {
            if let Some(message) = self.parse_response()? {
                return Ok(Some(message));
            };

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err("Soulseek connection reset by peer".into())
                };
            }
        }
    }

    /// Send a [`ServerRequest`] the soulseek server, using `[ToBytes]` to write to the buffer.
    #[instrument(level = "trace", skip(self))]
    pub async fn write_request(&mut self, request: &ServerRequest) -> tokio::io::Result<()> {
        request.write_to_buf(&mut self.stream).await?;
        info!("Request sent to Soulseek server : {:?}", request);
        self.stream.flush().await
    }

    /// Advance the soulseek tcp connection buffer. The amount of byte consumed is `message_len`
    /// four bytes for the u32 message length prefix and four bytes for the u32 message code prefix.
    fn consume(&mut self, message_len: usize) {
        self.buffer.advance(HEADER_LEN as usize + message_len)
    }

    #[instrument(level = "trace")]
    fn parse_response(&mut self) -> soulseek_protocol::Result<Option<ServerResponse>> {
        use soulseek_protocol::SlskError::Incomplete;
        let mut buf = Cursor::new(&self.buffer[..]);

        match ServerResponse::check(&mut buf) {
            Ok(header) => match ServerResponse::parse(&mut buf, &header) {
                Ok(server_response) => {
                    self.consume(header.message_len);
                    Ok(Some(server_response))
                }
                Err(e) => {
                    self.consume(header.message_len);
                    Err(SlskError::Other(Box::new(e)))
                }
            },
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl Drop for SlskConnection {
    fn drop(&mut self) {
        error!("Connection dropped");
    }
}
