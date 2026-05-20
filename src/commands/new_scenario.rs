use anyhow::Result;
use clap::Args;

use crate::config::AppConfig;

#[derive(Debug, Args)]
pub struct NewScenarioArgs {}

pub async fn run(_args: NewScenarioArgs, _config: AppConfig) -> Result<()> {
	println!("TODO: New Scenario");
	Ok(())
}