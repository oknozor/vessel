use crate::server_message::request::ServerRequest;
use crate::server_message::response::ServerResponse;
use crate::server_message::{ToBytes, HEADER_LEN};
use crate::SlskError;
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::time::Duration;

pub const DEFAULT_ADDRESS: &str = "server.slsknet.org:2242";

#[derive(Debug)]
pub struct SlskConnection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

pub async fn connect() -> SlskConnection {
    // Main stream with the soulseek server
    let stream = TcpStream::connect(DEFAULT_ADDRESS)
        .await
        .expect("Unable to connect to slsk");
    info!("connected to soulseek server");
    SlskConnection::new(stream)
}

impl SlskConnection {
    pub fn new(socket: TcpStream) -> SlskConnection {
        SlskConnection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_response_with_timeout(&mut self) -> crate::Result<Option<ServerResponse>> {
        match timeout(Duration::from_millis(10), self.read_response()).await {
            Ok(read_result) => read_result,
            Err(e) => Err(SlskError::TimeOut(e)),
        }
    }
    pub async fn read_response(&mut self) -> crate::Result<Option<ServerResponse>> {
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

    fn parse_response(&mut self) -> crate::Result<Option<ServerResponse>> {
        use crate::SlskError::Incomplete;
        let mut buf = Cursor::new(&self.buffer[..]);

        match ServerResponse::check(&mut buf) {
            Ok(header) => {
                let server_response = ServerResponse::parse(&mut buf, &header)?;
                // consume the message bytes
                self.consume(header.message_len);
                Ok(Some(server_response))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn write_request(&mut self, request: ServerRequest) -> tokio::io::Result<()> {
        request.write_to_buf(&mut self.stream).await?;
        info!("request sent to soulseek : {}", request.kind());
        self.stream.flush().await
    }

    fn consume(&mut self, message_len: usize) {
        self.buffer.advance(HEADER_LEN as usize + message_len)
    }
}
