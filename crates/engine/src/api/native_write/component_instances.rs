//! Component-instance bind/set/delete builders for the native write facade.
//!
//! Family A of the native-write migration: all operation authoring for
//! component instances lives here. The CLI callers in
//! `crates/cli/src/command_project_component_instances.rs` and
//! `crates/cli/src/command_project_schematic_symbol_component_instance.rs`
//! are thin argument-parsers over this module: they parse strings into a
//! typed [`ComponentInstanceSpec`], call a `build_*` function, and commit the
//! returned [`PreparedWrite`] via [`super::commit_prepared`].
//!
//! Builders are build-only; they never touch disk. Payload shape, id
//! derivation (see [`super::ids::derive_component_instance_id`]), guard
//! insertion, and error-message text are byte-for-byte the CLI's historical
//! behavior — journal records and shards must not drift.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;
use crate::substrate::{
    ComponentInstance, ComponentInstanceAuthority, ComponentInstanceId, DesignModel, ObjectId,
    Operation,
};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};
use super::ids;

/// An explicit role override for one placed-symbol or placed-package ref of a
/// component instance (parsed by the CLI from `<uuid>=<role>[:label]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentRoleAssignment {
    pub object_id: Uuid,
    pub role: String,
    pub label: Option<String>,
}

/// Typed request describing the desired state of a component instance: which
/// placed symbols and placed package it binds, the optional pool part, and
/// any explicit role overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentInstanceSpec {
    pub part_id: Option<Uuid>,
    pub symbol_ids: Vec<Uuid>,
    pub package_id: Uuid,
    pub symbol_roles: Vec<ComponentRoleAssignment>,
    pub package_roles: Vec<ComponentRoleAssignment>,
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
    let component_instance_id = component_instance_id.unwrap_or_else(|| {
        ids::derive_component_instance_id(
            &model.project.project_id,
            &spec.symbol_ids,
            spec.package_id,
        )
    });
    let payload = component_instance_payload(model, component_instance_id, 0, spec)?;
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
    let previous = authored_component_instance(model, component_instance_id)?;
    let previous_payload = component_instance_payload_from_instance(model, &previous)?;
    let next_revision = previous.object_revision.0 + 1;
    let payload = component_instance_payload(model, component_instance_id, next_revision, spec)?;
    BatchComposer::compose(model, provenance)
        .push_op(Operation::SetComponentInstance {
            component_instance_id,
            previous_component_instance: previous_payload,
            component_instance: payload,
        })
        .primary_object(component_instance_id)
        .finish()
}

/// Build the batch that deletes an existing authored component instance.
///
/// Returns the prepared write and the pre-delete instance so callers can
/// report on what was removed without re-resolving.
pub fn build_delete_component_instance(
    model: &DesignModel,
    provenance: WriteProvenance,
    component_instance_id: ComponentInstanceId,
) -> Result<(PreparedWrite, ComponentInstance), EngineError> {
    let previous = authored_component_instance(model, component_instance_id)?;
    let previous_payload = component_instance_payload_from_instance(model, &previous)?;
    let prepared = BatchComposer::compose(model, provenance)
        .push_op(Operation::DeleteComponentInstance {
            component_instance_id,
            component_instance: previous_payload,
        })
        .primary_object(component_instance_id)
        .finish()?;
    Ok((prepared, previous))
}

/// Build the `CreateComponentInstance` operation that binds a freshly placed
/// schematic symbol to its resolved pool part.
///
/// This is an operation contributor, not a full batch: the symbol-placement
/// flows compose it into their own `CreateSchematicSymbol` batches (and into
/// schematic proposals). The v5 seed is
/// `datum-eda:component-instance:schematic:<pool_symbol_id>:<placed_symbol_id>`
/// namespaced by the project id — byte-identical to the historical CLI
/// derivation. The placed symbol is referenced at object revision 0 because
/// it is created in the same batch.
pub fn build_placed_symbol_component_instance_op(
    model: &DesignModel,
    placed_symbol_id: Uuid,
    pool_symbol_id: Uuid,
    part_id: Uuid,
) -> Result<Operation, EngineError> {
    let part_ref = revisioned_ref(model, part_id)?;
    let component_instance_id = ids::derive_object_id(
        &model.project.project_id,
        "component-instance",
        &[
            "schematic".to_string(),
            pool_symbol_id.to_string(),
            placed_symbol_id.to_string(),
        ],
    );
    let library_bindings = part_library_binding_payload(model, component_instance_id, part_id)?;
    Ok(Operation::CreateComponentInstance {
        component_instance_id,
        component_instance: serde_json::json!({
            "uuid": component_instance_id,
            "object_revision": 0,
            "part_ref": part_ref,
            "library_bindings": library_bindings,
            "placed_symbol_refs": [{
                "object_id": placed_symbol_id,
                "object_revision": 0
            }],
            "placed_package_refs": [],
            "placed_symbol_roles": {
                (placed_symbol_id.to_string()): {
                    "role": "primary"
                }
            },
            "placed_package_roles": {}
        }),
    })
}

