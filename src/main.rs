mod cli;
mod commands;
mod config;
mod ldap;
mod models;
mod objects;
mod storage;
mod utilities;
use crate::objects::directory_objects::{DirectoryObject, read_directory_objects_from_bin_file};
use anyhow::Result;
use clap::Parser;
use cli::Cli;
use ldap::{LdapOptions, ldap_search};
use std::collections::{HashMap, HashSet};
use std::error::Error;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("an-app-has-no-name=debug,info")
        .init();

    let cli = Cli::parse();
    commands::dispatch(cli).await.expect("TODO: panic message");
    // let scenario_name = "esc1".to_string();
    // let config = config::app::AppConfig::load(cli.config.as_deref())?;
    // config.paths.ensure_directories()?;
    // config.logging.ensure_directories()?;
    // let ldap_options = LdapOptions {
    //     domain: config.domain.clone(),
    //     ldapfqdn: config.hostname.clone(),
    //     ip: Some("127.0.0.1".to_string()),
    //     port: Some(389),
    //     ldaps: false,
    //     ldap_filter: Some("(objectClass=*)".to_string()),
    // };
    // let mut ldap_results = Vec::new();
    // let total = ldap_search(
    //     ldap_options,
    //     &mut ldap_results,
    //     ldap::CollectionScope::FullDirectory,
    // )
    // .await
    // .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    // // print the total
    // let exe_path = std::env::current_exe().expect("Failed to get current executable path");
    // let base_dir = exe_path.parent().expect("Failed to get parent directory");
    // let scenarios_path = base_dir.join(&config.paths.scenarios_directory);
    // let scenario_path = scenarios_path.join(scenario_name);
    // std::fs::create_dir_all(&scenario_path)?;
    // let export_path = scenario_path.join("export.bin");
    // let schema_output_path = scenario_path.join("schema_attributes.yaml");
    //
    // let results = ldap::prepare_results_from_source(
    //     ldap_results,
    //     &config.domain,
    //     &export_path,
    //     &schema_output_path,
    //     Some(total),
    // )
    // .await
    // .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    //
    // // get the directory_objects to be compared against the .bin file
    // let directory_objects = results.directory_objects.clone();
    // let gold_path = scenarios_path.join("gold");
    // let gold_export_path = gold_path.join("export.bin");
    // let written_directory_objects = read_directory_objects_from_bin_file(&gold_export_path)
    //     .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    //
    // compare_bin_files(directory_objects, written_directory_objects);
    // println!("Total entries retrieved: {}", total);
    Ok(())

    // 	// let cli = Cli::parse();
    // 	// commands::dispatch(cli).await
}
