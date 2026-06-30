use super::*;
use eda_engine::schematic::PinElectricalType;
use eda_engine::substrate::ProjectResolver;

#[derive(Debug, Clone)]
pub(crate) struct PoolSymbolComponentBinding {
    pub(crate) symbol_id: Uuid,
    pub(crate) entity_id: Uuid,
    pub(crate) gate_id: Uuid,
    pub(crate) part_id: Option<Uuid>,
}

pub(crate) fn materialize_pool_symbol_pins(
    root: &Path,
    lib_id: Option<&str>,
) -> Result<Vec<SymbolPin>> {
    let Some(lib_id) = lib_id else {
        return Ok(Vec::new());
    };
    let Ok(symbol_id) = Uuid::parse_str(lib_id) else {
        return Ok(Vec::new());
    };
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let symbol = materialized_pool_object(&model, symbol_id, "symbols")?;
    let unit_id = uuid_field(&symbol, "unit", "pool symbol")?;
    let unit = materialized_pool_object(&model, unit_id, "units")?;
    let unit_pins = unit
        .get("pins")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("pool unit {unit_id} has no pins map"))?;
    let anchors = symbol
        .get("pin_anchors")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut pins = Vec::new();
    for anchor in anchors {
        let pin_id = uuid_field(anchor, "pin", "pool symbol pin anchor")?;
        let unit_pin = unit_pins
            .get(&pin_id.to_string())
            .ok_or_else(|| anyhow::anyhow!("pool unit {unit_id} has no pin {pin_id}"))?;
        let name = unit_pin
            .get("name")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("pool unit pin {pin_id} missing name"))?
            .to_string();
        let electrical_type = unit_pin
            .get("electrical_type")
            .or_else(|| unit_pin.get("direction"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Passive");
        pins.push(SymbolPin {
            uuid: pin_id,
            number: name.clone(),
            name,
            electrical_type: pool_pin_electrical_type_to_schematic(electrical_type),
            position: point_field(anchor, "position", "pool symbol pin anchor")?,
        });
    }
    Ok(pins)
}

pub(crate) fn resolve_pool_symbol_component_binding(
    root: &Path,
    lib_id: Option<&str>,
) -> Result<Option<PoolSymbolComponentBinding>> {
    let Some(lib_id) = lib_id else {
        return Ok(None);
    };
    let Ok(symbol_id) = Uuid::parse_str(lib_id) else {
        return Ok(None);
    };
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let symbol = materialized_pool_object(&model, symbol_id, "symbols")?;
    let unit_id = uuid_field(&symbol, "unit", "pool symbol")?;

    let mut matches = Vec::new();
    for object in model
        .objects
        .values()
        .filter(|object| object.domain == "pool" && object.kind == "entities")
    {
        let entity = materialized_pool_object(&model, object.object_id, "entities")?;
        let Some(gates) = entity.get("gates").and_then(serde_json::Value::as_object) else {
            continue;
        };
        for (gate_key, gate) in gates {
            if gate
                .get("symbol")
                .and_then(serde_json::Value::as_str)
                .and_then(|raw| Uuid::parse_str(raw).ok())
                != Some(symbol_id)
            {
                continue;
            }
            if gate
                .get("unit")
                .and_then(serde_json::Value::as_str)
                .and_then(|raw| Uuid::parse_str(raw).ok())
                != Some(unit_id)
            {
                continue;
            }
            let gate_id = Uuid::parse_str(gate_key).with_context(|| {
                format!(
                    "pool entity {} has invalid gate key {gate_key}",
                    object.object_id
                )
            })?;
            matches.push((object.object_id, gate_id));
        }
    }

    let [(entity_id, gate_id)] = matches.as_slice() else {
        return Ok(None);
    };
    let part_id = unique_part_for_entity(&model, *entity_id)?;
    Ok(Some(PoolSymbolComponentBinding {
        symbol_id,
        entity_id: *entity_id,
        gate_id: *gate_id,
        part_id,
    }))
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

fn unique_part_for_entity(
    model: &eda_engine::substrate::DesignModel,
    entity_id: Uuid,
) -> Result<Option<Uuid>> {
    let mut matches = Vec::new();
    for object in model
        .objects
        .values()
        .filter(|object| object.domain == "pool" && object.kind == "parts")
    {
        let part = materialized_pool_object(model, object.object_id, "parts")?;
        if part
            .get("entity")
            .and_then(serde_json::Value::as_str)
            .and_then(|raw| Uuid::parse_str(raw).ok())
            == Some(entity_id)
        {
            matches.push(object.object_id);
        }
    }
    Ok(match matches.as_slice() {
        [] => None,
        [part_id] => Some(*part_id),
        _ => None,
    })
}

fn uuid_field(value: &serde_json::Value, field: &str, label: &str) -> Result<Uuid> {
    let raw = value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{label} missing {field}"))?;
    Uuid::parse_str(raw).with_context(|| format!("{label} has invalid {field} uuid {raw}"))
}

fn point_field(value: &serde_json::Value, field: &str, label: &str) -> Result<Point> {
    let point = value
        .get(field)
        .ok_or_else(|| anyhow::anyhow!("{label} missing {field}"))?;
    let x = point
        .get("x")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| anyhow::anyhow!("{label} {field} missing x"))?;
    let y = point
        .get("y")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| anyhow::anyhow!("{label} {field} missing y"))?;
    Ok(Point { x, y })
}

fn pool_pin_electrical_type_to_schematic(electrical_type: &str) -> PinElectricalType {
    match electrical_type {
        "Input" => PinElectricalType::Input,
        "Output" | "OpenCollector" | "OpenEmitter" | "TriState" => PinElectricalType::Output,
        "Bidirectional" => PinElectricalType::Bidirectional,
        "PowerIn" => PinElectricalType::PowerIn,
        "PowerOut" => PinElectricalType::PowerOut,
        _ => PinElectricalType::Passive,
    }
}
