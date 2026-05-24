use crate::{
    models::ldap::{LdapNamingContexts, LdapOptions},
    objects::{
        attribute::SchemaEntry,
        directory_objects::{
            ADResults, DirectoryObject, save_directory_objects_to_bin_file,
            save_directory_objects_to_json_file,
        },
    },
    storage::{EntrySource, Storage, attributes::build_attribute_control_sets},
    utilities::banner::progress_bar,
};
use colored::Colorize;
use indicatif::ProgressBar;
use itertools::Itertools;

use ldap3::{
    LdapConnAsync, LdapConnSettings, Scope, SearchEntry,
    adapters::PagedResults,
    adapters::{Adapter, EntriesOnly},
    controls::RawControl,
};
use log::{debug, error, info, trace};
use oxicode::{Decode, Encode};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::process;

// #[derive(Clone, Debug)]
// pub struct LdapOptions {
//     pub domain: String,
//     pub ldapfqdn: String,
//     pub ip: Option<String>,
//     pub port: Option<u16>,
//     pub ldaps: bool,
//     pub ldap_filter: Option<String>,
// }

/// Function to request all AD values.
pub async fn ldap_search<S: Storage<LdapSearchEntry>>(
    options: LdapOptions,
    storage: &mut S,
    naming_contexts_file: &Path,
) -> Result<usize, Box<dyn Error>> {
    // Construct LDAP args

    let ldap_args = ldap_constructor(
        options.ldaps,
        options.ip.as_deref(),
        options.port,
        &options.domain,
        &options.ldapfqdn,
    )?;

    // LDAP connection
    let consettings = LdapConnSettings::new()
        .set_conn_timeout(std::time::Duration::from_secs(10))
        .set_no_tls_verify(true);
    let (conn, mut ldap) = LdapConnAsync::with_settings(consettings, &ldap_args.s_url).await?;
    ldap3::drive!(conn);

    debug!("Trying to connect with sasl_gssapi_bind() function (kerberos session)");
    if !options.ldapfqdn.contains("not set") {
        gssapi_connection(&mut ldap, &options.ldapfqdn, &options.domain).await?;
    } else {
        error!(
            "Need Domain Controller FQDN to bind GSSAPI connection. Please make sure the retcon.yaml has an entry for: '{}'\n",
            "domain: DC01.DOMAIN.LAB".bold()
        );
        process::exit(0x0100);
    }
    let mut total = 0; // for progress bar

    // Request all namingContexts for current DC
    let res = match get_all_naming_contexts(&mut ldap).await {
        Ok(res) => {
            trace!("naming_contexts: {:?}", &res);
            // save naming contexts to file for later use in other commands
            let naming_contexts = LdapNamingContexts {
                naming_contexts: res.clone(),
            };
            naming_contexts.save_to_file(naming_contexts_file)?;
            res
        }
        Err(err) => {
            error!("No namingContexts found! Reason: {err}\n");
            process::exit(0x0100);
        }
    };

    // namingContexts: DC=domain,DC=local
    // namingContexts: CN=Configuration,DC=domain,DC=local (needed for AD CS datas)
    if res.iter().any(|s| s.contains("Configuration")) {
        for cn in &res {
            // Set control LDAP_SERVER_SD_FLAGS_OID to get nTSecurityDescriptor
            // https://ldapwiki.com/wiki/LDAP_SERVER_SD_FLAGS_OID
            let sec_desc_flag_ctrl = RawControl {
                ctype: String::from("1.2.840.113556.1.4.801"),
                crit: true,
                val: Some(vec![48, 3, 2, 1, 5]),
            };
            // Setting control flag LDAP_SERVER_SHOW_DELETED_OID to include tombstoned objects
            // Required for ReanimateTombstones parsing
            let show_deleted_ctrl = RawControl {
                ctype: String::from("1.2.840.113556.1.4.417"),
                crit: true,
                val: None,
            };

            let show_deactivated_link_ctrl = RawControl {
                ctype: String::from("1.2.840.113556.1.4.2065"),
                crit: true,
                val: None,
            };
            ldap.with_controls(vec![
                sec_desc_flag_ctrl.to_owned(),
                show_deleted_ctrl.to_owned(),
                show_deactivated_link_ctrl.to_owned(),
            ]);

            info!(
                "Ldap filter : {}",
                options
                    .ldap_filter
                    .as_deref()
                    .unwrap_or("&((objectClass=*)(!(cn=DisplaySpecifiers)))")
                    .bold()
                    .green()
            );
            let _s_filter = options
                .ldap_filter
                .as_deref()
                .unwrap_or("&((objectClass=*)(!(cn=DisplaySpecifiers)))");

            // Every 999 max value in ldap response (err 4 ldap)
            let adapters: Vec<Box<dyn Adapter<_, _>>> = vec![
                Box::new(EntriesOnly::new()),
                Box::new(PagedResults::new(999)),
            ];

            // Streaming search with adapters and filters
            let mut search = ldap
                .streaming_search_with(
                    adapters, // Adapter which fetches Search results with a Paged Results control.
                    cn,
                    Scope::Subtree,
                    _s_filter,
                    vec!["*", "nTSecurityDescriptor"],
                    // Without the presence of this control, the server returns an SD only when the SD attribute name is explicitly mentioned in the requested attribute list.
                    // https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-adts/932a7a8d-8c93-4448-8093-c79b7d9ba499
                )
                .await?;

            // Wait and get next values
            let pb = ProgressBar::new(1);
            let mut count = 0;
            while let Some(entry) = search.next().await? {
                let entry = SearchEntry::construct(entry);
                //trace!("{:?}", &entry);
                total += 1;
                // Manage progress bar
                count += 1;
                progress_bar(
                    pb.to_owned(),
                    "LDAP objects retrieved".to_string(),
                    count,
                    "#".to_string(),
                );

                storage.add(entry.into())?;
            }
            pb.finish_and_clear();

            let res = search.finish().await.success();
            match res {
                Ok(_res) => info!("All data collected for NamingContext {}", &cn.bold()),
                Err(err) => {
                    error!("No data collected on {}! Reason: {err}", &cn.bold().red());
                }
            }
        }

        ldap.unbind().await?;
    }

    // drop ldap before final flush,
    // otherwise it will warn about an i/o error
    // "LDAP connection error: I/O error: Connection reset by peer (os error 54)"
    drop(ldap);
    if total == 0 {
        error!("No LDAP objects found! Exiting...");
        process::exit(0x0100);
    }

    storage.flush()?;

    // Return the vector with the result
    Ok(total)
}

