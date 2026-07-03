//! Pool/library authoring builders for the native write facade.
//!
//! Family B of the native-write migration: all operation authoring for pool
//! library objects lives here. The CLI callers in
//! `crates/cli/src/command_project_library.rs` and its satellites
//! (`command_project_library_payload.rs`, `_part_bindings.rs`, `_unit_pin.rs`,
//! `_symbol_geometry.rs`, `_package.rs`, `_package_geometry.rs`,
//! `_package_pad.rs`, `_pad_map.rs`, plus the `_proposals` files) are thin
//! argument-parsers over this module: they validate CLI input, assemble the
//! desired object payloads, describe the write as typed
//! [`PoolLibraryOperationSpec`]s, and either commit the returned
//! [`PreparedWrite`] via [`super::commit_prepared`] or feed its uncommitted
//! batch into `create_draft_proposal_from_batch` (proposal flows share these
//! builders; nothing forks).
//!
//! Builders are build-only; they never touch disk. Payload shapes, the
//! ensure-pool-ref rule, and the `previous_object`/`previous_attachments`
//! sourcing are byte-for-byte the CLI's historical behavior — journal records
//! and pool shards must not drift.
//!
//! Previous-payload sourcing: the pre-migration CLI derived `previous_object`
//! by resolving a fresh model and materializing the shard by relative path
//! (`DesignModel::materialized_source_shard_value_by_relative_path`, which
//! reads the shard file, hash-verifies it, and replays the journal when the
//! file is stale). This module performs the same materialization against the
//! model the batch is built on — the same disk state the CLI read — so no
//! separate disk-read hook is needed; the parity tests below prove the
//! model-sourced payload equals both the raw shard file and a fresh-resolve
//! read.
//!
//! The content-addressed pool model blob store ([`write_pool_model_blob`]) is
//! also engine-owned here: the blob file itself is idempotent, hash-named
//! evidence storage (not a canonical shard — the attachment pointer is what
//! commits through the journal), but its write no longer lives in the CLI.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;
use crate::pool::ModelRole;
use crate::substrate::{DesignModel, ObjectId, Operation, SourceShardKind};

use super::context::{BatchComposer, PreparedWrite, WriteProvenance};

/// Canonical relative path of a pool library object shard:
/// `<pool_path>/<object_kind>/<object_id>.json`.
pub fn pool_library_relative_path(pool_path: &str, object_kind: &str, object_id: Uuid) -> String {
    format!("{pool_path}/{object_kind}/{object_id}.json")
}

/// Identity of one pool library object shard targeted by a write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoolLibraryObjectTarget {
    pub object_id: ObjectId,
    pub object_kind: String,
    pub relative_path: String,
}

impl PoolLibraryObjectTarget {
    /// Target at the canonical `<pool_path>/<kind>/<id>.json` location.
    pub fn new(pool_path: &str, object_kind: &str, object_id: Uuid) -> Self {
        Self {
            object_id,
            object_kind: object_kind.to_string(),
            relative_path: pool_library_relative_path(pool_path, object_kind, object_id),
        }
    }

    /// Target at an explicit shard path (legacy package-compatibility flows
    /// address footprints by their existing shard path, which may not follow
    /// the canonical `<id>.json` naming for imported pools).
    pub fn at_relative_path(object_id: Uuid, object_kind: &str, relative_path: String) -> Self {
        Self {
            object_id,
            object_kind: object_kind.to_string(),
            relative_path,
        }
    }
}

/// Typed description of one pool-library mutation within a write.
///
/// `Set`, `Delete`, `AttachPartModel`, and `DetachPartModel` source their
/// previous payloads from the resolved model at build time (see the module
/// docs for the disk-equivalence argument); callers supply only the desired
/// next state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoolLibraryOperationSpec {
    Create {
        target: PoolLibraryObjectTarget,
        object: serde_json::Value,
    },
    Set {
        target: PoolLibraryObjectTarget,
        object: serde_json::Value,
    },
    Delete {
        target: PoolLibraryObjectTarget,
    },
    AttachPartModel {
        part_id: ObjectId,
        relative_path: String,
        attachments: Vec<serde_json::Value>,
    },
    DetachPartModel {
        part_id: ObjectId,
        relative_path: String,
        attachments: Vec<serde_json::Value>,
    },
}

