#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    ticket: u32,
    query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    username: String,
    ticket: u32,
    query: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomSearchQuery {
    room: String,
    ticket: u32,
    query: String,
}
