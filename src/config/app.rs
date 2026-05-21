use crate::config::defaults::{LoggingConfig, PathsConfig};
use anyhow::Result;
use config::{Config, ConfigError, File, FileFormat};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub domain: String,
    pub hostname: String,
    #[serde(default)]
    pub paths: PathsConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
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

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct LdapConfig {
//     pub ldapfqdn: String,
//     #[serde(default = "default_ldap_port")]
//     pub port: u16,
//     #[serde(default)]
//     pub ldaps: bool,
// }
// fn default_ldap_port() -> u16 {
//     389
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectChangeType {
    Created,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMatch {
    pub name_exact: Option<String>,
    pub name_prefix: Option<String>,
    pub name_contains: Option<String>,
    pub ou: Option<String>,
    pub dn_contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExclusionConfig {
    pub name: String,
    pub match_: ObjectMatch,
    pub rules: Vec<ChangeRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRule {
    pub change_type: ObjectChangeType,
    pub additional_ignored_attributes: Vec<String>,
}
