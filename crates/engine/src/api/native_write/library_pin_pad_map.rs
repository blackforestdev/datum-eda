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
        .ok_or_else(|| EngineError::Validation(format!("part {part_id} payload is not an object")))?
        .insert(
            "default_pin_pad_map".to_string(),
            serde_json::Value::String(map_id.to_string()),
        );
    Ok(PoolLibraryOperationSpec::Set {
        target: PoolLibraryObjectTarget::at_relative_path(part_id, "parts", relative_path),
        object,
    })
}

fn validation(message: String) -> EngineError {
    EngineError::Validation(message)
}

// Preserve legacy CLI anyhow Context rendering, including every source cause.
fn validation_context(context: String, error: &dyn std::error::Error) -> EngineError {
    let mut message = format!("{context}: {error}");
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(&format!(": {cause}"));
        source = cause.source();
    }
    validation(message)
}

/// Check existing typed PinPadMap rules for a manual generic Create.
/// Canonical schema, source ownership and UUID collisions retain precedence.
pub fn validate_pin_pad_map_creation_references(
    model: &DesignModel,
    relative_path: &str,
    object: &serde_json::Value,
) -> Result<(), EngineError> {
    validate_pin_pad_map_definition(model, relative_path, object, false)
}

/// Check existing typed PinPadMap rules for a manual generic Set.
pub fn validate_pin_pad_map_replacement_references(
    model: &DesignModel,
    relative_path: &str,
    object: &serde_json::Value,
) -> Result<(), EngineError> {
    validate_pin_pad_map_definition(model, relative_path, object, true)
}

fn validate_pin_pad_map_definition(
    model: &DesignModel,
    relative_path: &str,
    object: &serde_json::Value,
    replacement: bool,
) -> Result<(), EngineError> {
    if object
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
    {
        return Ok(());
    }
    let Ok(map) = serde_json::from_value::<crate::pool::PinPadMap>(object.clone()) else {
        return Ok(());
    };
    let blocked = if replacement {
        crate::substrate::pool_replacement_has_source_blocker(
            &model.objects,
            map.uuid,
            relative_path,
            object,
            "pin_pad_maps",
        )
    } else {
        crate::substrate::pool_creation_has_source_blocker(
            &model.objects,
            map.uuid,
            relative_path,
            object,
            "pin_pad_maps",
        )
    };
    if blocked {
        return Ok(());
    }
    // Read original JSON keys: differently cased UUID spellings must reach
    // the existing duplicate-pad check rather than coalescing in a HashMap.
    // The typed decode above owns shape admission; the supplied payload stays intact.
    let mappings = object
        .get("mappings")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flatten()
        .map(|(pad, entry)| {
            let pad = Uuid::parse_str(pad).map_err(|error| validation(error.to_string()))?;
            let entry: crate::pool::PadMapEntry = serde_json::from_value(entry.clone())?;
            Ok(PinPadMapMapping {
                pad,
                gate: entry.gate,
                pin: entry.pin,
            })
        })
        .collect::<Result<Vec<_>, EngineError>>()?;
    validate_pin_pad_map_payload(model, map.part, map.footprint, &mappings)
}

