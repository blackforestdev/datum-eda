//! Authored object registrations and revision diffs for promoted journal history.
use super::super::{DomainObject, ObjectId, ObjectRevision, Operation, TransactionRecord};
use std::collections::BTreeMap;
use uuid::Uuid;

pub(in crate::substrate) fn apply_promoted_journal_object_diffs(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    journal: &[TransactionRecord],
) {
    let final_sources = objects.clone();
    initialize_rules_epochs(objects, &final_sources, journal);
    for transaction in journal {
        for operation in &transaction.operations {
            if let Operation::CreateProjectRule {
                rules_root_id,
                rule_id,
                rule,
            } = operation
            {
                if let Some(root) = objects
                    .get(rules_root_id)
                    .filter(|root| root.domain == "rules" && root.kind == "$")
                {
                    objects.insert(
                        *rule_id,
                        DomainObject {
                            object_id: *rule_id,
                            object_revision: super::super::object_revision_for(rule),
                            source_shard_id: root.source_shard_id,
                            domain: "rules".into(),
                            kind: "rules".into(),
                        },
                    );
                }
            }
            if let Operation::CreateBoardGraphic {
                board_id,
                graphic_id,
                ..
            } = operation
                && let Some(board) = objects
                    .get(board_id)
                    .filter(|o| o.domain == "board" && o.kind == "$")
            {
                let shard_id = board.source_shard_id;
                objects.entry(*graphic_id).or_insert_with(|| DomainObject {
                    object_id: *graphic_id,
                    object_revision: ObjectRevision(0),
                    source_shard_id: shard_id,
                    domain: "board".into(),
                    kind: graphic_id.to_string(),
                });
            }
            if let Operation::CreateSchematicSheet {
                sheet_id,
                relative_path,
                ..
            } = operation
            {
                let shard_id = Uuid::new_v5(
                    &Uuid::NAMESPACE_URL,
                    format!("datum-eda:source-shard:schematic/{relative_path}").as_bytes(),
                );
                objects
                    .entry(*sheet_id)
                    .and_modify(|object| {
                        // Source collection sees a JSON root; the validated
                        // creation operation retains its canonical sheet kind.
                        if object.domain == "schematic"
                            && object.source_shard_id == shard_id
                            && object.kind == "$"
                        {
                            object.kind = "schematic_sheet".to_string();
                        }
                    })
                    .or_insert_with(|| DomainObject {
                        object_id: *sheet_id,
                        object_revision: ObjectRevision(0),
                        source_shard_id: shard_id,
                        domain: "schematic".to_string(),
                        kind: "schematic_sheet".to_string(),
                    });
            }
            if let Operation::CreateSchematicDefinition {
                definition_id,
                relative_path,
                ..
            } = operation
            {
                objects
                    .entry(*definition_id)
                    .or_insert_with(|| DomainObject {
                        object_id: *definition_id,
                        object_revision: ObjectRevision(0),
                        source_shard_id: Uuid::new_v5(
                            &Uuid::NAMESPACE_URL,
                            format!("datum-eda:source-shard:schematic/{relative_path}").as_bytes(),
                        ),
                        domain: "schematic".to_string(),
                        kind: "schematic_definition".to_string(),
                    });
            }
            if let Operation::CreatePoolLibraryObject {
                object_id,
                relative_path,
                object_kind,
                ..
            } = operation
            {
                objects.entry(*object_id).or_insert_with(|| DomainObject {
                    object_id: *object_id,
                    object_revision: ObjectRevision(0),
                    source_shard_id: Uuid::new_v5(
                        &Uuid::NAMESPACE_URL,
                        format!("datum-eda:source-shard:{relative_path}").as_bytes(),
                    ),
                    domain: "pool".to_string(),
                    kind: object_kind.clone(),
                });
            }
        }
        for id in &transaction.diff.created {
            if let Some(source) = final_sources.get(id) {
                objects.entry(*id).or_insert_with(|| source.clone());
            }
        }
        restore_component_pad_registrations(objects, transaction);
        for object_id in &transaction.diff.modified {
            if let Some(object) = objects.get_mut(object_id) {
                // A board import can set both outline and stackup. Canonical
                // application advances the root for each operation, while the
                // diff lists each modified object only once. Restore those
                // recorded advances rather than truncating valid history.
                let root_advances = if object.domain == "board" && object.kind == "$" {
                    transaction
                        .operations
                        .iter()
                        .filter(|op| match op {
                            Operation::AdoptBoardSourceProfile { board_id, .. }
                            | Operation::SetBoardOutline { board_id, .. }
                            | Operation::SetBoardStackup { board_id, .. }
                            | Operation::SetBoardName { board_id, .. } => board_id == object_id,
                            Operation::BumpObjectRevision { object_id: id } => id == object_id,
                            _ => false,
                        })
                        .count() as u64
                } else if object.domain == "project" && object.kind == "$" {
                    transaction
                        .operations
                        .iter()
                        .filter(|operation| match operation {
                            Operation::LibraryAssets { operation } => {
                                operation.owner_id() == *object_id
                            }
                            Operation::SetProjectName { project_id, .. }
                            | Operation::InitializeProjectDisplayUnits { project_id, .. }
                            | Operation::SetProjectDisplayUnits { project_id, .. }
                            | Operation::RemoveProjectDisplayUnits { project_id, .. } => {
                                project_id == object_id
                            }
                            Operation::Process { operation } => operation.owner_id() == *object_id,
                            Operation::BumpObjectRevision { object_id: id } => id == object_id,
                            _ => false,
                        })
                        .count() as u64
                } else if object.domain == "rules" && matches!(object.kind.as_str(), "$" | "rules")
                {
                    transaction.operations.iter().filter(|op| {
                        (object.kind=="$"&&rules_root(op)==Some(*object_id))
                            ||(object.kind=="rules"&&matches!(op,Operation::SetProjectRule {rule_id,..} if rule_id==object_id))
                            ||matches!(op,Operation::BumpObjectRevision {object_id:id} if id==object_id)
                    }).count() as u64
                } else if object.domain == "pool" {
                    transaction
                        .operations
                        .iter()
                        .map(|op| {
                            super::super::operation_application_pool_objects::root_revision_advance(
                                op, *object_id,
                            )
                        })
                        .sum()
                } else if object.domain == "board" {
                    let package_advances = transaction
                        .operations
                        .iter()
                        .filter(|operation| {
                            super::super::operation_application_objects::package_edit_target(
                                operation,
                            ) == Some(*object_id)
                        })
                        .count() as u64;
                    if package_advances == 0 {
                        0
                    } else {
                        package_advances
                            + transaction
                                .operations
                                .iter()
                                .filter(|operation| {
                                    matches!(operation,
                            Operation::BumpObjectRevision { object_id: id } if id == object_id)
                                })
                                .count() as u64
                    }
                } else {
                    0
                };
                object.object_revision =
                    ObjectRevision(object.object_revision.0 + root_advances.max(1));
            }
        }
        for object_id in &transaction.diff.deleted {
            objects.remove(object_id);
        }
        restore_library_source_registrations(objects, transaction);
        // Electrical sidecars already store their exact object revision. Generic
        // promoted diffs must not add historical advances a second time. Restore
        // the typed operation postimage, including inverse/redo records and
        // deletions, after the compatibility object-diff reconstruction.
        for operation in &transaction.operations {
            if let Some((id, record, delete)) =
                super::super::electrical_identity_store::write(operation)
            {
                if delete {
                    objects.remove(&id);
                } else {
                    let path = super::super::electrical_identity_store::relative_path(id);
                    let shard_id = Uuid::new_v5(
                        &Uuid::NAMESPACE_URL,
                        format!("datum-eda:source-shard:{path}").as_bytes(),
                    );
                    objects.insert(
                        id,
                        super::super::electrical_identity_store::object(record, shard_id),
                    );
                }
            }
        }
    }
}

