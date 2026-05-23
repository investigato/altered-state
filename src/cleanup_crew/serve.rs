use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    config::app::AppConfig, config::scenarios::ScenarioConfig, web::server::run as web_server_start,
};
pub struct ServeRequest {
    pub port: u16,
    pub state: ServerState,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    pub config: AppConfig,
    pub scenarios: Vec<ScenarioConfig>,
    pub active_scenario: Option<String>,
}
pub async fn run(config: AppConfig, request: ServeRequest) -> Result<()> {
    tracing::info!("resetting active scenario");

    web_server_start(config, request.state, request.port).await?;
    Ok(())
}
