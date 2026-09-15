use crate::*;
use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::api::native_write::library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec};
use eda_engine::api::native_write::library_pin_pad_map::{
    pin_pad_map_mapping_json, pin_pad_map_payload, set_part_default_pin_pad_map_spec,
};
use eda_engine::substrate::ProjectResolver;
use uuid::Uuid;

use super::library::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations,
    pool_library_mutation_view, pool_library_relative_path, validate_project_local_pool_path,
};
use super::payload::read_project_pool_object_payload;

pub(crate) fn create_native_project_pool_pin_pad_map(
    root: &Path,
    pool_path: &str,
    map_id: Uuid,
    part_id: Uuid,
    footprint_id: Option<Uuid>,
    entries: Vec<String>,
    set_default: bool,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let mappings = parse_mapping_entries(entries, &model, part_id)?;
    validate_pin_pad_map_payload(&model, part_id, footprint_id, &mappings)?;

    let relative_path = pool_library_relative_path(pool_path, "pin_pad_maps", map_id);
    let object = pin_pad_map_payload(map_id, part_id, footprint_id, &mappings);
    let mut operations = vec![PoolLibraryOperationSpec::Create {
        target: PoolLibraryObjectTarget::new(pool_path, "pin_pad_maps", map_id),
        object,
    }];
    if set_default {
        operations.push(set_part_default_pin_pad_map_spec(
            &model, pool_path, part_id, map_id,
        )?);
    }
    commit_pool_library_operations(
        root,
        format!("create native pool PinPadMap {map_id} for part {part_id}"),
        None,
        operations,
    )?;
    pool_library_mutation_view(
        root,
        "create_pin_pad_map",
        pool_path,
        "pin_pad_maps",
        map_id,
        &relative_path,
    )
}

