use crate::{
    comparison::comparer::compare_states,
    config::app::AppConfig,
    models::ldap::generate_ldap_options_from_config,
    ldap::{ldap_search, prepare_results_from_source},
    models::scenario::ScenarioExportType,
    objects::directory_objects::read_directory_objects_from_bin_file,
};
use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct CompareArgs {
    #[arg(long = "current")]
    pub current: String,
    #[arg(long = "target")]
    pub target: String,
    #[arg(long = "current-state")]
    pub current_state: String,
    #[arg(long = "target-state")]
    pub target_state: String,
}

pub async fn run(_args: CompareArgs, _config: AppConfig) -> Result<()> {
    println!("Comparing scenarios...");
    // write out all the passed args
    println!("Current scenario: {}", _args.current);
    println!("Target scenario: {}", _args.target);
    println!("Current state: {}", _args.current_state);
    println!("Target state: {}", _args.target_state);

    // current and target are required
    if _args.current.is_empty() || _args.target.is_empty() {
        println!("Current and target scenarios are required");
        return Ok(());
    }
    _config.paths.ensure_directories()?;
    _config.logging.ensure_directories()?;
    // current_state and target_state default to ScenarioExportType::Baseline if not provided
    let current_state = if _args.current_state.is_empty() {
        ScenarioExportType::Baseline
    } else {
        match _args.current_state.to_lowercase().as_str() {
            "baseline" => ScenarioExportType::Baseline,
            "current" => ScenarioExportType::Current,
            "working" => ScenarioExportType::Working,
            "snapshot" => ScenarioExportType::Snapshot,
            _ => {
                println!("Invalid current state provided, defaulting to baseline");
                ScenarioExportType::Baseline
            }
        }
    };

    let target_state = if _args.target_state.is_empty() {
        ScenarioExportType::Baseline
    } else {
        match _args.target_state.to_lowercase().as_str() {
            "baseline" => ScenarioExportType::Baseline,
            "current" => ScenarioExportType::Current,
            "working" => ScenarioExportType::Working,
            "snapshot" => ScenarioExportType::Snapshot,
            _ => {
                println!("Invalid target state provided, defaulting to baseline");
                ScenarioExportType::Baseline
            }
        }
    };

   
    let ldap_options = generate_ldap_options_from_config(&_config);
    let mut ldap_results = Vec::new();
    let total = ldap_search(
        ldap_options,
        &mut ldap_results,
        &_config.paths.naming_contexts_file,
    )
    .await
    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    let exe_path = std::env::current_exe().expect("Failed to get current executable path");
    let base_dir = exe_path.parent().expect("Failed to get parent directory");
    let scenarios_path = base_dir.join(&_config.paths.scenarios_directory);
    let current_scenario_path = scenarios_path.join(&_args.current);
    // create a directory for the current scenario if it doesn't exist
    std::fs::create_dir_all(&current_scenario_path)?;
    let current_export_file = format!("{}.bin", current_state);
    let current_export_path = current_scenario_path.join(current_export_file);
    let current_schema_output_path = current_scenario_path.join("schema_attributes.json");

    //
    let results = prepare_results_from_source(
        ldap_results,
        &_config.domain,
        &current_export_path,
        false,
        &current_schema_output_path,
        Some(total),
    )
    .await
    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    let directory_objects = results.directory_objects.clone();
    let target_scenario_path = scenarios_path.join(&_args.target);
    let target_scenario_export_file = format!("{}.bin", target_state);
    let target_export_path = target_scenario_path.join(target_scenario_export_file);
    let target_scenario_exported_objects =
        read_directory_objects_from_bin_file(&target_export_path)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    compare_states(directory_objects, target_scenario_exported_objects)
        .await
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    Ok(())
}
