pub mod activate;
// pub mod delete;
pub mod init;
pub mod list;
pub mod new;
pub mod update;
pub mod reset;
pub mod snapshot;
pub mod serve;
pub mod delete;
use anyhow::Result;
use crate::context::AppContext;
use crate::{
    cli::{Cli, Command},
    config::app::AppConfig,
};

pub async fn dispatch(cli: Cli) -> Result<()> {
    let config = AppConfig::load(cli.config.as_deref())?;
    let context = AppContext::new(cli.config.as_deref()).await?;


    match cli.command {
        Command::Serve(args) => serve::run(args, context).await,
        Command::Snapshot(args) => snapshot::run(args, context).await,
        Command::Init(args) => init::run(args, context).await,
        Command::New(args) => new::run(args, context).await,
        Command::Activate(args) => activate::run(args, context).await,
        Command::List(args) => list::run(args, context).await,
        Command::Reset(args) => reset::run(args, context).await,
        Command::Delete(args) => delete::run(args, context).await,
        Command::Update(args) => update::run(args, context).await,
        
    }
}
