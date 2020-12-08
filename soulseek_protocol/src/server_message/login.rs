use crate::server_message::MessageCode;
use std::io::Cursor;
use std::net::Ipv4Addr;
use tokio::io::AsyncWriteExt;
use tokio::io::BufWriter;
use tokio::net::TcpStream;

use crate::frame::{ParseBytes, ToBytes};
use crate::{read_ipv4, read_string, write_string, STR_LENGTH_PREFIX};
use bytes::Buf;

const VERSION: u32 = 157;
const MINOR_VERSION: u32 = 19;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
    version: u32,
    md5_digest: String,
    minor_version: u32,
}

impl LoginRequest {
    pub fn new(username: &str, password: &str) -> Self {
        let md5_digest = md5::compute(format!("{}{}", username, password));
        let md5_digest = format!("{:x}", md5_digest);

        Self {
            username: String::from(username),
            password: String::from(password),
            version: VERSION,
            md5_digest,
            minor_version: MINOR_VERSION,
        }
    }
}

#[async_trait]
impl ToBytes for LoginRequest {
    async fn write_to_buf(&self, buffer: &mut BufWriter<TcpStream>) -> tokio::io::Result<()> {
        let version_len = 4;
        let minor_version_len = 4;

        // Header
        let username = self.username.as_bytes().len() as u32 + 4;
        let password = self.password.as_bytes().len() as u32 + 4;
        let md5_digest = self.md5_digest.as_bytes().len() as u32 + 4;

        let length =
            STR_LENGTH_PREFIX + username + password + md5_digest + version_len + minor_version_len;

        buffer.write_u32_le(length).await?;
        buffer.write_u32_le(MessageCode::Login as u32).await?;

        // Content
        write_string(&self.username, buffer).await?;
        write_string(&self.password, buffer).await?;
        buffer.write_u32_le(self.version).await?;
        write_string(&self.md5_digest, buffer).await?;
        buffer.write_u32_le(self.minor_version).await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginResponse {
    /// Response code 0 indicate the login failed
    Failure {
        /// The failure cause, most of the time "INVALID PASSWORD"
        reason: String,
    },
    /// Response code 1 indicate the login succeeded
    Success {
        /// A mood of the day string
        greeting_message: String,
        /// User IP address
        user_ip: Ipv4Addr,
        ///MD5 hex digest of concatenated username & password
        password_md5_digest: String,
    },
}

impl ParseBytes for LoginResponse {
    type Output = Self;

    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        match src.get_u8() {
            0 => {
                let reason = read_string(src)?;
                Ok(LoginResponse::Failure { reason })
            }

            1 => {
                let greeting_message = read_string(src)?;
                let user_ip = read_ipv4(src)?;
                let password_md5_digest = read_string(src)?;

                Ok(LoginResponse::Success {
                    greeting_message,
                    user_ip,
                    password_md5_digest,
                })
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "unimplemented",
            )),
        }
    }
}
