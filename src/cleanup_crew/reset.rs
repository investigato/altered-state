use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    cleanup_crew::activate::{ActivateRequest, run as activate_scenario},
    config::app::AppConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetRequest {
    pub scenario: String,
    pub state: Option<String>,
}

pub async fn run(config: AppConfig, request: ResetRequest) -> Result<()> {
    tracing::info!("resetting active scenario");

    config.paths.ensure_directories()?;
    config.logging.ensure_directories()?;

    let target_scenario_directory =
        std::path::Path::new(&config.paths.scenarios_directory).join(&request.scenario);
    if !target_scenario_directory.exists() {
        println!(
            "Target scenario directory {} does not exist.",
            target_scenario_directory.to_string_lossy()
        );
        return Ok(());
    }

    activate_scenario(
        config,
        ActivateRequest {
            scenario: request.scenario.to_string(),
            state: Some("baseline".to_string()),
        },
    )
    .await?;
    Ok(())
}
