use std::path::Path;

use anyhow::{Context, Result, bail};
use eda_engine::substrate::{Operation, ProjectResolver};
use uuid::Uuid;

use super::command_project_library::{
    NativeProjectPoolLibraryObjectMutationView, commit_pool_library_operations,
    pool_library_mutation_view, pool_library_relative_path, read_pool_library_object_payload,
    validate_project_local_pool_path,
};

pub(crate) fn set_native_project_pool_part_pad_map_entry(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    pad_id: Uuid,
    gate_id: Uuid,
    pin_id: Uuid,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    set_native_project_pool_part_pad_map(
        root,
        pool_path,
        part_id,
        "merge".to_string(),
        vec![PartPadMapEntryInput {
            pad: pad_id,
            gate: gate_id,
            pin: pin_id,
        }],
        "set_part_pad_map_entry",
    )
}

pub(crate) fn set_native_project_pool_part_pad_map_from_entries(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    mode: String,
    entries: Vec<String>,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    let entries = entries
        .into_iter()
        .map(|entry| parse_pad_map_entry(&entry))
        .collect::<Result<Vec<_>>>()?;
    set_native_project_pool_part_pad_map(
        root,
        pool_path,
        part_id,
        mode,
        entries,
        "set_part_pad_map",
    )
}

struct PartPadMapEntryInput {
    pad: Uuid,
    gate: Uuid,
    pin: Uuid,
}

fn set_native_project_pool_part_pad_map(
    root: &Path,
    pool_path: &str,
    part_id: Uuid,
    mode: String,
    entries: Vec<PartPadMapEntryInput>,
    action: &'static str,
) -> Result<NativeProjectPoolLibraryObjectMutationView> {
    validate_project_local_pool_path(pool_path)?;
    let replace = match mode.as_str() {
        "merge" => false,
        "replace" => true,
        other => bail!("unsupported part pad-map mode {other}; expected merge or replace"),
    };
    if entries.is_empty() {
        bail!("part pad-map requires at least one entry");
    }
    let relative_path = pool_library_relative_path(pool_path, "parts", part_id);
    let previous_object = read_pool_library_object_payload(&root.join(&relative_path), part_id)?;
    let package_id = uuid_field(&previous_object, "package", "part")?;
    let entity_id = uuid_field(&previous_object, "entity", "part")?;
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    require_pool_object(&model, part_id, "parts")?;
    let package = materialized_pool_object(&model, package_id, "packages")?;
    let package_pads = package
        .get("pads")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("pool package {package_id} has no pads map"))?;
    let entity = materialized_pool_object(&model, entity_id, "entities")?;
    let mut requested_pads = std::collections::HashSet::new();
    let mut object = previous_object.clone();
    let pad_map = object
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} payload is not an object"))?
        .entry("pad_map")
        .or_insert_with(|| serde_json::json!({}));
    if replace {
        *pad_map = serde_json::json!({});
    }
    let pad_map = pad_map
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("part {part_id} pad_map is not an object"))?;
    for entry in entries {
        if !requested_pads.insert(entry.pad) {
            bail!("duplicate pad-map entry for pad {}", entry.pad);
        }
        validate_pad_map_entry(&model, package_id, package_pads, entity_id, &entity, &entry)?;
        pad_map.insert(
            entry.pad.to_string(),
            serde_json::json!({"gate": entry.gate, "pin": entry.pin}),
        );
    }
    commit_pool_library_operations(
        root,
        format!("set native pool part {part_id} pad map"),
        vec![Operation::SetPoolLibraryObject {
            object_id: part_id,
            relative_path: relative_path.clone(),
            object_kind: "parts".to_string(),
            previous_object,
            object,
        }],
    )?;
    pool_library_mutation_view(root, action, pool_path, "parts", part_id, &relative_path)
}

fn parse_pad_map_entry(entry: &str) -> Result<PartPadMapEntryInput> {
    let mut fields = entry.split(':');
    let pad = fields
        .next()
        .ok_or_else(|| anyhow::anyhow!("pad-map entry missing pad uuid"))?;
    let gate = fields
        .next()
        .ok_or_else(|| anyhow::anyhow!("pad-map entry missing gate uuid"))?;
    let pin = fields
        .next()
        .ok_or_else(|| anyhow::anyhow!("pad-map entry missing pin uuid"))?;
    if fields.next().is_some() {
        bail!("pad-map entry {entry} must be pad_uuid:gate_uuid:pin_uuid");
    }
    Ok(PartPadMapEntryInput {
        pad: Uuid::parse_str(pad)
            .with_context(|| format!("invalid pad uuid in pad-map entry {entry}"))?,
        gate: Uuid::parse_str(gate)
            .with_context(|| format!("invalid gate uuid in pad-map entry {entry}"))?,
        pin: Uuid::parse_str(pin)
            .with_context(|| format!("invalid pin uuid in pad-map entry {entry}"))?,
    })
}

fn validate_pad_map_entry(
    model: &eda_engine::substrate::DesignModel,
    package_id: Uuid,
    package_pads: &serde_json::Map<String, serde_json::Value>,
    entity_id: Uuid,
    entity: &serde_json::Value,
    entry: &PartPadMapEntryInput,
) -> Result<()> {
    if !package_pads.contains_key(&entry.pad.to_string()) {
        bail!("pool package {package_id} has no pad {}", entry.pad);
    }
    let gate = entity
        .get("gates")
        .and_then(serde_json::Value::as_object)
        .and_then(|gates| gates.get(&entry.gate.to_string()))
        .ok_or_else(|| anyhow::anyhow!("pool entity {entity_id} has no gate {}", entry.gate))?;
    let unit_id = uuid_field(gate, "unit", "entity gate")?;
    let unit = materialized_pool_object(model, unit_id, "units")?;
    let unit_pins = unit
        .get("pins")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("pool unit {unit_id} has no pins map"))?;
    if !unit_pins.contains_key(&entry.pin.to_string()) {
        bail!("pool unit {unit_id} has no pin {}", entry.pin);
    }
    Ok(())
}

fn materialized_pool_object(
    model: &eda_engine::substrate::DesignModel,
    object_id: Uuid,
    object_kind: &str,
) -> Result<serde_json::Value> {
    let object = require_pool_object(model, object_id, object_kind)?;
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

fn require_pool_object<'a>(
    model: &'a eda_engine::substrate::DesignModel,
    object_id: Uuid,
    object_kind: &str,
) -> Result<&'a eda_engine::substrate::DomainObject> {
    model
        .objects
        .get(&object_id)
        .filter(|object| object.domain == "pool" && object.kind == object_kind)
        .ok_or_else(|| anyhow::anyhow!("missing pool {object_kind} {object_id}"))
}

fn uuid_field(value: &serde_json::Value, field: &str, label: &str) -> Result<Uuid> {
    let raw = value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{label} missing {field}"))?;
    Uuid::parse_str(raw).with_context(|| format!("{label} has invalid {field} uuid {raw}"))
}