pub(crate) fn set_native_project_pool_pin_pad_map(
    root: &Path,
    pool_path: &str,
    map_id: Uuid,
    mode: String,
    entries: Vec<String>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let replace = match mode.as_str() {
        "merge" => false,
        "replace" => true,
        other => bail!("unsupported PinPadMap mode {other}; expected merge or replace"),
    };
    let relative_path = pool_library_relative_path(pool_path, "pin_pad_maps", map_id);
    let previous_object = read_project_pool_object_payload(root, &relative_path, map_id)?;
    let part_id = uuid_field(&previous_object, "part", "pin_pad_map")?;
    let footprint_id = optional_uuid_field(&previous_object, "footprint", "pin_pad_map")?;
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let entries = parse_mapping_entries(entries, &model, part_id)?;

    let mut object = previous_object.clone();
    let mappings = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("pin_pad_map {map_id} payload is not an object"))?
        .entry("mappings")
        .or_insert_with(|| serde_json::json!({}));
    if replace {
        *mappings = serde_json::json!({});
    }
    let mappings = mappings
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("pin_pad_map {map_id} mappings is not an object"))?;
    for entry in entries {
        mappings.insert(entry.pad.to_string(), pin_pad_map_mapping_json(&entry));
    }
    let merged = mappings
        .iter()
        .map(|(pad, entry)| {
            let pad = Uuid::parse_str(pad)
                .with_context(|| format!("pin_pad_map mapping key {pad} is not a UUID"))?;
            Ok(PinPadMapEntryInput {
                pad,
                gate: uuid_field(entry, "gate", "pin_pad_map mapping")?,
                pin: uuid_field(entry, "pin", "pin_pad_map mapping")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    validate_pin_pad_map_payload(&model, part_id, footprint_id, &merged)?;

    commit_pool_library_operations(
        root,
        format!("set native pool PinPadMap {map_id} mappings"),
        None,
        vec![PoolLibraryOperationSpec::Set {
            target: PoolLibraryObjectTarget::new(pool_path, "pin_pad_maps", map_id),
            object,
        }],
    )?;
    pool_library_mutation_view(
        root,
        "set_pin_pad_map",
        pool_path,
        "pin_pad_maps",
        map_id,
        &relative_path,
    )
}

pub(super) fn validate_pin_pad_map_payload(
    model: &eda_engine::substrate::DesignModel,
    part_id: Uuid,
    footprint_id: Option<Uuid>,
    mappings: &[PinPadMapEntryInput],
) -> Result<()> {
    if mappings.is_empty() {
        bail!("PinPadMap requires at least one mapping");
    }
    let part = materialized_pool_object(model, part_id, "parts")?;
    let entity_id = uuid_field(&part, "entity", "part")?;
    let package_id = uuid_field(&part, "package", "part")?;
    let entity = materialized_pool_object(model, entity_id, "entities")?;
    let valid_pads = if let Some(footprint_id) = footprint_id {
        let footprint = materialized_pool_object(model, footprint_id, "footprints")?;
        let footprint_package_id = uuid_field(&footprint, "package", "footprint")?;
        if footprint_package_id != package_id {
            bail!(
                "pool footprint {footprint_id} belongs to package {footprint_package_id}, not part package {package_id}"
            );
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
            bail!("duplicate PinPadMap mapping for pad {}", entry.pad);
        }
        if !seen_gate_pins.insert((entry.gate, entry.pin)) {
            bail!(
                "duplicate PinPadMap mapping for gate {} pin {}",
                entry.gate,
                entry.pin
            );
        }
        validate_gate_pin(model, entity_id, &entity, entry.gate, entry.pin)?;
        if !valid_pads.contains(&entry.pad) {
            bail!("PinPadMap target has no pad {}", entry.pad);
        }
    }
    Ok(())
}

fn infer_gate_for_pin(
    model: &eda_engine::substrate::DesignModel,
    part_id: Uuid,
    pin_id: Uuid,
) -> Result<Uuid> {
    let part = materialized_pool_object(model, part_id, "parts")?;
    let entity_id = uuid_field(&part, "entity", "part")?;
    let entity = materialized_pool_object(model, entity_id, "entities")?;
    let mut matches = Vec::new();
    let gates = entity
        .get("gates")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("pool entity {entity_id} has no gates map"))?;
    for (gate_id, gate) in gates {
        let gate_id = Uuid::parse_str(gate_id)
            .with_context(|| format!("pool entity {entity_id} gate key {gate_id} is not a UUID"))?;
        let unit_id = uuid_field(gate, "unit", "entity gate")?;
        let unit = materialized_pool_object(model, unit_id, "units")?;
        if unit
            .get("pins")
            .and_then(serde_json::Value::as_object)
            .is_some_and(|pins| pins.contains_key(&pin_id.to_string()))
        {
            matches.push(gate_id);
        }
    }
    match matches.as_slice() {
        [gate_id] => Ok(*gate_id),
        [] => bail!("pool entity {entity_id} has no pin {pin_id}"),
        _ => bail!(
            "PinPadMap entry {pin_id} is ambiguous across multiple gates; use pad_uuid:gate_uuid:pin_uuid"
        ),
    }
}

fn validate_gate_pin(
    model: &eda_engine::substrate::DesignModel,
    entity_id: Uuid,
    entity: &serde_json::Value,
    gate_id: Uuid,
    pin_id: Uuid,
) -> Result<()> {
    let gate = entity
        .get("gates")
        .and_then(serde_json::Value::as_object)
        .and_then(|gates| gates.get(&gate_id.to_string()))
        .ok_or_else(|| anyhow::anyhow!("pool entity {entity_id} has no gate {gate_id}"))?;
    let unit_id = uuid_field(gate, "unit", "entity gate")?;
    let unit = materialized_pool_object(model, unit_id, "units")?;
    let unit_pins = unit
        .get("pins")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("pool unit {unit_id} has no pins map"))?;
    if !unit_pins.contains_key(&pin_id.to_string()) {
        bail!("pool unit {unit_id} has no pin {pin_id}");
    }
    Ok(())
}

fn object_keys(
    value: &serde_json::Value,
    field: &str,
    label: &str,
) -> Result<std::collections::HashSet<Uuid>> {
    let object = value
        .get(field)
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("{label} has no {field} map"))?;
    object
        .keys()
        .map(|key| {
            Uuid::parse_str(key).with_context(|| format!("{label} {field} key {key} is not a UUID"))
        })
        .collect()
}

pub(crate) use eda_engine::api::native_write::library_pin_pad_map::PinPadMapMapping as PinPadMapEntryInput;

