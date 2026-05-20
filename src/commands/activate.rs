use anyhow::Result;
use clap::Args;

use crate::{
	config::app::AppConfig,
	// engine::activate::{self, ActivateRequest},
};

#[derive(Debug, Args)]
pub struct ActivateArgs {
	#[arg(long)]
	pub scenario: String,

	#[arg(long)]
	pub state: String,

	#[arg(long)]
	pub dry_run: bool,
}

// pub async fn run(args: ActivateArgs, config: AppConfig) -> Result<()> {
// 	let request = ActivateRequest {
// 		scenario: args.scenario,
// 		state: args.state,
// 		dry_run: args.dry_run,
// 	};

// 	activate::run(config, request).await
// }