//! Explicit Map declarations share the native ComponentInstance mutation path.
use super::*;
use crate::substrate::component_pad_source as source;
use crate::substrate::{LibraryBindingRole, SourceShardKind, SourceShardTaxon};
use serde_json::Value;

/// Adapter intent distinguishes omission from a current Part selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentInstancePartChoice {
    Selected,
    Preserve,
}

/// Build the batch that binds a new component instance.
///
/// When `component_instance_id` is `None` the id is derived deterministically
/// via [`ids::derive_component_instance_id`]. Returns the prepared write and
/// the (possibly derived) component-instance id.
pub fn build_bind_component_instance(
    model: &DesignModel,
    provenance: WriteProvenance,
    component_instance_id: Option<ComponentInstanceId>,
    spec: &ComponentInstanceSpec,
) -> Result<(PreparedWrite, ComponentInstanceId), EngineError> {
    build_bind_component_instance_with_pin_pad_map(
        model,
        provenance,
        component_instance_id,
        spec,
        None,
    )
}

/// Declare an explicitly selected current Map; absence never selects a default.
pub fn build_bind_component_instance_with_pin_pad_map(
    model: &DesignModel,
    provenance: WriteProvenance,
    component_instance_id: Option<ComponentInstanceId>,
    spec: &ComponentInstanceSpec,
    pin_pad_map: Option<Uuid>,
) -> Result<(PreparedWrite, ComponentInstanceId), EngineError> {
    let component_instance_id = component_instance_id.unwrap_or_else(|| {
        ids::derive_component_instance_id(
            &model.project.project_id,
            &spec.symbol_ids,
            spec.package_id,
        )
    });
    let mut payload = component_instance_payload(model, component_instance_id, 0, spec)?;
    declare_map(
        model,
        component_instance_id,
        spec,
        &mut payload,
        None,
        pin_pad_map,
    )?;
    let prepared = BatchComposer::compose(model, provenance)
        .push_op(Operation::CreateComponentInstance {
            component_instance_id,
            component_instance: payload,
        })
        .primary_object(component_instance_id)
        .finish()?;
    Ok((prepared, component_instance_id))
}

/// Build the batch that rewrites an existing authored component instance to
/// the state described by `spec` (object revision advances by one).
pub fn build_set_component_instance(
    model: &DesignModel,
    provenance: WriteProvenance,
    component_instance_id: ComponentInstanceId,
    spec: &ComponentInstanceSpec,
) -> Result<PreparedWrite, EngineError> {
    build_set_component_instance_with_pin_pad_map(
        model,
        provenance,
        component_instance_id,
        spec,
        None,
        ComponentInstancePartChoice::Selected,
    )
}

/// Omitted Map choice preserves an existing declaration, including its pins.
pub fn build_set_component_instance_with_pin_pad_map(
    model: &DesignModel,
    provenance: WriteProvenance,
    component_instance_id: ComponentInstanceId,
    spec: &ComponentInstanceSpec,
    pin_pad_map: Option<Uuid>,
    part_choice: ComponentInstancePartChoice,
) -> Result<PreparedWrite, EngineError> {
    let previous = authored_component_instance(model, component_instance_id)?;
    let mut selected_spec = spec.clone();
    if part_choice == ComponentInstancePartChoice::Preserve {
        if spec.part_id.is_some() && spec.part_id != previous.part_ref {
            return Err(invalid(
                "preserved Part conflicts with an explicit replacement",
            ));
        }
        selected_spec.part_id = previous.part_ref;
    }
    let spec = &selected_spec;
    let mapped = pin_pad_map.is_some()
        || previous
            .library_bindings
            .values()
            .any(|b| b.binding_role == LibraryBindingRole::PinPadMap);
    if part_choice == ComponentInstancePartChoice::Preserve && !mapped {
        return Err(invalid(
            "preserved Part intent requires a mapped declaration",
        ));
    }
    let previous_payload = if mapped {
        let object = &model.objects[&component_instance_id];
        let shard = model
            .source_shards
            .iter()
            .find(|s| {
                s.kind == SourceShardKind::ComponentInstance && s.shard_id == object.source_shard_id
            })
            .ok_or_else(|| invalid("missing authored ComponentInstance source"))?;
        model.materialized_source_shard_value_by_relative_path(&shard.relative_path)?["component_instance"].clone()
    } else {
        component_instance_payload_from_instance(model, &previous)?
    };
    let next_revision = previous.object_revision.0 + 1;
    let mut payload =
        component_instance_payload(model, component_instance_id, next_revision, spec)?;
    if mapped {
        // Retain authored siblings and bindings rather than reconstructing them.
        let mut merged = previous_payload.clone();
        for key in [
            "object_revision",
            "part_ref",
            "placed_symbol_refs",
            "placed_package_refs",
            "placed_symbol_roles",
            "placed_package_roles",
        ] {
            merged[key] = payload[key].clone();
        }
        if part_choice == ComponentInstancePartChoice::Preserve {
            merged["part_ref"] = previous_payload["part_ref"].clone();
        } else {
            let bindings = merged["library_bindings"]
                .as_object_mut()
                .ok_or_else(|| invalid("invalid authored bindings"))?;
            bindings.retain(|_, value| value["binding_role"] != "part");
            bindings.extend(
                payload["library_bindings"]
                    .as_object()
                    .ok_or_else(|| invalid("invalid Part binding"))?
                    .clone(),
            );
        }
        declare_map(
            model,
            component_instance_id,
            spec,
            &mut merged,
            Some(&previous),
            pin_pad_map,
        )?;
        payload = merged;
    }
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetComponentInstance {
            component_instance_id,
            previous_component_instance: previous_payload,
            component_instance: payload,
        })
        .primary_object(component_instance_id)
        .finish()
}

