use crate::frame::{read_string, ParseBytes};
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug)]
pub struct SearchRequest {
    pub unknown: u32,
    pub username: String,
    pub ticket: u32,
    pub query: String,
}

impl ParseBytes for SearchRequest {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let unknown = src.get_u32_le();
        let username = read_string(src)?;
        let ticket = src.get_u32_le();
        let query = read_string(src)?;

        Ok(Self {
            unknown,
            username,
            ticket,
            query,
        })
    }
}
