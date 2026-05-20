use crate::objects::attribute::{AttributeValueKind, SchemaEntry, SchemaObjectClass};
use serde::{Deserialize, Serialize};
use serde_saphyr::to_io_writer;
use std::collections::{HashMap, HashSet};
use std::fs::File;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeControlSet {
    pub system_attributes: HashSet<String>,
    pub allow_list_attributes: HashSet<String>,
    pub value_kinds: HashMap<String, AttributeValueKind>,
}

pub fn build_attribute_control_sets(
    entries: &[SchemaEntry],
    schema_output_path: &std::path::Path,
) -> AttributeControlSet {
    let mut system_attributes = HashSet::new();
    let mut allow_list_attributes = HashSet::new();
    let mut value_kinds = HashMap::new();

    for entry in entries {
        if entry.object_class != SchemaObjectClass::Attribute {
            continue;
        }

        let name = entry.ldap_display_name.to_lowercase();

        if let Some(kind) = entry.value_kind {
            value_kinds.insert(name.clone(), kind);
        }

        if entry.system_only || entry.is_constructed() || entry.is_not_replicated() {
            system_attributes.insert(name);
        } else {
            allow_list_attributes.insert(name);
        }
    }

    let mut file =
        File::create(schema_output_path).expect("Failed to create schema_attributes.yaml");
    to_io_writer(
        &mut file,
        &AttributeControlSet {
            system_attributes: system_attributes.clone(),
            allow_list_attributes: allow_list_attributes.clone(),
            value_kinds: value_kinds.clone(),
        },
    )
    .expect("Failed to write to schema_attributes.yaml");
    AttributeControlSet {
        system_attributes,
        allow_list_attributes,
        value_kinds,
    }
}
