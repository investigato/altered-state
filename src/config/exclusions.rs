use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectChangeType {
    Created,
    Removed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMatch {
    pub name_exact: Option<String>,
    pub name_prefix: Option<String>,
    pub name_contains: Option<String>,
    pub ou: Option<String>,
    pub dn_contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExclusionConfig {
    pub name: String,
    pub match_: ObjectMatch,
    pub rules: Vec<ChangeRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRule {
    pub change_type: ObjectChangeType,
    pub additional_ignored_attributes: Vec<String>,
}
