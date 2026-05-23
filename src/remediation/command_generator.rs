use crate::models::ldap::LdapNamingContexts;
use crate::objects::{
    attribute,
    directory_objects::DirectoryObject,
    remediation::{ActionType, CommandType, RemediationAction, RemediationCommand},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::collections::HashMap;
use std::path::Path;

//    public List<RemediationCommand> GenerateCommands(List<RemediationAction> actions)
//     {
//         var commands = new List<RemediationCommand>();
//         foreach (var action in actions) commands.AddRange(GenerateCommandsForAction(action));

//         return commands;
//     }

pub fn generate_commands(
    actions: HashMap<String, Vec<RemediationAction>>,
    naming_contexts_file: &Path,
) -> Vec<RemediationCommand> {
    let mut commands: Vec<RemediationCommand> = Vec::new();
    let naming_contexts =
        LdapNamingContexts::load_from_file(naming_contexts_file).unwrap_or_else(|_| {
            panic!(
                "Failed to load naming contexts from {}",
                naming_contexts_file.display()
            )
        });
    for (_, action_list) in actions {
        for action in action_list {
            let action_commands = generate_commands_for_action(action, &naming_contexts);
            commands.extend(action_commands);
        }
    }
    commands
}

// private List<RemediationCommand> GenerateCommandsForAction(RemediationAction action)
// {
//     return action.Action switch
//     {
//         ActionType.Create => GenerateCreateCommands(action),
//         ActionType.Reanimate => GenerateReanimateCommands(action),
//         ActionType.Delete => GenerateDeleteCommands(action),
//         ActionType.Modify => GenerateModifyCommands(action),
//         _ => new List<RemediationCommand>()
//     };
// }
pub fn generate_commands_for_action(
    action: RemediationAction,
    naming_contexts: &LdapNamingContexts,
) -> Vec<RemediationCommand> {
    match action.action {
        ActionType::Create => generate_create_commands(action),
        ActionType::Reanimate => generate_reanimate_commands(action),
        ActionType::Delete => generate_delete_commands(action, naming_contexts),
        ActionType::Modify => generate_modify_commands(action),
    }
}

//  private List<RemediationCommand> GenerateCreateCommands(RemediationAction action)
//     {
//         var record = action.Target;
//         var commands = new List<RemediationCommand>();
//         if (record == null) return commands;

//         // create fresh object with type based on record.ObjectClass
//         commands.Add(new RemediationCommand
//         {
//             Type = CommandType.Comment,
//             Command =
//                 $"Create new object of class {record.ObjectClass} with attributes from baseline",
//             IsComment = true
//         });
//         var createCommand = new StringBuilder();
//         createCommand.Append($"New-ADObject -Name '{record.Name}' -Type '{record.ObjectClass}' -Path '");
//         var parentDn = string.Join(",", record.DistinguishedName.Split(',').Skip(1));
//         createCommand.Append(parentDn);
//         createCommand.Append("'");
//         commands.Add(new RemediationCommand
//         {
//             Type = CommandType.PowerShell,
//             Command = createCommand.ToString(),
//             Description = "Create new object based on baseline"
//         });
//         var attributeCommands = GenerateRestoreAttributeCommands(record, null, record.DistinguishedName, true);
//         commands.AddRange(attributeCommands);
//         return commands;
//     }

fn generate_create_commands(action: RemediationAction) -> Vec<RemediationCommand> {
    let record = match action.target {
        Some(rec) => rec,
        None => return Vec::new(),
    };
    let mut commands: Vec<RemediationCommand> = Vec::new();
    // create fresh object with type based on record.ObjectClass
    commands.push(RemediationCommand {
        command_type: crate::objects::remediation::CommandType::Comment,
        command: format!(
            "Create new object of class {:?} with attributes from baseline",
            record.object_class
        ),
        description: None,
        object_name: None,
        is_comment: true,
    });
    let parent_dn = record
        .dn
        .split(',')
        .skip(1)
        .collect::<Vec<&str>>()
        .join(",");
    let object_name = record.name.as_deref().unwrap_or("");
    let object_class = record
        .object_class
        .iter()
        .rev()
        .find(|s| !s.eq_ignore_ascii_case("top"))
        .or_else(|| record.object_class.first())
        .map(|s| s.as_str())
        .unwrap_or("");
    let create_command = format!(
        "New-ADObject -Name '{}' -Type '{}' -Path '{}'",
        escape_dn_component(object_name),
        escape_dn_component(object_class),
        escape_dn_component(parent_dn.as_str())
    );

    commands.push(RemediationCommand {
        command_type: crate::objects::remediation::CommandType::PowerShell,
        command: create_command,
        description: Some("Create new object based on baseline".to_string()),
        object_name: record.name.clone(),
        is_comment: false,
    });
    let attribute_commands = generate_restore_attribute_commands(&record, None, &record.dn);
    commands.extend(attribute_commands);
    commands
}

// private List<RemediationCommand> GenerateDeleteCommands(RemediationAction
//     action)
// {
fn generate_delete_commands(
    action: RemediationAction,
    naming_contexts: &LdapNamingContexts,
) -> Vec<RemediationCommand> {
    let mut commands: Vec<RemediationCommand> = Vec::new();
    let target = action.target.as_ref();
    let current = action.current.as_ref();
    let mut new_dn = current.map(|c| c.dn.clone()).unwrap_or_default();
    // If we have a lastKnownParent, move the object there before deletion to preserve it in the Deleted Objects container
    if let Some(last_known_parent) = &action.last_known_parent {
        commands.push(RemediationCommand {
            command_type: CommandType::Comment,
            command: "Step 1: Move object to lastKnownParent before deletion".to_string(),
            description: None,
            object_name: None,
            is_comment: true,
        });

        commands.push(RemediationCommand {
            command_type: CommandType::PowerShell,
            command: format!(
                "Move-ADObject -Identity '{}' -TargetPath '{}'",
                action
                    .current
                    .as_ref()
                    .map(|c| c.dn.clone())
                    .unwrap_or_default(),
                last_known_parent
            ),
            description: Some("Move object to lastKnownParent before deletion".to_string()),
            object_name: None,
            is_comment: false,
        });

        if let Some(name) = current.and_then(|curr| curr.name.as_deref()) {
            new_dn = format!("CN={},{}", escape_dn_component(name), last_known_parent);
        }

        //  if (record != null && action.Current != null)
        //         {
        if let (Some(target_obj), Some(current_obj)) = (target, current) {
            let identity_dn = target_obj.dn.clone();
            let modified_attribute_commands =
                generate_restore_attribute_commands(target_obj, Some(current_obj), &identity_dn);
            commands.extend(modified_attribute_commands);
        }

        commands.push(RemediationCommand {
            command_type: CommandType::Comment,
            command: "Step 2: Delete the object".to_string(),
            description: None,
            object_name: None,
            is_comment: true,
        });
    } else {
        commands.push(RemediationCommand {
            command_type: CommandType::Comment,
            command: "WARNING: lastKnownParent not found in baseline, deleting directly (object may not be recoverable if not moved first)".to_string(),
            description: None,
            object_name: None,
            is_comment: true,
        });
    }

    if let Some(curr) = current {
        if curr
            .object_class
            .iter()
            .any(|c| c.eq_ignore_ascii_case("pkicertificatetemplate"))
            && target.is_none()
        {
            let template_name = curr.name.as_deref().unwrap_or("unknown-template");
            commands.push(RemediationCommand {
                command_type: CommandType::Comment,
                command: format!(
                    "Object '{}' is a Certificate Template, using CA cmdlets to delete to avoid orphaned CN=Deleted Objects entry",
                    escape_dn_component(template_name)
                ),
                description: None,
                object_name: None,
                is_comment: true,
            });
            commands.push(RemediationCommand {
                command_type: CommandType::PowerShell,
                command: format!(
                    "Get-CertificationAuthority | Get-CATemplate | Remove-CATemplate -Name '{}' | Set-CATemplate; Remove-ADObject -Identity '{}' -Recursive -Confirm:$false",
                    escape_dn_component(template_name), new_dn
                ),
                description: Some("Delete the object".to_string()),
                object_name: None,
                is_comment: false,
            });
        } else {
            commands.push(RemediationCommand {
                command_type: CommandType::PowerShell,
                command: format!(
                    "Remove-ADObject -Identity '{}' -Recursive -Confirm:$false",
                    new_dn
                ),
                description: Some("Delete the object".to_string()),
                object_name: None,
                is_comment: false,
            });
            // have to figure out where this one lives so match the end of the DN??
            // five possible naming contexts
        }
        let naming_context = extract_naming_context(&curr.dn, naming_contexts).unwrap_or("unknown");

        commands.push(RemediationCommand {
                command_type: CommandType::PowerShell,
                command: format!(
                    "Get-ADObject -Filter 'isDeleted -eq $true' -IncludeDeletedObjects -SearchBase 'CN=Deleted Objects,{}' | Where-Object {{ $_.DistinguishedName -ne 'CN=Deleted Objects,{}' }} |  Remove-ADObject -Recursive -Confirm:$false",
                    naming_context, naming_context
                ),
                description: Some("Clear the recycle bin of the deleted object".to_string()),
                object_name: None,
                is_comment: false,
            });
    }

    commands
}
fn extract_naming_context<'a>(
    dn: &str,
    naming_contexts: &'a LdapNamingContexts,
) -> Option<&'a str> {
    let mut sorted: Vec<&str> = naming_contexts
        .naming_contexts
        .iter()
        .map(|s| s.as_str())
        .collect();
    sorted.sort_by_key(|b| std::cmp::Reverse(b.len()));

    sorted.into_iter().find(|nc| {
        dn.eq_ignore_ascii_case(nc)
            || dn
                .to_ascii_lowercase()
                .ends_with(&format!(",{}", nc.to_lowercase()))
    })
}