/// Structure containing the LDAP connection arguments.
struct LdapArgs {
    s_url: String,
    _s_dc: Vec<String>,
}

/// Function to prepare LDAP arguments.
fn ldap_constructor(
    ldaps: bool,
    ip: Option<&str>,
    port: Option<u16>,
    domain: &str,
    ldapfqdn: &str,
) -> Result<LdapArgs, Box<dyn Error>> {
    // Prepare ldap url
    let s_url = prepare_ldap_url(ldaps, ip, port, domain);

    // Prepare full DC chain
    let s_dc = prepare_ldap_dc(domain);
    // Format username and email
    // Print infos if verbose mod is set
    debug!("IP: {}", ip.unwrap_or("not set"));
    debug!(
        "PORT: {}",
        match port {
            Some(p) => {
                p.to_string()
            }
            None => "not set".to_owned(),
        }
    );
    debug!("FQDN: {}", ldapfqdn);
    debug!("Url: {}", s_url);
    debug!("Domain: {}", domain);
    debug!("DC: {:?}", s_dc);

    Ok(LdapArgs {
        s_url: s_url.to_string(),
        _s_dc: s_dc,
    })
}

/// Function to prepare LDAP url.
/// credit to g0h4n & RustHound-CE
fn prepare_ldap_url(ldaps: bool, ip: Option<&str>, port: Option<u16>, domain: &str) -> String {
    let protocol = if ldaps || port.unwrap_or(0) == 636 {
        "ldaps"
    } else {
        "ldap"
    };

    let target = ip.unwrap_or(domain);

    match port {
        Some(port) => {
            format!("{protocol}://{target}:{port}")
        }
        None => {
            format!("{protocol}://{target}")
        }
    }
}

/// Function to prepare LDAP DC from DOMAIN.LOCAL
/// credit to g0h4n & RustHound-CE
pub fn prepare_ldap_dc(domain: &str) -> Vec<String> {
    let mut dc: String = "".to_owned();
    let mut naming_context: Vec<String> = Vec::new();

    // Format DC
    if !domain.contains(".") {
        dc.push_str("DC=");
        dc.push_str(domain);
        naming_context.push(dc[..].to_string());
    } else {
        naming_context.push(domain_to_dc(domain));
    }

    // For ADCS values
    naming_context.push(format!("{}{}", "CN=Configuration,", &dc[..]));
    naming_context
}

///credit to g0h4n & RustHound-CE
async fn gssapi_connection(
    ldap: &mut ldap3::Ldap,
    ldapfqdn: &str,
    domain: &str,
) -> Result<(), Box<dyn Error>> {
    let res = ldap.sasl_gssapi_bind(ldapfqdn).await?.success();
    match res {
        Ok(_res) => {
            info!(
                "Connected to {} Active Directory!",
                domain.to_uppercase().bold().green()
            );
            info!("Starting data collection...");
        }
        Err(err) => {
            error!(
                "Failed to authenticate to {} Active Directory. Reason: {err}\n",
                domain.to_uppercase().bold().red()
            );
            process::exit(0x0100);
        }
    }
    Ok(())
}

