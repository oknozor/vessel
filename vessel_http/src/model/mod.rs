use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SearchQuery {
    pub term: String,
}

#[derive(Deserialize, Serialize)]
pub struct QueueRequest {
    pub(crate) filename: String,
}

#[derive(Deserialize, Serialize)]
pub struct ChatMessage {
    pub(crate) message: String,
}
