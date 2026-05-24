use anyhow::Result;
use clap::Args;

use crate::{
    cleanup_crew::reset::{ResetRequest, run as reset_scenario},
    config::app::AppConfig,
};
#[derive(Debug, Args)]
pub struct ResetArgs {
    #[arg(long)]
    pub name: String,
}

pub async fn run(_args: ResetArgs, _config: AppConfig) -> Result<()> {
    let target_name = _args.name;
    if target_name.is_empty() {
        println!("Target scenario name cannot be empty.");
        return Ok(());
    }
    let request = ResetRequest {
        scenario: target_name,
        state: Some("baseline".to_string()),
    };
    reset_scenario(_config, request).await
}
