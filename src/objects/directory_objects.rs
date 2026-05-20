use crate::{objects::attribute::SchemaEntry, storage::attributes::AttributeControlSet};
use oxicode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct DirectoryObject {
    pub dn: String,
    pub name: Option<String>,
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

        // Filter entry.attrs to keep only keys present in care_set.allow_list_attributes
        let filtered_attributes: HashMap<String, Vec<String>> = entry
            .attrs
            .iter()
            .filter(|(k, _)| {
                attribute_control_set
                    .allow_list_attributes
                    .contains(&k.to_lowercase())
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        //   Filter entry.bin_attrs uses the same filter against allow_list_attributes
        let filtered_bin_attributes: HashMap<String, Vec<Vec<u8>>> = entry
            .bin_attrs
            .iter()
            .filter(|(k, _)| {
                attribute_control_set
                    .allow_list_attributes
                    .contains(&k.to_lowercase())
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        //   Sort values within each kept attribute (Vec<String> and Vec<Vec<u8>> both sort in place)
        let sorted_attributes: HashMap<String, Vec<String>> = filtered_attributes
            .into_iter()
            .map(|(k, mut v)| {
                v.sort();
                (k, v)
            })
            .collect();

        let sorted_bin_attributes: HashMap<String, Vec<Vec<u8>>> = filtered_bin_attributes
            .into_iter()
            .map(|(k, mut v)| {
                v.sort();
                (k, v)
            })
            .collect();
        //   Compute hash over the filtered and sorted attributes
        let attributes = sorted_attributes;
        let bin_attributes = sorted_bin_attributes;
        let hash = compute_hash(&attributes, &bin_attributes);
        //  don't forget about isdeleted ahh yeah it's lowercase dumbass but makes sure to make it lowercase just in case
        // Set is_deleted — check filtered attrs for "isdeleted" value "TRUE" or DN contains "CN=Deleted Objects"
        let is_deleted = attributes
            .get("isdeleted")
            .and_then(|v| v.first())
            .map(|s| s.eq_ignore_ascii_case("TRUE"))
            .unwrap_or(false)
            || dn.to_lowercase().contains("cn=deleted objects");

        DirectoryObject {
            dn,
            name,
            object_class,
            is_deleted,
            attributes,
            bin_attributes,
            hash,
        }
    }
}
fn compute_hash(
    attributes: &HashMap<String, Vec<String>>,
    bin_attributes: &HashMap<String, Vec<Vec<u8>>>,
) -> String {
    let mut hasher = Sha1::new();
    // Strings...make sure to sort the keys...otherwise we're gonna have a bad time
    let mut keys: Vec<&String> = attributes.keys().collect();
    keys.sort();
    for key in keys {
        let values = &attributes[key];
        hasher.update((key.len() as u32).to_be_bytes());
        hasher.update(key.as_bytes());
        hasher.update((values.len() as u32).to_be_bytes());
        for value in values {
            hasher.update((value.len() as u32).to_be_bytes());
            hasher.update(value.as_bytes());
        }
    }
    // Binaries...also sort the keys
    let mut bin_keys: Vec<&String> = bin_attributes.keys().collect();
    bin_keys.sort();
    for key in bin_keys {
        let values = &bin_attributes[key];
        hasher.update((key.len() as u32).to_be_bytes());
        hasher.update(key.as_bytes());
        hasher.update((values.len() as u32).to_be_bytes());
        for value in values {
            hasher.update((value.len() as u32).to_be_bytes());
            hasher.update(value);
        }
    }
    let result = hasher.finalize();
    hex::encode_upper(result)
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
