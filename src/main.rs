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

pub fn compare_bin_files(current: Vec<DirectoryObject>, stored: Vec<DirectoryObject>) -> bool {
    //   DirectoryObjects keyed by DN, then we tiptoe through the union of keys.
    let current_map: HashMap<String, DirectoryObject> = current
        .into_iter()
        .map(|obj| (obj.dn.clone(), obj))
        .collect();
    let stored_map: HashMap<String, DirectoryObject> = stored
        .into_iter()
        .map(|obj| (obj.dn.clone(), obj))
        .collect();
    let all_dns: HashSet<String> = current_map
        .keys()
        .chain(stored_map.keys())
        .cloned()
        .collect();
    for dn in all_dns {
        let current_obj = current_map.get(&dn);
        let stored_obj = stored_map.get(&dn);
        match (current_obj, stored_obj) {
            (Some(c), Some(s)) => {
                if c.hash != s.hash {
                    println!("Modified: {}", dn);
                    // i'll try to remember to remove this later, but I want to see it now
                    let empty_vals: Vec<String> = Vec::new();
                    for attr in c
                        .attributes
                        .keys()
                        .chain(s.attributes.keys())
                        .collect::<HashSet<_>>()
                    {
                        let c_vals = c.attributes.get(attr).unwrap_or(&empty_vals);
                        let s_vals = s.attributes.get(attr).unwrap_or(&empty_vals);
                        if c_vals != s_vals {
                            println!("  Attribute '{}' differs", attr);
                        }
                    }
                }
            }
            (Some(_), None) => println!("Removed: {}", dn),
            (None, Some(_)) => println!("Added: {}", dn),
            (None, None) => {}
        }
    }
    true
}
