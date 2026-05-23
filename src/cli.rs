use crate::commands::{
    activate::ActivateArgs, compare::CompareArgs, init::InitArgs, new::NewScenarioArgs,
    reset::ResetArgs, schema::SchemaArgs,serve::ServeArgs
};

use clap::{Parser, Subcommand};
#[derive(Debug, Parser)]
#[command(name = "an-app-has-no-name")]
#[command(about = "Clean up your mess")]
pub struct Cli {
    #[arg(long, global = true)]
    pub config: Option<String>,
    #[command(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Serve(ServeArgs),
    Init(InitArgs),
    Schema(SchemaArgs),
    New(NewScenarioArgs),
    Activate(ActivateArgs),
    // List(ListArgs),
    Compare(CompareArgs),
    Reset(ResetArgs),
    // Delete(DeleteArgs),
}
