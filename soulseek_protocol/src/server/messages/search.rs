use crate::frame::{read_string, ParseBytes};
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    ticket: u32,
    query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    username: String,
    ticket: u32,
    query: String,
}

impl ParseBytes for SearchQuery {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let username = read_string(src)?;
        let ticket = src.get_u32_le();
        let query = read_string(src)?;

        Ok(Self {
            username,
            ticket,
            query,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomSearchQuery {
    room: String,
    ticket: u32,
    query: String,
}
