use crate::frame::{read_string, ParseBytes};
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct SayInChat {
    pub room: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub room: String,
    pub username: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateMessage {
    id: u32,
    timestamp: u32,
    username: String,
    message: String,
    is_new: bool,
}

impl ParseBytes for PrivateMessage {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let id = src.get_u32_le();
        let timestamp = src.get_u32_le();
        let username = read_string(src)?;
        let message = read_string(src)?;
        let is_new = src.get_u8() == 1;

        Ok(Self {
            id,
            timestamp,
            username,
            message,
            is_new,
        })
    }
}
