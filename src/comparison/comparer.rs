use crate::{
    objects::directory_objects::DirectoryObject,
    objects::remediation::{ActionType, RemediationAction},
};
use std::collections::{HashMap, HashSet};

// |Target │ Current │ Action │
// ├────────────────────┼─────────────────────┼────────────────────────┤
// │ Live │ Missing │ Create │
// ├────────────────────┼─────────────────────┼────────────────────────┤
// │ Live │ Tombstone │ Reanimate │
// ├────────────────────┼─────────────────────┼────────────────────────┤
// │ Live │ Live (hash differs) │ Modify │
// ├────────────────────┼─────────────────────┼────────────────────────┤
// │ Tombstone / absent │ Live │ Delete (move + remove) │
// ├────────────────────┼─────────────────────┼────────────────────────┤
// │ Tombstone / absent │ Tombstone │ Skip it's already gone |
// public List<RemediationAction> Compare(Dictionary<string, AdObject> currentObjects,
//     Dictionary<string, AdObject> targetObjects)
// {
//     var actions = new List<RemediationAction>();
pub async fn compare_states(
    current: Vec<DirectoryObject>,
    target: Vec<DirectoryObject>,
) -> Result<HashMap<String, Vec<RemediationAction>>, Box<dyn std::error::Error + Send + Sync>> {
    let mut actions: HashMap<String, Vec<RemediationAction>> = HashMap::new();
    //   DirectoryObjects keyed by DN, then we tiptoe through the union of keys.
    let current_map: HashMap<String, DirectoryObject> = current
        .into_iter()
        .map(|obj| (obj.dn.clone(), obj))
        .collect();
    let target_map: HashMap<String, DirectoryObject> = target
        .into_iter()
        .map(|obj| (obj.dn.clone(), obj))
        .collect();
    let all_dns: HashSet<String> = current_map
        .keys()
        .chain(target_map.keys())
        .cloned()
        .collect();
    for dn in all_dns {
        let current_obj = current_map.get(&dn);
        let target_obj = target_map.get(&dn);
        match (current_obj, target_obj) {
            (Some(current), Some(target)) => {
                if current.hash != target.hash {
                    // was the object deleted and needs to be reanimated?
                    // if (IsDeletedObject(current) && !IsDeletedObject(target))
                    // {
                    if current.is_deleted && !target.is_deleted {
                        actions
                            .entry(dn.clone())
                            .or_default()
                            .push(RemediationAction {
                                action: ActionType::Reanimate,
                                target: Some(target.clone()),
                                current: Some(current.clone()),
                                last_known_parent: None,
                            });
                    } else if !current.is_deleted && target.is_deleted {
                        actions
                            .entry(dn.clone())
                            .or_default()
                            .push(RemediationAction {
                                action: ActionType::Delete,
                                target: Some(target.clone()),
                                current: Some(current.clone()),
                                last_known_parent: target
                                    .attributes
                                    .get("LastKnownParent")
                                    .and_then(|v| v.first().cloned()),
                            });
                    } else {
                        actions
                            .entry(dn.clone())
                            .or_default()
                            .push(RemediationAction {
                                action: ActionType::Modify,
                                target: Some(target.clone()),
                                current: Some(current.clone()),
                                last_known_parent: None,
                            });
                    }
                }
            }
            (Some(_), None) => actions
                .entry(dn.clone())
                .or_default()
                .push(RemediationAction {
                    action: ActionType::Delete,
                    target: None,
                    current: current_obj.cloned(),
                    last_known_parent: None,
                }),
            (None, Some(_)) => actions
                .entry(dn.clone())
                .or_default()
                .push(RemediationAction {
                    action: ActionType::Create,
                    target: target_obj.cloned(),
                    current: None,
                    last_known_parent: None,
                }),
            (None, None) => {
                // this should never happen since we're iterating over the union of keys
                continue;
            }
        }
    }
    let reconciled_actions: HashMap<String, Vec<RemediationAction>> = actions
        .into_iter()
        .map(|(dn, acts)| (dn, reconcile_tombstones(acts)))
        .map(|(dn, acts)| (dn, sort_actions(acts)))
        .collect();
    Ok(reconciled_actions)
}
// i'll try to remember to remove this later, but I want to see it now
// let empty_vals: Vec<String> = Vec::new();
// for attr in current
//     .attributes
//     .keys()
//     .chain(target.attributes.keys())
//     .collect::<HashSet<_>>()
// {
//     let current_vals = current.attributes.get(attr).unwrap_or(&empty_vals);
//     let target_vals = target.attributes.get(attr).unwrap_or(&empty_vals);
//     if current_vals != target_vals {
//         println!("  Attribute '{}' differs", attr);
//     }
// }
fn sort_actions(actions: Vec<RemediationAction>) -> Vec<RemediationAction> {
    let mut creates: Vec<RemediationAction> = actions
        .iter()
        .filter(|a| a.action == ActionType::Create)
        .cloned()
        .collect();
    let reanimates: Vec<RemediationAction> = actions
        .iter()
        .filter(|a| a.action == ActionType::Reanimate)
        .cloned()
        .collect();
    let modifies: Vec<RemediationAction> = actions
        .iter()
        .filter(|a| a.action == ActionType::Modify)
        .cloned()
        .collect();
    let mut deletes: Vec<RemediationAction> = actions
        .iter()
        .filter(|a| a.action == ActionType::Delete)
        .cloned()
        .collect();

    creates.sort_by_key(|a| a.target.as_ref().map_or(0, |t| t.dn.matches(',').count()));
    deletes.sort_by_key(|a| a.current.as_ref().map_or(0, |c| c.dn.matches(',').count()));
    deletes.reverse(); // sorry fam, kids before parents

    [creates, reanimates, modifies, deletes].concat()
}

