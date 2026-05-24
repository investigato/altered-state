use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    comparison::comparer::compare_states,
    config::app::AppConfig,
    config::scenarios::ScenarioConfig,
    config::scenarios::{ScenarioHookConfig, ScenarioHookType},
    ldap::{ldap_search, prepare_results_from_source},
    models::ldap::generate_ldap_options_from_config,
    models::scenario::{ScenarioExportType, ScenarioRef, ScenarioState},
    objects::directory_objects::read_directory_objects_from_bin_file,
    remediation::command_generator::generate_commands,
    utilities::hooks::execute_hooks,
    utilities::scripts::{execute_script, write_ps1},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateRequest {
    pub scenario: String,
    pub state: Option<String>,
}

pub async fn run(config: AppConfig, request: ActivateRequest) -> Result<()> {
    tracing::info!("activating scenario {}", request.scenario);
    config.paths.ensure_directories()?;
    config.logging.ensure_directories()?;
    let target_name = request.scenario;
    let target_state = request.state.unwrap_or_else(|| "baseline".to_string());
    // load the scenario file and make sure it exists and that we aren't trying to load the current scenario
    tracing::info!(
        "loading scenario state from file: {:?}",
        config.paths.scenario_state_file
    );
    let mut scenario_state = ScenarioState::load(&config.paths.scenario_state_file).await;
    let current_scenario = scenario_state.get_active_scenario().cloned();
    if let Some(current) = current_scenario.as_ref()
        && current.scenario == target_name
    {
        println!("Scenario {} is already active", target_name);
        return Ok(());
    }
    let current_config = if let Some(current) = current_scenario.as_ref() {
        Some(
            ScenarioConfig::load_for_scenario(&config.paths.scenarios_directory, &current.scenario)
                .map_err(|e| anyhow::Error::msg(e.to_string()))?,
        )
    } else {
        None
    };
    let target_scenario_directory =
        std::path::Path::new(&config.paths.scenarios_directory).join(&target_name);
    // check for the corresponding .bin file in the directory
    let target_scenario_file =
        target_scenario_directory.join(format!("{}.bin", target_state.to_lowercase()));
    if !target_scenario_file.exists() {
        println!(
            "Scenario file {} does not exist",
            target_scenario_file.to_string_lossy()
        );
        return Ok(());
    }

    // read the target_scenario_file to make sure it's a valid scenario file
    let target_scenario_exported_objects =
        read_directory_objects_from_bin_file(&target_scenario_file)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    // get the "where are we now?"
    let ldap_options = generate_ldap_options_from_config(&config);
    let mut ldap_results = Vec::new();
    let total = ldap_search(
        ldap_options,
        &mut ldap_results,
        &config.paths.naming_contexts_file,
    )
    .await
    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    let current_scenario_path = &config.paths.scenarios_directory.join(
        current_scenario
            .as_ref()
            .map_or("current_scenario", |s| &s.scenario),
    );
    // create a directory for the current scenario if it doesn't exist
    std::fs::create_dir_all(current_scenario_path)?;
    let current_export_file = format!(
        "{}.bin",
        ScenarioExportType::Current.to_string().to_lowercase()
    );
    let current_export_path = current_scenario_path.join(current_export_file);
    let current_schema_output_path = &config
        .paths
        .scenarios_directory
        .join("schema_attributes.json");
    //
    let results = prepare_results_from_source(
        ldap_results,
        &config.domain,
        &current_export_path,
        false,
        current_schema_output_path,
        &config.never_touch_these_attributes,
        Some(total),
    )
    .await
    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    let directory_objects = results.directory_objects.clone();

    let actions = compare_states(directory_objects, target_scenario_exported_objects)
        .await
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let commands = generate_commands(
        actions,
        &config.paths.naming_contexts_file,
        &config.paths.schema_attributes_file,
    );
    // write_to_console(&commands);

    // get scenario state
    let scenario_config_path = target_scenario_directory.join("config.json");
    if !scenario_config_path.exists() {
        println!(
            "Scenario config file {} does not exist",
            scenario_config_path.to_string_lossy()
        );
        return Ok(());
    }

    // hook shot
    // first cleanup from active_scenario if it exists BEFORE we change
    // does current_scenario have hooks?
    if let Some(current_config) = current_config.as_ref() {
        let cleanup_hooks: Vec<ScenarioHookConfig> = current_config
            .hooks
            .iter()
            .filter(|h| h.hook_type == ScenarioHookType::Cleanup)
            .cloned()
            .collect();

        let hook_outputs = execute_hooks(
            &cleanup_hooks,
            ScenarioHookType::Cleanup,
            current_scenario_path,
        );
        if let Ok(outputs) = hook_outputs {
            for output in outputs {
                println!(
                    "Executed cleanup hook: {:?}, success: {}, stdout: {}, stderr: {}",
                    output.path, output.success, output.stdout, output.stderr
                );
            }
        }
    }
    if let Some(current_config) = current_config.as_ref() {
        let preaction_hooks: Vec<ScenarioHookConfig> = current_config
            .hooks
            .iter()
            .filter(|h| h.hook_type == ScenarioHookType::PreAction)
            .cloned()
            .collect();
        let hook_outputs = execute_hooks(
            &preaction_hooks,
            ScenarioHookType::PreAction,
            current_scenario_path,
        );
        if let Ok(outputs) = hook_outputs {
            for output in outputs {
                println!(
                    "Executed pre-action hook: {:?}, success: {}, stdout: {}, stderr: {}",
                    output.path, output.success, output.stdout, output.stderr
                );
            }
        }
    }

    write_ps1(&commands, &config.paths.actions_script_file);
    execute_script(&config.paths.actions_script_file.to_string_lossy());

    // if we have an active scenario, move it to previous scenario
    if let Some(current) = current_scenario.as_ref() {
        scenario_state.previous_scenario = Some(current.clone());
    }
    // set the new active scenario
    scenario_state.active_scenario = Some(ScenarioRef {
        scenario: target_name.clone(),
        state_file: target_scenario_file.to_string_lossy().to_string(),
    });
    // save the scenario state
    scenario_state.save(&config.paths.scenario_state_file)?;
    if let Some(current) = current_scenario.as_ref() {
        println!(
            "Scenario {} with state {} is now active. Previous scenario was {} with state {}",
            target_name, target_state, current.scenario, current.state_file
        );
    } else {
        println!(
            "Scenario {} with state {} is now active. No previous scenario.",
            target_name, target_state
        );
    }

    // does new active_scenario have hooks?
    let new_config =
        ScenarioConfig::load_for_scenario(&config.paths.scenarios_directory, &target_name)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let activation_hooks: Vec<ScenarioHookConfig> = new_config
        .hooks
        .iter()
        .filter(|h| h.hook_type == ScenarioHookType::Activation)
        .cloned()
        .collect();
    let hook_outputs = execute_hooks(
        &activation_hooks,
        ScenarioHookType::Activation,
        &target_scenario_directory,
    );
    if let Ok(outputs) = hook_outputs {
        for output in outputs {
            println!(
                "Executed activation hook: {:?}, success: {}, stdout: {}, stderr: {}",
                output.path, output.success, output.stdout, output.stderr
            );
        }
    }

    Ok(())
}
