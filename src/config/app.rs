use crate::{
    models::scenario::ScenarioState,
    config::defaults::{LoggingConfig, PathsConfig},
};
use anyhow::Result;
use config::{Config, ConfigError, File, FileFormat};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub domain: String,
    pub hostname: String,
    #[serde(default)]
    pub never_touch_these_attributes: Vec<String>,
    #[serde(default)]
    pub paths: PathsConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub scenario_state: ScenarioState,
}

impl AppConfig {
    pub fn load(path: Option<&str>) -> Result<Self, ConfigError> {
        // config file should be there
        let config_path = match path {
            Some(p) => std::path::PathBuf::from(p),
            None => {
                let exe_path =
                    std::env::current_exe().expect("Failed to get current executable path");
                exe_path
                    .parent()
                    .expect("Failed to get parent directory")
                    .join("config.json")
            }
        };
        Config::builder()
            .add_source(File::from(config_path).format(FileFormat::Json))
            .build()?
            .try_deserialize()
    }
}
