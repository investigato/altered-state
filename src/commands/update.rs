use anyhow::Result;
use clap::Args;

use crate::{config::scenarios::ScenarioConfig, context::AppContext};

#[derive(Debug, Args)]
pub struct UpdateArgs {
    #[arg(long, help = "Name of the scenario to delete")]
    pub name: String,
    #[arg(
        long = "description",
        short = 'd',
        help = "New description for the scenario"
    )]
    pub description: Option<String>,
    #[arg(
        long = "set-playable",
        help = "Set the active snapshot for the scenario"
    )]
    pub set_playable: Option<String>,
}

pub async fn run(_args: UpdateArgs, _context: AppContext) -> Result<()> {
    if _args.name.is_empty() {
        println!("Scenario name is required for deletion");
        return Ok(());
    }
    if _args.description.is_none() && _args.set_playable.is_none() {
        println!("At least one of --description or --set-playable must be provided");
        return Ok(());
    }

    // see if scenario exists
    let scenario_path = &_context.config.paths.scenarios_directory.join(&_args.name);
    if !scenario_path.exists() {
        println!("Scenario {} does not exist", _args.name);
        return Ok(());
    }
    let config_path = scenario_path.join("config.json");
    if !config_path.exists() {
        println!("Scenario {} does not have a config.json file", _args.name);
        return Ok(());
    }
    let mut scenario_config =
        ScenarioConfig::load_for_scenario(&_context.config.paths.scenarios_directory, &_args.name)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    if let Some(description) = _args.description {
        scenario_config.description = Some(description);
    }
    if let Some(playable) = _args.set_playable {
        scenario_config.playable_state = Some(playable);
    }
    scenario_config.save_to_path(&config_path.to_string_lossy())?;
    println!("Scenario {} has been updated", _args.name);
    Ok(())
}
