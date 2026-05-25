use anyhow::Result;
use clap::Args;

use crate::config::{app::AppConfig, scenarios::load_all};

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(long = "detailed")]
    pub detailed: bool,
}

pub async fn run(_args: ListArgs, _config: AppConfig) -> Result<()> {
    _config.paths.ensure_directories()?;
    _config.logging.ensure_directories()?;

    let all_scenarios = load_all(&_config.paths.scenarios_directory).map_err(|e| {
        anyhow::anyhow!(
            "Failed to load scenarios from directory {}: {}",
            _config.paths.scenarios_directory.display(),
            e
        )
    })?;
    for scenario in all_scenarios {
        println!("Scenario: {}", scenario.name);
        if _args.detailed {
            for snapshot in scenario.snapshots {
                println!("  Snapshot: {}", snapshot.name);
            }
        }
    }

    Ok(())
}
