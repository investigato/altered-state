use anyhow::Result;
use clap::Args;

use crate::{
    config::app::AppConfig,
    models::scenario::{ScenarioExportType, ScenarioState},
};

#[derive(Debug, Args)]
pub struct CompareArgs {
    #[arg(long = "current")]
    pub current: String,
    #[arg(long = "target")]
    pub target: String,
    #[arg(long = "current-state")]
    pub current_state: String,
    #[arg(long = "target-state")]
    pub target_state: String,
}

pub async fn run(_args: CompareArgs, _config: AppConfig) -> Result<()> {
    println!("Comparing scenarios...");
    // write out all the passed args
    println!("Current scenario: {}", _args.current);
    println!("Target scenario: {}", _args.target);
    println!("Current state: {}", _args.current_state);
    println!("Target state: {}", _args.target_state);
    Ok(())
}
