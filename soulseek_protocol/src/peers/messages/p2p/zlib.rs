use std::io::{Cursor, Write};

use bytes::Buf;
use flate2::write::ZlibEncoder;
use flate2::{Compression, Decompress, FlushDecompress};

pub(crate) fn decompress(src: &mut Cursor<&[u8]>) -> std::io::Result<Vec<u8>> {
    // FIXME : 1032 is zlib max ratio
    let mut data = Vec::with_capacity(src.remaining() * 1032);
    let decompress_result =
        Decompress::new(true).decompress_vec(&src.chunk(), &mut data, FlushDecompress::Sync);

    match decompress_result {
        Ok(status) => {
            debug!("Data successfully decompressed : {:?}", status)
        }
        Err(e) => {
            error!("Decompress error: {}", e);
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }
    };

    Ok(data)
}

pub(crate) fn compress(data: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(data)?;
    let compressed_data = e.finish()?;

    Ok(compressed_data)
}

#[cfg(test)]
mod test {
    use crate::frame::ToBytes;
    use crate::peers::messages::p2p::shared_directories::{Directory, File, SharedDirectories};
    use crate::peers::messages::p2p::zlib::{compress, decompress};
    use crate::peers::messages::p2p::PeerMessageCode;
    use bytes::Buf;
    use futures::executor::block_on;
    use std::io::Cursor;
    use tokio::io::{AsyncWriteExt, BufWriter};

    #[test]
    fn should_decompress() {
        let data = vec![1, 2, 3, 4];
        let out = &mut vec![];
        let cursor = Cursor::new(data.as_slice());
        let mut message_buffer = BufWriter::new(out);

        block_on(async { message_buffer.write_all(data.as_slice()).await.unwrap() });

        let compressed_data = compress(message_buffer.buffer()).unwrap();
        let mut compressed_data = Cursor::new(compressed_data.as_slice());

        let decompressed_data = decompress(&mut compressed_data).unwrap();

        assert_eq!(decompressed_data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn should_decompress_share_reply() {
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
        let data = buff.buffer();

        let mut cursor = Cursor::new(data);
        let len = cursor.get_u32_le();
        let code = cursor.get_u32_le();

        assert_eq!(len, 40);
        assert_eq!(code, PeerMessageCode::SharesReply as u32);
        assert!(cursor.has_remaining());

        let decompressed_data = decompress(&mut cursor).unwrap();

        assert_eq!(
            decompressed_data,
            vec![
                1, 0, 0, 0, 4, 0, 0, 0, 116, 101, 115, 116, 1, 0, 0, 0, 1, 4, 0, 0, 0, 102, 105,
                108, 101, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 109, 100, 0, 0, 0, 0
            ]
        );
    }
}
