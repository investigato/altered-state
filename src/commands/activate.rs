use anyhow::{Ok, Result};
use clap::Args;

use crate::{
    cleanup_crew::activate::{ActivateRequest, run as activate_scenario},
    config::app::AppConfig,
};
#[derive(Debug, Args)]
pub struct ActivateArgs {
    #[arg(long)]
    pub scenario: String,

    #[arg(long = "state", default_value = "baseline")]
    pub state: Option<String>,
}

pub async fn run(args: ActivateArgs, config: AppConfig) -> Result<()> {
    if args.scenario.is_empty() {
        println!("Scenario name is required");
        return Ok(());
    }
    let request = ActivateRequest {
        scenario: args.scenario,
        state: args.state,
    };
    activate_scenario(config, request).await
}
