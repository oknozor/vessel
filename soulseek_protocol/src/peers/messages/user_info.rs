use std::io::Cursor;

use bytes::Buf;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::frame::{ParseBytes, read_string, ToBytes, write_string};
use crate::peers::messages::MessageCode;

#[derive(Debug)]
pub struct UserInfo {
    pub description: String,
    pub picture: Option<String>,
    pub total_upload: u32,
    pub queue_size: u32,
    pub slots_free: bool,
}

impl ParseBytes for UserInfo {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self::Output> {
        let description = read_string(src)?;
        let has_picture = src.get_u8() != 0;
        let picture = if has_picture {
            read_string(src).ok()
        } else {
            None
        };
        let total_upload = src.get_u32_le();
        let queue_size = src.get_u32_le();
        let slots_free = src.get_u8() == 1;

        Ok(UserInfo {
            description,
            picture,
            total_upload,
            queue_size,
            slots_free,
        })
    }
}

#[async_trait]
impl ToBytes for UserInfo {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        let description = 4 + self.description.bytes().len() as u32;
        let has_picture = 1;
        let picture_len = self
            .picture
            .as_ref()
            .map(|picture| picture.len() as u32 + 4);
        let total_upload = 4;
        let queue_size = 4;
        let slots_free = 1;

        let length = if let Some(picture_len) = picture_len {
            4 + description + has_picture + picture_len + total_upload + queue_size + slots_free
        } else {
            4 + description + has_picture + total_upload + queue_size + slots_free
        };

        buffer.write_u32_le(length).await?;
        buffer
            .write_u32_le(MessageCode::UserInfoReply as u32)
            .await?;
        write_string(&self.description, buffer).await?;
        if let Some(picture) = &self.picture {
            buffer.write_u8(1).await?;
            write_string(picture, buffer).await?;
        } else {
            buffer.write_u8(0).await?;
        }
        buffer.write_u32_le(self.total_upload).await?;
        buffer.write_u32_le(self.queue_size).await?;
        let slot_free = if self.slots_free { 1 } else { 0 };
        buffer.write_u8(slot_free).await?;
        Ok(())
    }
}
