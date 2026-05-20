use crate::commands::init::InitArgs;
use clap::{Parser, Subcommand};
#[derive(Debug, Parser)]
#[command(name = "an-app-has-no-name")]
#[command(about = "Clean up your mess")]
pub struct Cli {
    #[arg(long, global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    // Serve(ServeArgs),
    Init(InitArgs),
    // New(NewArgs),
    // Activate(ActivateArgs),
    // List(ListArgs),
    // Compare(CompareArgs),
    // Reset(ResetArgs),
    // Delete(DeleteArgs),
}
