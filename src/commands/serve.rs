// use anyhow::Result;
// use clap::Args;

// use crate::{config::app::AppConfig, web};

// #[derive(Debug, Args)]
// pub struct ServeArgs {
// 	#[arg(long, default_value_t = 5000)]
// 	pub port: u16,
// }

// pub async fn run(args: ServeArgs, config: AppConfig) -> Result<()> {
// 	web::server::run(config, args.port).await
// }