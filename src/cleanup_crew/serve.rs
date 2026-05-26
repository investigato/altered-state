use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use crate::{
    config::scenarios::ScenarioConfig, context::AppContext, web::server::run as web_server_start,
};
pub struct ServeRequest {
    pub port: u16,
    pub state: Arc<RwLock<ServerState>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    pub context: AppContext,
    pub scenarios: Vec<ScenarioConfig>,
}
pub async fn run(context: AppContext, request: ServeRequest) -> Result<()> {
    tracing::info!("resetting active scenario");

    web_server_start(context, request.state, request.port).await?;
    Ok(())
}
