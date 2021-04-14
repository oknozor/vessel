#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendations {
    recommendation: Vec<Recommendation>,
    unrecommendation: Vec<Recommendation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemRecommendations {
    item: String,
    recommendation: Vec<Recommendation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    content: String,
    note: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interests {
    username: String,
    liked: Vec<String>,
    hated: Vec<String>,
}
