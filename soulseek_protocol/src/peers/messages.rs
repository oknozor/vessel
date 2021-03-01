use std::io::Cursor;

use bytes::Buf;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::frame::{read_string, write_string, ToBytes};
use crate::message_common::ConnectionType;
use crate::peers::messages::InitMessageCode::{PeerInit, PierceFireWall, Unknown};
use crate::peers::request::PeerRequest;
use crate::peers::response::PeerResponse;
use crate::SlskError;

pub(crate) const CONNECTION_MSG_HEADER_LEN: u32 = 5;
pub(crate) const PEER_MSG_HEADER_LEN: u32 = 8;

#[derive(Debug)]
pub struct ConnectionMessageHeader {
    pub(crate) code: InitMessageCode,
    pub(crate) message_len: usize,
}

#[derive(Debug)]
pub struct PeerMessageHeader {
    pub(crate) code: MessageCode,
    pub(crate) message_len: usize,
}

#[derive(Debug)]
pub enum PeerRequestPacket {
    Message(PeerRequest),
    ConnectionMessage(PeerConnectionMessage),
    None,
}

#[derive(Debug)]
pub enum PeerResponsePacket {
    Message(PeerResponse),
    ConnectionMessage(PeerConnectionMessage),
    None,
}

impl ConnectionMessageHeader {
    pub fn read(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let message_length = src.get_u32_le();
        let code = src.get_u8();
        let code = InitMessageCode::from(code);

        // We can subtract message code from the length since we already know it
        let message_len = message_length as usize - 1;

        Ok(Self { message_len, code })
    }
}

impl PeerMessageHeader {
    pub fn read(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let message_length = src.get_u32_le();
        let code = src.get_u32_le();
        let code = MessageCode::from(code);

        // We can subtract message code from the length since we already know it
        let message_len = message_length as usize - 4;

        Ok(Self { message_len, code })
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum InitMessageCode {
    PierceFireWall = 0,
    PeerInit = 1,
    Unknown,
}

#[repr(u32)]
#[derive(Debug)]
pub enum MessageCode {
    SharesRequest = 4,
    SharesReply = 5,
    SearchRequest = 8,
    SearchReply = 9,
    UserInfoRequest = 15,
    UserInfoReply = 16,
    FolderContentsRequest = 36,
    FolderContentsReply = 37,
    TransferRequest = 40,
    TransferReply = 41,
    UploadPlacehold = 42,
    QueueDownload = 43,
    PlaceInQueueReply = 44,
    UploadFailed = 46,
    QueueFailed = 50,
    PlaceInQueueRequest = 51,
    UploadQueueNotification = 52,
    Unknown,
}

impl From<u32> for MessageCode {
    fn from(code: u32) -> Self {
        match code {
            4 => MessageCode::SharesRequest,
            5 => MessageCode::SharesReply,
            8 => MessageCode::SearchRequest,
            9 => MessageCode::SearchReply,
            15 => MessageCode::UserInfoRequest,
            16 => MessageCode::UserInfoReply,
            36 => MessageCode::FolderContentsRequest,
            37 => MessageCode::FolderContentsReply,
            40 => MessageCode::TransferRequest,
            41 => MessageCode::TransferReply,
            42 => MessageCode::UploadPlacehold,
            43 => MessageCode::QueueDownload,
            44 => MessageCode::PlaceInQueueReply,
            46 => MessageCode::UploadFailed,
            50 => MessageCode::QueueFailed,
            51 => MessageCode::PlaceInQueueRequest,
            52 => MessageCode::UploadQueueNotification,
            _ => MessageCode::Unknown,
        }
    }
}

impl From<u8> for InitMessageCode {
    fn from(code: u8) -> Self {
        match code {
            0 => PierceFireWall,
            1 => PeerInit,
            _ => Unknown,
        }
    }
}

#[derive(Debug)]
pub enum PeerConnectionMessage {
    PierceFirewall(u32),
    PeerInit {
        username: String,
        connection_type: ConnectionType,
        token: u32,
    },
}

#[derive(Debug)]
pub struct Attribute {
    pub place: u32,
    pub attribute: u32,
}

#[async_trait]
impl ToBytes for Attribute {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        buffer.write_u32_le(self.attribute).await?;
        buffer.write_u32_le(self.place).await?;
        Ok(())
    }
}

#[derive(Debug)]
struct Directory {
    pub name: String,
    pub files: Vec<File>,
}

#[async_trait]
impl ToBytes for Directory {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        todo!()
    }
}

