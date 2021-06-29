use crate::{
    frame::{read_string, write_string, ParseBytes, ToBytes},
    peers::p2p::{zlib, zlib::decompress, PeerMessageCode},
};
use bytes::Buf;
use std::io::Cursor;
use tokio::io::{AsyncWrite, AsyncWriteExt, BufWriter};

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct SharedDirectories {
    pub dirs: Vec<Directory>,
}

#[async_trait]
impl ToBytes for SharedDirectories {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        // Pack message
        let inner = &mut vec![];
        let mut message_buffer = BufWriter::new(inner);

        message_buffer.write_u32_le(self.dirs.len() as u32).await?;

        for dir in &self.dirs {
            dir.write_to_buf(&mut message_buffer).await?;
        }

        message_buffer.flush().await?;
        let data = message_buffer.into_inner();
        // Compress message
        let compressed_data = zlib::compress(data)?;

        // Write to connection buffer
        buffer
            .write_u32_le(compressed_data.len() as u32 + 4)
            .await?;
        buffer
            .write_u32_le(PeerMessageCode::SharesReply as u32)
            .await?;
        buffer.write_all(compressed_data.as_slice()).await?;

        Ok(())
    }
}

impl ParseBytes for SharedDirectories {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let data = decompress(src)?;
        let mut cursor = Cursor::new(data.as_slice());

        let directory_nth = cursor.get_u32_le();
        let mut dirs = Vec::with_capacity(directory_nth as usize);

        for _ in 0..directory_nth {
            dirs.push(Directory::parse(&mut cursor)?);
        }

        Ok(SharedDirectories { dirs })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Directory {
    pub name: String,
    pub files: Vec<File>,
}

#[async_trait]
impl ToBytes for Directory {
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
        write_string(&self.name, buffer).await?;
        buffer.write_u32_le(self.files.len() as u32).await?;

        for file in &self.files {
            file.write_to_buf(buffer).await?;
        }

        Ok(())
    }
}

impl ParseBytes for Directory {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let name = read_string(src)?;
        let file_nth = src.get_u32_le();
        let mut files = Vec::with_capacity(file_nth as usize);
        for _ in 0..file_nth {
            files.push(File::parse(src)?);
        }

        Ok(Directory { name, files })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct File {
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
        buffer.write_u8(1).await?; // unused char
        write_string(&self.name, buffer).await?;
        buffer.write_u64_le(self.size).await?;
        write_string(&self.extension, buffer).await?;
        buffer.write_u32_le(self.attributes.len() as u32).await?;

        for attribute in &self.attributes {
            attribute.write_to_buf(buffer).await?
        }

        Ok(())
    }
}

impl ParseBytes for File {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let _unused_char = src.get_u8();
        let name = read_string(src)?;
        let size = src.get_u64_le();
        let extension = read_string(src)?;
        let attribute_size = src.get_u32_le();

        let mut attributes = Vec::with_capacity(attribute_size as usize);

        for _ in 0..attribute_size {
            attributes.push(Attribute::parse(src)?);
        }

        Ok(File {
            name,
            size,
            extension,
            attributes,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
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

impl ParseBytes for Attribute {
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        let place = src.get_u32_le();
        let attribute = src.get_u32_le();

        Ok(Attribute { place, attribute })
    }
}

#[cfg(test)]
mod test {
    use crate::{
        frame::{ParseBytes, ToBytes},
        peers::p2p::{
            shared_directories::{Directory, File, SharedDirectories},
            PeerMessageCode,
        },
    };
    use bytes::Buf;
    use tokio::io::BufWriter;
    use tokio_test::block_on;

    #[test]
    fn write_share_reply_ok() {
        let shared_dirs = SharedDirectories {
            dirs: vec![Directory {
                name: "test".to_string(),
                files: vec![File {
                    name: "file".to_string(),
                    size: 0,
                    extension: "md".to_string(),
                    attributes: vec![],
                }],
            }],
        };

        let mut vec = vec![];
        let mut buff = BufWriter::new(&mut vec);
        block_on(shared_dirs.write_to_buf(&mut buff)).unwrap();
        let mut cursor = std::io::Cursor::new(buff.buffer());

        let len = cursor.get_u32_le();
        let code = cursor.get_u32_le();
        assert_eq!(code, PeerMessageCode::SharesReply as u32);
        assert_eq!(cursor.position(), 8);

        let parse_result = SharedDirectories::parse(&mut cursor).unwrap();

        assert_eq!(parse_result, shared_dirs);
    }
}
