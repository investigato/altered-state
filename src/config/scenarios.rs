use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::config::app::ExclusionConfig;
use crate::models::scenario::ScenarioState;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConfig {
	pub name: String,
	pub description: Option<String>,
	pub image: Option<String>,
	pub states: Vec<ScenarioState>,
	pub hooks: Vec<ScenarioHookConfig>,
	pub exclusions: Vec<ExclusionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioHookConfig {
	pub hook_type: ScenarioHookType,
	pub path: PathBuf,
	pub arguments: Vec<String>,
	pub run_as: Option<RunAs>,
	pub timeout_seconds: Option<u64>,
	pub continue_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScenarioHookType {
	PreAction,
	Cleanup,
	Activation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunAs {
	LocalSystem,
	CurrentUser,
}