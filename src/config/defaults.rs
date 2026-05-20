use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PathsConfig {
    pub actions_script_file: String,
    pub images_directory: String,
    pub scenarios_directory: String,
    pub scenario_state_file: String,
    pub schema_attributes_file: String,
    pub working_directory: String,
    pub temp_directory: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        PathsConfig {
            actions_script_file: "actions.ps1".to_string(),
            images_directory: "images".to_string(),
            scenarios_directory: "scenarios".to_string(),
            scenario_state_file: "scenario_state.yaml".to_string(),
            schema_attributes_file: "schema_attributes.yaml".to_string(),
            working_directory: "working".to_string(),
            temp_directory: "temp".to_string(),
        }
    }
}

impl PathsConfig {
    pub fn ensure_directories(&self) -> std::io::Result<()> {
        let exe_path = std::env::current_exe().expect("Failed to get current executable path");
        let base_dir = exe_path.parent().expect("Failed to get parent directory");
        create_directory_if_not_exists(&base_dir.join(&self.images_directory))?;
        create_directory_if_not_exists(&base_dir.join(&self.scenarios_directory))?;
        create_directory_if_not_exists(&base_dir.join(&self.working_directory))?;
        create_directory_if_not_exists(&base_dir.join(&self.temp_directory))?;
        Ok(())
    }
}

pub fn create_directory_if_not_exists(path: &std::path::Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub directory: String,
    pub prefix: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            directory: "logs".to_string(),
            prefix: "log".to_string(),
        }
    }
}

impl LoggingConfig {
    pub fn ensure_directories(&self) -> std::io::Result<()> {
        let exe_path = std::env::current_exe().expect("Failed to get current executable path");
        let base_dir = exe_path.parent().expect("Failed to get parent directory");
        create_directory_if_not_exists(&base_dir.join(&self.directory))
    }
}
