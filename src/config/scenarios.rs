use crate::config::exclusions::ExclusionConfig;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConfig {
    pub name: String,
    pub description: Option<String>,
    pub image_path: Option<String>,
    pub hooks: Vec<ScenarioHookConfig>,
    pub exclusions: Vec<ExclusionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioHookConfig {
    pub hook_type: ScenarioHookType,
    pub path: PathBuf,
    pub arguments: Vec<String>,
    pub continue_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScenarioHookType {
    PreAction,
    Cleanup,
    Activation,
}

impl fmt::Display for ScenarioHookType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScenarioHookType::PreAction => write!(f, "preaction"),
            ScenarioHookType::Cleanup => write!(f, "cleanup"),
            ScenarioHookType::Activation => write!(f, "activation"),
        }
    }
}

impl ScenarioConfig {
    pub fn load_from_path(path: &str) -> Result<Self, String> {
        let config_str = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read scenario config file: {}", e))?;
        let mut config: ScenarioConfig = serde_json::from_str(&config_str)
            .map_err(|e| format!("Failed to parse scenario config JSON: {}", e))?;
        config.validate(path)?;
        Ok(config)
    }
    pub fn load_all(path: &Path) -> Result<Vec<Self>, String> {
        let mut scenarios = Vec::new();
        let entries = std::fs::read_dir(path)
            .map_err(|e| format!("Failed to read scenarios directory: {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let entry_path = entry.path();
            // inside each scenario folder, the config is named config.json
            if entry_path.is_dir() {
                let config_path = entry_path.join("config.json");
                if config_path.exists() {
                    let scenario = Self::load_from_path(config_path.to_str().unwrap())?;
                    scenarios.push(scenario);
                }
            }
        }
        Ok(scenarios)
    }
    pub fn load_for_scenario(scenarios_dir: &Path, scenario_name: &str) -> Result<Self, String> {
        let scenario_path = scenarios_dir.join(scenario_name).join("config.json");
        if !scenario_path.exists() {
            return Err(format!(
                "Scenario config not found for scenario '{}'",
                scenario_name
            ));
        }
        Self::load_from_path(scenario_path.to_str().unwrap())
    }
    pub fn validate(&mut self, path: &str) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Scenario name cannot be empty".to_string());
        }
        // valid locations for hook scripts and images either next to the executable (same directory)
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get current executable path: {}", e))?;
        let exe_directory = exe_path
            .parent()
            .ok_or_else(|| "Failed to get executable parent directory".to_string())?;
        // or in the same path as the config file passed in
        let config_directory = Path::new(path)
            .parent()
            .ok_or_else(|| "Failed to get scenario config directory".to_string())?;
        let config_directory = config_directory
            .canonicalize()
            .unwrap_or_else(|_| config_directory.to_path_buf());

        let mut changed = false;

        if let Some(image_path) = &self.image_path {
            let image_path_buf = PathBuf::from(image_path);
            if image_path_buf.is_absolute() {
                if !image_path_buf.exists() {
                    return Err(format!(
                        "Image file not found at path: {:?}",
                        image_path_buf
                    ));
                }
            } else {
                // check them both
                let exe_image_path = exe_directory.join(&image_path_buf);
                let config_image_path = config_directory.join(&image_path_buf);
                let resolved_image_path = if exe_image_path.exists() {
                    exe_image_path
                } else if config_image_path.exists() {
                    config_image_path
                } else {
                    return Err(format!(
                        "Image file not found at either path: {:?} or {:?}",
                        exe_image_path, config_image_path
                    ));
                };

                self.image_path = Some(resolved_image_path.to_string_lossy().to_string());
                changed = true;
            }
        }

        for hook in &mut self.hooks {
            let hook_path = if hook.path.is_absolute() {
                if !hook.path.exists() {
                    return Err(format!("Hook script not found at path: {:?}", hook.path));
                }
                hook.path.clone()
            } else {
                // check them both
                let exe_hook_path = exe_directory.join(&hook.path);
                let config_hook_path = config_directory.join(&hook.path);
                if exe_hook_path.exists() {
                    exe_hook_path
                } else if config_hook_path.exists() {
                    config_hook_path
                } else {
                    return Err(format!(
                        "Hook script not found at either path: {:?} or {:?}",
                        exe_hook_path, config_hook_path
                    ));
                }
            };

            if hook.path != hook_path {
                hook.path = hook_path;
                changed = true;
            }
        }

        if changed {
            self.save_to_path(path)
                .map_err(|e| format!("Failed to persist normalized scenario config: {}", e))?;
        }

        Ok(())
    }
    pub fn copy_scripts_to_directory(
        hooks: Vec<ScenarioHookConfig>,
        target_directory: &Path,
    ) -> Result<(), String> {
        for hook in &hooks {
            let file_name = hook
                .path
                .file_name()
                .ok_or_else(|| format!("Invalid hook path: {:?}", hook.path))?;
            let target_path = target_directory.join(file_name);
            std::fs::copy(&hook.path, &target_path).map_err(|e| {
                format!(
                    "Failed to copy hook script from {:?} to {:?}: {}",
                    hook.path, target_path, e
                )
            })?;
        }
        Ok(())
    }

    pub fn copy_image_to_directory(
        image_path: &PathBuf,
        target_directory: &Path,
    ) -> Result<(), String> {
        let file_name = image_path
            .file_name()
            .ok_or_else(|| format!("Invalid image path: {:?}", image_path))?;
        let target_path = target_directory.join(file_name);
        std::fs::copy(image_path, &target_path).map_err(|e| {
            format!(
                "Failed to copy image from {:?} to {:?}: {}",
                image_path, target_path, e
            )
        })?;
        Ok(())
    }
    pub fn save_to_path(&self, path: &str) -> Result<(), std::io::Error> {
        let config_str =
            serde_json::to_string_pretty(self).map_err(|e| std::io::Error::other(e.to_string()))?;
        std::fs::write(path, config_str)
    }
}
