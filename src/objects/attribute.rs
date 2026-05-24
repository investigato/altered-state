use ldap3::SearchEntry;
use log::trace;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single attributeSchema or classSchema object from
/// CN=Schema,CN=Configuration,DC=whatever,DC=local
/// Used to build the GUID resolution map.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SchemaEntry {
    pub ldap_display_name: String,                 // lDAPDisplayName
    pub schema_id_guid: Option<[u8; 16]>,          // schemaIDGUID (from result_bin)
    pub attribute_security_guid: Option<[u8; 16]>, // attributeSecurityGUID (from result_bin)
    pub object_class: SchemaObjectClass,
    pub admin_display_name: String, // adminDisplayName
    pub dn: String,
    pub attribute_syntax: Option<String>,
    pub om_syntax: Option<i32>,
    pub is_single_valued: bool,
    pub system_only: bool,
    pub system_flags: i32,
    pub link_id: Option<i32>,
    pub value_kind: Option<AttributeValueKind>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeValueKind {
    Boolean,
    Integer,
    LargeInteger,
    String,
    OctetString,
    Sid,
    SecurityDescriptor,
    Dn,
    Time,
    Object,
    Unknown,
}
impl AttributeValueKind {
    pub fn from_schema_pair(attribute_syntax: &str, om_syntax: i32) -> Self {
        match (attribute_syntax, om_syntax) {
            ("2.5.5.8", 1) => Self::Boolean,

            ("2.5.5.9", 2) => Self::Integer,
            ("2.5.5.9", 10) => Self::Integer,

            ("2.5.5.16", 65) => Self::LargeInteger,

            ("2.5.5.1", 127) => Self::Dn,

            ("2.5.5.15", 66) => Self::SecurityDescriptor,
            ("2.5.5.17", 4) => Self::Sid,

            ("2.5.5.10", 4) => Self::OctetString,

            ("2.5.5.11", 23) => Self::Time,
            ("2.5.5.11", 24) => Self::Time,

            ("2.5.5.3", 27)
            | ("2.5.5.5", 22)
            | ("2.5.5.5", 19)
            | ("2.5.5.6", 18)
            | ("2.5.5.2", 6)
            | ("2.5.5.4", 20)
            | ("2.5.5.12", 64) => Self::String,

            ("2.5.5.7", 127) | ("2.5.5.10", 127) | ("2.5.5.13", 127) | ("2.5.5.14", 127) => {
                Self::Object
            }

            _ => Self::Unknown,
        }
    }
}
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub enum SchemaObjectClass {
    #[default]
    Attribute,
    Class,
}

impl SchemaEntry {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn is_not_replicated(&self) -> bool {
        self.system_flags & 0x1 != 0
    }

    pub fn is_constructed(&self) -> bool {
        self.system_flags & 0x4 != 0
    }

    
    pub fn parse(&mut self, result: SearchEntry) -> Result<(), Box<dyn std::error::Error>> {
        let result_dn: String = result.dn.to_uppercase();
        let result_attrs: HashMap<String, Vec<String>> = result.attrs;
        let result_bin: HashMap<String, Vec<Vec<u8>>> = result.bin_attrs;
        self.dn = result_dn;
        // Trace all result attributes
        for (key, value) in &result_attrs {
            trace!("  {key:?}:{value:?}");
        }
        // Trace all bin result attributes
        for (key, value) in &result_bin {
            trace!("  {key:?}:{value:?}");
        }

        for (key, value) in &result_attrs {
            match key.as_str() {
                "lDAPDisplayName" => {
                    let ldap_display_name = &value[0];
                    self.ldap_display_name = ldap_display_name.to_lowercase()
                }
                "adminDisplayName" => {
                    let admin_display_name = &value[0];
                    self.admin_display_name = admin_display_name.to_lowercase().to_string()
                }
                "objectClass" => {
                    let object_class = &value[0];
                    self.object_class = if object_class.to_lowercase() == "classschema" {
                        SchemaObjectClass::Class
                    } else {
                        SchemaObjectClass::Attribute
                    }
                }
                "attributeSyntax" => {
                    self.attribute_syntax = value.first().cloned();
                }
                "oMSyntax" => {
                    self.om_syntax = value.first().and_then(|v| v.parse::<i32>().ok());
                }
                "isSingleValued" => {
                    self.is_single_valued = value
                        .first()
                        .map(|v| v.eq_ignore_ascii_case("TRUE"))
                        .unwrap_or(false);
                }
                "systemOnly" => {
                    self.system_only = value
                        .first()
                        .map(|v| v.eq_ignore_ascii_case("TRUE"))
                        .unwrap_or(false);
                }
                "systemFlags" => {
                    self.system_flags = value
                        .first()
                        .and_then(|v| v.parse::<i32>().ok())
                        .unwrap_or(0);
                }
                "linkID" => {
                    self.link_id = value.first().and_then(|v| v.parse::<i32>().ok());
                }
                _ => {}
            }
        }
        for (key, value) in &result_bin {
            match key.as_str() {
                "schemaIDGUID" => {
                    if let Some(guid_bytes) = value.first() {
                        self.schema_id_guid = guid_bytes.as_slice().try_into().ok();
                    }
                }
                "attributeSecurityGUID" => {
                    if let Some(guid_bytes) = value.first() {
                        self.attribute_security_guid = guid_bytes.as_slice().try_into().ok();
                    }
                }
                _ => {}
            }
        }
        if let (Some(attribute_syntax), Some(om_syntax)) = (&self.attribute_syntax, self.om_syntax)
        {
            self.value_kind = Some(AttributeValueKind::from_schema_pair(
                attribute_syntax,
                om_syntax,
            ));
        }
        Ok(())
    }
}

// type SchemaMap = HashMap<String, [u8; 16]>;
// type PropertySetMap = HashMap<[u8; 16], Vec<[u8; 16]>>;

// pub fn build_maps(entries: Vec<SchemaEntry>) -> (SchemaMap, PropertySetMap) {
//     let mut schema_map: HashMap<String, [u8; 16]> = HashMap::new();
//     let mut property_set_map: HashMap<[u8; 16], Vec<[u8; 16]>> = HashMap::new();

//     for entry in entries {
//         if let Some(guid) = entry.schema_id_guid {
//             schema_map.insert(entry.admin_display_name.to_lowercase(), guid);

//             if let Some(prop_set_guid) = entry.attribute_security_guid {
//                 property_set_map
//                     .entry(prop_set_guid)
//                     .or_default()
//                     .push(guid);
//             }
//         }
//     }

//     (schema_map, property_set_map)
// }
