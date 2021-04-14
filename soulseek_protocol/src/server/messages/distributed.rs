use crate::frame::ParseBytes;
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddedDistributedMessage {
    code: u8,
    message: Vec<u8>,
}

impl ParseBytes for EmbeddedDistributedMessage {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let code = src.get_u8();
        let message = src.bytes().to_vec();

        Ok(Self { code, message })
    }
}
