use crate::{
    cleanup_crew::serve::{ServeRequest, ServerState, run as server_run},
    config::app::AppConfig,
    config::scenarios::ScenarioConfig,
    web::server::run as run_server,
};
use an_app_has_no_name::ScenarioState;
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

    let all_scenarios = ScenarioConfig::load_all(&config.paths.scenarios_directory).map_err(|e| {
        anyhow::anyhow!(
            "Failed to load scenarios from directory {}: {}",
            config.paths.scenarios_directory.display(),
            e
        )
    })?;
    let active_scenario = ScenarioState::load(&config.paths.scenarios_directory);
    let active_scenario_name = active_scenario.get_active_scenario_name();
    let server_state = ServerState {
        config: config.clone(),
        scenarios: all_scenarios,
        active_scenario: active_scenario_name,
    };

    let serve_request = ServeRequest {
        port: args.port,
        state: server_state,
    };
    server_run(config, serve_request).await?;
    Ok(())
}
