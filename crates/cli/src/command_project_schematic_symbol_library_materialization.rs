use super::*;
use eda_engine::schematic::PinElectricalType;
use eda_engine::substrate::ProjectResolver;

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
        let direction = unit_pin
            .get("direction")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Passive");
        pins.push(SymbolPin {
            uuid: pin_id,
            number: name.clone(),
            name,
            electrical_type: pool_pin_direction_to_schematic(direction),
            position: point_field(anchor, "position", "pool symbol pin anchor")?,
        });
    }
    Ok(pins)
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

fn pool_pin_direction_to_schematic(direction: &str) -> PinElectricalType {
    match direction {
        "Input" => PinElectricalType::Input,
        "Output" | "OpenCollector" | "OpenEmitter" | "TriState" => PinElectricalType::Output,
        "Bidirectional" => PinElectricalType::Bidirectional,
        "PowerIn" => PinElectricalType::PowerIn,
        "PowerOut" => PinElectricalType::PowerOut,
        _ => PinElectricalType::Passive,
    }
}
