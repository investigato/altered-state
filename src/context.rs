use crate::{config::app::AppConfig, models::scenario::ScenarioState};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppContext {
    pub config: AppConfig,
    pub scenario_state: ScenarioState,
}

impl AppContext {
    pub async fn new(config_path: Option<&str>) -> Result<Self> {
        let config = AppConfig::load(config_path)?;
        config.paths.ensure_directories()?;
        config.logging.ensure_directories()?;
        let scenario_state = ScenarioState::load(&config.paths.scenario_state_file).await;

        if scenario_state.active_scenario.is_none() {
            let scenarios_dir = &config.paths.scenarios_directory;
            if scenarios_dir.exists() && scenarios_dir.read_dir()?.next().is_some() {
                // if we got here...wtf is going on
                anyhow::bail!(
                    "Invariant violation: No active scenario in state but scenarios directory is not empty"
                );
            }
        }

        Ok(AppContext {
            config,
            scenario_state,
        })
    }
    pub fn save_scenario_state(&self) -> Result<()> // writes to scenario_state_file
    {
        self.scenario_state
            .save(&self.config.paths.scenario_state_file)?;
        Ok(())
    }
}