#[derive(Debug)]
struct File {
    pub name: String,
    pub size: u64,
    pub extension: String,
    pub attributes: Vec<Attribute>,
}

#[async_trait]
impl ToBytes for File {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        todo!()
    }
}

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

#[derive(Debug)]
pub struct UserInfo {
    pub description: String,
    pub picture: Option<String>,
    pub total_upload: u32,
    pub queue_size: u32,
    pub slots_free: bool,
}

#[async_trait]
impl ToBytes for UserInfo {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        let picture_len = if let Some(picture) = &self.picture {
            picture.len() as u32
        } else {
            0
        };

        let length = 4 + self.description.bytes().len() as u32 + 1 + picture_len + 4 + 4 + 1;

        buffer.write_u32_le(length).await?;
        buffer
            .write_u32_le(MessageCode::UserInfoReply as u32)
            .await?;
        write_string(&self.description, buffer).await?;
        buffer.write_u32_le(self.total_upload).await?;
        buffer.write_u32_le(self.queue_size).await?;
        let slot_free = if self.slots_free { 1 } else { 0 };
        buffer.write_u8(slot_free).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FolderContentsRequest {
    files: Vec<String>,
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

#[derive(Debug)]
pub struct FolderContentsReply {
    dirs: Vec<Directory>,
}

#[async_trait]
impl ToBytes for FolderContentsReply {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        todo!()
    }
}

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

#[async_trait]
impl ToBytes for PeerConnectionMessage {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        match self {
            PeerConnectionMessage::PierceFirewall(token) => {
                buffer.write_u32_le(CONNECTION_MSG_HEADER_LEN).await?;
                buffer
                    .write_u8(InitMessageCode::PierceFireWall as u8)
                    .await?;
                buffer.write_u32_le(*token).await?;
            }
            PeerConnectionMessage::PeerInit {
                username,
                connection_type,
                token,
            } => {
                let username_len = username.bytes().len() as u32 + 4;
                let connection_type_len = connection_type.bytes().len() as u32 + 4;

                buffer
                    .write_u32_le(1 + username_len + connection_type_len + 4)
                    .await?;
                buffer.write_u8(InitMessageCode::PeerInit as u8).await?;
                write_string(&username, buffer).await?;
                write_string(connection_type.as_ref(), buffer).await?;

                buffer.write_u32_le(*token).await?;
            }
        }

        Ok(())
    }
}

impl PeerConnectionMessage {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<ConnectionMessageHeader, SlskError> {
        // Check if the buffer contains enough bytes to parse the message error
        if src.remaining() < CONNECTION_MSG_HEADER_LEN as usize {
            return Err(SlskError::Incomplete);
        }

        // Check if the buffer contains the full message already
        let header = ConnectionMessageHeader::read(src)?;
        if src.remaining() < header.message_len {
            Err(SlskError::Incomplete)
        } else {
            // discard header data
            src.set_position(0);
            src.advance(CONNECTION_MSG_HEADER_LEN as usize);

            Ok(header)
        }
    }

    pub(crate) fn parse(
        src: &mut Cursor<&[u8]>,
        header: &ConnectionMessageHeader,
    ) -> std::io::Result<Self> {
        let message = match header.code {
            InitMessageCode::PierceFireWall => {
                PeerConnectionMessage::PierceFirewall(src.get_u32_le())
            }
            InitMessageCode::PeerInit => PeerConnectionMessage::PeerInit {
                username: read_string(src)?,
                connection_type: ConnectionType::from(read_string(src)?),
                token: src.get_u32_le(),
            },
            InitMessageCode::Unknown => panic!("Unkown message kind, code"),
        };

        Ok(message)
    }

    pub fn kind(&self) -> &str {
        match self {
            PeerConnectionMessage::PierceFirewall(_) => "PierceFirewall",
            PeerConnectionMessage::PeerInit { .. } => "PeerInit",
        }
    }
}
