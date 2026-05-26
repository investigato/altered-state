use anyhow::Result;
use clap::Args;

use crate::{
    cleanup_crew::reset::{ResetRequest, run as reset_scenario},
    context::AppContext,
    models::scenario::ScenarioState,
};
#[derive(Debug, Args)]
pub struct ResetArgs {
    #[arg(long)]
    pub name: String,
}

pub async fn run(_args: ResetArgs, _context: AppContext) -> Result<()> {
    // if _args.name is empty, read the _config.scenario_state and use that as the target name, otherwise use _args.name
    let _config = &_context.config;
    let target_name: Option<String>;
    if !_args.name.is_empty() {
        target_name = Some(_args.name);
    } else {
        let scenario_state = ScenarioState::load(&_config.paths.scenario_state_file).await;
        target_name = Some(
            scenario_state
                .active_scenario
                .clone()
                .map_or_else(|| "default".to_string(), |s| s.scenario),
        );
    }

    let request = ResetRequest {
        scenario: target_name.unwrap_or_else(|| "default".to_string()),
    };
    reset_scenario(_context, request).await
}
