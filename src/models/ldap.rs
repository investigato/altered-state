use crate::config::app::AppConfig;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer as to_io_writer};
use std::error::Error;
use std::fs::File;
use std::path::Path;
#[derive(Clone, Debug)]
pub struct LdapOptions {
    pub domain: String,
    pub ldapfqdn: String,
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub ldaps: bool,
    pub ldap_filter: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LdapNamingContexts {
    pub naming_contexts: Vec<String>,
}

impl LdapNamingContexts {
    pub fn load_from_file(path: &Path) -> Result<LdapNamingContexts, Box<dyn Error>> {
        // if it doesn't exist, create a new file with empty naming contexts and return that
        if !path.exists() {
            let naming_contexts = LdapNamingContexts {
                naming_contexts: Vec::new(),
            };

            let mut file = File::create(path)?;
            to_io_writer(&mut file, &naming_contexts)?;
            return Ok(naming_contexts);
        }
        let file = File::open(path)?;
        let naming_contexts: LdapNamingContexts = from_reader(file)?;
        Ok(naming_contexts)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        to_io_writer(&mut file, self)?;
        Ok(())
    }
}

pub fn generate_ldap_options_from_config(config: &AppConfig) -> LdapOptions {
    LdapOptions {
        domain: config.domain.to_string(),
        ldapfqdn: config.hostname.to_string(),
        ip: Some(config.hostname.to_string()),
        port: Some(389),
        ldaps: false,
        ldap_filter: Some("(objectClass=*)".to_string()),
    }
}
