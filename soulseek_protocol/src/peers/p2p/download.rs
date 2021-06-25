#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum DownloadProgress {
    Init {
        file_name: String,
        user_name: String,
        ticket: u32,
    },
    Progress {
        ticket: u32,
        percent: usize,
    },
}
