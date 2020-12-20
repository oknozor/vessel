pub mod messages;

use crate::peer_message::MessageCode::{PeerInit, PierceFireWall, Unknown};
use bytes::Buf;
use std::io::Cursor;

pub(crate) const HEADER_LEN: u32 = 5;

#[derive(Debug)]
pub struct Header {
    pub(crate) code: MessageCode,
    pub(crate) message_len: usize,
}

impl Header {
    pub fn read(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let message_length = src.get_u32_le();
        let code = src.get_u8();
        let code = MessageCode::from(code);

        // We can subtract message code from the length since we already know it
        let message_len = message_length as usize - 1;

        Ok(Self { message_len, code })
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum MessageCode {
    PierceFireWall = 0,
    PeerInit = 1,
    Unknown,
}

impl From<u8> for MessageCode {
    fn from(code: u8) -> Self {
        match code {
            0 => PierceFireWall,
            1 => PeerInit,
            _ => Unknown,
        }
    }
}
