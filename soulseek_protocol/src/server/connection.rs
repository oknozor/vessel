use crate::frame::ToBytes;
use crate::server::messages::request::ServerRequest;
use crate::server::messages::response::ServerResponse;
use crate::server::messages::HEADER_LEN;
use crate::SlskError;
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

const DEFAULT_ADDRESS: &str = "server.slsknet.org:2242";

#[derive(Debug)]
pub struct SlskConnection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

/// Connect to the official soulseek server at `server.slsknet.org:2242`.
pub async fn connect() -> SlskConnection {
    // Main stream with the soulseek server
    let stream = TcpStream::connect(DEFAULT_ADDRESS)
        .await
        .expect("Unable to connect to slsk");
    info!("connected to soulseek server");
    SlskConnection::new(stream)
}

impl SlskConnection {
    /// Attempt to read a message from the soulseek server.
    /// First parse the message [`Header`], if there are at least as much bytes as the header content
    /// length, try to parse it, otherwise, try to read more bytes from the soulseek TcpStream buffer.
    /// **WARNING**  :
    /// [`Header`]: crate::server.messages::Header
    pub async fn read_response(&mut self) -> crate::Result<Option<ServerResponse>> {
        loop {
            if let Some(message) = self.parse_response()? {
                return Ok(Some(message));
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

    /// Send a [`ServerRequest`] the soulseek server, using `[ToBytes]` to write to the buffer.
    #[instrument(level = "debug", skip(self))]
    pub async fn write_request(&mut self, request: &ServerRequest) -> tokio::io::Result<()> {
        request.write_to_buf(&mut self.stream).await?;
        info!("Request sent to Soulseek server : {}", request.kind());
        self.stream.flush().await
    }

    /// Advance the soulseek tcp connection buffer. The amount of byte consumed is `message_len`
    /// four bytes for the u32 message length prefix and four bytes for the u32 message code prefix.
    fn consume(&mut self, message_len: usize) {
        self.buffer.advance(HEADER_LEN as usize + message_len)
    }

    fn new(socket: TcpStream) -> SlskConnection {
        SlskConnection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    fn parse_response(&mut self) -> crate::Result<Option<ServerResponse>> {
        use crate::SlskError::Incomplete;
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