/// Build one guarded batch for a pool-library write.
///
/// When `ensure_pool_ref` names a pool path that the project manifest does
/// not yet reference, an `AddProjectPoolRef` operation is prepended with the
/// next priority (`max + 1`, starting at 1) — exactly the pre-migration CLI
/// rule. The first spec's object id becomes the write's primary object.
pub fn build_pool_library_write(
    model: &DesignModel,
    provenance: WriteProvenance,
    ensure_pool_ref: Option<&str>,
    operations: Vec<PoolLibraryOperationSpec>,
) -> Result<PreparedWrite, EngineError> {
    let mut composer = BatchComposer::compose(model, provenance);
    if let Some(pool_path) = ensure_pool_ref
        && let Some(operation) = ensure_pool_ref_operation(model, pool_path)?
    {
        composer = composer.push_op(operation);
    }
    let mut primary_object_id = None;
    for spec in operations {
        let (object_id, operation) = pool_library_operation(model, spec)?;
        if primary_object_id.is_none() {
            primary_object_id = Some(object_id);
        }
        composer = composer.push_op(operation);
    }
    if let Some(object_id) = primary_object_id {
        composer = composer.primary_object(object_id);
    }
    composer.finish()
}

/// The `AddProjectPoolRef` operation required before writing into
/// `pool_path`, or `None` when the manifest already references the pool.
pub fn ensure_pool_ref_operation(
    model: &DesignModel,
    pool_path: &str,
) -> Result<Option<Operation>, EngineError> {
    let pools = manifest_pool_refs(model)?;
    if pools.iter().any(|(path, _)| path == pool_path) {
        return Ok(None);
    }
    let priority = pools
        .iter()
        .map(|(_, priority)| *priority)
        .max()
        .unwrap_or(0)
        + 1;
    Ok(Some(Operation::AddProjectPoolRef {
        path: pool_path.to_string(),
        priority,
    }))
}

/// The materialized payload of one pool library object, sourced from the
/// resolved model (shard read + hash check + journal replay), with the same
/// uuid validation the pre-migration CLI applied.
pub fn pool_library_object_payload(
    model: &DesignModel,
    relative_path: &str,
    object_id: Uuid,
) -> Result<serde_json::Value, EngineError> {
    let object = model.materialized_source_shard_value_by_relative_path(relative_path)?;
    let payload_id = object
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| EngineError::Validation("pool library object missing uuid".to_string()))?;
    let payload_id = Uuid::parse_str(payload_id).map_err(|_| {
        EngineError::Validation(format!("invalid pool library object uuid {payload_id}"))
    })?;
    if payload_id != object_id {
        return Err(EngineError::Validation(format!(
            "pool library object uuid {payload_id} does not match target object {object_id}"
        )));
    }
    Ok(object)
}

fn pool_library_operation(
    model: &DesignModel,
    spec: PoolLibraryOperationSpec,
) -> Result<(ObjectId, Operation), EngineError> {
    Ok(match spec {
        PoolLibraryOperationSpec::Create { target, object } => (
            target.object_id,
            Operation::CreatePoolLibraryObject {
                object_id: target.object_id,
                relative_path: target.relative_path,
                object_kind: target.object_kind,
                object,
            },
        ),
        PoolLibraryOperationSpec::Set { target, object } => {
            let previous_object =
                pool_library_object_payload(model, &target.relative_path, target.object_id)?;
            (
                target.object_id,
                Operation::SetPoolLibraryObject {
                    object_id: target.object_id,
                    relative_path: target.relative_path,
                    object_kind: target.object_kind,
                    previous_object,
                    object,
                },
            )
        }
        PoolLibraryOperationSpec::Delete { target } => {
            let object =
                pool_library_object_payload(model, &target.relative_path, target.object_id)?;
            (
                target.object_id,
                Operation::DeletePoolLibraryObject {
                    object_id: target.object_id,
                    relative_path: target.relative_path,
                    object_kind: target.object_kind,
                    object,
                },
            )
        }
        PoolLibraryOperationSpec::AttachPartModel {
            part_id,
            relative_path,
            attachments,
        } => {
            let previous_attachments =
                part_behavioural_model_attachments(model, &relative_path, part_id)?;
            (
                part_id,
                Operation::AttachPoolPartModel {
                    part_id,
                    relative_path,
                    previous_attachments,
                    attachments,
                },
            )
        }
        PoolLibraryOperationSpec::DetachPartModel {
            part_id,
            relative_path,
            attachments,
        } => {
            let previous_attachments =
                part_behavioural_model_attachments(model, &relative_path, part_id)?;
            (
                part_id,
                Operation::DetachPoolPartModel {
                    part_id,
                    relative_path,
                    previous_attachments,
                    attachments,
                },
            )
        }
    })
}

