use std::io::Cursor;

use bytes::Buf;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

use crate::{
    frame::{read_string, ParseBytes, ToBytes},
    MessageCode, ProtocolHeader, ProtocolMessage,
};

use self::search::SearchRequest;

mod branch;
mod search;

#[derive(Debug)]
pub struct DistributedMessageHeader {
    pub(crate) code: DistributedMessageCode,
    pub(crate) message_len: usize,
}

impl ProtocolHeader for DistributedMessageHeader {
    const LEN: usize = 5;
    type Code = DistributedMessageCode;

    fn message_len(&self) -> usize {
        self.message_len
    }

    fn new(message_len: usize, code: Self::Code) -> Self {
        Self { code, message_len }
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum DistributedMessageCode {
    Ping = 0,
    SearchRequest = 3,
    BranchLevel = 4,
    BranchRoot = 5,
    ChildDepth = 7,
    ServerSearchRequest = 93,
    Unknown,
}

impl MessageCode for DistributedMessageCode {
    const LEN: usize = 1;

    fn read(src: &mut Cursor<&[u8]>) -> Self {
        let code = src.get_u8();
        Self::from(code)
    }
}

impl From<u8> for DistributedMessageCode {
    fn from(code: u8) -> Self {
        match code {
            0 => DistributedMessageCode::Ping,
            3 => DistributedMessageCode::SearchRequest,
            4 => DistributedMessageCode::BranchLevel,
            5 => DistributedMessageCode::BranchRoot,
            7 => DistributedMessageCode::ChildDepth,
            93 => DistributedMessageCode::ServerSearchRequest,
            _ => DistributedMessageCode::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum DistributedMessage {
    Ping,
    SearchRequest(SearchRequest),
    BranchLevel(u32),
    BranchRoot(String),
    ChildDepth(u32),
    ServerSearchRequest,
    Unknown,
}

#[async_trait]
impl ToBytes for DistributedMessage {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        match self {
            DistributedMessage::Ping => {
                write_empty_distributed_msg(DistributedMessageCode::Ping, buffer).await?
            }
            DistributedMessage::SearchRequest(_) => todo!(),
            DistributedMessage::BranchLevel(_) => todo!(),
            DistributedMessage::BranchRoot(_) => todo!(),
            DistributedMessage::ChildDepth(_) => todo!(),
            DistributedMessage::ServerSearchRequest => todo!(),
            DistributedMessage::Unknown => todo!(),
        }

        Ok(())
    }
}

impl ProtocolMessage for DistributedMessage {
    type Header = DistributedMessageHeader;

    fn parse(src: &mut Cursor<&[u8]>, header: &DistributedMessageHeader) -> std::io::Result<Self> {
        match header.code {
            DistributedMessageCode::Ping => Ok(DistributedMessage::Ping),
            DistributedMessageCode::SearchRequest => {
                SearchRequest::parse(src).map(DistributedMessage::SearchRequest)
            }
            DistributedMessageCode::BranchLevel => {
                Ok(DistributedMessage::BranchLevel(src.get_u32_le()))
            }
            DistributedMessageCode::BranchRoot => {
                Ok(DistributedMessage::BranchRoot(read_string(src)?))
            }
            DistributedMessageCode::ChildDepth => {
                Ok(DistributedMessage::ChildDepth(src.get_u32_le()))
            }
            DistributedMessageCode::ServerSearchRequest => {
                Ok(DistributedMessage::ServerSearchRequest)
            }
            _ => {
                error!("Unknown distributed message type {:?}", src);
                Ok(DistributedMessage::Unknown)
            }
        }
    }
}

async fn write_empty_distributed_msg(
    code: DistributedMessageCode,
    buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
) -> tokio::io::Result<()> {
    buffer.write_u32_le(1).await?;
    buffer.write_u8(code as u8).await?;
    Ok(())
}
