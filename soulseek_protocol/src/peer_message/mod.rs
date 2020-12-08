mod messages;

use crate::peer_message::MessageCode::{PeerInit, PierceFireWall, Unknown};

const HEADER_LEN: u32 = 5;

#[derive(Debug)]
pub struct Header {
    pub(crate) code: MessageCode,
    pub(crate) message_len: usize,
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