/// Existing typed manual/proposal map validation, shared with generic admission.
pub fn validate_pin_pad_map_payload(
    model: &DesignModel,
    part_id: Uuid,
    footprint_id: Option<Uuid>,
    mappings: &[PinPadMapMapping],
) -> Result<(), EngineError> {
    if mappings.is_empty() {
        return Err(validation(
            "PinPadMap requires at least one mapping".to_string(),
        ));
    }
    let part = materialized_pool_object(model, part_id, "parts")?;
    let entity_id = uuid_field(&part, "entity", "part")?;
    let package_id = uuid_field(&part, "package", "part")?;
    let entity = materialized_pool_object(model, entity_id, "entities")?;
    let valid_pads = if let Some(footprint_id) = footprint_id {
        let footprint = materialized_pool_object(model, footprint_id, "footprints")?;
        let footprint_package_id = uuid_field(&footprint, "package", "footprint")?;
        if footprint_package_id != package_id {
            return Err(validation(format!(
                "pool footprint {footprint_id} belongs to package {footprint_package_id}, not part package {package_id}"
            )));
        }
        object_keys(
            &footprint,
            "pads",
            &format!("pool footprint {footprint_id}"),
        )?
    } else {
        let package = materialized_pool_object(model, package_id, "packages")?;
        object_keys(&package, "pads", &format!("pool package {package_id}"))?
    };
    let mut seen_pads = std::collections::HashSet::new();
    let mut seen_gate_pins = std::collections::HashSet::new();
    for entry in mappings {
        if !seen_pads.insert(entry.pad) {
            return Err(validation(format!(
                "duplicate PinPadMap mapping for pad {}",
                entry.pad
            )));
        }
        if !seen_gate_pins.insert((entry.gate, entry.pin)) {
            return Err(validation(format!(
                "duplicate PinPadMap mapping for gate {} pin {}",
                entry.gate, entry.pin
            )));
        }
        validate_gate_pin(model, entity_id, &entity, entry.gate, entry.pin)?;
        if !valid_pads.contains(&entry.pad) {
            return Err(validation(format!(
                "PinPadMap target has no pad {}",
                entry.pad
            )));
        }
    }
    Ok(())
}
fn validate_gate_pin(
    model: &DesignModel,
    entity_id: Uuid,
    entity: &serde_json::Value,
    gate_id: Uuid,
    pin_id: Uuid,
) -> Result<(), EngineError> {
    let gate = entity
        .get("gates")
        .and_then(serde_json::Value::as_object)
        .and_then(|gates| gates.get(&gate_id.to_string()))
        .ok_or_else(|| validation(format!("pool entity {entity_id} has no gate {gate_id}")))?;
    let unit_id = uuid_field(gate, "unit", "entity gate")?;
    let unit = materialized_pool_object(model, unit_id, "units")?;
    let unit_pins = unit
        .get("pins")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| validation(format!("pool unit {unit_id} has no pins map")))?;
    if !unit_pins.contains_key(&pin_id.to_string()) {
        return Err(validation(format!(
            "pool unit {unit_id} has no pin {pin_id}"
        )));
    }
    Ok(())
}

fn object_keys(
    value: &serde_json::Value,
    field: &str,
    label: &str,
) -> Result<std::collections::HashSet<Uuid>, EngineError> {
    let object = value
        .get(field)
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| validation(format!("{label} has no {field} map")))?;
    object
        .keys()
        .map(|key| {
            Uuid::parse_str(key).map_err(|error| {
                validation_context(format!("{label} {field} key {key} is not a UUID"), &error)
            })
        })
        .collect()
}
fn materialized_pool_object(
    model: &DesignModel,
    object_id: Uuid,
    object_kind: &str,
) -> Result<serde_json::Value, EngineError> {
    let object = model
        .objects
        .get(&object_id)
        .filter(|object| object.domain == "pool" && object.kind == object_kind)
        .ok_or_else(|| validation(format!("missing pool {object_kind} {object_id}")))?;
    let shard = model
        .source_shards
        .iter()
        .find(|shard| shard.shard_id == object.source_shard_id)
        .ok_or_else(|| {
            validation(format!(
                "missing source shard for pool {object_kind} {object_id}"
            ))
        })?;
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .map_err(|error| {
            validation_context(
                format!("failed to materialize pool {object_kind} {object_id}"),
                &error,
            )
        })
}

fn uuid_field(value: &serde_json::Value, field: &str, label: &str) -> Result<Uuid, EngineError> {
    let raw = value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| validation(format!("{label} missing {field}")))?;
    Uuid::parse_str(raw).map_err(|error| {
        validation_context(format!("{label} has invalid {field} uuid {raw}"), &error)
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
        assert_eq!(target.relative_path, format!("pool/parts/{part_id}.json"));
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
