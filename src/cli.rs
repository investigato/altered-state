use crate::commands::{
    activate::ActivateArgs, compare::CompareArgs, delete::DeleteArgs, init::InitArgs,
    list::ListArgs, new::NewScenarioArgs, reset::ResetArgs, serve::ServeArgs,
    snapshot::SnapshotArgs,
};

use clap::{Parser, Subcommand};
#[derive(Debug, Parser)]
#[command(name = "altered-state")]
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
    New(NewScenarioArgs),
    Delete(DeleteArgs),
    Activate(ActivateArgs),
    Snapshot(SnapshotArgs),
    List(ListArgs),
    Compare(CompareArgs),
    Reset(ResetArgs),
}
