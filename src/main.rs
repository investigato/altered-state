pub(crate) mod cleanup_crew;
pub(crate) mod cli;
pub(crate) mod commands;
pub(crate) mod comparison;
pub(crate) mod config;
pub(crate) mod ldap;
pub(crate) mod models;
pub(crate) mod objects;
pub(crate) mod remediation;
pub(crate) mod storage;
pub(crate) mod utilities;
pub(crate) mod web;

extern crate bitflags;
extern crate chrono;
extern crate regex;

// Reimport key functions and structure
#[doc(inline)]
pub use ldap::ldap_search;
#[doc(inline)]
pub use ldap3::SearchEntry;

pub use comparison::comparer::compare_states;
pub use config::scenarios::{ScenarioConfig, ScenarioHookConfig, ScenarioHookType};
pub use ldap::prepare_results_from_source;
pub use models::scenario::{ScenarioExportType, ScenarioRef, ScenarioState};
pub use objects::{attribute::SchemaEntry, directory_objects::save_directory_objects_to_bin_file};
pub use remediation::command_generator::generate_commands;
pub use storage::{DiskStorage, DiskStorageReader, EntrySource, Storage};
pub use utilities::banner::print_banner;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

use std::error::Error;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .init();

    commands::dispatch(args).await.expect("panic at the disco!");
    Ok(())
}