fn is_nt_security_descriptor_attr(attr_name: &str) -> bool {
    attr_name
        .split(';')
        .next()
        .map(|base| base.eq_ignore_ascii_case("nTSecurityDescriptor"))
        .unwrap_or(false)
}

fn escape_powershell_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

fn is_probably_sddl_string(bytes: &[u8]) -> Option<&str> {
    let s = std::str::from_utf8(bytes).ok()?.trim();
    if s.is_empty() {
        return None;
    }

    // Typical SDDL starts with one or more of O:, G:, D:, S:
    if s.starts_with("O:") || s.starts_with("G:") || s.starts_with("D:") || s.starts_with("S:") {
        return Some(s);
    }

    None
}

fn escape_dn_component(value: &str) -> String {
    value.replace('\n', "\\0A").replace('\r', "\\0D")
}
fn generate_modify_commands(action: RemediationAction) -> Vec<RemediationCommand> {
    let mut commands: Vec<RemediationCommand> = Vec::new();
    let target = match action.target {
        Some(t) => t,
        None => return commands,
    };
    let attribute_commands =
        generate_restore_attribute_commands(&target, action.current.as_ref(), &target.dn);
    commands.extend(attribute_commands);
    commands
}

//  private List<RemediationCommand> GenerateReanimateCommands(RemediationAction action)
//     {
//         var commands = new List<RemediationCommand>();
//         var target = action.Target;
//         if (target == null) return commands;
fn generate_reanimate_commands(action: RemediationAction) -> Vec<RemediationCommand> {
    let mut commands: Vec<RemediationCommand> = Vec::new();
    let target = match action.target {
        Some(t) => t,
        None => return commands,
    };
    //         // Restore to original location
    //         if (!string.Equals(target.DistinguishedName, action.Current?.DistinguishedName,
    //                 StringComparison.OrdinalIgnoreCase) &&
    //             !string.IsNullOrEmpty(target.DistinguishedName))
    //         {
    //             // where should it go back to?
    //             var parentOu =
    //                 string.Join(",",
    //                     target.DistinguishedName.Split(',').Skip(1));
    //             commands.Add(new RemediationCommand
    //             {
    //                 Type = CommandType.PowerShell,
    //                 Command =
    //                     $"Restore-ADObject -Identity '{action.Current!.DistinguishedName}' -TargetPath '{parentOu}' -Confirm:$false",
    //                 Description = "Move object back to original location"
    //             });
    //         }
    if !target.dn.is_empty()
        && !action
            .current
            .as_ref()
            .map(|c| c.dn.as_str())
            .unwrap_or("")
            .eq_ignore_ascii_case(target.dn.as_str())
    {
        let parent_ou = target
            .dn
            .split(',')
            .skip(1)
            .collect::<Vec<&str>>()
            .join(",");
        commands.push(RemediationCommand {
            command_type: CommandType::PowerShell,
            command: format!(
                "Restore-ADObject -Identity '{:?}' -TargetPath '{:?}' -Confirm:$false",
                action
                    .current
                    .as_ref()
                    .map(|c| c.dn.clone())
                    .unwrap_or_default(),
                parent_ou
            ),
            description: Some("Move object back to original location".to_string()),
            object_name: None,
            is_comment: false,
        });
    }
    //         var attributeCommands =
    //             GenerateRestoreAttributeCommands(target, action.Current, action.Current!.DistinguishedName, false);
    //         commands.AddRange(attributeCommands);
    let attribute_commands = generate_restore_attribute_commands(
        &target,
        action.current.as_ref(),
        &action
            .current
            .as_ref()
            .map(|c| c.dn.clone())
            .unwrap_or_default(),
    );
    commands.extend(attribute_commands);
    commands
}