fn invalid(message: &str) -> EngineError {
    EngineError::Validation(format!("ComponentInstance Map binding: {message}"))
}

fn declare_map(
    model: &DesignModel,
    id: Uuid,
    spec: &ComponentInstanceSpec,
    payload: &mut Value,
    previous: Option<&ComponentInstance>,
    selected: Option<Uuid>,
) -> Result<(), EngineError> {
    let prior = previous
        .into_iter()
        .flat_map(|p| &p.library_bindings)
        .filter(|(_, b)| b.binding_role == LibraryBindingRole::PinPadMap)
        .collect::<Vec<_>>();
    if prior.len() > 1 {
        return Err(invalid("ambiguous declared Maps"));
    }
    let Some(target) = selected.or_else(|| prior.first().map(|(_, b)| b.target_object_id)) else {
        return Ok(());
    };
    let part = spec
        .part_id
        .ok_or_else(|| invalid("an explicit Part is required"))?;
    if selected.is_none() && previous.is_some_and(|p| p.part_ref != Some(part)) {
        return Err(invalid(
            "changing Part conflicts with the existing declaration",
        ));
    }
    let revision = if selected.is_some() {
        source::capture_root(model, target, SourceShardTaxon::PoolPinPadMap)?.object_revision
    } else {
        prior[0].1.pinned_object_revision
    };
    if let Some((_, old)) = prior.first()
        && selected.is_some()
        && (old.target_object_id != target || old.pinned_object_revision != revision)
    {
        return Err(invalid(
            "changing a placed Map binding requires a reviewed library update",
        ));
    }
    let part_revision = serde_json::from_value(payload["part_ref"]["object_revision"].clone())?;
    validate_map(
        model,
        target,
        revision,
        part,
        part_revision,
        spec.package_id,
    )?;
    if let Some((_, old)) = prior.first() {
        for override_ref in &old.local_override_refs {
            validate_map(
                model,
                override_ref.object_id,
                override_ref.object_revision,
                part,
                part_revision,
                spec.package_id,
            )?;
        }
        return Ok(());
    }
    let bindings = payload["library_bindings"]
        .as_object_mut()
        .ok_or_else(|| invalid("invalid authored bindings"))?;
    let binding_id = ids::derive_object_id(
        &model.project.project_id,
        "library-binding",
        &[id.to_string(), "pin_pad_map".into(), target.to_string()],
    );
    bindings.insert(
        binding_id.to_string(),
        serde_json::json!({
            "target_object_id": target,
            "pinned_object_revision": revision,
            "binding_role": "pin_pad_map"
        }),
    );
    Ok(())
}

