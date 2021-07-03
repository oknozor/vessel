use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SearchQuery {
    pub term: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchTicket {
    pub ticket: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub cause: String,
}

#[derive(Deserialize, Serialize)]
pub struct QueueRequest {
    pub(crate) file_name: String,
}

#[derive(Deserialize, Serialize)]
pub struct ChatMessage {
    pub(crate) message: String,
}