//     private static List<RemediationAction> ReconcileTombstones(List<RemediationAction> actions)
//     {
fn reconcile_tombstones(actions: Vec<RemediationAction>) -> Vec<RemediationAction> {
    let mut actions = actions;
    // Find actions where Action == Create && IsDeletedObject(action.Target)
    // These are unmatched tombstones that shouldn't be created

    let unmatched_tombstones: Vec<RemediationAction> = actions
        .iter()
        .filter(|action| {
            action.action == ActionType::Create
                && action.target.is_some()
                && action
                    .target
                    .as_ref()
                    .is_some_and(|target| target.is_deleted)
        })
        .cloned()
        .collect::<Vec<RemediationAction>>();
    // Find actions where Action == Delete && action.LastKnownParent == null
    // These are live objects with no target, no parent info

    let mut parentless_deletes: Vec<RemediationAction> = actions
        .iter()
        .filter(|action| action.action == ActionType::Delete && action.last_known_parent.is_none())
        .cloned()
        .collect::<Vec<RemediationAction>>();

    // For each unmatched tombstone, reconstruct its CN by stripping \0ADEL:
    for tombstone in &unmatched_tombstones {
        //             var tombstoneTarget = tombstone.Target;
        let tombstone_target = tombstone.target.as_ref();
        //             if (tombstoneTarget == null)
        //                 continue;
        if tombstone_target.is_none() {
            continue;
        }
        //             var cn = tombstoneTarget.Name.Split(new[] { "_x000A_DEL:" }, StringSplitOptions.None)[0];
        let cn = tombstone_target
            .and_then(|target| target.name.as_deref())
            .and_then(|name| name.split("_x000A_DEL:").next())
            .unwrap_or("");
        //             var match = parentlessDeletes.FirstOrDefault(d =>
        //                 d.Current != null && d.Current.Name.Equals(cn, StringComparison.OrdinalIgnoreCase));

        let matching_delete_index = parentless_deletes.iter().position(|directory_object| {
            directory_object
                .current
                .as_ref()
                .and_then(|current| current.name.as_deref())
                .is_some_and(|name| name.eq_ignore_ascii_case(cn))
        });
        //             if (match != null)
        //             {
        if let Some(matching_delete_index) = matching_delete_index {
            let matching_delete = parentless_deletes.remove(matching_delete_index);
            // If there's a match remove both, add one Delete action with Current from the live object and
            // LastKnownParent pulled from the tombstone's lastKnownParent attribute
            actions.retain(|action| action != tombstone && action != &matching_delete);
            actions.push(RemediationAction {
                action: ActionType::Delete,
                target: tombstone_target.cloned(),
                current: matching_delete.current.clone(),
                last_known_parent: tombstone_target.and_then(|target| {
                    target
                        .attributes
                        .get("lastknownparent")
                        .and_then(|value| value.first().cloned())
                        .or_else(|| {
                            target
                                .attributes
                                .get("LastKnownParent")
                                .and_then(|value| value.first().cloned())
                        })
                }),
            });
        }
    }

    // Any unmatched tombstones left over after reconciliation should be removed from
    // actions entirely, a tombstone in target with no live counterpart means the object
    // is already properly deleted
    actions.retain(|action| {
        !unmatched_tombstones
            .iter()
            .any(|target_action| target_action == action)
    });
    actions
}
