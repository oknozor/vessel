use crate::frame::ToBytes;
use crate::server::MessageCode;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

#[derive(Debug, Serialize, Deserialize)]
pub struct SharedFolderAndFiles {
    dirs: u32,
    files: u32,
}

#[async_trait]
impl ToBytes for SharedFolderAndFiles {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        buffer.write_u32_le(8).await?;
        buffer
            .write_u32_le(MessageCode::SharedFoldersAndFiles as u32)
            .await?;
        buffer.write_u32_le(self.dirs).await?;
        buffer.write_u32_le(self.files).await?;
        Ok(())
    }
}
