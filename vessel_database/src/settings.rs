use config::{Config, ConfigError, File};
use std::path::PathBuf;

lazy_static! {
    pub static ref CONFIG: Settings = Settings::get().unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub(crate) shared_directories: Vec<PathBuf>,
    pub download_folder: PathBuf,
    pub username: String,
    pub password: String,
}

impl Settings {
    pub fn get() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::from(PathBuf::from("vessel.toml")))?;
        let settings: Result<Settings, ConfigError> = s.try_into();

        settings
    }
}
