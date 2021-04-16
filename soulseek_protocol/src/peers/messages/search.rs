use bytes::Buf;
use tokio::io::{AsyncWrite , BufWriter};

use crate::frame::{ParseBytes, ToBytes, read_string};
use std::{io::Cursor, usize};

use super::{shared_directories::File, zlib::decompress};

#[derive(Debug)]
pub struct SearchReply {
    pub username: String,
    pub ticket: u32,
    pub files: Vec<File>,
    pub slot_free: bool,
    pub average_speed: u32,
    pub queue_length: u64,
    pub locked_results:Vec<File>
}

#[async_trait]
impl ToBytes for SearchReply {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        todo!()
    }
}


impl ParseBytes for SearchReply {
    type Output = Self;

    fn parse(src: &mut std::io::Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let data= decompress(src)?;
        let src = &mut Cursor::new(data.as_slice());
        let username = read_string(src)?;
        let ticket = src.get_u32_le();
        
        let result_nth = src.get_u32_le();
        let mut files = Vec::with_capacity(result_nth as usize);

        
        for _ in 0..result_nth {
            files.push(File::parse(src)?);
        }
        
        let slot_free = src.get_u8() == 1; 
        let average_speed = src.get_u32_le();
        let queue_length = src.get_u64_le();

        let lock_result_nth = src.get_u32_le();
        let mut locked_results = Vec::with_capacity(lock_result_nth as usize);

        
        for _ in 0..lock_result_nth {
            locked_results.push(File::parse(src)?);
        }

        Ok(Self {
            username,
            ticket,
            files,
            slot_free,
            average_speed,
            queue_length,
            locked_results,
        }) 
    }
}