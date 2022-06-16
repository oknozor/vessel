use crate::entity::Entity;
use soulseek_protocol::peers::p2p::transfer::TransferRequest;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DownloadEntity {
    pub file_name: String,
    pub user: String,
    pub ticket: u32,
    pub file_size: u64,
    pub progress: u64,
}

impl DownloadEntity {
    pub fn key_from(username: &str, ticket: u32) -> String {
        format!("{}@{}", username, ticket)
    }
}

impl Entity for DownloadEntity {
    fn get_key(&self) -> Vec<u8> {
        let key = DownloadEntity::key_from(&self.user, self.ticket);
        key.as_bytes().to_vec()
    }

    const COLLECTION: &'static str = "downloads";
}

impl From<(String, &TransferRequest)> for DownloadEntity {
    fn from((user, request): (String, &TransferRequest)) -> Self {
        let file_name = request.file_name.replace('\\', "/");
        let file_name = Path::new(&file_name)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        DownloadEntity {
            file_name,
            user,
            ticket: request.ticket,
            file_size: request
                .file_size
                .expect("Accepted transfer request should have a file size"),
            progress: 0,
        }
    }
}
