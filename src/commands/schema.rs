use anyhow::Result;
use clap::Args;

use crate::config::app::AppConfig;

#[derive(Debug, Args)]
pub struct SchemaArgs {}

pub async fn run(args: SchemaArgs, config: AppConfig) -> Result<()> {
    Ok(())
}
