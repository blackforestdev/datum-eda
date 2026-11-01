//! Manual Unit and Symbol creation through the existing canonical pool writer.
use super::*;
use crate::pool::{LibraryPinElectricalType, Primitive, Symbol};
use std::collections::HashSet;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PinIntent {
    name: String,
    electrical_type: LibraryPinElectricalType,
}

pub(super) fn build(
    context: &NativeWriteContext<'_>,
    provenance: WriteProvenance,
    request: LibraryDefinitionRequest,
    create: bool,
) -> Result<PreparedWrite, EngineError> {
    if !create {
        return Err(refusal("Saved Unit and Symbol editing is unavailable"));
    }
    let allowed: &[&str] = match request.kind.as_str() {
        "units" => &["name", "pins"],
        "symbols" => &["name", "unit", "drawings", "pin_anchors"],
        _ => return Err(refusal("Expected a Unit or Symbol creation request")),
    };
    if request
        .fields
        .keys()
        .any(|key| !allowed.contains(&key.as_str()))
    {
        return Err(refusal(
            "Creation contains an unsupported field; approval is not editable here",
        ));
    }
    let name = request
        .fields
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| refusal("Library name must be non-empty"))?;
    let object = if request.kind == "units" {
        unit_payload(&request, name)?
    } else {
        symbol_payload(context, &request, name)?
    };
    library::build_pool_library_write(
        context.model,
        provenance,
        Some("pool"),
        vec![PoolLibraryOperationSpec::Create {
            target: PoolLibraryObjectTarget::new("pool", &request.kind, request.object_id),
            object,
        }],
    )
}

fn unit_payload(request: &LibraryDefinitionRequest, name: &str) -> Result<Value, EngineError> {
    let rows = request
        .fields
        .get("pins")
        .and_then(Value::as_object)
        .ok_or_else(|| refusal("Unit pins must be a map of named logical pins"))?;
    let mut identities = HashSet::new();
    let mut pins = Map::new();
    for (key, row) in rows {
        let id = Uuid::parse_str(key).map_err(|_| refusal("Unit pin identity is invalid"))?;
        if !identities.insert(id) {
            return Err(refusal("Duplicate Unit pin identity"));
        }
        let intent: PinIntent = serde_json::from_value(row.clone())?;
        let pin_name = intent.name.trim();
        if pin_name.is_empty() {
            return Err(refusal("Unit pin name must be non-empty"));
        }
        pins.insert(
            id.to_string(),
            json!({
                "uuid":id,"name":pin_name,"direction":intent.electrical_type,
                "electrical_type":intent.electrical_type,"swap_group":0,"alternates":[]
            }),
        );
    }
    let mut object = library::pool_unit_payload(request.object_id, name, "");
    object["pins"] = Value::Object(pins);
    Ok(object)
}

fn symbol_payload(
    context: &NativeWriteContext<'_>,
    request: &LibraryDefinitionRequest,
    name: &str,
) -> Result<Value, EngineError> {
    let unit = request
        .fields
        .get("unit")
        .and_then(Value::as_str)
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| refusal("Choose an existing named Unit"))?;
    let unit_payload =
        super::super::schematic_library::materialized_pool_object(context.model, unit, "units")?;
    let pins = unit_payload
        .get("pins")
        .and_then(Value::as_object)
        .ok_or_else(|| refusal("Selected Unit has no valid logical pin map"))?;
    let mut object = library::pool_symbol_payload(request.object_id, unit, name);
    for field in ["drawings", "pin_anchors"] {
        object[field] = request
            .fields
            .get(field)
            .cloned()
            .ok_or_else(|| refusal(format!("Symbol creation requires {field}")))?;
    }
    let symbol: Symbol = serde_json::from_value(object.clone())?;
    match symbol.drawings.as_slice() {
        [Primitive::Rect { min, max, width }] if min.x < max.x && min.y < max.y && *width > 0 => {}
        _ => {
            return Err(refusal(
                "Symbol body requires one rectangle with ordered corners and positive line width",
            ));
        }
    }
    let mut anchored = HashSet::new();
    for anchor in &symbol.pin_anchors {
        if !pins.contains_key(&anchor.pin.to_string()) {
            return Err(refusal(
                "Symbol anchor must reference a pin in the selected Unit",
            ));
        }
        if !anchored.insert(anchor.pin) {
            return Err(refusal("A Unit pin may have only one Symbol anchor"));
        }
        if anchor.style.length_nm.is_some_and(|length| length < 0) {
            return Err(refusal("Pin stub length must not be negative"));
        }
    }
    Ok(object)
}
