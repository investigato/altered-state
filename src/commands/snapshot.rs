use anyhow::Result;

use crate::{
    config::{
        app::AppConfig,
        scenarios::{ScenarioConfig, SnapshotEntry},
    },
    ldap::{ldap_search, prepare_results_from_source},
    models::{
        ldap::generate_ldap_options_from_config,
        scenario::{ScenarioExportType, ScenarioState},
    },
};
use clap::Args;

#[derive(Debug, Args)]
pub struct SnapshotArgs {
    #[arg(long = "scenario")]
    pub scenario: String,
    #[arg(long = "description")]
    pub description: String,
}

pub async fn run(args: SnapshotArgs, config: AppConfig) -> Result<()> {
    if args.description.is_empty() {
        println!("Description is required");
        return Ok(());
    }
    config.paths.ensure_directories()?;
    config.logging.ensure_directories()?;
    let description = args.description;
    let scenario_state = ScenarioState::load(&config.paths.scenario_state_file).await;
    let current_scenario = scenario_state.get_active_scenario().cloned();
    // if args.scenario.is_empty() { get current scenario? }
    let scenario_name = match args.scenario.is_empty() {
        true => {
            if let Some(current) = current_scenario.as_ref() {
                println!(
                    "No scenario provided, using current active scenario: {}",
                    current.scenario
                );
                current.scenario.clone()
            } else {
                println!("No scenario provided and no active scenario found");
                return Ok(());
            }
        }
        false => args.scenario,
    };

    let scenario_config =
        ScenarioConfig::load_for_scenario(&config.paths.scenarios_directory, &scenario_name)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    let current_scenario_path = config.paths.scenarios_directory.join(&scenario_name);
    let export_path = match current_scenario_path.exists() {
        true => {
            println!(
                "Scenario {} already exists, adding a snapshot",
                scenario_name
            );
            let next_snapshot_number = scenario_config.snapshots.len() + 1;
            let snapshot_name = format!("snapshot-{}", next_snapshot_number);
            current_scenario_path.join(format!("{}.bin", snapshot_name.to_lowercase()))
        }
        false => {
            std::fs::create_dir_all(&current_scenario_path)?;
            current_scenario_path.join(format!(
                "{}.bin",
                ScenarioExportType::Baseline.to_string().to_lowercase()
            ))
        }
    };

    // LDAP section
    let ldap_options = generate_ldap_options_from_config(&config);
    let mut ldap_results = Vec::new();
    let total = ldap_search(
        ldap_options,
        &mut ldap_results,
        &config.paths.naming_contexts_file,
    )
    .await
    .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let current_schema_output_path = &config
        .paths
        .scenarios_directory
        .join("schema_attributes.json");
    //
    prepare_results_from_source(
        ldap_results,
        &config.domain,
        &export_path,
        false,
        current_schema_output_path,
        &config.never_touch_these_attributes,
        Some(total),
    )
    .await
    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    // we need to add a snapshot entry to the config

    let mut scenario_config = scenario_config.clone();
    let snapshot_entry = SnapshotEntry {
        name: format!("snapshot-{}", scenario_config.snapshots.len() + 1),
        description,
        file_path: export_path.to_string_lossy().to_string(),
        created_at: chrono::Utc::now().to_string(),
    };
    scenario_config.snapshots.push(snapshot_entry);
    scenario_config.save_to_path(&current_scenario_path.join("config.json").to_string_lossy())?;
    println!(
        "Snapshot saved successfully at {}",
        export_path.to_string_lossy()
    );

    Ok(())
}