fn rules_root(operation: &Operation) -> Option<ObjectId> {
    match operation {
        Operation::SetProjectRules { rules_root_id, .. }
        | Operation::CreateProjectRule { rules_root_id, .. }
        | Operation::SetProjectRule { rules_root_id, .. }
        | Operation::DeleteProjectRule { rules_root_id, .. } => Some(*rules_root_id),
        _ => None,
    }
}

/// Rules persist owning counters in the final source. Start compatibility replay
/// at their witnessed pre-journal epochs, then apply actual operation advances.
/// A declared creation supplies its own postimage; it must not inherit a future
/// row's counter from final-source discovery. Other object families are unchanged.
fn initialize_rules_epochs(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    final_sources: &BTreeMap<ObjectId, DomainObject>,
    journal: &[TransactionRecord],
) {
    for source in final_sources
        .values()
        .filter(|source| source.domain == "rules" && matches!(source.kind.as_str(), "$" | "rules"))
    {
        let mut advances = 0u64;
        let mut witness = None;
        let mut created = false;
        for op in journal.iter().flat_map(|tx| &tx.operations) {
            if source.kind == "$" {
                advances += u64::from(rules_root(op) == Some(source.object_id));
            } else {
                match op {
                    Operation::CreateProjectRule { rule_id, .. }
                        if *rule_id == source.object_id =>
                    {
                        created = true;
                        break;
                    }
                    Operation::DeleteProjectRule { rule_id, rule, .. }
                        if *rule_id == source.object_id =>
                    {
                        witness = Some(
                            super::super::object_revision_for(rule)
                                .0
                                .checked_sub(advances),
                        );
                        break;
                    }
                    Operation::SetProjectRule { rule_id, .. } if *rule_id == source.object_id => {
                        advances += 1
                    }
                    _ => {}
                }
            }
        }
        if created {
            objects.remove(&source.object_id);
        } else if let Some(epoch) =
            witness.unwrap_or_else(|| source.object_revision.0.checked_sub(advances))
        {
            if let Some(object) = objects.get_mut(&source.object_id) {
                object.object_revision = ObjectRevision(epoch);
            }
        }
    }
}