fn authored_component_instance(
    model: &DesignModel,
    component_instance_id: ComponentInstanceId,
) -> Result<ComponentInstance, EngineError> {
    let instance = model
        .component_instances
        .get(&component_instance_id)
        .ok_or_else(|| {
            EngineError::Validation(format!(
                "component instance {component_instance_id} was not found"
            ))
        })?;
    if instance.authority != ComponentInstanceAuthority::Authored {
        return Err(EngineError::Validation(format!(
            "component instance {component_instance_id} is compatibility-derived; author an explicit ComponentInstance before mutation"
        )));
    }
    Ok(instance.clone())
}

fn component_instance_payload_from_instance(
    model: &DesignModel,
    instance: &ComponentInstance,
) -> Result<serde_json::Value, EngineError> {
    let symbol_refs = instance
        .placed_symbol_refs
        .iter()
        .map(|symbol_id| revisioned_ref(model, *symbol_id))
        .collect::<Result<Vec<_>, _>>()?;
    let package_refs = instance
        .placed_package_refs
        .iter()
        .map(|package_id| revisioned_ref(model, *package_id))
        .collect::<Result<Vec<_>, _>>()?;
    let part_ref = instance
        .part_ref
        .map(|part_id| revisioned_ref(model, part_id))
        .transpose()?;
    Ok(serde_json::json!({
        "uuid": instance.id,
        "object_revision": instance.object_revision.0,
        "part_ref": part_ref,
        "library_bindings": instance.library_bindings,
        "placed_symbol_refs": symbol_refs,
        "placed_package_refs": package_refs,
        "placed_symbol_roles": instance.placed_symbol_roles,
        "placed_package_roles": instance.placed_package_roles
    }))
}

fn component_instance_payload(
    model: &DesignModel,
    component_instance_id: ComponentInstanceId,
    object_revision: u64,
    spec: &ComponentInstanceSpec,
) -> Result<serde_json::Value, EngineError> {
    if spec.symbol_ids.is_empty() {
        return Err(EngineError::Validation(
            "component instance payload requires at least one symbol ref".to_string(),
        ));
    }
    let package_ids = [spec.package_id];
    let symbol_refs = spec
        .symbol_ids
        .iter()
        .map(|symbol_id| revisioned_ref(model, *symbol_id))
        .collect::<Result<Vec<_>, _>>()?;
    let package_refs = package_ids
        .iter()
        .map(|package_id| revisioned_ref(model, *package_id))
        .collect::<Result<Vec<_>, _>>()?;
    let part_ref = spec
        .part_id
        .map(|part_id| revisioned_ref(model, part_id))
        .transpose()?;
    let library_bindings = spec
        .part_id
        .map(|part_id| part_library_binding_payload(model, component_instance_id, part_id))
        .transpose()?
        .unwrap_or_default();
    let placed_symbol_roles =
        component_role_map(&spec.symbol_ids, &spec.symbol_roles, "primary", "unit")?;
    let placed_package_roles =
        component_role_map(&package_ids, &spec.package_roles, "primary", "alternate")?;
    Ok(serde_json::json!({
        "uuid": component_instance_id,
        "object_revision": object_revision,
        "part_ref": part_ref,
        "library_bindings": library_bindings,
        "placed_symbol_refs": symbol_refs,
        "placed_package_refs": package_refs,
        "placed_symbol_roles": placed_symbol_roles,
        "placed_package_roles": placed_package_roles
    }))
}

fn part_library_binding_payload(
    model: &DesignModel,
    component_instance_id: ComponentInstanceId,
    part_id: ObjectId,
) -> Result<BTreeMap<Uuid, serde_json::Value>, EngineError> {
    let part_ref = revisioned_ref(model, part_id)?;
    let binding_id = ids::derive_object_id(
        &model.project.project_id,
        "library-binding",
        &[
            component_instance_id.to_string(),
            "part".to_string(),
            part_id.to_string(),
        ],
    );
    let mut bindings = BTreeMap::new();
    bindings.insert(
        binding_id,
        serde_json::json!({
            "target_object_id": part_ref["object_id"],
            "pinned_object_revision": part_ref["object_revision"],
            "binding_role": "part"
        }),
    );
    Ok(bindings)
}