// private List<RemediationCommand> GenerateRestoreAttributeCommands(AdObject target,
//         AdObject? current, string identityDn, bool fullRestore)
fn generate_restore_attribute_commands(
    target: &DirectoryObject,
    current: Option<&DirectoryObject>,
    identity_dn: &str,
) -> Vec<RemediationCommand> {
    let mut commands: Vec<RemediationCommand> = Vec::new();
    //         if (!string.Equals(target.Sddl, current?.Sddl, StringComparison.OrdinalIgnoreCase) &&
    //             commands.AddRange(GenerateSddlCommands(identityDn, target.Sddl));

    if let Some(sddl) = target.sddl.as_deref()
        && !sddl.is_empty()
        && current.and_then(|c| c.sddl.as_deref()) != Some(sddl)
    {
        let sddl_commands = generate_sddl_commands(identity_dn, Some(sddl));
        commands.extend(sddl_commands);
    }

    //         var targetAttributes = target.Attributes;
    //         var currentAttributes = current?.Attributes ?? new Dictionary<string, object?>();
    //         // Two buckets of commanding fun: one for Replace (new or modified attributes) and one for Clear (removed attributes)
    //         var toReplace = new Dictionary<string, object>();
    //         var toClear = new List<string>();
    //  target.attributes are what values SHOULD be set
    //  current.attributes (optional) are what values exist now, to diff against
    //  bin_attributes need to base64 encode each Vec<u8> value for PowerShell's FromBase64String
    //  toClear are keys in current.attributes not present in target.attributes
    //  toReplace are keys in target.attributes where the value differs from current.attributes (or key not present in current.attributes)
    let target_attributes = &target.attributes;
    let target_bin_attributes = &target.bin_attributes;
    let empty_attributes: HashMap<String, Vec<String>> = HashMap::new();
    let empty_bin_attributes: HashMap<String, Vec<Vec<u8>>> = HashMap::new();
    let current_attributes = current.map(|c| &c.attributes).unwrap_or(&empty_attributes);
    let current_bin_attributes = current
        .map(|c| &c.bin_attributes)
        .unwrap_or(&empty_bin_attributes);
    let mut to_replace: HashMap<String, String> = HashMap::new();
    let mut to_replace_bin: HashMap<String, Vec<String>> = HashMap::new();
    let mut to_clear: Vec<String> = Vec::new();
    // attribute replacement logic using target.attributes.keys() as the effective watch list
    for attr in target_attributes.keys() {
        let target_value = target_attributes.get(attr);
        let current_value = current_attributes.get(attr);
        if target_value != current_value
            && let Some(val) = target_value
        {
            to_replace.insert(attr.clone(), val.join(","));
        }
    }
    for attr in current_attributes.keys() {
        if !target_attributes.contains_key(attr) {
            to_clear.push(attr.clone());
        }
    }
    // same logic for bin_attributes
    for attr in target_bin_attributes.keys() {
        if is_nt_security_descriptor_attr(attr) {
            continue;
        }
        let target_value = target_bin_attributes.get(attr);
        let current_value = current_bin_attributes.get(attr);
        if target_value != current_value
            && let Some(val) = target_value
        {
            // Keep base64 text here; the generated PowerShell command will decode each value with FromBase64String.
            // EXCEEEEEEEEEPT...for things like pkikeyusage which is single valued OCTET-STRING and this base64 makes it look like 2 entries
            let encoded_vals: Vec<String> = val.iter().map(|v| STANDARD.encode(v)).collect();
            to_replace_bin.insert(attr.clone(), encoded_vals);
        }
    }
    for attr in current_bin_attributes.keys() {
        if is_nt_security_descriptor_attr(attr) {
            continue;
        }
        if !target_bin_attributes.contains_key(attr) {
            to_clear.push(attr.clone());
        }
    }
    //         if (toReplace.Any())
    //         {
    if to_replace.is_empty() && to_replace_bin.is_empty() && to_clear.is_empty() {
        return commands;
    }
    //             var replaceObject = new StringBuilder();

    // need to make a string object to hold the replacement values that will go into powershell's -Replace parameter, which is a hashtable in the form @{key=value;key2=value2}

    let mut replace_object = String::from("@{");
    for (k, v) in to_replace {
        replace_object.push_str(&format!("'{}'='{}';", k, v));
    }
    for (k, v) in to_replace_bin {
        let joined_vals: Vec<String> = v
            .iter()
            .map(|val| format!("[Convert]::FromBase64String('{}')", val))
            .collect();
        replace_object.push_str(&format!("'{}'=@({});", k, joined_vals.join(",")));
    }
    replace_object.push('}');
    //             replaceObject.Append("}");
    //             commands.Add(new RemediationCommand
    //             {
    //                 Type = CommandType.PowerShell,
    //                 Command =
    //                     $"Set-ADObject -Identity '{identityDn}' -Replace {replaceObject}",
    //                 Description = "Replace modified attributes"
    //             });
    commands.push(RemediationCommand {
        command_type: CommandType::PowerShell,
        command: format!(
            "Set-ADObject -Identity '{}' -Replace {}",
            identity_dn, replace_object
        ),
        description: Some("Replace modified attributes".to_string()),
        object_name: None,
        is_comment: false,
    });

    // if toClear is not empty, join the list into a comma-separated string and Set-ADObject -Identity '<dn>' -Clear attr1,attr2,...
    //             if (toClear.Any())
    //             {
    //                 var clearArray = string.Join(",", toClear.Select(a => $"'{a}'"));
    //                 commands.Add(new RemediationCommand
    //                 {
    //                     Type = CommandType.PowerShell,
    //                     Command =
    //                         $"Set-ADObject -Identity '{identityDn}' -Clear @({clearArray})",
    //                     Description = "Clear removed attributes"
    //                 });
    //             }
    //         }
    if !to_clear.is_empty() {
        let clear_array = to_clear
            .iter()
            .map(|a| format!("'{}'", a))
            .collect::<Vec<String>>()
            .join(",");
        commands.push(RemediationCommand {
            command_type: CommandType::PowerShell,
            command: format!(
                "Set-ADObject -Identity '{}' -Clear @({})",
                identity_dn, clear_array
            ),
            description: Some("Clear removed attributes".to_string()),
            object_name: None,
            is_comment: false,
        });
    }

    commands
}

