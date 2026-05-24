use anyhow::Result;
use clap::Args;

use crate::config::app::AppConfig;

#[derive(Debug, Args)]
pub struct ListArgs {}

pub async fn run(_args: ListArgs, _config: AppConfig) -> Result<()> {
	println!("TODO: list scenarios");
	Ok(())
}