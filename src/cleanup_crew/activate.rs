use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    comparison::comparer::compare_states,
    config::scenarios::ScenarioConfig,
    config::scenarios::{ScenarioHookConfig, ScenarioHookType},
    context::AppContext,
    ldap::{ldap_search, prepare_results_from_source},
    models::ldap::generate_ldap_options_from_config,
    models::scenario::{ScenarioExportType, ScenarioRef},
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

pub async fn run(mut context: AppContext, request: ActivateRequest) -> Result<()> {
    tracing::info!("activating scenario {}", request.scenario);
    let config = &context.config;

    let target_name = request.scenario;
    let target_scenario_directory =
        std::path::Path::new(&config.paths.scenarios_directory).join(&target_name);
    // if scenario doesn't exist, error out
    if !target_scenario_directory.exists() {
        println!(
            "Scenario directory {} does not exist",
            target_scenario_directory.to_string_lossy()
        );
        return Ok(());
    }
    // load the config file, if error, then error out
    if !target_scenario_directory.join("config.json").exists() {
        println!(
            "Scenario config file {} does not exist",
            target_scenario_directory
                .join("config.json")
                .to_string_lossy()
        );
        return Ok(());
    }
    let target_scenario_config =
        ScenarioConfig::load_for_scenario(&config.paths.scenarios_directory, &target_name)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    // Resolve target file:
    // - request.state is Some → format!("{}.bin", state) → if file doesn't exist, exit
    // - request.state is None → playable_state from config, fallback to "baseline" → same existence check
    let target_state = if let Some(state) = request.state {
        // if it doesn't end with bin, add .bin
        if !state.to_lowercase().ends_with(".bin") {
            let formatted_state = format!("{}.bin", state);
            if !target_scenario_directory.join(&formatted_state).exists() {
                println!(
                    "Scenario file {} does not exist",
                    target_scenario_directory
                        .join(&formatted_state)
                        .to_string_lossy()
                );
                return Ok(());
            }
            formatted_state
        } else {
            if !target_scenario_directory.join(&state).exists() {
                println!(
                    "Scenario file {} does not exist",
                    target_scenario_directory.join(&state).to_string_lossy()
                );
                return Ok(());
            }
            state
        }
    } else if let Some(playable) = target_scenario_config.playable_state {
        // ensure the format is correct with ScenarioExportType::Type.bin
        if !playable.to_lowercase().ends_with(".bin") {
            let formatted_playable = format!("{}.bin", playable);
            if !target_scenario_directory.join(&formatted_playable).exists() {
                println!(
                    "Scenario file {} does not exist",
                    target_scenario_directory
                        .join(&formatted_playable)
                        .to_string_lossy()
                );
                return Ok(());
            }
            formatted_playable
        } else {
            playable
        }
    } else {
        ScenarioExportType::Baseline.to_string() + ".bin"
    };
    let target_scenario_file = target_scenario_directory.join(target_state.to_lowercase());
    if !target_scenario_file.exists() {
        println!(
            "Scenario file {} does not exist",
            target_scenario_file.to_string_lossy()
        );
        return Ok(());
    }

    // check if this scenario is already active, exit if so

    // load current scenario's config (needed for cleanup/pre-action hooks)

    // whoops, this should never ever be none at this point :-(
    let current_config = ScenarioConfig::load_for_scenario(
        &config.paths.scenarios_directory,
        &context
            .scenario_state
            .get_active_scenario()
            .ok_or_else(|| {
                anyhow::Error::msg("Expected an active scenario in state, but none was found")
            })?
            .scenario,
    )
    .ok();

    // get the "where are we now?"
    let ldap_options = generate_ldap_options_from_config(config);
    let mut ldap_results = Vec::new();
    let total = ldap_search(
        ldap_options,
        &mut ldap_results,
        &config.paths.naming_contexts_file,
    )
    .await
    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    let current = context
        .scenario_state
        .get_active_scenario()
        .ok_or_else(|| {
            anyhow::Error::msg("Expected an active scenario in state, but none was found")
        })?;
    let current_scenario_path = config.paths.scenarios_directory.join(&current.scenario);
    // create a directory for the current scenario if it doesn't exist
    std::fs::create_dir_all(&current_scenario_path)?;
    let temp_export_file = format!(
        "{}.bin",
        ScenarioExportType::Current.to_string().to_lowercase()
    );
    let temp_export_path = config.paths.temp_directory.join(&temp_export_file);
    let current_schema_output_path = &config
        .paths
        .scenarios_directory
        .join("schema_attributes.json");
    //
    let results = prepare_results_from_source(
        ldap_results,
        &config.domain,
        &temp_export_path,
        false,
        current_schema_output_path,
        &config.never_touch_these_attributes,
        Some(total),
    )
    .await
    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    let directory_objects = results.directory_objects.clone();
    let target_scenario_exported_objects =
        read_directory_objects_from_bin_file(&target_scenario_file)
            .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    //  compare_states → generate_commands

    let actions = compare_states(directory_objects, target_scenario_exported_objects)
        .await
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let commands = generate_commands(
        actions,
        &config.paths.naming_contexts_file,
        &config.paths.schema_attributes_file,
    );

    //  run cleanup hooks on current scenario
    // let scenario_config_path = target_scenario_directory.join("config.json");
    // if !scenario_config_path.exists() {
    //     println!(
    //         "Scenario config file {} does not exist",
    //         scenario_config_path.to_string_lossy()
    //     );
    //     return Ok(());
    // }

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
            &current_scenario_path,
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

    //  run pre-action hooks on current scenario
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
            &current_scenario_path,
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

    //  write_ps1 + execute_script
    write_ps1(&commands, &config.paths.actions_script_file);
    execute_script(&config.paths.actions_script_file.to_string_lossy());

    //  update and save ScenarioState
    let active_scenario_ref = ScenarioRef {
        scenario: target_name.clone(),
        state_file: target_scenario_directory
            .join(target_state.to_lowercase())
            .to_string_lossy()
            .to_string(),
    };
    // update the state file
    context
        .scenario_state
        .set_active_scenario(active_scenario_ref)
        .await;
    // save the scenario state
    context
        .save_scenario_state()
        .map_err(|e| anyhow::anyhow!("Failed to update scenario state file: {}", e))?;

    //  run activation hooks on new scenario
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
    //  print result

    println!(
        "Scenario {} with state {} is now active.",
        target_name, target_state,
    );

    Ok(())
}
