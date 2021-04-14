// FIXME : what does this message mean ?
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminCommand {
    string: String,
    strings: Vec<String>,
}
