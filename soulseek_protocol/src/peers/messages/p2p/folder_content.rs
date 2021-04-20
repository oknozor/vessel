use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::frame::{read_string, ParseBytes, ToBytes};
use crate::peers::messages::p2p::MessageCode;
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug)]
pub struct FolderContentsRequest {
    files: Vec<String>,
}

impl ParseBytes for FolderContentsRequest {
    type Output = FolderContentsRequest;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let file_nth = src.get_u32_le();
        let mut folder_content_request = FolderContentsRequest { files: vec![] };

        for _ in 0..file_nth {
            folder_content_request.files.push(read_string(src)?);
        }

        Ok(folder_content_request)
    }
}

#[async_trait]
impl ToBytes for FolderContentsRequest {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        let mut files_size = 0;
        for file in &self.files {
            files_size += 4;
            files_size += file.bytes().len() as u32;
        }

        let length = 4 + self.files.len() as u32 + files_size;

        buffer.write_u32_le(length).await?;
        buffer
            .write_u32_le(MessageCode::FolderContentsRequest as u32)
            .await?;
        Ok(())
    }
}
