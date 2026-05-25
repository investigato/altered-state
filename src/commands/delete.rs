use anyhow::Result;
use clap::Args;

use crate::config::app::AppConfig;

#[derive(Debug, Args)]
pub struct DeleteArgs {
    #[arg(long, help = "Name of the scenario to delete")]
    pub name: String,
    #[arg(
        long = "force",
        short = 'f',
        help = "Force delete without confirmation"
    )]
    pub force: bool,
}

pub async fn run(_args: DeleteArgs, _config: AppConfig) -> Result<()> {
    if _args.name.is_empty() {
        println!("Scenario name is required for deletion");
        return Ok(());
    }
    _config.paths.ensure_directories()?;
    _config.logging.ensure_directories()?;
    // see if scenario exists
    let scenario_path = _config.paths.scenarios_directory.join(&_args.name);
    if !scenario_path.exists() {
        println!("Scenario {} does not exist", _args.name);
        return Ok(());
    }
    if !_args.force {
        println!(
            "Are you sure you want to delete scenario {}? This action cannot be undone. (y/N)",
            _args.name
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Cancelling deletion of scenario {}", _args.name);
            return Ok(());
        }
    }
    std::fs::remove_dir_all(&scenario_path)?;
    println!("Scenario {} has been deleted", _args.name);

    Ok(())
}
