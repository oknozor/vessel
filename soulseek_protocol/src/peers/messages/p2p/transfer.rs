use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::frame::{read_string, write_string, ParseBytes, ToBytes, STR_LENGTH_PREFIX};
use crate::peers::messages::p2p::PeerMessageCode;
use bytes::Buf;
use std::io::Cursor;

#[derive(Debug, Serialize)]
pub struct TransferRequest {
    direction: u32,
    pub ticket: u32,
    pub filename: String,
    pub file_size: Option<u64>,
}

impl ParseBytes for TransferRequest {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let direction = src.get_u32_le();
        let ticket = src.get_u32_le();
        let filename = read_string(src)?;

        let file_size = if direction == 1 {
            Some(src.get_u64_le())
        } else {
            None
        };

        Ok(Self {
            direction,
            ticket,
            filename,
            file_size,
        })
    }
}

#[async_trait]
impl ToBytes for TransferRequest {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        todo!()
    }
}

#[derive(Debug, Serialize)]
pub struct PlaceInQueueReply {
    filename: String,
    place: String,
}

#[async_trait]
impl ToBytes for PlaceInQueueReply {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        todo!()
    }
}

#[derive(Debug, Serialize)]
pub struct UploadFailed {
    filename: String,
}

impl ParseBytes for UploadFailed {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let filename = read_string(src)?;

        Ok(Self { filename })
    }
}

#[async_trait]
impl ToBytes for UploadFailed {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        todo!()
    }
}

#[derive(Debug, Serialize)]
pub struct QueueFailed {
    filename: String,
    reason: String,
}

#[async_trait]
impl ToBytes for QueueFailed {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        let len = STR_LENGTH_PREFIX
            + self.filename.as_bytes().len() as u32
            + STR_LENGTH_PREFIX
            + self.reason.as_bytes().len() as u32
            + 4;

        buffer.write_u32_le(len).await?;
        buffer
            .write_u32_le(PeerMessageCode::QueueFailed as u32)
            .await?;
        write_string(&self.filename, buffer).await?;
        write_string(&self.reason, buffer).await?;

        Ok(())
    }
}

impl ParseBytes for QueueFailed {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let filename = read_string(src)?;
        let reason = read_string(src)?;

        Ok(Self { filename, reason })
    }
}

#[derive(Debug, Serialize)]
pub struct PlaceInQueueRequest {
    filename: String,
}

#[async_trait]
impl ToBytes for PlaceInQueueRequest {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        todo!()
    }
}

#[derive(Debug, Serialize)]
pub enum TransferReply {
    TransferReplyOk { ticket: u32, file_size: u64 },
    TransferRejected { ticket: u32, reason: String },
}

#[async_trait]
impl ToBytes for TransferReply {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        match self {
            TransferReply::TransferReplyOk { ticket, file_size } => {
                let len = 4 + 4 + 1 + 8;
                buffer.write_u32_le(len).await?;
                buffer
                    .write_u32_le(PeerMessageCode::TransferReply as u32)
                    .await?;
                buffer.write_u32_le(*ticket).await?;
                buffer.write_u8(1).await?;
                buffer.write_u64_le(*file_size).await?;
            }
            TransferReply::TransferRejected { ticket, reason } => {
                let len = 4 + 4 + 1 + STR_LENGTH_PREFIX + reason.bytes().len() as u32;
                buffer.write_u32_le(len).await?;
                buffer
                    .write_u32_le(PeerMessageCode::TransferReply as u32)
                    .await?;
                buffer.write_u32_le(*ticket).await?;
                buffer.write_u8(0).await?;
                write_string(reason, buffer).await?;
            }
        };

        Ok(())
    }
}
