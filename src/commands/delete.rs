use anyhow::Result;
use clap::Args;

use crate::{cleanup_crew::activate::ActivateRequest, context::AppContext};

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

pub async fn run(_args: DeleteArgs, _context: AppContext) -> Result<()> {
    if _args.name.is_empty() {
        println!("Scenario name is required for deletion");
        return Ok(());
    }
    let _config = &_context.config;
    let _state = &_context.scenario_state;

    // see if scenario exists
    let scenario_path = _config.paths.scenarios_directory.join(&_args.name);
    if !scenario_path.exists() {
        println!("Scenario {} does not exist", _args.name);
        return Ok(());
    }
    // check scenario_state to see if the scenario is active, if active, activate default scenario first before deletion
    let active_scenario = _state.get_active_scenario();
    if let Some(active) = active_scenario
        && active.scenario == _args.name
    {
        println!(
            "Scenario {} is currently active. Activating default scenario before deletion.",
            _args.name
        );

        crate::cleanup_crew::activate::run(
            _context.clone(),
            ActivateRequest {
                scenario: "default".to_string(),
                state: None,
            },
        )
        .await?;
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
