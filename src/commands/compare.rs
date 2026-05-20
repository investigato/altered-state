use anyhow::Result;
use clap::Args;

use crate::config::app::AppConfig;

#[derive(Debug, Args)]
pub struct CompareArgs {}

pub async fn run(_args: CompareArgs, _config: AppConfig) -> Result<()> {
	println!("TODO: compare scenarios");
	Ok(())
}