/// The part's current `behavioural_models` array (empty when the field is
/// absent), matching the pre-migration CLI's read-modify-write baseline.
fn part_behavioural_model_attachments(
    model: &DesignModel,
    relative_path: &str,
    part_id: Uuid,
) -> Result<Vec<serde_json::Value>, EngineError> {
    let part = pool_library_object_payload(model, relative_path, part_id)?;
    match part.get("behavioural_models") {
        None => Ok(Vec::new()),
        Some(serde_json::Value::Array(models)) => Ok(models.clone()),
        Some(_) => Err(EngineError::Validation(format!(
            "part {part_id} behavioural_models field is not an array"
        ))),
    }
}

fn manifest_pool_refs(model: &DesignModel) -> Result<Vec<(String, u32)>, EngineError> {
    let manifest = model.materialized_source_shard_value(SourceShardKind::ProjectManifest)?;
    let pools = manifest
        .get("pools")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            EngineError::Validation("project manifest pools must be an array".to_string())
        })?;
    pools
        .iter()
        .map(|pool| {
            let path = pool
                .get("path")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    EngineError::Validation(
                        "project manifest pool ref missing path".to_string(),
                    )
                })?;
            let priority = pool
                .get("priority")
                .and_then(serde_json::Value::as_u64)
                .and_then(|priority| u32::try_from(priority).ok())
                .ok_or_else(|| {
                    EngineError::Validation(
                        "project manifest pool ref missing priority".to_string(),
                    )
                })?;
            Ok((path.to_string(), priority))
        })
        .collect()
}

/// Write one content-addressed pool model blob (`<pool>/models/<role>/<sha>.<ext>`).
///
/// The blob store is idempotent: parent directories are created and the file
/// is written only when absent (a hash-named blob never changes content).
/// This is evidence storage, not a canonical shard — the attachment pointer
/// that references the blob commits through the journal separately.
pub fn write_pool_model_blob(
    project_root: &Path,
    relative_path: &str,
    bytes: &[u8],
) -> Result<PathBuf, EngineError> {
    let target = project_root.join(relative_path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            EngineError::Operation(format!(
                "failed to create model target directory {}: {error}",
                parent.display()
            ))
        })?;
    }
    if !target.exists() {
        std::fs::write(&target, bytes).map_err(|error| {
            EngineError::Operation(format!(
                "failed to write content-addressed model {}: {error}",
                target.display()
            ))
        })?;
    }
    Ok(target)
}

/// Deterministic id of a content-addressed pool model blob. The v5 seed is
/// `datum-eda:pool-model:<sha256>` in the URL namespace — byte-identical to
/// the historical CLI derivation.
pub fn derive_pool_model_uuid(sha256: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("datum-eda:pool-model:{sha256}").as_bytes(),
    )
}

/// Deterministic id of a part↔model attachment. The v5 seed is
/// `datum-eda:model-attachment:<part>:<model>:<role:?>:<model_names:?>` in
/// the URL namespace — byte-identical to the historical CLI derivation
/// (including the `Debug` formatting of role and names).
pub fn derive_pool_model_attachment_uuid(
    part_id: Uuid,
    model_uuid: Uuid,
    role: &ModelRole,
    model_names: &[String],
) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!(
            "datum-eda:model-attachment:{part_id}:{model_uuid}:{:?}:{model_names:?}",
            role
        )
        .as_bytes(),
    )
}

/// Historical create-payload template for a pool unit.
pub fn pool_unit_payload(unit_id: Uuid, name: &str, manufacturer: &str) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "uuid": unit_id,
        "name": name,
        "manufacturer": manufacturer,
        "pins": {},
        "tags": []
    })
}

/// Historical create-payload template for a pool symbol.
pub fn pool_symbol_payload(symbol_id: Uuid, unit_id: Uuid, name: &str) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "uuid": symbol_id,
        "name": name,
        "unit": unit_id
    })
}

