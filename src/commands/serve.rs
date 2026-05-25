use crate::{
    cleanup_crew::serve::{ServeRequest, ServerState, run as server_run},
    config::app::AppConfig,
    config::scenarios::load_all,
    models::scenario::ScenarioState,
};

use anyhow::Result;
use clap::Args;
use std::sync::{Arc, RwLock};

#[derive(Debug, Args)]
pub struct ServeArgs {
    #[arg(long, default_value_t = 5000)]
    pub port: u16,
}

pub async fn run(args: ServeArgs, config: AppConfig) -> Result<()> {
    config.paths.ensure_directories()?;
    config.logging.ensure_directories()?;

    let all_scenarios =
        load_all(&config.paths.scenarios_directory).map_err(|e| {
            anyhow::anyhow!(
                "Failed to load scenarios from directory {}: {}",
                config.paths.scenarios_directory.display(),
                e
            )
        })?;
    let active_scenario = ScenarioState::load(&config.paths.scenario_state_file).await;
    let scenario_name = if let Some(ref scenario) = active_scenario.active_scenario {
        tracing::info!("Loaded active scenario: {:?}", scenario);
        Some(scenario.scenario.clone())
    } else {
        tracing::info!("No active scenario found");
        None
    };
    let server_state = ServerState {
        config: config.clone(),
        scenarios: all_scenarios,
        active_scenario: scenario_name,
    };

    let serve_request = ServeRequest {
        port: args.port,
        state: Arc::new(RwLock::new(server_state)),
    };
    server_run(config, serve_request).await?;
    Ok(())
}
