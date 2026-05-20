mod cli;
mod commands;
mod config;
mod ldap;
mod objects;
mod storage;
mod utilities;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use ldap::LdapOptions;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("an-app-has-no-name=debug,info")
        .init();

    let cli = Cli::parse();
    let config = config::app::AppConfig::load(cli.config.as_deref())?;
    let ldap_options = LdapOptions {
        domain: config.domain.clone(),
        ldapfqdn: config.ldap.ldapfqdn.clone(),
        ip: Some("127.0.0.1".to_string()),
        port: Some(config.ldap.port),
        ldaps: config.ldap.ldaps,
        ldap_filter: Some("(objectClass=*)".to_string()),
    };
    let mut ldap_results = Vec::new();
    let total = ldap::ldap_search(
        ldap_options,
        &mut ldap_results,
        ldap::CollectionScope::SchemaOnly,
    )
    .await?;
    // print the total
    ldap::prepare_results_from_source(ldap_results, &config, Some(total)).await?;
    println!("Total entries retrieved: {}", total);
    Ok(())

    // 	// let cli = Cli::parse();
    // 	// commands::dispatch(cli).await
}
