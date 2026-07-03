//! Pin-pad-map builders for the native write facade.
//!
//! Family B (pin-pad-map slice): the PinPadMap payload templates and the
//! set-part-default composition live here, shared by the direct CLI commands
//! in `crates/cli/src/command_project_library_pin_pad_map.rs` and
//! `crates/cli/src/command_project_library_pad_map.rs` and the proposal flows
//! in `crates/cli/src/command_project_library_proposals.rs` (both feed the
//! same [`super::library::PoolLibraryOperationSpec`]s into
//! [`super::library::build_pool_library_write`]).
//!
//! Payload shapes are byte-for-byte the CLI's historical behavior — PinPadMap
//! shards must not drift.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::EngineError;
use crate::substrate::DesignModel;

use super::library::{
    PoolLibraryObjectTarget, PoolLibraryOperationSpec, pool_library_object_payload,
    pool_library_relative_path,
};

/// One validated pad→(gate, pin) mapping of a PinPadMap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinPadMapMapping {
    pub pad: Uuid,
    pub gate: Uuid,
    pub pin: Uuid,
}

/// Historical create-payload template for a native PinPadMap.
pub fn pin_pad_map_payload(
    map_id: Uuid,
    part_id: Uuid,
    footprint_id: Option<Uuid>,
    mappings: &[PinPadMapMapping],
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "uuid": map_id,
        "part": part_id,
        "footprint": footprint_id,
        "mappings": pin_pad_map_mappings_json(mappings),
        "tags": []
    })
}

/// The `mappings` object of a PinPadMap payload: `{ pad: { gate, pin } }`.
pub fn pin_pad_map_mappings_json(mappings: &[PinPadMapMapping]) -> serde_json::Value {
    serde_json::Value::Object(
        mappings
            .iter()
            .map(|mapping| (mapping.pad.to_string(), pin_pad_map_mapping_json(mapping)))
            .collect(),
    )
}

/// One mapping entry of a PinPadMap payload: `{ gate, pin }`.
pub fn pin_pad_map_mapping_json(mapping: &PinPadMapMapping) -> serde_json::Value {
    serde_json::json!({
        "gate": mapping.gate,
        "pin": mapping.pin
    })
}

/// The `Set` spec that points a part's `default_pin_pad_map` at `map_id`.
///
/// The part payload is sourced from the resolved model (the same
/// materialization the pre-migration CLI used) and edited in place; composed
/// after a `Create` spec this yields the historical one-batch
/// create-with-default flow.
pub fn set_part_default_pin_pad_map_spec(
    model: &DesignModel,
    pool_path: &str,
    part_id: Uuid,
    map_id: Uuid,
) -> Result<PoolLibraryOperationSpec, EngineError> {
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let mut object = pool_library_object_payload(model, &relative_path, part_id)?;
    object
        .as_object_mut()
        .ok_or_else(|| {
            EngineError::Validation(format!("part {part_id} payload is not an object"))
        })?
        .insert(
            "default_pin_pad_map".to_string(),
            serde_json::Value::String(map_id.to_string()),
        );
    Ok(PoolLibraryOperationSpec::Set {
        target: PoolLibraryObjectTarget::at_relative_path(part_id, "parts", relative_path),
        object,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::super::context::{WriteProvenance, commit_prepared};
    use super::super::library::{build_pool_library_write, pool_part_payload};
    use super::super::test_support::{temp_project_root, write_minimal_project};
    use super::*;
    use crate::substrate::{CommitSource, Operation, ProjectResolver};

    fn test_provenance() -> WriteProvenance {
        WriteProvenance::new("unit-test", CommitSource::Test, "pin pad map facade test")
    }

    fn resolved_model_with_pool_part(name: &str) -> (PathBuf, DesignModel, Uuid) {
        let root = temp_project_root(name);
        write_minimal_project(&root, Uuid::new_v4(), Uuid::new_v4());
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("fixture project should resolve");
        let part_id = Uuid::new_v4();
        let prepared = build_pool_library_write(
            &model,
            test_provenance(),
            Some("pool"),
            vec![PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", "parts", part_id),
                object: pool_part_payload(
                    part_id,
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    "MPN",
                    "M",
                    "V",
                    "D",
                    "DS",
                    "Active",
                ),
            }],
        )
        .expect("part create should build");
        commit_prepared(&mut model, &root, prepared).expect("part create should commit");
        let model = ProjectResolver::new(&root)
            .resolve()
            .expect("project should re-resolve");
        (root, model, part_id)
    }

    #[test]
    fn pin_pad_map_payload_matches_historical_shape() {
        let map_id = Uuid::new_v4();
        let part_id = Uuid::new_v4();
        let mapping = PinPadMapMapping {
            pad: Uuid::new_v4(),
            gate: Uuid::new_v4(),
            pin: Uuid::new_v4(),
        };

        assert_eq!(
            pin_pad_map_payload(map_id, part_id, None, &[mapping]),
            serde_json::json!({
                "schema_version": 1,
                "uuid": map_id,
                "part": part_id,
                "footprint": null,
                "mappings": {
                    mapping.pad.to_string(): { "gate": mapping.gate, "pin": mapping.pin }
                },
                "tags": []
            })
        );

        let footprint_id = Uuid::new_v4();
        assert_eq!(
            pin_pad_map_payload(map_id, part_id, Some(footprint_id), &[])["footprint"],
            serde_json::json!(footprint_id)
        );
    }

    #[test]
    fn set_default_spec_edits_model_sourced_part_payload() {
        let (_root, model, part_id) = resolved_model_with_pool_part("pin_pad_map_default");
        let map_id = Uuid::new_v4();

        let spec = set_part_default_pin_pad_map_spec(&model, "pool", part_id, map_id)
            .expect("set-default spec should build");

        let PoolLibraryOperationSpec::Set { target, object } = &spec else {
            panic!("expected Set spec");
        };
        assert_eq!(target.object_id, part_id);
        assert_eq!(target.object_kind, "parts");
        assert_eq!(
            target.relative_path,
            format!("pool/parts/{part_id}.json")
        );
        assert_eq!(
            object["default_pin_pad_map"],
            serde_json::json!(map_id.to_string())
        );

        // Composed after a Create, the batch carries the model-sourced
        // previous part payload — the historical one-batch default flow.
        let map_payload = pin_pad_map_payload(map_id, part_id, None, &[]);
        let prepared = build_pool_library_write(
            &model,
            test_provenance(),
            None,
            vec![
                PoolLibraryOperationSpec::Create {
                    target: PoolLibraryObjectTarget::new("pool", "pin_pad_maps", map_id),
                    object: map_payload,
                },
                spec,
            ],
        )
        .expect("composite write should build");
        assert_eq!(prepared.batch.operations.len(), 2);
        let Operation::SetPoolLibraryObject {
            previous_object,
            object,
            ..
        } = &prepared.batch.operations[1]
        else {
            panic!("expected SetPoolLibraryObject");
        };
        assert_eq!(
            previous_object["default_pin_pad_map"],
            serde_json::Value::Null
        );
        assert_eq!(
            object["default_pin_pad_map"],
            serde_json::json!(map_id.to_string())
        );
    }

    #[test]
    fn set_default_spec_rejects_missing_part() {
        let (_root, model, _part_id) = resolved_model_with_pool_part("pin_pad_map_missing");
        let missing = Uuid::new_v4();
        let error = set_part_default_pin_pad_map_spec(&model, "pool", missing, Uuid::new_v4())
            .expect_err("missing part should fail");
        assert!(error.to_string().contains("source shard"));
    }
}
