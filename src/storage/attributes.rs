use crate::objects::attribute::{AttributeValueKind, SchemaEntry, SchemaObjectClass};
use serde::{Deserialize, Serialize};
use serde_json::to_writer as to_io_writer;
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
    update_schema_file: bool,
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

        let _these_must_be_ignored_to_avoid_fucking_things_up = [
            "certificatetemplates",
            "objectcategory",
            "cn",
        ];
        if entry.system_only
            || entry.is_constructed()
            || entry.is_not_replicated()
            || _these_must_be_ignored_to_avoid_fucking_things_up.contains(&name.as_str())
        {
            system_attributes.insert(name);
        } else {
            allow_list_attributes.insert(name);
        }
    }

    if update_schema_file {
        let mut file = File::create(schema_output_path)
            .unwrap_or_else(|_| panic!("Failed to create {}", schema_output_path.display()));
        to_io_writer(
            &mut file,
            &AttributeControlSet {
                system_attributes: system_attributes.clone(),
                allow_list_attributes: allow_list_attributes.clone(),
                value_kinds: value_kinds.clone(),
            },
        )
        .expect("Failed to write to schema_attributes.json");
    }
    AttributeControlSet {
        system_attributes,
        allow_list_attributes,
        value_kinds,
    }
}
