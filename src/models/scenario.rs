use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveScenario {
    pub scenario: String,
    pub state: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioState {
    pub active_scenario: Option<ScenarioRef>,
    pub previous_scenario: Option<ScenarioRef>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
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
