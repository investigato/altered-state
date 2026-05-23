mod cli;
mod commands;
mod comparison;
mod config;
mod cleanup_crew;
mod ldap;
mod models;
mod remediation;
mod objects;
mod storage;
mod utilities;
mod web;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

use std::error::Error;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .init();

    commands::dispatch(args).await.expect("panic at the disco!");
    Ok(())
}
