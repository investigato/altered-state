use crate::{objects::attribute::SchemaEntry, storage::attributes::AttributeControlSet};
use oxicode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use sha1::{Digest, Sha1};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq, Eq)]
pub struct DirectoryObject {
    pub dn: String,
    pub name: Option<String>,
    pub sddl: Option<Vec<u8>>,
    pub object_class: Vec<String>,
    pub is_deleted: bool,
    pub attributes: HashMap<String, Vec<String>>,
    pub bin_attributes: HashMap<String, Vec<Vec<u8>>>,
    pub hash: String,
}
impl DirectoryObject {
    pub fn from_ldap_entry(
        entry: &ldap3::SearchEntry,
        attribute_control_set: &AttributeControlSet,
    ) -> Self {
        let dn = entry.dn.to_string();
        let name = entry.attrs.get("name").and_then(|v| v.first()).cloned();
        let object_class = entry.attrs.get("objectClass").cloned().unwrap_or_default();
        let sddl = entry
            .bin_attrs
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("ntsecuritydescriptor"))
            .and_then(|(_, values)| values.first())
            .cloned();

        // Normalize keys to lowercase so LDAP casing differences do not change hashes, that will fuck shit up
        let mut attributes: HashMap<String, Vec<String>> = HashMap::new();
        for (key, values) in &entry.attrs {
            let normalized_key = key.to_lowercase();
            if attribute_control_set
                .allow_list
                .contains_key(&normalized_key)
            {
                attributes
                    .entry(normalized_key)
                    .or_default()
                    .extend(values.iter().cloned());
            }
        }

        let mut bin_attributes: HashMap<String, Vec<Vec<u8>>> = HashMap::new();
        for (key, values) in &entry.bin_attrs {
            let normalized_key = key.to_lowercase();
            if normalized_key == "ntsecuritydescriptor" {
                continue;
            }
            if attribute_control_set
                .allow_list
                .contains_key(&normalized_key)
            {
                bin_attributes
                    .entry(normalized_key)
                    .or_default()
                    .extend(values.iter().cloned());
            }
        }

        // Keep stored value vectors stable as well.
        for values in attributes.values_mut() {
            values.sort();
        }
        for values in bin_attributes.values_mut() {
            values.sort();
        }

        //  don't forget about isdeleted ahh yeah it's lowercase dumbass but makes sure to make it lowercase just in case
        // Set is_deleted by checking filtered_attributes for "isdeleted" value "TRUE" or DN contains "CN=Deleted Objects"
        let is_deleted = attributes
            .get("isdeleted")
            .and_then(|v| v.first())
            .map(|s| s.eq_ignore_ascii_case("TRUE"))
            .unwrap_or(false)
            || dn.to_lowercase().contains("cn=deleted objects");
        let hash = compute_hash(&attributes, &bin_attributes);
        DirectoryObject {
            dn,
            name,
            sddl,
            object_class,
            is_deleted,
            attributes,
            bin_attributes,
            hash,
        }
    }
}

// fn is_nt_security_descriptor_attr(attr_name: &str) -> bool {
//     attr_name
//         .split([';', ':'])
//         .next()
//         .map(str::trim)
//         .map(|base| base.eq_ignore_ascii_case("nTSecurityDescriptor"))
//         .unwrap_or(false)
// }

fn compute_hash(
    attributes: &HashMap<String, Vec<String>>,
    bin_attributes: &HashMap<String, Vec<Vec<u8>>>,
) -> String {
    let mut hasher = Sha1::new();
    // Strings...make sure to sort the keys...otherwise we're gonna have a bad time
    let mut keys: Vec<&String> = attributes.keys().collect();
    keys.sort();
    for key in keys {
        let mut values = attributes.get(key).cloned().unwrap_or_default();
        values.sort();
        hasher.update((key.len() as u32).to_be_bytes());
        hasher.update(key.as_bytes());
        hasher.update((values.len() as u32).to_be_bytes());
        for value in &values {
            hasher.update((value.len() as u32).to_be_bytes());
            hasher.update(value.as_bytes());
        }
    }
    // Binaries...also sort the keys
    let mut bin_keys: Vec<&String> = bin_attributes.keys().collect();
    bin_keys.sort();
    for key in bin_keys {
        let mut values = bin_attributes.get(key).cloned().unwrap_or_default();
        values.sort();
        hasher.update((key.len() as u32).to_be_bytes());
        hasher.update(key.as_bytes());
        hasher.update((values.len() as u32).to_be_bytes());
        for value in &values {
            hasher.update((value.len() as u32).to_be_bytes());
            hasher.update(value);
        }
    }
    let result = hasher.finalize();
    hex::encode_upper(result)
}

pub fn save_directory_objects_to_bin_file(
    objects: &[DirectoryObject],
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let owned_objects: Vec<DirectoryObject> = objects.to_vec();
    oxicode::encode_to_file(&owned_objects, path)?;
    Ok(())
}

pub fn save_directory_objects_to_json_file(
    objects: &[DirectoryObject],
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = to_string_pretty(objects)?;
    // change extension to .json
    let mut json_path = path.to_path_buf();
    json_path.set_extension("json");
    std::fs::write(json_path, json)?;
    Ok(())
}

pub fn read_directory_objects_from_bin_file(
    path: &std::path::Path,
) -> Result<Vec<DirectoryObject>, Box<dyn std::error::Error>> {
    let objects: Vec<DirectoryObject> = oxicode::decode_from_file(path)?;
    Ok(objects)
}

#[derive(Default)]
pub struct ADResults {
    pub schema_entries: Vec<SchemaEntry>,
    pub directory_objects: Vec<DirectoryObject>,
    // pub users: Vec<User>,
    // pub dmsas: Vec<DelegatedMSA>,
    // pub groups: Vec<Group>,
    // pub computers: Vec<Computer>,
    // pub ous: Vec<Ou>,
    // pub domains: Vec<Domain>,
    // pub gpos: Vec<Gpo>,
    // pub fsps: Vec<Fsp>,
    // pub containers: Vec<Container>,
    // pub trusts: Vec<Trust>,
    // pub ntauthstores: Vec<NtAuthStore>,
    // pub aiacas: Vec<AIACA>,
    // pub rootcas: Vec<RootCA>,
    // pub enterprisecas: Vec<EnterpriseCA>,
    // pub certtemplates: Vec<CertTemplate>,
    // pub issuancepolicies: Vec<IssuancePolicy>,
    pub mappings: DomainMappings,
}

#[derive(Default)]
pub struct DomainMappings {
    /// DN to SID
    pub dn_sid: HashMap<String, String>,
    ///  DN to Type
    pub sid_type: HashMap<String, String>,
    /// FQDN to SID
    pub fqdn_sid: HashMap<String, String>,
    /// fqdn to an ip address
    pub fqdn_ip: HashMap<String, String>,
}

impl ADResults {
    pub fn new() -> Self {
        Self::default()
    }
}
