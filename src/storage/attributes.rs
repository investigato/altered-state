use crate::objects::attribute::{AttributeValueKind, SchemaEntry, SchemaObjectClass};
use serde::{Deserialize, Serialize};
use serde_json::to_writer as to_io_writer;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Error, Result};
use std::path::Path;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeControlSet {
    pub system_attributes: HashSet<String>,
    pub allow_list: HashMap<String, AllowedAttribute>, // key is lowercase ldap_display_name
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedAttribute {
    pub is_single_valued: bool,
    pub value_kind: Option<AttributeValueKind>,
    pub link_id: Option<i32>,
}

pub fn build_attribute_control_sets(
    entries: &[SchemaEntry],
    schema_output_path: &std::path::Path,
    update_schema_file: bool,
) -> AttributeControlSet {
    let mut system_attributes = HashSet::new();
    let mut allow_list_attributes = HashMap::new();

    for entry in entries {
        if entry.object_class != SchemaObjectClass::Attribute {
            continue;
        }

        let name = entry.ldap_display_name.to_lowercase();

        let _these_must_be_ignored_to_avoid_fucking_things_up =
            ["certificatetemplates", "objectcategory", "cn"];
        if entry.system_only
            || entry.is_constructed()
            || entry.is_not_replicated()
            || _these_must_be_ignored_to_avoid_fucking_things_up.contains(&name.as_str())
        {
            system_attributes.insert(name);
        } else {
            allow_list_attributes.insert(
                name,
                AllowedAttribute {
                    is_single_valued: entry.is_single_valued,
                    value_kind: entry.value_kind,
                    link_id: entry.link_id,
                },
            );
        }
    }

    if update_schema_file {
        let mut file = File::create(schema_output_path)
            .unwrap_or_else(|_| panic!("Failed to create {}", schema_output_path.display()));
        to_io_writer(
            &mut file,
            &AttributeControlSet {
                system_attributes: system_attributes.clone(),
                allow_list: allow_list_attributes.clone(),
            },
        )
        .expect("Failed to write to schema_attributes.json");
    }
    AttributeControlSet {
        system_attributes,
        allow_list: allow_list_attributes,
    }
}

pub fn load_attribute_control_set(schema_path: &Path) -> Result<AttributeControlSet> {
    if !schema_path.exists() {
        return Ok(AttributeControlSet {
            system_attributes: HashSet::new(),
            allow_list: HashMap::new(),
        });
    }
    let file = File::open(schema_path)?;
    let attribute_control_set: AttributeControlSet =
        serde_json::from_reader(file).map_err(|e| Error::other(e.to_string()))?;
    Ok(attribute_control_set)
}