/// (Not needed yet...yes it is! [Gato]) Get all namingContext for DC
pub async fn get_all_naming_contexts(
    ldap: &mut ldap3::Ldap,
) -> Result<Vec<String>, Box<dyn Error>> {
    // Every 999 max value in ldap response (err 4 ldap)
    let adapters: Vec<Box<dyn Adapter<_, _>>> = vec![
        Box::new(EntriesOnly::new()),
        Box::new(PagedResults::new(999)),
    ];

    // First LDAP request to get all namingContext
    let mut search = ldap
        .streaming_search_with(
            adapters,
            "",
            Scope::Base,
            "(objectClass=*)",
            vec!["namingContexts"],
        )
        .await?;

    // Prepare LDAP result vector
    let mut rs: Vec<SearchEntry> = Vec::new();
    while let Some(entry) = search.next().await? {
        let entry = SearchEntry::construct(entry);
        rs.push(entry);
    }
    let res = search.finish().await.success();

    // Prepare vector for all namingContexts result
    let mut naming_contexts: Vec<String> = Vec::new();
    match res {
        Ok(_res) => {
            debug!("All namingContexts collected!");
            for result in rs {
                let result_attrs: HashMap<String, Vec<String>> = result.attrs;

                for value in result_attrs.values() {
                    for naming_context in value {
                        debug!("namingContext found: {}", &naming_context.bold().green());
                        naming_contexts.push(naming_context.to_string());
                    }
                }
            }
            return Ok(naming_contexts);
        }
        Err(err) => {
            error!("No namingContexts found! Reason: {err}");
        }
    }
    // Empty result if no namingContexts found
    Ok(Vec::new())
}

// New type to implement Serialize and Deserialize for SearchEntry
#[derive(Debug, Clone, Encode, Decode)]
pub struct LdapSearchEntry {
    /// Entry DN.
    pub dn: String,
    /// Attributes.
    pub attrs: HashMap<String, Vec<String>>,
    /// Binary-valued attributes.
    pub bin_attrs: HashMap<String, Vec<Vec<u8>>>,
}

impl From<SearchEntry> for LdapSearchEntry {
    fn from(entry: SearchEntry) -> Self {
        LdapSearchEntry {
            dn: entry.dn,
            attrs: entry.attrs,
            bin_attrs: entry.bin_attrs,
        }
    }
}

impl From<LdapSearchEntry> for SearchEntry {
    fn from(entry: LdapSearchEntry) -> Self {
        SearchEntry {
            dn: entry.dn,
            attrs: entry.attrs,
            bin_attrs: entry.bin_attrs,
        }
    }
}

pub fn domain_to_dc(domain: &str) -> String {
    let split = domain.split('.');
    let mut dc = String::new();

    for (i, s) in split.enumerate() {
        dc.push_str("DC=");
        dc.push_str(s);

        if i < domain.split('.').count() - 1 {
            dc.push(',');
        }
    }
    dc
}

pub enum Type {
    Unknown,
    SchemaEntry,
}
/// Get object type, like ("user","group","computer","ou", "container", "gpo", "domain" "trust").
pub fn get_type(result: &SearchEntry) -> Result<Type, Type> {
    let result_attrs = &result.attrs;

    let contains = |values: &Vec<String>, to_find: &str| values.iter().any(|elem| elem == to_find);
    let object_class_vals = result_attrs.get("objectClass");
    result_attrs.get("flags");

    if let Some(vals) = object_class_vals {
        match () {
            _ if contains(vals, "attributeSchema") || contains(vals, "classSchema") => {
                return Ok(Type::SchemaEntry);
            }

            _ => {}
        }
    }
    Err(Type::Unknown)
}
pub async fn prepare_results_from_source<S: EntrySource>(
    source: S,
    domain: &str,
    export_path: &Path,
    update_schema_file: bool,
    source_output_path: &Path,
    total_objects: Option<usize>,
) -> Result<ADResults, Box<dyn Error>> {
    let ad_results = parse_result_type_from_source(
        domain,
        export_path,
        update_schema_file,
        source_output_path,
        source,
        total_objects,
    )?;

    // Functions to replace and add missing values
    // check_all_result(
    //     config,
    //     &mut ad_results.schema_entries,
    // &mut ad_results.users,
    // &mut ad_results.dmsas,
    // &mut ad_results.groups,
    // &mut ad_results.computers,
    // &mut ad_results.ous,
    // &mut ad_results.domains,
    // &mut ad_results.gpos,
    // &mut ad_results.fsps,
    // &mut ad_results.containers,
    // &mut ad_results.trusts,
    // &mut ad_results.ntauthstores,
    // &mut ad_results.aiacas,
    // &mut ad_results.rootcas,
    // &mut ad_results.enterprisecas,
    // &mut ad_results.certtemplates,
    // &mut ad_results.issuancepolicies,
    //     &ad_results.mappings.dn_sid,
    //     &ad_results.mappings.sid_type,
    //     &ad_results.mappings.fqdn_sid,
    //     &ad_results.mappings.fqdn_ip,
    // )?;

    Ok(ad_results)
}

