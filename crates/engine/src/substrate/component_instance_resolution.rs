//! Validate ComponentInstance projections against the accepted final object graph.
//! Provisional load errors are never substituted for this current-state result.
use super::*;
pub(in crate::substrate) fn resolve_component_instance_sources(
    sources: ComponentInstanceSources,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    diagnostics: &mut Vec<ResolveDiagnostic>,
) -> BTreeMap<ComponentInstanceId, ComponentInstance> {
    let mut instances = BTreeMap::new();
    for (_, (shard, input)) in sources {
        insert_component_instance(&shard, input, objects, &mut instances, diagnostics, true);
    }
    instances
}
pub(super) fn insert_component_instance(
    shard: &SourceShardRef,
    input: PersistedComponentInstance,
    objects: &mut BTreeMap<ObjectId, DomainObject>,
    instances: &mut BTreeMap<ComponentInstanceId, ComponentInstance>,
    diagnostics: &mut Vec<ResolveDiagnostic>,
    final_state: bool,
) {
    let Some(filename_id) = shard
        .path
        .file_stem()
        .and_then(|value| value.to_str())
        .and_then(|value| Uuid::parse_str(value).ok())
    else {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_invalid_filename".to_string(),
            message: format!(
                "component instance shard filename must be <uuid>.json: {}",
                shard.relative_path
            ),
            path: Some(shard.path.clone()),
        });
        return;
    };
    if filename_id != input.uuid {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_filename_mismatch".to_string(),
            message: format!(
                "component instance shard filename {} does not match embedded uuid {}",
                filename_id, input.uuid
            ),
            path: Some(shard.path.clone()),
        });
        return;
    }
    let same_source = final_state
        && objects.get(&input.uuid).is_some_and(|o| {
            o.source_shard_id == shard.shard_id
                && o.domain == "component_instance"
                && o.kind == "component_instance"
        });
    if instances.contains_key(&input.uuid) || (objects.contains_key(&input.uuid) && !same_source) {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_duplicate_id".to_string(),
            message: format!("duplicate component instance id {}", input.uuid),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if input.placed_symbol_refs.is_empty() {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_empty_refs".to_string(),
            message: format!(
                "component instance {} must reference at least one symbol",
                input.uuid
            ),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if !refs_match_objects(&input.placed_symbol_refs, objects)
        || (!input.placed_package_refs.is_empty()
            && !refs_match_objects(&input.placed_package_refs, objects))
    {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_unresolved_ref".to_string(),
            message: format!(
                "component instance {} references missing or stale symbol/package objects",
                input.uuid
            ),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if let Err(message) = validate_role_map(
        &input.placed_symbol_refs,
        &input.placed_symbol_roles,
        "symbol",
    ) {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_invalid_symbol_roles".to_string(),
            message: format!("component instance {} {message}", input.uuid),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if let Err(message) = validate_role_map(
        &input.placed_package_refs,
        &input.placed_package_roles,
        "package",
    ) {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_invalid_package_roles".to_string(),
            message: format!("component instance {} {message}", input.uuid),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if let Some(part_ref) = &input.part_ref
        && !revisioned_pool_ref_matches(
            part_ref.object_id,
            part_ref.object_revision,
            objects,
            "parts",
        )
    {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_invalid_part_ref".to_string(),
            message: format!(
                "component instance {} part_ref {} must target a current pool parts object",
                input.uuid, part_ref.object_id
            ),
            path: Some(shard.path.clone()),
        });
        return;
    }
    if let Err(message) = validate_library_bindings(&input, objects) {
        diagnostics.push(ResolveDiagnostic {
            code: "component_instance_invalid_library_binding".to_string(),
            message: format!("component instance {} {message}", input.uuid),
            path: Some(shard.path.clone()),
        });
        return;
    }
    objects.insert(
        input.uuid,
        DomainObject {
            object_id: input.uuid,
            object_revision: input.object_revision,
            source_shard_id: shard.shard_id,
            domain: "component_instance".to_string(),
            kind: "component_instance".to_string(),
        },
    );
    instances.insert(input.uuid, component_instance_from_persisted(&input));
}

fn validate_library_bindings(
    input: &PersistedComponentInstance,
    objects: &BTreeMap<ObjectId, DomainObject>,
) -> Result<(), String> {
    let mut part_binding = None;
    for (binding_id, binding) in &input.library_bindings {
        if *binding_id == Uuid::nil() || *binding_id == binding.target_object_id {
            return Err(format!(
                "library binding {binding_id} has invalid binding identity"
            ));
        }
        let expected_kind = match binding.binding_role {
            LibraryBindingRole::Part => "parts",
            LibraryBindingRole::Symbol => "symbols",
            LibraryBindingRole::Package => "packages",
            LibraryBindingRole::Footprint => "footprints",
            LibraryBindingRole::PinPadMap => "pin_pad_maps",
            LibraryBindingRole::ModelAttachment => "models",
        };
        if !revisioned_pool_ref_matches(
            binding.target_object_id,
            binding.pinned_object_revision,
            objects,
            expected_kind,
        ) {
            return Err(format!(
                "library binding {binding_id} target {} must resolve to current pool {expected_kind}",
                binding.target_object_id
            ));
        }
        for reference in &binding.local_override_refs {
            if !revisioned_pool_ref_matches(
                reference.object_id,
                reference.object_revision,
                objects,
                expected_kind,
            ) {
                return Err(format!(
                    "library binding {binding_id} local override {} must resolve to current pool {expected_kind}",
                    reference.object_id
                ));
            }
        }
        if binding.binding_role == LibraryBindingRole::Part
            && part_binding.replace(binding).is_some()
        {
            return Err("must not contain multiple part LibraryBindings".to_string());
        }
    }
    if let (Some(part_ref), Some(part_binding)) = (&input.part_ref, part_binding)
        && (part_ref.object_id != part_binding.target_object_id
            || part_ref.object_revision != part_binding.pinned_object_revision)
    {
        return Err(format!(
            "part_ref {}@{} does not match part LibraryBinding {}@{}",
            part_ref.object_id,
            part_ref.object_revision.0,
            part_binding.target_object_id,
            part_binding.pinned_object_revision.0
        ));
    }
    Ok(())
}

fn refs_match_objects(refs: &[RevisionedRef], objects: &BTreeMap<ObjectId, DomainObject>) -> bool {
    refs.iter().all(|reference| {
        revisioned_object_matches(reference.object_id, reference.object_revision, objects)
    })
}

fn revisioned_object_matches(
    object_id: ObjectId,
    object_revision: ObjectRevision,
    objects: &BTreeMap<ObjectId, DomainObject>,
) -> bool {
    objects
        .get(&object_id)
        .map(|object| object.object_revision == object_revision)
        .unwrap_or(false)
}

fn revisioned_pool_ref_matches(
    object_id: ObjectId,
    object_revision: ObjectRevision,
    objects: &BTreeMap<ObjectId, DomainObject>,
    expected_kind: &str,
) -> bool {
    objects
        .get(&object_id)
        .map(|object| {
            object.object_revision == object_revision
                && object.domain == "pool"
                && object.kind == expected_kind
        })
        .unwrap_or(false)
}
