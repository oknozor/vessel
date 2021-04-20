use tokio::io::{AsyncWrite, BufWriter};

use crate::frame::ToBytes;

#[derive(Debug)]
pub struct TransferRequest {
    direction: u32,
    ticket: u32,
    filename: String,
    file_size: Option<u64>,
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct UploadFailed {
    filename: String,
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

#[derive(Debug)]
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
        todo!()
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct QueueDownload {
    filename: String,
}

#[async_trait]
impl ToBytes for QueueDownload {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        todo!()
    }
}

#[derive(Debug)]
pub enum TransferReply {
    TransferReplyOk {
        ticket: String,
        file_size: Option<u64>,
    },
    TransferRejected {
        ticket: String,
        reason: String,
    },
}

#[async_trait]
impl ToBytes for TransferReply {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        todo!()
    }
}
