use crate::frame::{read_string, write_string, ParseBytes, ToBytes, STR_LENGTH_PREFIX};
use crate::server::messages::MessageCode;
use bytes::Buf;
use std::io::Cursor;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    ticket: u32,
    query: String,
}

impl SearchRequest {
    pub(crate) async fn write_to_buf_with_code(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
        code: MessageCode,
    ) -> tokio::io::Result<()> {
        let len = STR_LENGTH_PREFIX + self.query.bytes().len() as u32 + 4;
        buffer.write_u32_le(len).await?;
        buffer.write_u32_le(code as u32).await?;
        buffer.write_u32_le(self.ticket).await?;
        write_string(&self.query, buffer).await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    username: String,
    ticket: u32,
    query: String,
}

#[async_trait]
impl ToBytes for SearchQuery {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        let len = STR_LENGTH_PREFIX
            + self.username.bytes().len() as u32
            + 4
            + STR_LENGTH_PREFIX
            + self.username.bytes().len() as u32;

        buffer.write_u32_le(len).await?;
        buffer
            .write_u32_le(MessageCode::SharedFoldersAndFiles as u32)
            .await?;
        write_string(&self.username, buffer).await?;
        buffer.write_u32_le(self.ticket).await?;
        write_string(&self.query, buffer).await?;

        Ok(())
    }
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