pub(super) fn parse_mapping_entries(
    entries: Vec<String>,
    model: &eda_engine::substrate::DesignModel,
    part_id: Uuid,
) -> Result<Vec<PinPadMapEntryInput>> {
    let mut parsed = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for entry in entries {
        let fields = entry.split(':').collect::<Vec<_>>();
        let parsed_entry = match fields.as_slice() {
            [pin, pad] => {
                let pin = Uuid::parse_str(pin)
                    .with_context(|| format!("invalid pin uuid in PinPadMap entry {entry}"))?;
                let pad = Uuid::parse_str(pad)
                    .with_context(|| format!("invalid pad uuid in PinPadMap entry {entry}"))?;
                PinPadMapEntryInput {
                    pad,
                    gate: infer_gate_for_pin(model, part_id, pin)?,
                    pin,
                }
            }
            [pad, gate, pin] => PinPadMapEntryInput {
                pad: Uuid::parse_str(pad)
                    .with_context(|| format!("invalid pad uuid in PinPadMap entry {entry}"))?,
                gate: Uuid::parse_str(gate)
                    .with_context(|| format!("invalid gate uuid in PinPadMap entry {entry}"))?,
                pin: Uuid::parse_str(pin)
                    .with_context(|| format!("invalid pin uuid in PinPadMap entry {entry}"))?,
            },
            _ => bail!(
                "PinPadMap entry {entry} must be pin_uuid:pad_uuid or pad_uuid:gate_uuid:pin_uuid"
            ),
        };
        if !seen.insert(parsed_entry.pad) {
            bail!("duplicate PinPadMap mapping for pad {}", parsed_entry.pad);
        }
        parsed.push(parsed_entry);
    }
    Ok(parsed)
}

fn materialized_pool_object(
    model: &eda_engine::substrate::DesignModel,
    object_id: Uuid,
    object_kind: &str,
) -> Result<serde_json::Value> {
    let object = model
        .objects
        .get(&object_id)
        .filter(|object| object.domain == "pool" && object.kind == object_kind)
        .ok_or_else(|| anyhow::anyhow!("missing pool {object_kind} {object_id}"))?;
    let shard = model
        .source_shards
        .iter()
        .find(|shard| shard.shard_id == object.source_shard_id)
        .ok_or_else(|| {
            anyhow::anyhow!("missing source shard for pool {object_kind} {object_id}")
        })?;
    model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .with_context(|| format!("failed to materialize pool {object_kind} {object_id}"))
}

fn uuid_field(value: &serde_json::Value, field: &str, label: &str) -> Result<Uuid> {
    let raw = value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{label} missing {field}"))?;
    Uuid::parse_str(raw).with_context(|| format!("{label} has invalid {field} uuid {raw}"))
}

fn optional_uuid_field(
    value: &serde_json::Value,
    field: &str,
    label: &str,
) -> Result<Option<Uuid>> {
    match value.get(field) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(value) => {
            Ok(Some(parse_uuid_value(value, field).with_context(|| {
                format!("{label} has invalid {field} uuid")
            })?))
        }
    }
}

fn parse_uuid_value(value: &serde_json::Value, label: &str) -> Result<Uuid> {
    let raw = value
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("{label} must be a UUID string"))?;
    Uuid::parse_str(raw).with_context(|| format!("{label} is not a valid UUID: {raw}"))
}

// Phase 5: exec-layer dissolution — variant run() impls (the former
// command_exec destructure-and-forward glue, now inherent methods on the
// clap args structs).

impl ProjectCreatePoolPinPadMapArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            pool,
            map_uuid,
            part_uuid,
            footprint_uuid,
            entries,
            set_default,
        } = self;
        Ok((
            render_output(
                format,
                &create_native_project_pool_pin_pad_map(
                    &path,
                    &pool,
                    map_uuid,
                    part_uuid,
                    footprint_uuid,
                    entries,
                    set_default,
                )?,
            ),
            0,
        ))
    }
}

impl ProjectSetPoolPinPadMapArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            pool,
            map_uuid,
            mode,
            entries,
        } = self;
        Ok((
            render_output(
                format,
                &set_native_project_pool_pin_pad_map(&path, &pool, map_uuid, mode, entries)?,
            ),
            0,
        ))
    }
}
