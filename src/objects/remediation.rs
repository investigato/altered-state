use crate::objects::directory_objects::DirectoryObject;

#[derive(Debug, Clone)]
pub struct RemediationCommand {
    pub command_type: CommandType,
    pub command: String,
    pub description: Option<String>,
    pub object_name: Option<String>,
    pub is_comment: bool,
}

#[derive(Debug, Clone)]
pub enum CommandType {
    PowerShell,
    DsAcls,
    Comment,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    Create,
    Reanimate,
    Modify,
    Delete,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemediationAction {
    pub action: ActionType,
    pub target: Option<DirectoryObject>, // what it should look like
    pub current: Option<DirectoryObject>, // what it looks like now (can be null for Create actions)
    pub last_known_parent: Option<String>, // for delete actions, where to move it before deletion (if known)
}