/// Historical create-payload template for a pool entity with one gate.
#[allow(clippy::too_many_arguments)]
pub fn pool_entity_payload(
    entity_id: Uuid,
    gate_id: Uuid,
    unit_id: Uuid,
    symbol_id: Uuid,
    name: &str,
    prefix: &str,
    manufacturer: &str,
    gate_name: &str,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "uuid": entity_id,
        "name": name,
        "prefix": prefix,
        "manufacturer": manufacturer,
        "gates": {
            gate_id.to_string(): {
                "uuid": gate_id,
                "name": gate_name,
                "unit": unit_id,
                "symbol": symbol_id
            }
        },
        "tags": []
    })
}

/// Historical create-payload template for a pool padstack; `aperture` is the
/// already-validated aperture value (null, circle, or rect object).
pub fn pool_padstack_payload(
    padstack_id: Uuid,
    name: &str,
    aperture: serde_json::Value,
    drill_nm: Option<i64>,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "uuid": padstack_id,
        "name": name,
        "aperture": aperture,
        "drill_nm": drill_nm
    })
}

/// Historical create-payload template for a pool part.
#[allow(clippy::too_many_arguments)]
pub fn pool_part_payload(
    part_id: Uuid,
    entity_id: Uuid,
    package_id: Uuid,
    mpn: &str,
    manufacturer: &str,
    value: &str,
    description: &str,
    datasheet: &str,
    lifecycle: &str,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "uuid": part_id,
        "entity": entity_id,
        "package": package_id,
        "default_footprint": null,
        "default_pin_pad_map": null,
        "pad_map": {},
        "mpn": mpn,
        "manufacturer": manufacturer,
        "manufacturer_jep106": null,
        "value": value,
        "description": description,
        "datasheet": datasheet,
        "parametric": {},
        "orderable_mpns": [],
        "packaging_options": [],
        "tags": [],
        "lifecycle": lifecycle,
        "base": null,
        "behavioural_models": [],
        "thermal": null
    })
}

