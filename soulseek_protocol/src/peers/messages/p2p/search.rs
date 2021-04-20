use std::{io::Cursor, usize};

use bytes::Buf;
use tokio::io::{AsyncWrite, BufWriter};

use crate::frame::{read_string, ParseBytes, ToBytes};
use crate::peers::messages::p2p::shared_directories::File;
use crate::peers::messages::p2p::zlib::decompress;

#[derive(Debug)]
pub struct SearchReply {
    pub username: String,
    pub ticket: u32,
    pub files: Vec<File>,
    pub slot_free: bool,
    pub average_speed: u32,
    pub queue_length: u64,
    pub locked_results: Vec<File>,
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
        let data = decompress(src)?;

        let src = &mut Cursor::new(data.as_slice());

        let username = read_string(src)?;
        let ticket = src.get_u32_le();

        let result_nth = src.get_u32_le();

        let mut files = Vec::with_capacity(result_nth as usize);

        for _ in 0..result_nth {
            let file = File::parse(src)?;
            files.push(file);
        }

        let slot_free = src.get_u8() == 1;
        let average_speed = src.get_u32_le();
        let queue_length = src.get_u64_le();

        let mut locked_results = vec![];
        // Sometimes this is not present in the search reply.
        // We should investigate this further
        if src.has_remaining() {
            let lock_result_nth = src.get_u32_le();

            for _ in 0..lock_result_nth {
                let file = File::parse(src)?;
                locked_results.push(file);
            }
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

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::frame::ParseBytes;

    use super::SearchReply;

    #[test]
    fn parse_search_reply() {
        let data: Vec<u8> = vec![
            120, 218, 197, 151, 205, 107, 212, 64, 20, 192, 39, 91, 176, 130, 222, 180, 23, 79,
            131, 224, 199, 178, 24, 54, 187, 110, 183, 139, 135, 54, 31, 174, 45, 116, 75, 113,
            219, 189, 52, 69, 166, 201, 36, 155, 110, 54, 19, 39, 147, 110, 187, 234, 93, 232, 127,
            225, 217, 155, 55, 5, 145, 122, 18, 17, 65, 207, 126, 156, 244, 230, 77, 40, 158, 156,
            36, 173, 154, 189, 121, 152, 236, 64, 194, 204, 203, 192, 239, 199, 204, 123, 15, 50,
            11, 0, 160, 100, 40, 135, 40, 60, 58, 254, 46, 95, 226, 75, 41, 224, 47, 179, 71, 252,
            120, 136, 35, 179, 119, 251, 174, 166, 110, 172, 116, 224, 178, 97, 118, 73, 236, 71,
            24, 15, 254, 76, 96, 183, 143, 40, 182, 205, 74, 165, 98, 118, 188, 136, 97, 10, 59,
            196, 38, 240, 42, 220, 116, 253, 3, 216, 65, 22, 212, 48, 15, 222, 72, 195, 65, 204,
            96, 141, 207, 55, 181, 117, 83, 153, 135, 107, 30, 221, 67, 1, 146, 135, 97, 253, 240,
            227, 50, 72, 198, 12, 127, 248, 178, 4, 178, 177, 36, 1, 112, 58, 151, 188, 255, 182,
            82, 249, 48, 79, 40, 28, 171, 249, 24, 89, 125, 120, 93, 105, 45, 180, 202, 124, 173,
            180, 90, 53, 184, 213, 94, 85, 245, 109, 88, 51, 171, 74, 182, 101, 36, 59, 62, 178,
            94, 189, 104, 74, 224, 159, 33, 197, 130, 233, 201, 185, 180, 125, 114, 96, 195, 141,
            62, 134, 26, 162, 59, 152, 166, 38, 227, 119, 95, 64, 206, 228, 190, 96, 147, 58, 15,
            169, 59, 132, 223, 149, 10, 239, 120, 212, 79, 45, 190, 61, 184, 156, 63, 143, 129, 96,
            139, 155, 60, 212, 181, 250, 132, 100, 252, 183, 215, 46, 228, 249, 129, 96, 126, 131,
            135, 86, 201, 30, 191, 138, 120, 60, 78, 21, 158, 220, 123, 152, 87, 32, 130, 21, 230,
            121, 104, 29, 133, 188, 124, 244, 152, 69, 169, 195, 203, 79, 135, 121, 135, 72, 176,
            67, 147, 135, 214, 176, 139, 152, 199, 143, 66, 167, 24, 135, 169, 199, 149, 139, 213,
            188, 199, 174, 96, 143, 133, 52, 29, 136, 227, 164, 248, 197, 199, 79, 139, 205, 134,
            86, 130, 31, 161, 16, 118, 48, 102, 169, 194, 155, 15, 70, 94, 129, 138, 85, 80, 170,
            73, 19, 165, 50, 111, 164, 113, 196, 248, 70, 156, 106, 212, 194, 48, 175, 225, 11,
            214, 72, 186, 100, 215, 115, 152, 23, 184, 89, 85, 252, 104, 151, 138, 172, 10, 37,
            105, 148, 154, 231, 66, 189, 143, 113, 148, 29, 194, 215, 237, 71, 133, 54, 39, 37,
            105, 145, 6, 25, 5, 39, 45, 250, 243, 214, 115, 80, 36, 95, 165, 108, 68, 232, 192,
            212, 144, 53, 144, 119, 67, 119, 201, 62, 151, 231, 123, 197, 240, 117, 35, 161, 55,
            203, 103, 64, 145, 41, 120, 74, 111, 83, 18, 176, 68, 96, 241, 217, 249, 188, 0, 22,
            43, 144, 125, 149, 173, 24, 191, 158, 5, 121, 178, 37, 150, 188, 18, 56, 68, 102, 251,
            236, 248, 236, 4, 87, 112, 209, 77, 238, 147, 29, 39, 60, 42, 77, 217, 193, 39, 238,
            175, 185, 41, 59, 12, 235, 241, 156, 52, 109, 7, 187, 209, 152, 153, 112, 112, 197, 58,
            160, 88, 55, 24, 182, 88, 146, 139, 189, 201, 59, 16, 92, 125, 14, 241, 109, 222, 119,
            121, 221, 223, 42, 23, 75, 230, 63, 38, 12, 185, 201, 157, 191, 159, 204, 253, 253,
            159, 127, 35, 191, 1, 213, 178, 180, 153,
        ];
        let mut cursor = Cursor::new(data.as_slice());

        let res = SearchReply::parse(&mut cursor);
        assert!(res.is_ok());
    }
}