// private static List<RemediationCommand> GenerateSddlCommands(string dn, string? baselineSddl)
// {
fn generate_sddl_commands(dn: &str, baseline_sd: Option<&[u8]>) -> Vec<RemediationCommand> {
    //     var commands = new List<RemediationCommand>
    //     {
    //         new()
    //         {
    //             Type = CommandType.Comment,
    //             Command = "Restore security descriptor from baseline",
    //             IsComment = true
    //         }
    //     };
    let mut commands: Vec<RemediationCommand> = Vec::new();
    commands.push(RemediationCommand {
        command_type: CommandType::Comment,
        command: "Restore security descriptor from baseline".to_string(),
        description: None,
        object_name: None,
        is_comment: true,
    });
    //     if (!string.IsNullOrEmpty(baselineSddl))
    //     {
    //         commands.Add(new RemediationCommand
    //         {
    //             Type = CommandType.PowerShell,
    //             Command = $"$sddl = '{baselineSddl}'",
    //             Description = "Set SDDL variable"
    //         });
    if let Some(baseline_sd) = baseline_sd
        && !baseline_sd.is_empty()
    {
        if let Some(sddl) = is_probably_sddl_string(baseline_sd) {
            commands.push(RemediationCommand {
                command_type: CommandType::PowerShell,
                command: format!("$sddl = '{}'", escape_powershell_single_quoted(sddl)),
                description: Some("Set SDDL variable".to_string()),
                object_name: None,
                is_comment: false,
            });
        } else {
            let baseline_sd_b64 = STANDARD.encode(baseline_sd);
            commands.push(RemediationCommand {
                command_type: CommandType::PowerShell,
                command: format!(
                    "$baselineSd = [Convert]::FromBase64String('{}')",
                    baseline_sd_b64
                ),
                description: Some("Decode baseline security descriptor".to_string()),
                object_name: None,
                is_comment: false,
            });
            commands.push(RemediationCommand {
                command_type: CommandType::PowerShell,
                command: "$sddl = (New-Object System.Security.AccessControl.RawSecurityDescriptor($baselineSd, 0)).GetSddlForm([System.Security.AccessControl.AccessControlSections]::All)".to_string(),
                description: Some("Set SDDL variable".to_string()),
                object_name: None,
                is_comment: false,
            });
        }
        //         commands.Add(new RemediationCommand
        //         {
        //             Type = CommandType.PowerShell,
        //             Command = $"$acl = Get-Acl 'AD:\\{dn}'",
        //             Description = "Get current ACL"
        //         });
        commands.push(RemediationCommand {
            command_type: CommandType::PowerShell,
            command: format!("$acl = Get-Acl 'AD:\\{dn}'"),
            description: Some("Get current ACL".to_string()),
            object_name: None,
            is_comment: false,
        });
        //         commands.Add(new RemediationCommand
        //         {
        //             Type = CommandType.PowerShell,
        //             Command = "$acl.SetSecurityDescriptorSddlForm($sddl)",
        //             Description = "Set SDDL form"
        //         });
        commands.push(RemediationCommand {
            command_type: CommandType::PowerShell,
            command: "$acl.SetSecurityDescriptorSddlForm($sddl)".to_string(),
            description: Some("Set SDDL form".to_string()),
            object_name: None,
            is_comment: false,
        });
        //         commands.Add(new RemediationCommand
        //         {
        //             Type = CommandType.PowerShell,
        //             Command = $"Set-Acl -Path 'AD:\\{dn}' -AclObject $acl",
        //             Description = "Apply ACL to object"
        //         });
        commands.push(RemediationCommand {
            command_type: CommandType::PowerShell,
            command: format!("Set-Acl -Path 'AD:\\{dn}' -AclObject $acl"),
            description: Some("Apply ACL to object".to_string()),
            object_name: None,
            is_comment: false,
        });
        //         commands.Add(new RemediationCommand
        //         {
        //             Type = CommandType.Comment,
        //             Command = "OR use dsacls:",
        //             IsComment = true
        //         });
        commands.push(RemediationCommand {
            command_type: CommandType::Comment,
            command: "# OR use dsacls:".to_string(),
            description: None,
            object_name: None,
            is_comment: false,
        });
        //         commands.Add(new RemediationCommand
        //         {
        //             Type = CommandType.DsAcls,
        //             Command = $"dsacls '{dn}' /S /T",
        //             Description = "Alternative using dsacls",
        //             IsComment = true
        //         });
        commands.push(RemediationCommand {
            command_type: CommandType::DsAcls,
            command: format!("# dsacls '{dn}' /S /T"),
            description: Some("Alternative using dsacls".to_string()),
            object_name: None,
            is_comment: true,
        });
    } else {
        //     }
        //     else
        //     {
        //         commands.Add(new RemediationCommand
        //         {
        //             Type = CommandType.Comment,
        //             Command = "Manual review required - baseline SDDL not captured",
        //             IsComment = true
        //         });
        //     }
        commands.push(RemediationCommand {
            command_type: CommandType::Comment,
            command: "Manual review is required, no baseline SDDL was captured".to_string(),
            description: None,
            object_name: None,
            is_comment: true,
        });
    }
    commands
}
