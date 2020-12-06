use crate::server_message::request::ServerRequest;
use crate::server_message::response::ServerResponse;
use crate::server_message::{ToBytes, HEADER_LEN};
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct SlskConnection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl SlskConnection {
    pub fn new(socket: TcpStream) -> SlskConnection {
        SlskConnection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_response(&mut self) -> crate::Result<Option<ServerResponse>> {
        loop {
            if let Some(server_message) = self.parse_response()? {
                return Ok(Some(server_message));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
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
        self.stream.flush().await
    }

    fn consume(&mut self, message_len: usize) {
        self.buffer.advance(HEADER_LEN as usize + message_len)
    }
}
