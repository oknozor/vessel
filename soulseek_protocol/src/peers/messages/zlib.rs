use std::io::Cursor;

use bytes::Buf;
use flate2::{Decompress, FlushDecompress};

pub(crate) fn decompress(src: &mut Cursor<&[u8]>) -> std::io::Result<Vec<u8>> {
    let mut data = Vec::with_capacity(1000000);
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
