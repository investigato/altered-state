use crate::{models::ldap::LdapNamingContexts, storage::attributes::AttributeControlSet};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PathsConfig {
    pub actions_script_file: PathBuf,
    pub images_directory: PathBuf,
    pub scenarios_directory: PathBuf,
    pub scenario_state_file: PathBuf,
    pub schema_attributes_file: PathBuf,
    pub naming_contexts_file: PathBuf,
    pub working_directory: PathBuf,
    pub temp_directory: PathBuf,
}

impl Default for PathsConfig {
    fn default() -> Self {
        let exe_path = std::env::current_exe().expect("Failed to get current executable path");
        let base_dir = exe_path.parent().expect("Failed to get parent directory");
        let web_directory = base_dir.join("web").join("wwwroot");
        let temp_directory = base_dir.join("temp");
        let actions_script_file = temp_directory.join("actions.ps1");
        let images_directory = web_directory.join("images");
        let scenarios_directory = base_dir.join("scenarios");
        let scenario_state_file = "scenario_state.json".to_string();
        let scenario_state_path = scenarios_directory.join(&scenario_state_file);
        let schema_attributes_file = scenarios_directory.join("schema_attributes.json");
        let naming_contexts_file = scenarios_directory.join("naming_contexts.json");
        let working_directory = base_dir.join("working");
        PathsConfig {
            actions_script_file,
            images_directory,
            scenarios_directory,
            scenario_state_file: scenario_state_path,
            schema_attributes_file,
            naming_contexts_file,
            working_directory,
            temp_directory,
        }
    }
}

impl PathsConfig {
    pub fn ensure_directories(&self) -> std::io::Result<()> {
        create_directory_if_not_exists(&self.images_directory)?;
        create_directory_if_not_exists(&self.scenarios_directory)?;
        // create the scenario_state.json file if it doesn't exist and fill with empty ScenarioState JSON
        if !self.scenario_state_file.exists() {
            let scenario_state = crate::models::scenario::ScenarioState::new();
            let state_str = serde_json::to_string_pretty(&scenario_state)
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            std::fs::write(&self.scenario_state_file, state_str)?;
        }
        // create the schema_attributes.json file if it doesn't exist and fill with empty AttributeControlSet JSON
        if !self.schema_attributes_file.exists() {
            let attribute_control_set = AttributeControlSet {
                system_attributes: HashSet::new(),
                allow_list: HashMap::new(),
            };
            let attributes_str = serde_json::to_string_pretty(&attribute_control_set)
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            std::fs::write(&self.schema_attributes_file, attributes_str)?;
        }
        if !self.naming_contexts_file.exists() {
            let naming_contexts = LdapNamingContexts {
                naming_contexts: Vec::new(),
            };
            let contexts_str = serde_json::to_string_pretty(&naming_contexts)
                .map_err(|e| std::io::Error::other(e.to_string()))?;
            std::fs::write(&self.naming_contexts_file, contexts_str)?;
        }
        create_directory_if_not_exists(&self.working_directory)?;
        create_directory_if_not_exists(&self.temp_directory)?;
        Ok(())
    }
}

fn create_directory_if_not_exists(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub directory: PathBuf,
    pub prefix: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let exe_path = std::env::current_exe().expect("Failed to get current executable path");
        let base_dir = exe_path.parent().expect("Failed to get parent directory");
        let logs_directory = base_dir.join("logs");
        LoggingConfig {
            directory: logs_directory,
            prefix: "log".to_string(),
        }
    }
}

impl LoggingConfig {
    pub fn ensure_directories(&self) -> std::io::Result<()> {
        create_directory_if_not_exists(&self.directory)
    }
}
