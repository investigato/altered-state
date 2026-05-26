use crate::{
    cleanup_crew::serve::{ServeRequest, ServerState, run as server_run},
    config::scenarios::load_all,
    context::AppContext,
};

use anyhow::Result;
use clap::Args;
use std::sync::{Arc, RwLock};

#[derive(Debug, Args)]
pub struct ServeArgs {
    #[arg(long, default_value_t = 5000)]
    pub port: u16,
}

pub async fn run(args: ServeArgs, context: AppContext) -> Result<()> {
    let config = &context.config;
    let all_scenarios = load_all(&config.paths.scenarios_directory).map_err(|e| {
        anyhow::anyhow!(
            "Failed to load scenarios from directory {}: {}",
            config.paths.scenarios_directory.display(),
            e
        )
    })?;

    let server_state = ServerState {
        context: context.clone(),
        scenarios: all_scenarios,
    };

    let serve_request = ServeRequest {
        port: args.port,
        state: Arc::new(RwLock::new(server_state)),
    };
    server_run(context, serve_request).await?;
    Ok(())
}
