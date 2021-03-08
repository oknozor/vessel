use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::frame::ToBytes;
use crate::peers::messages::MessageCode;

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

#[derive(Debug)]
pub struct FolderContentsRequest {
    files: Vec<String>,
}
