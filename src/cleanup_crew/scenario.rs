use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    cleanup_crew::activate::{ActivateRequest, run as activate_scenario},
    config::app::AppConfig,config::scenarios::ScenarioResponseDto,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRequest {
    pub scenario: String,
}


pub async fn run(config: AppConfig, request: ListRequest) -> Result<Vec<ScenarioResponseDto>, anyhow::Error> {
    tracing::info!("listing active scenario");

    config.paths.ensure_directories()?;
    config.logging.ensure_directories()?;

    
  
    Ok(())
}
