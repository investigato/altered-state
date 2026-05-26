use crate::{
    config::scenarios::ScenarioConfig,
    context::AppContext,
    ldap::{ldap_search, prepare_results_from_source},
    models::ldap::generate_ldap_options_from_config,
    models::scenario::{ScenarioExportType, ScenarioRef},
};
use anyhow::Result;
use clap::Args;

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long = "overwrite")]
    pub overwrite: bool,

    #[arg(long = "template")]
    pub template_configuration: Option<String>,
}

pub async fn run(_args: InitArgs, mut _context: AppContext) -> Result<()> {
    let _config = &_context.config;
    let overwrite = _args.overwrite;
    // check command specific options
    // var templateConfigPath = parseResult.GetValue(_templateConfigOption);

    // var scenarioName = "gold";
    // var scenarioDescription = "Baseline for the default scenario";
    let target_scenario_name = "default".to_string();
    let target_scenario_description = "Baseline for the default scenario".to_string();
    let target_scenario_directory =
        std::path::Path::new(&_config.paths.scenarios_directory).join(&target_scenario_name);
    let target_export_type = ScenarioExportType::Baseline;
    // make sure it exists and is empty (if overwrite is true)
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

    // List<ScenarioHook> scenarioScripts;
    // List<Exclusion> scenarioExclusions;
    // bool scenarioIncludeDefaultWatchedAttributes;
    // List<string> scenarioWatchedAttributes;

    // if (templateConfigPath == null)
    // {
    let mut scenario_config = match _args.template_configuration {
        None => ScenarioConfig {
            name: target_scenario_name,
            description: Some(target_scenario_description),
            image_path: None,
            hooks: Vec::new(),
            exclusions: Vec::new(),
            snapshots: Vec::new(),
            playable_state: Some(ScenarioExportType::Baseline.to_string() + ".bin"),
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
                    &_config.paths.images_directory,
                )
                .map_err(|e| anyhow::anyhow!(e))?;
                ScenarioConfig::copy_image_to_directory(
                    &template_image_path,
                    &target_scenario_directory,
                )
                .map_err(|e| anyhow::anyhow!(e))?;
            }
            if !template_config.snapshots.is_empty() {
                println!(
                    "Template scenario has {} snapshots defined, they will be copied to the new scenario folder and included in the new scenario config",
                    template_config.snapshots.len()
                );
                ScenarioConfig::copy_snapshots_to_directory(
                    template_config.snapshots.clone(),
                    &target_scenario_directory,
                )
                .map_err(|e| anyhow::anyhow!(e))?;
            }
            let image_path = template_config.image_path.clone();
            let scenario_scripts = template_config.hooks;
            let scenario_exclusions = template_config.exclusions;
            let scenario_snapshots = template_config.snapshots;
            let playable_state = template_config
                .playable_state
                .as_deref()
                .unwrap_or("baseline.bin")
                .to_string();
            ScenarioConfig {
                name: target_scenario_name,
                description: Some(target_scenario_description),
                image_path,
                hooks: scenario_scripts,
                exclusions: scenario_exclusions,
                snapshots: scenario_snapshots,
                playable_state: Some(playable_state),
            }
        }
    };

    scenario_config.save_to_path(
        &target_scenario_directory
            .join("config.json")
            .to_string_lossy(),
    )?;
    scenario_config
        .finalize_paths(
            &target_scenario_directory,
            &_config.paths.images_directory,
            &target_scenario_directory.join("config.json"),
        )
        .map_err(|e| anyhow::anyhow!("Failed to finalize scenario config paths: {}", e))?;
    // get the schema and write it out to have the system_attributes.yaml file created with the default system attributes
    // AdExporter.ExportSchema(retconConfig);
    // generate ldap options from config file

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
        &_config.never_touch_these_attributes,
        Some(total),
    )
    .await
    .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    // var goldConfigPath = Path.Combine(fullPath, "config.json");
    // try
    // {
    // save it
    let target_scenario_config_path = target_scenario_directory.join("config.json");
    // serialize the config and write it out to the scenario folder
    scenario_config
        .save_to_path(target_scenario_config_path.to_str().unwrap())
        .map_err(|e| anyhow::anyhow!("Failed to write scenario config to path: {}", e))?;

    // activate it

    let active_scenario_ref = ScenarioRef {
        scenario: scenario_config.name.clone(),
        state_file: target_scenario_directory
            .join(format!(
                "{}.bin",
                target_export_type.to_string().to_lowercase()
            ))
            .to_string_lossy()
            .to_string(),
    };

    // update the state file
    _context
        .scenario_state
        .set_active_scenario(active_scenario_ref)
        .await;
    _context
        .save_scenario_state()
        .map_err(|e| anyhow::anyhow!("Failed to update scenario state file: {}", e))?;

    Ok(())
}
