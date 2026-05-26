use anyhow::Result;
use clap::Args;

use crate::{config::scenarios::load_all, context::AppContext};

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(long = "detailed")]
    pub detailed: bool,
}

pub async fn run(_args: ListArgs, _context: AppContext) -> Result<()> {
    let _config = &_context.config;
    let _state = &_context.scenario_state;
    if let Some(active) = &_state.active_scenario {
        println!("Active scenario: {}", active.scenario);
    } else {
        println!("No active scenario");
    }
    let all_scenarios = load_all(&_config.paths.scenarios_directory).map_err(|e| {
        anyhow::anyhow!(
            "Failed to load scenarios from directory {}: {}",
            _config.paths.scenarios_directory.display(),
            e
        )
    })?;
    for scenario in all_scenarios {
        println!("Scenario: {} ", scenario.name);
        if _args.detailed {
            for snapshot in scenario.snapshots {
                println!("  Snapshot: {}", snapshot.name);
            }
        }
    }

    Ok(())
}