/// Historical create-payload template for a pool package; `pads` is the
/// already-assembled legacy pads map (usually empty).
pub fn pool_package_payload(
    package_id: Uuid,
    name: &str,
    pads: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "uuid": package_id,
        "name": name,
        "pads": pads,
        "courtyard": {"vertices": [], "closed": true},
        "silkscreen": [],
        "models_3d": [],
        "body_height_nm": null,
        "body_height_mounted_nm": null,
        "tags": []
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::context::commit_prepared;
    use super::super::test_support::{temp_project_root, write_minimal_project};
    use super::*;
    use crate::substrate::{CommitSource, OperationBatch, ProjectResolver};

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, "library facade test")
    }

    fn resolved_minimal_model(name: &str) -> (PathBuf, DesignModel) {
        let root = temp_project_root(name);
        write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture project should resolve");
        (root, model)
    }

    /// Commit one unit into `pool` through the facade and re-resolve.
    fn resolved_model_with_pool_unit(name: &str) -> (PathBuf, DesignModel, Uuid) {
        let (root, mut model) = resolved_minimal_model(name);
        let unit_id = Uuid::new_v4();
        let prepared = build_pool_library_write(
            &model,
            test_provenance(),
            Some("pool"),
            vec![PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "units", unit_id),
                object: pool_unit_payload(unit_id, "Test Unit", "ACME"),
            }],
        )
        .expect("unit create should build");
        commit_prepared(&mut model, &root, prepared).expect("unit create should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("committed pool project should re-resolve");
        (root, model, unit_id)
    }

    #[test]
    fn create_prepends_pool_ref_for_unreferenced_pool() {
        let (_root, model) = resolved_minimal_model("library_create_pool_ref");
        let unit_id = Uuid::new_v4();
        let object = pool_unit_payload(unit_id, "Test Unit", "ACME");

        let prepared = build_pool_library_write(
            &model,
            test_provenance(),
            Some("pool"),
            vec![PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "units", unit_id),
                object: object.clone(),
            }],
        )
        .expect("create should build");

        assert_eq!(prepared.primary_object_id, Some(unit_id));
        assert_eq!(
            prepared.batch.operations,
            vec![
                Operation::AddProjectPoolRef {
                    path: "pool".to_string(),
                    priority: 1,
                },
                Operation::CreatePoolLibraryObject {
                    object_id: unit_id,
                    relative_path: format!("pool/units/{unit_id}.json"),
                    object_kind: "units".to_string(),
                    object,
                },
            ]
        );
        assert_eq!(
            prepared.batch.expected_model_revision,
            Some(model.model_revision.clone())
        );
    }

    #[test]
    fn create_skips_pool_ref_when_pool_already_referenced() {
        let (_root, model, _unit_id) = resolved_model_with_pool_unit("library_pool_ref_skip");
        let symbol_id = Uuid::new_v4();

        let prepared = build_pool_library_write(
            &model,
            test_provenance(),
            Some("pool"),
            vec![PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "symbols", symbol_id),
                object: pool_symbol_payload(symbol_id, Uuid::new_v4(), "SYM"),
            }],
        )
        .expect("create should build");

        assert_eq!(prepared.batch.operations.len(), 1);
        assert!(matches!(
            prepared.batch.operations[0],
            Operation::CreatePoolLibraryObject { object_id, .. } if object_id == symbol_id
        ));
    }

    #[test]
    fn ensure_pool_ref_assigns_next_priority() {
        let (_root, model, _unit_id) = resolved_model_with_pool_unit("library_pool_priority");

        // "pool" was added at priority 1; a second pool gets max + 1 = 2.
        let operation = ensure_pool_ref_operation(&model, "pool-b")
            .expect("ensure should succeed")
            .expect("unreferenced pool should produce an operation");
        assert_eq!(
            operation,
            Operation::AddProjectPoolRef {
                path: "pool-b".to_string(),
                priority: 2,
            }
        );
        assert_eq!(
            ensure_pool_ref_operation(&model, "pool").expect("ensure should succeed"),
            None
        );
    }

    /// The exact previous-payload derivation the CLI performed pre-migration:
    /// a fresh resolve plus shard materialization by relative path.
    fn cli_previous_payload_oracle(root: &std::path::Path, relative_path: &str) -> serde_json::Value {
        let model = ProjectResolver::new(root)
            .resolve()
            .expect("oracle resolve should succeed");
        model
            .materialized_source_shard_value_by_relative_path(relative_path)
            .expect("oracle materialization should succeed")
    }

    #[test]
    fn previous_payload_matches_cli_fresh_resolve_and_raw_disk() {
        let (root, model, unit_id) = resolved_model_with_pool_unit("library_previous_parity");
        let relative_path = pool_library_relative_path("pool", "units", unit_id);

        let facade = pool_library_object_payload(&model, &relative_path, unit_id)
            .expect("facade payload should materialize");
        let cli_oracle = cli_previous_payload_oracle(&root, &relative_path);
        let raw_disk: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(&relative_path))
                .expect("pool shard should exist on disk"),
        )
        .expect("pool shard should parse");

        assert_eq!(facade, cli_oracle);
        assert_eq!(facade, raw_disk);
    }

    #[test]
    fn set_sources_previous_object_from_model_and_threads_object_verbatim() {
        let (root, model, unit_id) = resolved_model_with_pool_unit("library_set");
        let relative_path = pool_library_relative_path("pool", "units", unit_id);
        let previous = pool_library_object_payload(&model, &relative_path, unit_id)
            .expect("previous payload should materialize");
        let mut object = previous.clone();
        object["name"] = serde_json::json!("Renamed Unit");

        let prepared = build_pool_library_write(
            &model,
            test_provenance(),
            None,
            vec![PoolLibraryOperationSpec::Set {
                target: PoolLibraryObjectTarget::new("pool", "units", unit_id),
                object: object.clone(),
            }],
        )
        .expect("set should build");

        assert_eq!(
            prepared.batch.operations,
            vec![Operation::SetPoolLibraryObject {
                object_id: unit_id,
                relative_path: relative_path.clone(),
                object_kind: "units".to_string(),
                previous_object: previous,
                object,
            }]
        );

        // Commit and confirm the on-disk shard advanced to the new payload.
        let mut model = model;
        commit_prepared(&mut model, &root, prepared).expect("set should commit");
        let updated: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(root.join(&relative_path)).expect("shard should exist"),
        )
        .expect("shard should parse");
        assert_eq!(updated["name"], serde_json::json!("Renamed Unit"));
    }

    #[test]
    fn delete_sources_stored_payload_from_model() {
        let (_root, model, unit_id) = resolved_model_with_pool_unit("library_delete");
        let relative_path = pool_library_relative_path("pool", "units", unit_id);
        let stored = pool_library_object_payload(&model, &relative_path, unit_id)
            .expect("stored payload should materialize");

        let prepared = build_pool_library_write(
            &model,
            test_provenance(),
            None,
            vec![PoolLibraryOperationSpec::Delete {
                target: PoolLibraryObjectTarget::new("pool", "units", unit_id),
            }],
        )
        .expect("delete should build");

        assert_eq!(
            prepared.batch.operations,
            vec![Operation::DeletePoolLibraryObject {
                object_id: unit_id,
                relative_path,
                object_kind: "units".to_string(),
                object: stored,
            }]
        );
    }

    #[test]
    fn set_rejects_unknown_shard_and_mismatched_uuid() {
        let (_root, model, unit_id) = resolved_model_with_pool_unit("library_set_unknown");
        let missing = Uuid::new_v4();

        let error = build_pool_library_write(
            &model,
            test_provenance(),
            None,
            vec![PoolLibraryOperationSpec::Set {
                target: PoolLibraryObjectTarget::new("pool", "units", missing),
                object: serde_json::json!({}),
            }],
        )
        .expect_err("unknown shard should fail");
        assert!(error.to_string().contains("source shard"));

        // Addressing an existing shard with the wrong object id must fail the
        // uuid check.
        let relative_path = pool_library_relative_path("pool", "units", unit_id);
        let error = pool_library_object_payload(&model, &relative_path, missing)
            .expect_err("mismatched uuid should fail");
        assert!(
            error
                .to_string()
                .contains(&format!("does not match target object {missing}"))
        );
    }

    #[test]
    fn attach_and_detach_source_previous_attachments_from_model() {
        let (root, mut model) = resolved_minimal_model("library_attach");
        let part_id = Uuid::new_v4();
        // A part payload without behavioural_models: previous baseline is [].
        let prepared = build_pool_library_write(
            &model,
            test_provenance(),
            Some("pool"),
            vec![PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "parts", part_id),
                object: serde_json::json!({
                    "schema_version": 1,
                    "uuid": part_id,
                    "entity": Uuid::new_v4(),
                    "package": Uuid::new_v4(),
                    "pad_map": {},
                    "mpn": "",
                    "manufacturer": "",
                    "value": "R",
                    "description": "",
                    "datasheet": "",
                    "parametric": {},
                    "orderable_mpns": [],
                    "tags": [],
                    "lifecycle": "Active",
                    "base": null
                }),
            }],
        )
        .expect("part create should build");
        commit_prepared(&mut model, &root, prepared).expect("part create should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");

        let relative_path = pool_library_relative_path("pool", "parts", part_id);
        let attachment = serde_json::to_value(crate::pool::ModelAttachment {
            uuid: Uuid::new_v4(),
            model_uuid: Uuid::new_v4(),
            role: ModelRole::Spice,
            dialect: None,
            model_names: vec!["OPA1656".to_string()],
            encrypted: false,
            encryption_scheme: None,
            provenance: None,
            format_metadata: crate::pool::ModelFormatMetadata::None,
            reviewed: None,
            notes: Vec::new(),
        })
        .expect("attachment should serialize");
        let prepared = build_pool_library_write(
            &model,
            test_provenance(),
            None,
            vec![PoolLibraryOperationSpec::AttachPartModel {
                part_id,
                relative_path: relative_path.clone(),
                attachments: vec![attachment.clone()],
            }],
        )
        .expect("attach should build");
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::AttachPoolPartModel {
                part_id,
                relative_path: relative_path.clone(),
                previous_attachments: Vec::new(),
                attachments: vec![attachment.clone()],
            }]
        );

        // Commit the attach, then a detach must see the attachment as the
        // previous baseline.
        let mut model = model;
        commit_prepared(&mut model, &root, prepared).expect("attach should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");
        let prepared = build_pool_library_write(
            &model,
            test_provenance(),
            None,
            vec![PoolLibraryOperationSpec::DetachPartModel {
                part_id,
                relative_path: relative_path.clone(),
                attachments: Vec::new(),
            }],
        )
        .expect("detach should build");
        assert_eq!(
            prepared.batch.operations,
            vec![Operation::DetachPoolPartModel {
                part_id,
                relative_path,
                previous_attachments: vec![attachment],
                attachments: Vec::new(),
            }]
        );
    }

    #[test]
    fn build_matches_cli_hand_rolled_batch_shape() {
        // Parity oracle: the pre-migration CLI committed a raw OperationBatch
        // (fresh v4 batch id, Some(model_revision), provenance, operations —
        // with no revision guards, since pool objects are shard-addressed).
        let (_root, model) = resolved_minimal_model("library_batch_parity");
        let unit_id = Uuid::new_v4();
        let object = pool_unit_payload(unit_id, "Parity Unit", "ACME");

        let prepared = build_pool_library_write(
            &model,
            test_provenance(),
            Some("pool"),
            vec![PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "units", unit_id),
                object: object.clone(),
            }],
        )
        .expect("create should build");

        let cli_batch = OperationBatch {
            batch_id: prepared.batch.batch_id,
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: test_provenance().into(),
            operations: vec![
                Operation::AddProjectPoolRef {
                    path: "pool".to_string(),
                    priority: 1,
                },
                Operation::CreatePoolLibraryObject {
                    object_id: unit_id,
                    relative_path: format!("pool/units/{unit_id}.json"),
                    object_kind: "units".to_string(),
                    object,
                },
            ],
        };
        assert_eq!(prepared.batch, cli_batch);
    }

    #[test]
    fn pool_model_blob_write_is_idempotent_and_creates_directories() {
        let root = temp_project_root("library_blob");
        let relative_path = "pool/models/spice/abc123.lib";

        let target = write_pool_model_blob(&root, relative_path, b"first")
            .expect("blob write should succeed");
        assert_eq!(target, root.join(relative_path));
        assert_eq!(std::fs::read(&target).expect("blob should read"), b"first");

        // A hash-named blob never changes: rewriting must be a no-op.
        write_pool_model_blob(&root, relative_path, b"second")
            .expect("idempotent write should succeed");
        assert_eq!(std::fs::read(&target).expect("blob should read"), b"first");
    }

    #[test]
    fn model_and_attachment_uuid_derivations_match_cli() {
        // Byte-exact historical CLI derivations
        // (crates/cli/src/command_project_library.rs pre-migration).
        let sha256 = "d0a3f1";
        assert_eq!(
            derive_pool_model_uuid(sha256),
            Uuid::new_v5(
                &Uuid::NAMESPACE_URL,
                format!("datum-eda:pool-model:{sha256}").as_bytes(),
            )
        );

        let part_id = Uuid::new_v4();
        let model_uuid = derive_pool_model_uuid(sha256);
        let role = ModelRole::Spice;
        let model_names = vec!["OPA1656".to_string(), "ALT".to_string()];
        assert_eq!(
            derive_pool_model_attachment_uuid(part_id, model_uuid, &role, &model_names),
            Uuid::new_v5(
                &Uuid::NAMESPACE_URL,
                format!(
                    "datum-eda:model-attachment:{part_id}:{model_uuid}:{:?}:{model_names:?}",
                    role
                )
                .as_bytes(),
            )
        );
    }

    #[test]
    fn create_payload_templates_match_historical_shapes() {
        let id = Uuid::new_v4();
        let other = Uuid::new_v4();
        assert_eq!(
            pool_unit_payload(id, "U", "M"),
            serde_json::json!({
                "schema_version": 1, "uuid": id, "name": "U",
                "manufacturer": "M", "pins": {}, "tags": []
            })
        );
        assert_eq!(
            pool_symbol_payload(id, other, "S"),
            serde_json::json!({
                "schema_version": 1, "uuid": id, "name": "S", "unit": other
            })
        );
        assert_eq!(
            pool_padstack_payload(id, "P", serde_json::Value::Null, None),
            serde_json::json!({
                "schema_version": 1, "uuid": id, "name": "P",
                "aperture": null, "drill_nm": null
            })
        );
        let part = pool_part_payload(id, other, other, "MPN", "M", "V", "D", "DS", "Active");
        assert_eq!(part["schema_version"], serde_json::json!(1));
        assert_eq!(part["behavioural_models"], serde_json::json!([]));
        assert_eq!(part["lifecycle"], serde_json::json!("Active"));
        assert_eq!(part["default_pin_pad_map"], serde_json::Value::Null);
        let package = pool_package_payload(id, "PKG", serde_json::json!({}));
        assert_eq!(
            package["courtyard"],
            serde_json::json!({"vertices": [], "closed": true})
        );
        assert_eq!(package["body_height_nm"], serde_json::Value::Null);
    }
}