fn validate_map(
    model: &DesignModel,
    id: Uuid,
    revision: crate::substrate::ObjectRevision,
    part_id: Uuid,
    part_revision: crate::substrate::ObjectRevision,
    placement_id: Uuid,
) -> Result<(), EngineError> {
    let value = model.materialized_pool_object_revision(id, "pin_pad_maps", revision)?;
    let map: crate::pool::PinPadMap = serde_json::from_value(value)?;
    if map.uuid != id || map.part != part_id {
        return Err(invalid("Map belongs to another Part"));
    }
    let footprint_id = map
        .footprint
        .ok_or_else(|| invalid("Map has no Footprint"))?;
    let footprint = source::capture_root(model, footprint_id, SourceShardTaxon::PoolFootprint)?;
    let footprint: crate::pool::Footprint =
        serde_json::from_value(source::source_value(model, &footprint)?)?;
    let part: crate::pool::Part = serde_json::from_value(
        model.materialized_pool_object_revision(part_id, "parts", part_revision)?,
    )?;
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let placement: crate::board::PlacedPackage =
        serde_json::from_value(board["packages"][placement_id.to_string()].clone())?;
    if placement.part != part_id
        || placement.package != part.package
        || footprint.package != part.package
    {
        return Err(invalid(
            "Part, Footprint and placed Package are incompatible",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::native_write::{
        board_components::{BoardPackagePlacement, build_place_board_package},
        commit_prepared,
        library::{self, PoolLibraryObjectTarget as Target, PoolLibraryOperationSpec as Spec},
    };
    use crate::substrate::{CommitSource, Operation, ProjectResolver};
    use serde_json::json;
    use std::path::PathBuf;
    fn provenance() -> WriteProvenance {
        WriteProvenance::new("test", CommitSource::Manual, "explicit Map intent")
    }
    fn fixture(name: &str) -> (PathBuf, DesignModel, ComponentInstanceSpec, Uuid, Uuid) {
        let (root, mut model, symbol, other_part) =
            crate::api::native_write::schematic_assignment::tests::fixture(name);
        let part = "90000000-0000-4000-8000-000000000001".parse().unwrap();
        let map = "a0000000-0000-4000-8000-000000000001".parse().unwrap();
        let package = crate::board::PlacedPackage {
            uuid: Uuid::new_v4(),
            part,
            package: "50000000-0000-4000-8000-000000000001".parse().unwrap(),
            reference: "U_MAP".into(),
            value: "native".into(),
            position: crate::ir::geometry::Point::zero(),
            rotation: 0,
            layer: 1,
            locked: false,
        };
        let p = build_place_board_package(
            &model,
            provenance(),
            &BoardPackagePlacement {
                package: package.clone(),
                materialized: json!({}),
            },
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        (
            root,
            model,
            ComponentInstanceSpec {
                part_id: Some(part),
                symbol_ids: vec![symbol],
                package_id: package.uuid,
                symbol_roles: vec![],
                package_roles: vec![],
            },
            map,
            other_part,
        )
    }
    fn raw(model: &DesignModel, id: Uuid) -> Value {
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.shard_id == model.objects[&id].source_shard_id)
            .unwrap();
        model
            .materialized_source_shard_value_by_relative_path(&shard.relative_path)
            .unwrap()
    }
    #[test]
    fn explicit_map_intent_pins_current_and_refuses_foreign_missing_or_wrong_kind_without_writes() {
        let (root, mut model, spec, map, other_part) = fixture("native-map-intent");
        let journal = std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap();
        let revision = model.model_revision.clone();
        for choice in [Uuid::new_v4(), spec.part_id.unwrap()] {
            assert!(
                build_bind_component_instance_with_pin_pad_map(
                    &model,
                    provenance(),
                    None,
                    &spec,
                    Some(choice)
                )
                .is_err()
            );
        }
        let mut wrong = spec.clone();
        wrong.part_id = Some(other_part);
        assert!(
            build_bind_component_instance_with_pin_pad_map(
                &model,
                provenance(),
                None,
                &wrong,
                Some(map)
            )
            .is_err()
        );
        let mut no_part = spec.clone();
        no_part.part_id = None;
        assert!(
            build_bind_component_instance_with_pin_pad_map(
                &model,
                provenance(),
                None,
                &no_part,
                Some(map)
            )
            .is_err()
        );
        assert_eq!(
            std::fs::read(crate::substrate::transaction_journal_path(&root)).unwrap(),
            journal
        );
        assert_eq!(model.model_revision, revision);
        let (p, ci) = build_bind_component_instance_with_pin_pad_map(
            &model,
            provenance(),
            None,
            &spec,
            Some(map),
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let bindings = &model.component_instances[&ci].library_bindings;
        assert_eq!(bindings.len(), 2);
        let declared = bindings
            .values()
            .find(|b| b.binding_role == LibraryBindingRole::PinPadMap)
            .unwrap();
        assert_eq!(declared.target_object_id, map);
        assert_eq!(
            declared.pinned_object_revision,
            model.objects[&map].object_revision
        );
        assert_eq!(
            ProjectResolver::new(&root)
                .resolve()
                .unwrap()
                .component_instances[&ci]
                .library_bindings,
            *bindings
        );
    }
    #[test]
    fn omitted_map_preserves_historical_revision_overrides_and_unrelated_authored_fields() {
        let (root, mut model, spec, map, _) = fixture("native-map-preservation");
        let (p, ci) = build_bind_component_instance_with_pin_pad_map(
            &model,
            provenance(),
            None,
            &spec,
            Some(map),
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let previous = raw(&model, ci)["component_instance"].clone();
        let mut next = previous.clone();
        next["object_revision"] = json!(model.objects[&ci].object_revision.0 + 1);
        let bindings = next["library_bindings"].as_object_mut().unwrap();
        let binding = bindings
            .values_mut()
            .find(|b| b["binding_role"] == "pin_pad_map")
            .unwrap();
        binding["provenance"] = json!({"retain":"map"});
        binding["extension"] = json!({"retain":"binding"});
        binding["local_override_refs"] = json!([{"object_id":map,"object_revision":0}]);
        let expected = next["library_bindings"].clone();
        let p = BatchComposer::compose(&model, provenance())
            .push_op(Operation::SetComponentInstance {
                component_instance_id: ci,
                previous_component_instance: previous,
                component_instance: next,
            })
            .finish()
            .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let mut map_value = raw(&model, map);
        map_value["tags"] = json!(["new-current"]);
        let p = library::build_pool_library_write(
            &model,
            provenance(),
            Some("pool"),
            vec![Spec::Set {
                target: Target::new("pool", "pin_pad_maps", map),
                object: map_value,
            }],
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        assert!(model.objects[&map].object_revision.0 > 0);
        let p = build_set_component_instance(&model, provenance(), ci, &spec).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let reopened = ProjectResolver::new(&root).resolve().unwrap();
        let source = raw(&reopened, ci);
        assert_eq!(source["component_instance"]["library_bindings"], expected);
        assert!(
            build_set_component_instance_with_pin_pad_map(
                &reopened,
                provenance(),
                ci,
                &spec,
                Some(map),
                ComponentInstancePartChoice::Selected,
            )
            .is_err(),
            "a new selection cannot silently discard old local overrides"
        );
    }

    #[test]
    fn first_explicit_part_and_map_choice_keeps_existing_component_identity() {
        let (root, mut model, spec, map, _) = fixture("native-first-map-set");
        let mut unbound = spec.clone();
        unbound.part_id = None;
        let (p, ci) = build_bind_component_instance(&model, provenance(), None, &unbound).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        assert!(model.component_instances[&ci].library_bindings.is_empty());
        let p = build_set_component_instance_with_pin_pad_map(
            &model,
            provenance(),
            ci,
            &spec,
            Some(map),
            ComponentInstancePartChoice::Selected,
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let declared = &model.component_instances[&ci];
        assert_eq!(declared.id, ci);
        assert_eq!(declared.part_ref, spec.part_id);
        assert_eq!(declared.library_bindings.len(), 2);
        assert!(
            declared
                .library_bindings
                .values()
                .any(|b| b.binding_role == LibraryBindingRole::Part
                    && Some(b.target_object_id) == spec.part_id)
        );
        assert!(
            declared
                .library_bindings
                .values()
                .any(|b| b.binding_role == LibraryBindingRole::PinPadMap
                    && b.target_object_id == map)
        );
    }

    #[test]
    fn preserved_part_intent_retains_old_pin_while_native_selected_intent_remains_current() {
        let (root, mut model, spec, map, _) = fixture("native-map-part-intent");
        let (p, ci) = build_bind_component_instance_with_pin_pad_map(
            &model,
            provenance(),
            None,
            &spec,
            Some(map),
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let expected = model.component_instances[&ci].library_bindings.clone();
        let part = spec.part_id.unwrap();
        let mut value = raw(&model, part);
        value["mpn"] = json!("New current Part");
        let p = library::build_pool_library_write(
            &model,
            provenance(),
            Some("pool"),
            vec![Spec::Set {
                target: Target::new("pool", "parts", part),
                object: value,
            }],
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let mut omitted = spec.clone();
        omitted.part_id = None;
        let p = build_set_component_instance_with_pin_pad_map(
            &model,
            provenance(),
            ci,
            &omitted,
            None,
            ComponentInstancePartChoice::Preserve,
        )
        .unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        assert_eq!(model.component_instances[&ci].library_bindings, expected);
        let p = build_set_component_instance(&model, provenance(), ci, &spec).unwrap();
        commit_prepared(&mut model, &root, p).unwrap();
        let current = model.component_instances[&ci]
            .library_bindings
            .values()
            .find(|b| b.binding_role == LibraryBindingRole::Part)
            .unwrap();
        assert_eq!(
            current.pinned_object_revision,
            model.objects[&part].object_revision
        );
        assert!(current.pinned_object_revision.0 > 0);
    }
}