fn component_role_map(
    object_ids: &[Uuid],
    assignments: &[ComponentRoleAssignment],
    first_role: &str,
    later_role: &str,
) -> Result<BTreeMap<Uuid, serde_json::Value>, EngineError> {
    let mut roles = object_ids
        .iter()
        .enumerate()
        .map(|(index, object_id)| {
            (
                *object_id,
                serde_json::json!({
                    "role": if index == 0 { first_role } else { later_role },
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for assignment in assignments {
        if !roles.contains_key(&assignment.object_id) {
            return Err(EngineError::Validation(format!(
                "component role spec {} does not match a selected ref",
                assignment.object_id
            )));
        }
        roles.insert(
            assignment.object_id,
            serde_json::json!({
                "role": assignment.role,
                "label": assignment.label,
            }),
        );
    }
    Ok(roles)
}

fn revisioned_ref(
    model: &DesignModel,
    object_id: ObjectId,
) -> Result<serde_json::Value, EngineError> {
    let object = model.objects.get(&object_id).ok_or_else(|| {
        EngineError::Validation(format!(
            "component instance target object {object_id} was not found"
        ))
    })?;
    Ok(serde_json::json!({
        "object_id": object_id,
        "object_revision": object.object_revision.0
    }))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::super::context::commit_prepared;
    use super::super::test_support::resolved_model_with_board_package;
    use super::*;
    use crate::substrate::{CommitSource, ObjectRevision, ProjectResolver};

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new(
            "unit-test",
            CommitSource::Test,
            "component instance facade test",
        )
    }

    fn write_json(path: &Path, value: serde_json::Value) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("parent dir should create");
        }
        std::fs::write(
            path,
            serde_json::to_string_pretty(&value).expect("json should serialize"),
        )
        .expect("json should write");
    }

    /// Overlay one placed schematic symbol on the board-package fixture so
    /// component-instance refs resolve against real schematic/board domain
    /// objects (mirrors the substrate tests' symbol+package fixture writer).
    /// Returns `(project_root, model, symbol_id, package_id)`.
    fn resolved_model_with_symbol_and_package(name: &str) -> (PathBuf, DesignModel, Uuid, Uuid) {
        let (root, model, _board_id, package_id) = resolved_model_with_board_package(name);
        let project_id = model.project.project_id;
        let symbol_id = Uuid::new_v4();
        write_json(
            &root.join("schematic/sheets/main.json"),
            serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v5(&project_id, b"sheet"),
                "name": "Main",
                "symbols": {
                    (symbol_id.to_string()): {
                        "uuid": symbol_id,
                        "part": Uuid::new_v5(&project_id, b"part"),
                        "entity": Uuid::new_v5(&project_id, b"entity"),
                        "gate": Uuid::new_v5(&project_id, b"gate"),
                        "lib_id": "test:R",
                        "reference": "U1",
                        "value": "OLD",
                        "fields": [],
                        "pins": [],
                        "position": { "x": 0, "y": 0 },
                        "rotation": 0,
                        "mirrored": false,
                        "unit_selection": null,
                        "display_mode": "LibraryDefault",
                        "pin_overrides": [],
                        "hidden_power_behavior": "SourceDefinedImplicit"
                    }
                },
                "wires": {},
                "junctions": {},
                "labels": {},
                "buses": {},
                "bus_entries": {},
                "ports": {},
                "noconnects": {},
                "texts": {},
                "drawings": {}
            }),
        );
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture project with symbol should resolve");
        (root, model, symbol_id, package_id)
    }

    fn fixture_spec(symbol_id: Uuid, package_id: Uuid) -> ComponentInstanceSpec {
        ComponentInstanceSpec {
            part_id: None,
            symbol_ids: vec![symbol_id],
            package_id,
            symbol_roles: Vec::new(),
            package_roles: Vec::new(),
        }
    }

    #[test]
    fn bind_derives_id_and_authors_create_payload() {
        let (_root, model, symbol_id, package_id) =
            resolved_model_with_symbol_and_package("component_instances_bind");
        let spec = fixture_spec(symbol_id, package_id);

        let (prepared, component_instance_id) =
            build_bind_component_instance(&model, test_provenance(), None, &spec)
                .expect("bind should build");

        assert_eq!(
            component_instance_id,
            ids::derive_component_instance_id(
                &model.project.project_id,
                &spec.symbol_ids,
                spec.package_id
            )
        );
        assert_eq!(prepared.primary_object_id, Some(component_instance_id));
        // Creation is not an existing-object mutation: no guard is inserted.
        assert_eq!(prepared.batch.operations.len(), 1);
        let Operation::CreateComponentInstance {
            component_instance_id: op_id,
            component_instance,
        } = &prepared.batch.operations[0]
        else {
            panic!("expected CreateComponentInstance");
        };
        assert_eq!(*op_id, component_instance_id);
        assert_eq!(
            component_instance,
            &serde_json::json!({
                "uuid": component_instance_id,
                "object_revision": 0,
                "part_ref": null,
                "library_bindings": {},
                "placed_symbol_refs": [
                    { "object_id": symbol_id, "object_revision": 0 }
                ],
                "placed_package_refs": [
                    { "object_id": package_id, "object_revision": 0 }
                ],
                "placed_symbol_roles": {
                    (symbol_id.to_string()): { "role": "primary" }
                },
                "placed_package_roles": {
                    (package_id.to_string()): { "role": "primary" }
                }
            })
        );
    }

    #[test]
    fn bind_honors_explicit_id_and_role_assignments() {
        let (_root, model, symbol_id, package_id) =
            resolved_model_with_symbol_and_package("component_instances_bind_roles");
        let explicit_id = Uuid::new_v4();
        let mut spec = fixture_spec(symbol_id, package_id);
        spec.symbol_roles = vec![ComponentRoleAssignment {
            object_id: symbol_id,
            role: "unit".to_string(),
            label: Some("A".to_string()),
        }];

        let (prepared, component_instance_id) =
            build_bind_component_instance(&model, test_provenance(), Some(explicit_id), &spec)
                .expect("bind should build");

        assert_eq!(component_instance_id, explicit_id);
        let Operation::CreateComponentInstance {
            component_instance, ..
        } = &prepared.batch.operations[0]
        else {
            panic!("expected CreateComponentInstance");
        };
        assert_eq!(
            component_instance["placed_symbol_roles"][symbol_id.to_string()],
            serde_json::json!({ "role": "unit", "label": "A" })
        );
    }

    #[test]
    fn bind_rejects_role_assignment_for_unselected_ref() {
        let (_root, model, symbol_id, package_id) =
            resolved_model_with_symbol_and_package("component_instances_bad_role");
        let stray = Uuid::new_v4();
        let mut spec = fixture_spec(symbol_id, package_id);
        spec.package_roles = vec![ComponentRoleAssignment {
            object_id: stray,
            role: "alternate".to_string(),
            label: None,
        }];

        let error = build_bind_component_instance(&model, test_provenance(), None, &spec)
            .expect_err("stray role assignment should fail");
        assert!(error.to_string().contains(&format!(
            "component role spec {stray} does not match a selected ref"
        )));
    }

    #[test]
    fn bind_requires_symbol_refs_and_resolvable_targets() {
        let (_root, model, symbol_id, package_id) =
            resolved_model_with_symbol_and_package("component_instances_bad_refs");

        let mut empty = fixture_spec(symbol_id, package_id);
        empty.symbol_ids.clear();
        let error = build_bind_component_instance(&model, test_provenance(), None, &empty)
            .expect_err("empty symbol refs should fail");
        assert!(
            error
                .to_string()
                .contains("component instance payload requires at least one symbol ref")
        );

        let missing = Uuid::new_v4();
        let mut dangling = fixture_spec(symbol_id, package_id);
        dangling.symbol_ids = vec![missing];
        let error = build_bind_component_instance(&model, test_provenance(), None, &dangling)
            .expect_err("dangling symbol ref should fail");
        assert!(error.to_string().contains(&format!(
            "component instance target object {missing} was not found"
        )));
    }

    #[test]
    fn set_guards_previous_revision_and_bumps_payload_revision() {
        let (root, mut model, symbol_id, package_id) =
            resolved_model_with_symbol_and_package("component_instances_set");
        let spec = fixture_spec(symbol_id, package_id);
        let (prepared, component_instance_id) =
            build_bind_component_instance(&model, test_provenance(), None, &spec)
                .expect("bind should build");
        commit_prepared(&mut model, &root, prepared).expect("bind should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("committed project should re-resolve");

        let prepared =
            build_set_component_instance(&model, test_provenance(), component_instance_id, &spec)
                .expect("set should build");

        assert_eq!(prepared.batch.operations.len(), 2);
        assert_eq!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision {
                object_id: component_instance_id,
                expected_object_revision: ObjectRevision(0),
            }
        );
        let Operation::SetComponentInstance {
            previous_component_instance,
            component_instance,
            ..
        } = &prepared.batch.operations[1]
        else {
            panic!("expected SetComponentInstance");
        };
        assert_eq!(previous_component_instance["object_revision"], 0);
        assert_eq!(component_instance["object_revision"], 1);
    }

    #[test]
    fn delete_returns_previous_instance_and_guards_it() {
        let (root, mut model, symbol_id, package_id) =
            resolved_model_with_symbol_and_package("component_instances_delete");
        let spec = fixture_spec(symbol_id, package_id);
        let (prepared, component_instance_id) =
            build_bind_component_instance(&model, test_provenance(), None, &spec)
                .expect("bind should build");
        commit_prepared(&mut model, &root, prepared).expect("bind should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("committed project should re-resolve");

        let (prepared, previous) =
            build_delete_component_instance(&model, test_provenance(), component_instance_id)
                .expect("delete should build");

        assert_eq!(previous.id, component_instance_id);
        assert_eq!(prepared.primary_object_id, Some(component_instance_id));
        assert_eq!(prepared.batch.operations.len(), 2);
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::GuardObjectRevision { object_id, .. } if object_id == component_instance_id
        ));
        assert!(matches!(
            &prepared.batch.operations[1],
            Operation::DeleteComponentInstance { component_instance_id: id, .. }
                if *id == component_instance_id
        ));
    }

    #[test]
    fn set_and_delete_reject_unknown_instance() {
        let (_root, model, symbol_id, package_id) =
            resolved_model_with_symbol_and_package("component_instances_unknown");
        let missing = Uuid::new_v4();
        let spec = fixture_spec(symbol_id, package_id);

        let set_error = build_set_component_instance(&model, test_provenance(), missing, &spec)
            .expect_err("set of unknown instance should fail");
        assert!(
            set_error
                .to_string()
                .contains(&format!("component instance {missing} was not found"))
        );

        let delete_error = build_delete_component_instance(&model, test_provenance(), missing)
            .expect_err("delete of unknown instance should fail");
        assert!(
            delete_error
                .to_string()
                .contains(&format!("component instance {missing} was not found"))
        );
    }

    #[test]
    fn placed_symbol_op_matches_historical_derivation_and_payload() {
        let (_root, model, _board_id, package_id) =
            resolved_model_with_board_package("component_instances_placed_symbol");
        let placed_symbol_id = Uuid::new_v4();
        let pool_symbol_id = Uuid::new_v4();
        // The fixture package doubles as the pool-part target object.
        let part_id = package_id;

        let operation = build_placed_symbol_component_instance_op(
            &model,
            placed_symbol_id,
            pool_symbol_id,
            part_id,
        )
        .expect("placed-symbol op should build");

        // Byte-exact historical CLI derivation
        // (command_project_schematic_symbol_component_instance.rs).
        let expected_id = Uuid::new_v5(
            &model.project.project_id,
            format!("datum-eda:component-instance:schematic:{pool_symbol_id}:{placed_symbol_id}")
                .as_bytes(),
        );
        let Operation::CreateComponentInstance {
            component_instance_id,
            component_instance,
        } = &operation
        else {
            panic!("expected CreateComponentInstance");
        };
        assert_eq!(*component_instance_id, expected_id);
        let expected_binding_id = ids::derive_object_id(
            &model.project.project_id,
            "library-binding",
            &[
                expected_id.to_string(),
                "part".to_string(),
                part_id.to_string(),
            ],
        );
        assert_eq!(
            component_instance,
            &serde_json::json!({
                "uuid": expected_id,
                "object_revision": 0,
                "part_ref": { "object_id": part_id, "object_revision": 0 },
                "library_bindings": {
                    (expected_binding_id.to_string()): {
                        "target_object_id": part_id,
                        "pinned_object_revision": 0,
                        "binding_role": "part"
                    }
                },
                "placed_symbol_refs": [
                    { "object_id": placed_symbol_id, "object_revision": 0 }
                ],
                "placed_package_refs": [],
                "placed_symbol_roles": {
                    (placed_symbol_id.to_string()): { "role": "primary" }
                },
                "placed_package_roles": {}
            })
        );
    }

    #[test]
    fn placed_symbol_op_rejects_missing_part_object() {
        let (_root, model, _board_id, _package_id) =
            resolved_model_with_board_package("component_instances_placed_missing");
        let missing_part = Uuid::new_v4();

        let error = build_placed_symbol_component_instance_op(
            &model,
            Uuid::new_v4(),
            Uuid::new_v4(),
            missing_part,
        )
        .expect_err("missing part object should fail");
        assert!(error.to_string().contains(&format!(
            "component instance target object {missing_part} was not found"
        )));
    }
}
