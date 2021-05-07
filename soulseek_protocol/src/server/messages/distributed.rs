use crate::frame::ParseBytes;
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddedDistributedMessage {
    code: u8,
    message: Vec<u8>,
}

impl ParseBytes for EmbeddedDistributedMessage {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let code = src.get_u8();
        let message = src.chunk().to_vec();

        Ok(Self { code, message })
    }
}