// for `total_objects`, the total number of objects may not be known if the ldap query was never run
// (e.g run was resumed from cached results)
pub fn parse_result_type_from_source(
    _domain: &str,
    export_path: &Path,
    update_schema_file: bool,
    schema_output_path: &Path,
    source: impl EntrySource,
    total_objects: Option<usize>,
) -> Result<ADResults, Box<dyn Error>> {
    let mut results = ADResults::default();
    // Domain name

    // Needed for progress bar stats
    let pb = ProgressBar::new(1);
    let mut count = 0;
    let total = total_objects;
    // "DOMAIN_SID".to_owned();

    info!("Starting the LDAP objects parsing...");
    // The schema_guids from (CN=Schema,CN=Configuration...) get here late,
    // but they are needed to fully parse the ACLs. Reason: ACL parsing is inline
    // for every object so without the GUID mapping available, it is incomplete.

    let all_entries: Vec<LdapSearchEntry> =
        source.into_entry_iter().collect::<Result<Vec<_>, _>>()?;
    // All the schema entries first (both attributeSchema and classSchema)
    let (schema_entries, non_schema_entries): (Vec<SchemaEntry>, Vec<LdapSearchEntry>) =
        all_entries.into_iter().partition_map(|e| {
            let se = SearchEntry::from(e.clone());
            if matches!(get_type(&se), Ok(Type::SchemaEntry)) {
                let mut schema_entry = SchemaEntry::new();
                match SchemaEntry::parse(&mut schema_entry, se) {
                    Ok(()) => itertools::Either::Left(schema_entry),
                    Err(_) => itertools::Either::Right(e),
                }
            } else {
                itertools::Either::Right(e)
            }
        });
    // .iter()
    // .filter_map(|e| {
    //     let se = SearchEntry::from(e.clone());
    //     if matches!(get_type(&se), Ok(Type::SchemaEntry)) {
    //         let mut schema_entry = SchemaEntry::new();
    //         match SchemaEntry::parse(&mut schema_entry, se) {
    //             Ok(()) => Some(schema_entry),
    //             Err(_) => None,
    //         }
    //     } else {
    //         None
    //     }
    // })
    // .collect();
    // info!(
    //     "Schema finder: {} schema entries found",
    //     schema_entries.len()
    // );
    // build attribute control sets for later use in ACL parsing and value parsing
    // construct config.paths.scenarios_directory + "/schema_attributes.yaml"

    let attribute_control_set =
        build_attribute_control_sets(&schema_entries, schema_output_path, update_schema_file);

    // let (schema_map, property_set_map) = build_maps(schema_entries);
    // init_maps(schema_map, property_set_map);

    for raw_entry in non_schema_entries {
        let entry: SearchEntry = raw_entry.into();
        // Start parsing with Type matching

        results
            .directory_objects
            .push(DirectoryObject::from_ldap_entry(
                &entry,
                &attribute_control_set,
            ));

        // Percentage (%) = 100 x (part / total)
        if let Some(total) = total {
            count += 1;
            let percentage = 100 * count / total;
            progress_bar(
                pb.to_owned(),
                "Parsing LDAP objects".to_string(),
                percentage.try_into()?,
                "%".to_string(),
            );
        }
    }
    // if there are directory objects, write them to file
    save_directory_objects_to_bin_file(&results.directory_objects, export_path)?;
    save_directory_objects_to_json_file(&results.directory_objects, export_path)?;
    pb.finish_and_clear();
    info!("Parsing LDAP objects finished!");
    Ok(results)
}

// /// Function to parse and replace value for unknown object.
// pub fn parse_unknown(result: SearchEntry, _domain: &str) -> serde_json::value::Value {
//     let _result_dn = result.dn.to_uppercase();
//     let _result_attrs: HashMap<String, Vec<String>> = result.attrs;
//     let _result_bin: HashMap<String, Vec<Vec<u8>>> = result.bin_attrs;

//     let unknown_json = json!({
//         "unknown": null,
//     });

//     // Debug for current object
//     trace!("Parse Unknown object: {}", _result_dn);
//     // for (key, value) in &_result_attrs {
//     //    println!("  {:?}:{:?}", key, value);
//     // }
//     // //trace result bin
//     // for (key, value) in &_result_bin {
//     //    println!("  {:?}:{:?}", key, value);
//     // }

//     unknown_json
// }
