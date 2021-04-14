use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::frame::{write_string, ToBytes};
use crate::peers::messages::shared_directories::Attribute;
use crate::peers::messages::MessageCode;

#[derive(Debug)]
pub struct SearchRequest {
    pub ticket: u32,
    pub query: String,
}

#[async_trait]
impl ToBytes for SearchRequest {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        let length = 4 + 4 + self.query.bytes().len() as u32;
        buffer.write_u32_le(length).await?;
        buffer
            .write_u32_le(MessageCode::SearchRequest as u32)
            .await?;
        buffer.write_u32_le(self.ticket).await?;
        write_string(&self.query, buffer).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SearchReply {
    pub username: String,
    pub ticket: u32,
    pub size: u64,
    pub ext: String,
    pub attributes: Vec<Attribute>,
    pub slot_free: bool,
    pub average_speed: u32,
    pub queue_length: u64,
}

#[async_trait]
impl ToBytes for SearchReply {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        let length = self.username.bytes().len() as u32
            + 8
            + self.ext.bytes().len() as u32
            + 4
            + self.attributes.len() as u32 * 8
            + 1
            + 4
            + 8;
        buffer.write_u32_le(length).await?;
        buffer.write_u32_le(MessageCode::SearchReply as u32).await?;
        write_string(&self.username, buffer).await?;
        buffer.write_u32_le(self.ticket).await?;
        buffer.write_u64_le(self.size).await?;
        buffer.write_u32_le(self.attributes.len() as u32).await?;

        for attribute in &self.attributes {
            attribute.write_to_buf(buffer).await?;
        }

        let slot_free = if self.slot_free { 1u8 } else { 0u8 };
        buffer.write_u8(slot_free);

        buffer.write_u32_le(self.average_speed).await?;
        buffer.write_u64_le(self.queue_length).await?;
        Ok(())
    }
}
