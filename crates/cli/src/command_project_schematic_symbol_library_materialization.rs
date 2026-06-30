use super::*;
use eda_engine::pool::Symbol;
use eda_engine::schematic::PinElectricalType;
use eda_engine::substrate::ProjectResolver;

#[derive(Debug, Clone)]
pub(crate) struct PoolSymbolComponentBinding {
    pub(crate) symbol_id: Uuid,
    pub(crate) symbol_revision: u64,
    pub(crate) unit_id: Uuid,
    pub(crate) unit_revision: u64,
    pub(crate) entity_id: Uuid,
    pub(crate) entity_revision: u64,
    pub(crate) gate_id: Uuid,
    pub(crate) part: Option<PoolSymbolPartBinding>,
}

#[derive(Debug, Clone)]
pub(crate) struct PoolSymbolPartBinding {
    pub(crate) part_id: Uuid,
    pub(crate) part_revision: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct PoolSymbolBindingResolution {
    pub(crate) binding: Option<PoolSymbolComponentBinding>,
    pub(crate) status: &'static str,
    pub(crate) diagnostics: Vec<String>,
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
    let symbol_value = materialized_pool_object(&model, symbol_id, "symbols")?;
    let symbol: Symbol = serde_json::from_value(symbol_value.clone())
        .with_context(|| format!("failed to parse pool symbol {symbol_id}"))?;
    let unit_id = symbol.unit;
    let unit = materialized_pool_object(&model, unit_id, "units")?;
    let unit_pins = unit
        .get("pins")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("pool unit {unit_id} has no pins map"))?;
    let mut pins = Vec::new();
    for anchor in &symbol.pin_anchors {
        let pin_id = anchor.pin;
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
            position: anchor.position,
        });
    }
    Ok(pins)
}

pub(crate) fn resolve_pool_symbol_component_binding(
    root: &Path,
    lib_id: Option<&str>,
) -> Result<PoolSymbolBindingResolution> {
    let Some(lib_id) = lib_id else {
        return Ok(PoolSymbolBindingResolution {
            binding: None,
            status: "no_lib_id",
            diagnostics: Vec::new(),
        });
    };
    let Ok(symbol_id) = Uuid::parse_str(lib_id) else {
        return Ok(PoolSymbolBindingResolution {
            binding: None,
            status: "compatibility_lib_id",
            diagnostics: vec![format!(
                "lib_id {lib_id} is not a pool symbol UUID; placed as compatibility identifier"
            )],
        });
    };
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let symbol = materialized_pool_object(&model, symbol_id, "symbols")?;
    let symbol_revision = object_revision(&model, symbol_id, "pool symbol")?;
    let unit_id = uuid_field(&symbol, "unit", "pool symbol")?;
    let _unit = materialized_pool_object(&model, unit_id, "units")?;
    let unit_revision = object_revision(&model, unit_id, "pool unit")?;

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
    matches.sort();

    let [(entity_id, gate_id)] = matches.as_slice() else {
        let diagnostics = match matches.as_slice() {
            [] => vec![format!(
                "pool symbol {symbol_id} did not resolve to any entity gate"
            )],
            _ => vec![format!(
                "pool symbol {symbol_id} resolves to multiple entity gates: {}",
                matches
                    .iter()
                    .map(|(entity_id, gate_id)| format!("{entity_id}/{gate_id}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )],
        };
        return Ok(PoolSymbolBindingResolution {
            binding: None,
            status: "ambiguous_entity_gate",
            diagnostics,
        });
    };
    let entity_revision = object_revision(&model, *entity_id, "pool entity")?;
    let part = unique_part_for_entity(&model, *entity_id)?;
    let (part, part_diagnostics, status) = match part {
        UniquePartResolution::Unique(part) => (
            Some(part),
            Vec::new(),
            "bound_with_part",
        ),
        UniquePartResolution::None => (
            None,
            vec![format!(
                "pool entity {entity_id} has no pool part; placed symbol is bound to entity/gate only"
            )],
            "bound_without_part",
        ),
        UniquePartResolution::Ambiguous(part_ids) => (
            None,
            vec![format!(
                "pool entity {entity_id} has multiple pool parts and cannot assign a unique part: {}",
                part_ids
                    .iter()
                    .map(Uuid::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )],
            "ambiguous_part",
        ),
    };
    Ok(PoolSymbolBindingResolution {
        binding: Some(PoolSymbolComponentBinding {
            symbol_id,
            symbol_revision,
            unit_id,
            unit_revision,
            entity_id: *entity_id,
            entity_revision,
            gate_id: *gate_id,
            part,
        }),
        status,
        diagnostics: part_diagnostics,
    })
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

enum UniquePartResolution {
    None,
    Unique(PoolSymbolPartBinding),
    Ambiguous(Vec<Uuid>),
}

fn unique_part_for_entity(
    model: &eda_engine::substrate::DesignModel,
    entity_id: Uuid,
) -> Result<UniquePartResolution> {
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
    matches.sort();
    Ok(match matches.as_slice() {
        [] => UniquePartResolution::None,
        [part_id] => UniquePartResolution::Unique(PoolSymbolPartBinding {
            part_id: *part_id,
            part_revision: object_revision(model, *part_id, "pool part")?,
        }),
        _ => UniquePartResolution::Ambiguous(matches),
    })
}

fn object_revision(
    model: &eda_engine::substrate::DesignModel,
    object_id: Uuid,
    label: &str,
) -> Result<u64> {
    model
        .objects
        .get(&object_id)
        .map(|object| object.object_revision.0)
        .ok_or_else(|| anyhow::anyhow!("{label} {object_id} was not found in resolved model"))
}

fn uuid_field(value: &serde_json::Value, field: &str, label: &str) -> Result<Uuid> {
    let raw = value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{label} missing {field}"))?;
    Uuid::parse_str(raw).with_context(|| format!("{label} has invalid {field} uuid {raw}"))
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
