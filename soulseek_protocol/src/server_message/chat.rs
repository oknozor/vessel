#[derive(Debug, Serialize, Deserialize)]
pub struct SendChatMessage {
    pub room: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub room: String,
    pub username: String,
    pub message: String,
}


