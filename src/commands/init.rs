use anyhow::Result;
use clap::Args;

use crate::config::app::AppConfig;

#[derive(Debug, Args)]
pub struct InitArgs {}

pub async fn run(_args: InitArgs, _config: AppConfig) -> Result<()> {
	println!("TODO: initialize default scenario");
	Ok(())
}