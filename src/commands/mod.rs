pub mod activate;
pub mod compare;
// pub mod delete;
pub mod init;
pub mod list;
pub mod new;
// pub mod new;
pub mod reset;
pub mod schema;
pub mod serve;

use anyhow::Result;

use crate::{
    cli::{Cli, Command},
    config::app::AppConfig,
};

pub async fn dispatch(cli: Cli) -> Result<()> {
    let config = AppConfig::load(cli.config.as_deref())?;

    match cli.command {
        Command::Serve(args) => serve::run(args, config).await,
        Command::Schema(args) => schema::run(args, config).await,
        Command::Init(args) => init::run(args, config).await,
        Command::New(args) => new::run(args, config).await,
        Command::Activate(args) => activate::run(args, config).await,
        // Command::List(args) => list::run(args, config).await,
        Command::Compare(args) => compare::run(args, config).await,
        Command::Reset(args) => reset::run(args, config).await,
        // Command::Delete(args) => delete::run(args, config).await,
    }
}
