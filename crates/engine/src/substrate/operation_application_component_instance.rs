use std::collections::BTreeSet;

use uuid::Uuid;

use super::{
    CommitDiff, ComponentInstance, DesignModel, DomainObject, EngineError, LibraryBinding,
    LibraryBindingRole, ObjectId, Operation, RevisionedRef, SourceShardDirtyState, SourceShardKind,
    SourceShardRef,
    component_instance::{
        PersistedComponentInstance, persisted_component_instance_from_value, validate_role_map,
    },
    source_shard::source_shard_taxon_for_path,
    source_shard_authority_for_kind,
};

pub(super) fn apply_component_instance_operation(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    operation: &Operation,
) -> Result<bool, EngineError> {
    match operation {
        Operation::CreateComponentInstance {
            component_instance_id,
            component_instance,
        } => {
            apply_component_instance_create(model, diff, *component_instance_id, component_instance)
        }
        Operation::DeleteComponentInstance {
            component_instance_id,
            ..
        } => apply_component_instance_delete(model, diff, *component_instance_id),
        Operation::SetComponentInstance {
            component_instance_id,
            component_instance,
            ..
        } => apply_component_instance_set(model, diff, *component_instance_id, component_instance),
        _ => return Ok(false),
    }?;
    Ok(true)
}

pub(super) fn apply_component_instance_create(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    component_instance_id: ObjectId,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    let component_instance = persisted_component_instance_from_value(value)?;
    let persisted_component_instance: PersistedComponentInstance =
        serde_json::from_value(value.clone())?;
    validate_component_instance_id(component_instance.id, component_instance_id)?;
    validate_component_instance_refs(
        &component_instance,
        &persisted_component_instance,
        persisted_component_instance.part_ref.as_ref(),
        model,
    )?;
    let shard_id = authored_shard_id(component_instance_id);
    model.objects.insert(
        component_instance_id,
        DomainObject {
            object_id: component_instance_id,
            object_revision: component_instance.object_revision,
            source_shard_id: shard_id,
            domain: "component_instance".to_string(),
            kind: "component_instance".to_string(),
        },
    );
    model
        .component_instances
        .insert(component_instance_id, component_instance);
    ensure_authored_shard(model, component_instance_id);
    diff.created.push(component_instance_id);
    Ok(())
}

pub(super) fn apply_component_instance_delete(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    component_instance_id: ObjectId,
) -> Result<(), EngineError> {
    model.component_instances.remove(&component_instance_id);
    model.objects.remove(&component_instance_id);
    remove_authored_shard(model, component_instance_id);
    diff.deleted.push(component_instance_id);
    Ok(())
}

pub(super) fn apply_component_instance_set(
    model: &mut DesignModel,
    diff: &mut CommitDiff,
    component_instance_id: ObjectId,
    value: &serde_json::Value,
) -> Result<(), EngineError> {
    let component_instance = persisted_component_instance_from_value(value)?;
    let persisted_component_instance: PersistedComponentInstance =
        serde_json::from_value(value.clone())?;
    validate_component_instance_id(component_instance.id, component_instance_id)?;
    validate_component_instance_refs(
        &component_instance,
        &persisted_component_instance,
        persisted_component_instance.part_ref.as_ref(),
        model,
    )?;
    let shard_id = authored_shard_id(component_instance_id);
    model.objects.insert(
        component_instance_id,
        DomainObject {
            object_id: component_instance_id,
            object_revision: component_instance.object_revision,
            source_shard_id: shard_id,
            domain: "component_instance".to_string(),
            kind: "component_instance".to_string(),
        },
    );
    model
        .component_instances
        .insert(component_instance_id, component_instance);
    ensure_authored_shard(model, component_instance_id);
    diff.modified.push(component_instance_id);
    Ok(())
}

fn validate_component_instance_id(actual: ObjectId, expected: ObjectId) -> Result<(), EngineError> {
    if actual != expected {
        return Err(EngineError::Validation(format!(
            "component instance id {actual} does not match operation id {expected}"
        )));
    }
    Ok(())
}

fn validate_component_instance_refs(
    component_instance: &ComponentInstance,
    persisted_component_instance: &PersistedComponentInstance,
    part_ref: Option<&RevisionedRef>,
    model: &DesignModel,
) -> Result<(), EngineError> {
    if component_instance.placed_symbol_refs.is_empty() {
        return Err(EngineError::Validation(format!(
            "component instance {} must reference at least one symbol",
            component_instance.id
        )));
    }
    validate_component_instance_ref_set(
        component_instance.id,
        "symbol",
        &component_instance.placed_symbol_refs,
        model,
        "schematic",
    )?;
    if !component_instance.placed_package_refs.is_empty() {
        validate_component_instance_ref_set(
            component_instance.id,
            "package",
            &component_instance.placed_package_refs,
            model,
            "board",
        )?;
    }
    validate_role_map(
        &persisted_component_instance.placed_symbol_refs,
        &persisted_component_instance.placed_symbol_roles,
        "symbol",
    )
    .map_err(|message| {
        EngineError::Validation(format!(
            "component instance {} {message}",
            component_instance.id
        ))
    })?;
    validate_role_map(
        &persisted_component_instance.placed_package_refs,
        &persisted_component_instance.placed_package_roles,
        "package",
    )
    .map_err(|message| {
        EngineError::Validation(format!(
            "component instance {} {message}",
            component_instance.id
        ))
    })?;
    if let Some(part_ref) = part_ref {
        validate_revisioned_pool_ref(
            model,
            component_instance.id,
            "part_ref",
            part_ref.object_id,
            part_ref.object_revision,
            "parts",
        )?;
    }
    validate_library_bindings(component_instance, model, part_ref)?;
    Ok(())
}

