use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use std::{
    fmt,
    fs::{File, write},
    io::{Error, Result},
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScenarioState {
    pub active_scenario: Option<ScenarioRef>,
    pub previous_scenario: Option<ScenarioRef>,
}

impl ScenarioState {
    // load or create new if file not found/error
    pub async fn load(path: &Path) -> Self {
        if path.exists() {
            let state_str = std::fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
            from_str(&state_str).unwrap_or_else(|_| ScenarioState::new())
        } else {
            ScenarioState::new()
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if !path.exists() {
            File::create(path)?;
        }

        let state_str = to_string_pretty(self).map_err(|e| Error::other(e.to_string()))?;

        write(path, state_str)
    }
    pub fn new() -> Self {
        ScenarioState {
            active_scenario: None,
            previous_scenario: None,
        }
    }
    pub async fn set_active_scenario(&mut self, scenario: ScenarioRef) {
        self.previous_scenario = self.active_scenario.clone();
        self.active_scenario = Some(scenario);
    }

    pub fn get_active_scenario(&self) -> Option<&ScenarioRef> {
        if let Some(ref active) = self.active_scenario {
            Some(active)
        } else {
            // try to load from file if active_scenario is None, this handles the case where the state is not in memory but exists on disk
            None
        }
    }
    pub fn get_active_scenario_name(&self) -> Option<String> {
        // load() first, then get the active scenario name if it exists
        self.active_scenario.as_ref().map(|s| s.scenario.clone())
    }
}

impl Default for ScenarioState {
    fn default() -> Self {
        ScenarioState::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioRef {
    pub scenario: String,
    pub state_file: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScenarioExportType {
    Baseline,
    Current,
    Working,
    Snapshot,
}

impl fmt::Display for ScenarioExportType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScenarioExportType::Baseline => write!(f, "baseline"),
            ScenarioExportType::Current => write!(f, "current"),
            ScenarioExportType::Working => write!(f, "working"),
            ScenarioExportType::Snapshot => write!(f, "snapshot"),
        }
    }
}
