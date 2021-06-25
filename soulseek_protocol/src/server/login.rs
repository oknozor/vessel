use crate::server::MessageCode;
use std::io::Cursor;
use std::net::Ipv4Addr;
use tokio::io::BufWriter;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::frame::{read_ipv4, read_string, write_string, ParseBytes, ToBytes, STR_LENGTH_PREFIX};
use bytes::Buf;

const VERSION: u32 = 157;
const MINOR_VERSION: u32 = 19;

/// # Login request
/// Send your username, password, and client version.
///
///  ## Protocol message
/// | Description | Message Length | Message Code | Username Length | Username                | Password Length | Password                |
/// | ----------- | -------------- | ------------  | --------------- | ----------------------- | --------------- | ----------------------- |
/// | Human       | `72u32`        | `1u32`        | `8u32`          | `"username"`            | 8               | `"password"`            |
/// | Hex         | 48 00 00 00    | 01 00 00 00  | 08 00 00 00      | 75 73 65 72 6e 61 6d 65 | 08 00 00 00     | 70 61 73 73 77 6f 72 64 |
///
///
/// | Description | Version     | Length      | Hash                                                                                            | Minor Version |
/// | ----------- | ----------- | ----------- | ----------------------------------------------------------------------------------------------- | ------------- |
/// | Human       | `157u32`         | `32u32`| "d51c9a7e9353746a6020f9602d452929"                                                              | 19            |
/// | Hex         | 9d 00 00 00 | 20 00 00 00 | 64 35 31 63 39 61 37 65 39 33 35 33 37 34 36 61 36 30 32 30 66 39 36 30 32 64 34 35 32 39 32 39 | 13 00 00 00   |
///
/// **Note : ** :
///
/// Don't create [`LoginRequest`] struct manually, use [`LoginRequest::new`] to generate the md5
/// digest and use the correct version numbers.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
    version: u32,
    md5_digest: String,
    minor_version: u32,
}

impl LoginRequest {
    /// Create a new login request from username and password, this will generate a md5 password hash
    /// and append the correct minor and major version to the request (see: [`LoginRequest`])
    ///
    /// ## Example :
    /// ```
    /// let login_request = LoginRequest::new("bob", "hunter2");
    /// ```
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
    async fn write_to_buf(
        &self,
        buffer: &mut BufWriter<impl AsyncWrite + Unpin + Send>,
    ) -> tokio::io::Result<()> {
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
    fn parse(src: &mut Cursor<&[u8]>) -> std::io::Result<Self> {
        match src.get_u8() {
            0 => {
                let reason = read_string(src)?;
                Ok(LoginResponse::Failure { reason })
            }

            1 => {
                let greeting_message = read_string(src)?;
                let user_ip = read_ipv4(src);
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
