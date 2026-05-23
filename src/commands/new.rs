use anyhow::Result;
use clap::Args;

use crate::{
    config::{app::AppConfig, scenarios::ScenarioConfig},
    ldap::{ldap_search, prepare_results_from_source},
    models::ldap::generate_ldap_options_from_config,
    models::scenario::{ScenarioExportType, ScenarioRef, ScenarioState},
};

// private readonly Option<bool> _activateOption;
// private readonly Option<string> _descriptionOption;
// private readonly Option<string> _nameOption;
#[derive(Debug, Args)]
pub struct NewScenarioArgs {
    #[arg(long = "overwrite", short = 'o')]
    pub overwrite: bool,
    #[arg(long = "template", short = 't')]
    pub template_configuration: Option<String>,
    #[arg(long = "description", short = 'd')]
    pub description: Option<String>,
    #[arg(long = "name", short = 'n')]
    pub name: String,
}

pub async fn run(_args: NewScenarioArgs, _config: AppConfig) -> Result<()> {
    _config.paths.ensure_directories()?;
    _config.logging.ensure_directories()?;
    let overwrite = _args.overwrite;

    let description = match _args.description {
        Some(desc) => desc,
        None => _args.name.to_string(),
    };
    let name = _args.name;
    let target_scenario_directory =
        std::path::Path::new(&_config.paths.scenarios_directory).join(&name);
    let target_export_type = ScenarioExportType::Baseline;
    let target_export_path = target_scenario_directory.join(format!(
        "{}.bin",
        target_export_type.to_string().to_lowercase()
    ));
    if target_scenario_directory.exists() {
        if overwrite {
            std::fs::remove_dir_all(&target_scenario_directory)?;
        } else {
            println!(
                "Target scenario directory {} already exists. Use --overwrite to overwrite it.",
                target_scenario_directory.to_string_lossy()
            );
            return Ok(());
        }
    }
    std::fs::create_dir_all(&target_scenario_directory)?;
    let scenario_config = match _args.template_configuration {
        None => ScenarioConfig {
            name: name.clone(),
            description: Some(description.clone()),
            image_path: None,
            hooks: Vec::new(),
            exclusions: Vec::new(),
        },
        Some(template_configuration) => {
            let template_config = ScenarioConfig::load_from_path(template_configuration.as_str())
                .map_err(|e| {
                anyhow::anyhow!("Failed to load template scenario config: {}", e)
            })?;

            if !template_config.hooks.is_empty() {
                println!(
                    "Template scenario has {} scripts defined, they will be copied to the new scenario folder and included in the new scenario config",
                    template_config.hooks.len()
                );
                ScenarioConfig::copy_scripts_to_directory(
                    template_config.hooks.clone(),
                    &target_scenario_directory,
                )
                .map_err(|e| anyhow::anyhow!(e))?;
            }

            if let Some(template_image) = template_config.image_path.as_ref() {
                println!(
                    "Template scenario has an image defined at {}, it will be copied to the new scenario folder and included in the new scenario config",
                    template_image
                );
                let template_image_path = std::path::PathBuf::from(template_image);
                ScenarioConfig::copy_image_to_directory(
                    &template_image_path,
                    &target_scenario_directory,
                )
                .map_err(|e| anyhow::anyhow!(e))?;
            }
            let image_path = template_config.image_path.clone();
            let scenario_scripts = template_config.hooks;
            let scenario_exclusions = template_config.exclusions;

            ScenarioConfig {
                name: name.clone(),
                description: Some(description.clone()),
                image_path,
                hooks: scenario_scripts,
                exclusions: scenario_exclusions,
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
    prepare_results_from_source(
        ldap_results,
        &_config.domain,
        &target_export_path,
        true,
        &_config.paths.schema_attributes_file,
        Some(total),
    )
    .await
    .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let target_scenario_config_path = target_scenario_directory.join("config.json");
    // serialize the config and write it out to the scenario folder
    scenario_config
        .save_to_path(target_scenario_config_path.to_str().unwrap())
        .map_err(|e| anyhow::anyhow!("Failed to write scenario config to path: {}", e))?;

    // activate it
    let mut scenario_state = ScenarioState::load(&_config.paths.scenario_state_file);
    let scenario_ref = ScenarioRef {
        scenario: scenario_config.name.clone(),
        state_file: format!("{}.bin", target_export_type.to_string().to_lowercase()),
    };
    // update the state file
    scenario_state.set_active_scenario(scenario_ref);
    scenario_state
        .save(&_config.paths.scenario_state_file)
        .map_err(|e| anyhow::anyhow!("Failed to update scenario state file: {}", e))?;

    // activate_scenario(
    //     _config,
    //     ActivateRequest {
    //         scenario: name,
    //         state: Some(target_export_type.to_string()),
    //     },
    // )
    // .await?;

    //   ScenarioManager.ExecuteScripts(config,newActiveScenario, ScriptType.Activation,config.PowerShellExecutableLocation);

    Ok(())
}