fn restore_component_pad_registrations(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    transaction: &TransactionRecord,
) {
    for op in &transaction.operations {
        let (package, materialized) = match op {
            Operation::CreateBoardPackage {
                package_id,
                materialized,
                ..
            }
            | Operation::SetBoardPackagePackage {
                package_id,
                materialized,
                ..
            } => (*package_id, materialized),
            _ => continue,
        };
        let Some(shard) = objects.get(&package).map(|o| o.source_shard_id) else {
            continue;
        };
        let Some(pads) = materialized
            .get("component_pads")
            .and_then(serde_json::Value::as_array)
        else {
            continue;
        };
        for value in pads {
            if let Ok(pad) = serde_json::from_value::<
                super::super::component_pad_source::ComponentPadSource,
            >(value.clone())
                && (pad.source_binding.is_some() || super::super::component_pad_source::touches(op))
            {
                let registration = DomainObject {
                    object_id: pad.uuid,
                    object_revision: ObjectRevision(0),
                    source_shard_id: shard,
                    domain: "board".into(),
                    kind: package.to_string(),
                };
                if transaction.diff.created.contains(&pad.uuid) {
                    objects.insert(pad.uuid, registration);
                } else {
                    objects.entry(pad.uuid).or_insert(registration);
                }
            }
        }
    }
}
fn restore_library_source_registrations(
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    transaction: &TransactionRecord,
) {
    for op in &transaction.operations {
        let materialized = match op {
            Operation::CreateBoardPackage { materialized, .. }
            | Operation::SetBoardPackagePackage { materialized, .. } => materialized,
            _ => continue,
        };
        if let Some(value)=materialized.get("pad_source_transition") && let Ok(proof)=serde_json::from_value::<crate::api::native_write::board_components::pad_materialization::ComponentPadTransition>(value.clone()) {objects.extend(proof.library_source_registrations);}
    }
}