fn validate_library_bindings(
    component_instance: &ComponentInstance,
    model: &DesignModel,
    part_ref: Option<&RevisionedRef>,
) -> Result<(), EngineError> {
    let mut part_binding = None;
    for (binding_id, binding) in &component_instance.library_bindings {
        if *binding_id == binding.target_object_id {
            return Err(EngineError::Validation(format!(
                "component instance {} library binding {} must have a stable binding id distinct from target object id",
                component_instance.id, binding_id
            )));
        }
        validate_library_binding(component_instance.id, *binding_id, binding, model)?;
        if binding.binding_role == LibraryBindingRole::Part
            && part_binding.replace(binding).is_some() {
                return Err(EngineError::Validation(format!(
                    "component instance {} must not contain multiple part LibraryBindings",
                    component_instance.id
                )));
            }
    }
    if let (Some(part_ref), Some(part_binding)) = (part_ref, part_binding)
        && (part_ref.object_id != part_binding.target_object_id
            || part_ref.object_revision != part_binding.pinned_object_revision)
        {
            return Err(EngineError::Validation(format!(
                "component instance {} part_ref {}@{} does not match part LibraryBinding {}@{}",
                component_instance.id,
                part_ref.object_id,
                part_ref.object_revision.0,
                part_binding.target_object_id,
                part_binding.pinned_object_revision.0
            )));
        }
    Ok(())
}

fn validate_library_binding(
    component_instance_id: ObjectId,
    binding_id: ObjectId,
    binding: &LibraryBinding,
    model: &DesignModel,
) -> Result<(), EngineError> {
    let expected_kind = match binding.binding_role {
        LibraryBindingRole::Part => "parts",
        LibraryBindingRole::Symbol => "symbols",
        LibraryBindingRole::Package => "packages",
        LibraryBindingRole::Footprint => "footprints",
        LibraryBindingRole::PinPadMap => "pin_pad_maps",
        LibraryBindingRole::ModelAttachment => "models",
    };
    validate_revisioned_pool_ref(
        model,
        component_instance_id,
        "library binding target",
        binding.target_object_id,
        binding.pinned_object_revision,
        expected_kind,
    )?;
    for reference in &binding.local_override_refs {
        validate_revisioned_pool_ref(
            model,
            component_instance_id,
            "library binding local override",
            reference.object_id,
            reference.object_revision,
            expected_kind,
        )?;
    }
    if binding_id == Uuid::nil() {
        return Err(EngineError::Validation(format!(
            "component instance {component_instance_id} library binding id must not be nil"
        )));
    }
    Ok(())
}

fn validate_revisioned_pool_ref(
    model: &DesignModel,
    component_instance_id: ObjectId,
    label: &str,
    object_id: ObjectId,
    object_revision: super::ObjectRevision,
    expected_kind: &str,
) -> Result<(), EngineError> {
    let Some(object) = model.objects.get(&object_id) else {
        return Err(EngineError::NotFound {
            object_type: "component_instance_library_ref",
            uuid: object_id,
        });
    };
    if object.object_revision != object_revision
        || object.domain != "pool"
        || object.kind != expected_kind
    {
        return Err(EngineError::Validation(format!(
            "component instance {component_instance_id} {label} {object_id} must target a current pool/{expected_kind} object, got {}/{} revision {:?}",
            object.domain, object.kind, object.object_revision
        )));
    }
    Ok(())
}

fn validate_component_instance_ref_set(
    component_instance_id: ObjectId,
    label: &str,
    object_ids: &[ObjectId],
    model: &DesignModel,
    expected_domain: &str,
) -> Result<(), EngineError> {
    let mut seen = BTreeSet::new();
    for object_id in object_ids {
        if !seen.insert(*object_id) {
            return Err(EngineError::Validation(format!(
                "component instance {component_instance_id} has duplicate {label} ref {object_id}"
            )));
        }
        let Some(object) = model.objects.get(object_id) else {
            return Err(EngineError::NotFound {
                object_type: "component_instance_ref",
                uuid: *object_id,
            });
        };
        if object.domain != expected_domain {
            return Err(EngineError::Validation(format!(
                "component instance {component_instance_id} {label} ref {object_id} must target {expected_domain} domain, got {}/{}",
                object.domain, object.kind
            )));
        }
    }
    Ok(())
}

pub(super) fn authored_relative_path(component_instance_id: ObjectId) -> String {
    format!(".datum/component_instances/{component_instance_id}.json")
}

pub(super) fn authored_shard_id(component_instance_id: ObjectId) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!(
            "datum-eda:source-shard:{}",
            authored_relative_path(component_instance_id)
        )
        .as_bytes(),
    )
}

fn ensure_authored_shard(model: &mut DesignModel, component_instance_id: ObjectId) {
    let relative_path = authored_relative_path(component_instance_id);
    if model
        .source_shards
        .iter()
        .any(|shard| shard.relative_path == relative_path)
    {
        return;
    }
    model.source_shards.push(SourceShardRef {
        shard_id: authored_shard_id(component_instance_id),
        kind: SourceShardKind::ComponentInstance,
        taxon: source_shard_taxon_for_path(&SourceShardKind::ComponentInstance, &relative_path),
        path: std::path::PathBuf::from(&relative_path),
        relative_path,
        authority: source_shard_authority_for_kind(&SourceShardKind::ComponentInstance),
        dirty_state: SourceShardDirtyState::Clean,
        schema_version: Some(1),
        content_hash: String::new(),
    });
}

fn remove_authored_shard(model: &mut DesignModel, component_instance_id: ObjectId) {
    let relative_path = authored_relative_path(component_instance_id);
    model
        .source_shards
        .retain(|shard| shard.relative_path != relative_path);
}
