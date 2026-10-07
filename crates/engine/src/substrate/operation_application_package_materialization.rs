//! Existing package application with stable registered PM057 Pad lifetimes.
use super::operation_application_board_payloads::{
    board_payload_objects, materialized_payload_objects,
};
use super::{CommitDiff, DesignModel, EngineError, ObjectRevision, Operation};
use std::collections::BTreeSet;
pub(super) fn apply(
    model: &mut DesignModel,
    operation: &Operation,
    diff: &mut CommitDiff,
) -> Result<bool, EngineError> {
    match operation {
        Operation::CreateBoardPackage {
            package_id,
            package,
            materialized,
        } => {
            let created = board_payload_objects(model, *package_id, package, materialized)?;
            for object_id in created.keys() {
                diff.created.push(*object_id);
            }
            model.objects.extend(created);
            recover_library_registrations(model, materialized)?;
            Ok(true)
        }
        Operation::DeleteBoardPackage {
            package_id,
            package,
            materialized,
        } => {
            let deleted = board_payload_objects(model, *package_id, package, materialized)?;
            for object_id in deleted.keys() {
                if model.objects.remove(object_id).is_some() {
                    diff.deleted.push(*object_id);
                }
            }
            Ok(true)
        }
        Operation::SetBoardPackagePackage {
            package_id,
            previous_materialized,
            materialized,
            ..
        } => {
            let deleted = materialized_payload_objects(model, *package_id, previous_materialized)?;
            let mut created = materialized_payload_objects(model, *package_id, materialized)?;
            let bound = super::component_pad_source::touches(operation)
                || previous_materialized
                    .get("component_pads")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|v| {
                        v.iter()
                            .any(|p| p.get("source_binding").is_some_and(|v| !v.is_null()))
                    });
            let kept: BTreeSet<_> = if bound {
                deleted
                    .keys()
                    .filter(|id| created.contains_key(id))
                    .copied()
                    .collect()
            } else {
                BTreeSet::new()
            };
            for id in deleted.keys().filter(|id| !kept.contains(id)) {
                if model
                    .objects
                    .get(id)
                    .is_some_and(|o| !bound || o.domain == "board")
                {
                    model.objects.remove(id);
                    diff.deleted.push(*id);
                }
            }
            for (id, object) in &mut created {
                if kept.contains(id) {
                    let previous = model.objects.get(id).ok_or_else(|| {
                        super::component_pad_source::invalid("missing preserved registration")
                    })?;
                    if previous.domain != "board"
                        || previous.source_shard_id != object.source_shard_id
                    {
                        return Err(super::component_pad_source::invalid(
                            "foreign preserved registration",
                        ));
                    }
                    object.object_revision =
                        ObjectRevision(previous.object_revision.0.checked_add(1).ok_or_else(
                            || super::component_pad_source::invalid("Pad revision overflow"),
                        )?);
                    diff.modified.push(*id);
                } else {
                    diff.created.push(*id);
                }
            }
            model.objects.extend(created);
            recover_library_registrations(model, materialized)?;
            let object = model
                .objects
                .get_mut(package_id)
                .ok_or(EngineError::NotFound {
                    object_type: "domain_object",
                    uuid: *package_id,
                })?;
            object.object_revision = ObjectRevision(object.object_revision.0 + 1);
            diff.modified.push(*package_id);
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn recover_library_registrations(
    model: &mut DesignModel,
    materialized: &serde_json::Value,
) -> Result<(), EngineError> {
    if let Some(value) = materialized.get("pad_source_transition") {
        let transition:crate::api::native_write::board_components::pad_materialization::ComponentPadTransition=serde_json::from_value(value.clone())?;
        model
            .objects
            .extend(transition.library_source_registrations);
    }
    Ok(())
}
