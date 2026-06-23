use super::*;
use eda_engine::substrate::ProjectResolver;
use serde::de::DeserializeOwned;

pub(super) fn load_native_sheet(path: &Path) -> Result<NativeSheetRoot> {
    let sheet_text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&sheet_text).with_context(|| format!("failed to parse {}", path.display()))
}

pub(super) fn native_sheet_into_engine_sheet(native_sheet: NativeSheetRoot) -> Sheet {
    Sheet {
        uuid: native_sheet.uuid,
        name: native_sheet.name,
        frame: native_sheet.frame,
        symbols: native_sheet
            .symbols
            .into_values()
            .map(|symbol| (symbol.uuid, symbol))
            .collect(),
        wires: native_sheet
            .wires
            .into_values()
            .map(|wire| (wire.uuid, wire))
            .collect(),
        junctions: native_sheet
            .junctions
            .into_values()
            .map(|junction| (junction.uuid, junction))
            .collect(),
        labels: native_sheet
            .labels
            .into_values()
            .map(|label| (label.uuid, label))
            .collect(),
        buses: native_sheet
            .buses
            .into_values()
            .map(|bus| (bus.uuid, bus))
            .collect(),
        bus_entries: native_sheet
            .bus_entries
            .into_values()
            .map(|entry| (entry.uuid, entry))
            .collect(),
        ports: native_sheet
            .ports
            .into_values()
            .map(|port| (port.uuid, port))
            .collect(),
        noconnects: native_sheet
            .noconnects
            .into_values()
            .map(|marker| (marker.uuid, marker))
            .collect(),
        texts: native_sheet
            .texts
            .into_values()
            .map(|text| (text.uuid, text))
            .collect(),
        drawings: native_sheet
            .drawings
            .into_values()
            .map(|drawing| (drawing_uuid(&drawing), drawing))
            .collect(),
    }
}

pub(super) fn json_object_len(value: &serde_json::Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(serde_json::Value::as_object)
        .map(|items| items.len())
        .unwrap_or(0)
}

pub(super) fn render_label_kind(kind: &LabelKind) -> &'static str {
    match kind {
        LabelKind::Local => "local",
        LabelKind::Global => "global",
        LabelKind::Hierarchical => "hierarchical",
        LabelKind::Power => "power",
    }
}

pub(super) fn render_port_direction(direction: &PortDirection) -> &'static str {
    match direction {
        PortDirection::Input => "input",
        PortDirection::Output => "output",
        PortDirection::Bidirectional => "bidirectional",
        PortDirection::Passive => "passive",
    }
}

pub(super) fn load_native_label_mutation_target(
    project: &LoadedNativeProject,
    label_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, NetLabel)> {
    load_native_materialized_sheet_map_target(project, label_uuid, "labels", "label")
}

pub(super) fn load_native_symbol_mutation_target(
    project: &LoadedNativeProject,
    symbol_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, PlacedSymbol)> {
    load_native_materialized_sheet_map_target(project, symbol_uuid, "symbols", "symbol")
}

pub(super) fn load_native_field_mutation_target(
    project: &LoadedNativeProject,
    field_uuid: Uuid,
) -> Result<(
    Uuid,
    std::path::PathBuf,
    serde_json::Value,
    Uuid,
    PlacedSymbol,
    SymbolField,
)> {
    let model = ProjectResolver::new(&project.root)
        .resolve()
        .with_context(|| {
            format!(
                "failed to resolve native project {}",
                project.root.display()
            )
        })?;
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_value = model
            .materialized_source_shard_value_by_relative_path(&format!("schematic/{relative_path}"))
            .with_context(|| format!("failed to materialize {}", sheet_path.display()))?;
        if let Some(entries) = sheet_value
            .get("symbols")
            .and_then(serde_json::Value::as_object)
        {
            for entry in entries.values() {
                let symbol: PlacedSymbol =
                    serde_json::from_value(entry.clone()).with_context(|| {
                        format!("failed to parse symbol in {}", sheet_path.display())
                    })?;
                if let Some(field) = symbol.fields.iter().find(|field| field.uuid == field_uuid) {
                    return Ok((
                        parsed_sheet_uuid,
                        sheet_path,
                        sheet_value,
                        symbol.uuid,
                        symbol.clone(),
                        field.clone(),
                    ));
                }
            }
        }
    }

    bail!("symbol field not found in native project: {field_uuid}");
}

pub(super) fn load_native_text_mutation_target(
    project: &LoadedNativeProject,
    text_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, SchematicText)> {
    load_native_materialized_sheet_map_target(project, text_uuid, "texts", "text")
}

pub(super) fn load_native_drawing_mutation_target(
    project: &LoadedNativeProject,
    drawing_uuid: Uuid,
) -> Result<(
    Uuid,
    std::path::PathBuf,
    serde_json::Value,
    SchematicPrimitive,
)> {
    load_native_materialized_sheet_map_target(project, drawing_uuid, "drawings", "drawing")
}

pub(super) fn drawing_uuid(drawing: &SchematicPrimitive) -> Uuid {
    match drawing {
        SchematicPrimitive::Line { uuid, .. }
        | SchematicPrimitive::Rect { uuid, .. }
        | SchematicPrimitive::Circle { uuid, .. }
        | SchematicPrimitive::Arc { uuid, .. } => *uuid,
    }
}

pub(super) fn render_drawing_query_view(
    sheet_uuid: Uuid,
    drawing: SchematicPrimitive,
) -> Option<serde_json::Value> {
    match drawing {
        SchematicPrimitive::Line { uuid, from, to } => Some(serde_json::json!({
            "uuid": uuid,
            "sheet": sheet_uuid,
            "kind": "line",
            "from": from,
            "to": to,
        })),
        SchematicPrimitive::Rect { uuid, min, max } => Some(serde_json::json!({
            "uuid": uuid,
            "sheet": sheet_uuid,
            "kind": "rect",
            "min": min,
            "max": max,
        })),
        SchematicPrimitive::Circle {
            uuid,
            center,
            radius,
        } => Some(serde_json::json!({
            "uuid": uuid,
            "sheet": sheet_uuid,
            "kind": "circle",
            "center": center,
            "radius": radius,
        })),
        SchematicPrimitive::Arc { uuid, arc } => Some(serde_json::json!({
            "uuid": uuid,
            "sheet": sheet_uuid,
            "kind": "arc",
            "arc": arc,
        })),
    }
}

pub(super) fn load_native_wire_mutation_target(
    project: &LoadedNativeProject,
    wire_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, SchematicWire)> {
    load_native_materialized_sheet_map_target(project, wire_uuid, "wires", "wire")
}

pub(super) fn load_native_junction_mutation_target(
    project: &LoadedNativeProject,
    junction_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, Junction)> {
    load_native_materialized_sheet_map_target(project, junction_uuid, "junctions", "junction")
}

pub(super) fn load_native_port_mutation_target(
    project: &LoadedNativeProject,
    port_uuid: Uuid,
) -> Result<(
    Uuid,
    std::path::PathBuf,
    serde_json::Value,
    HierarchicalPort,
)> {
    load_native_materialized_sheet_map_target(project, port_uuid, "ports", "port")
}

pub(super) fn load_native_bus_mutation_target(
    project: &LoadedNativeProject,
    bus_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, Bus)> {
    load_native_materialized_sheet_map_target(project, bus_uuid, "buses", "bus")
}

pub(super) fn load_native_bus_entry_mutation_target(
    project: &LoadedNativeProject,
    bus_entry_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, BusEntry)> {
    load_native_materialized_sheet_map_target(project, bus_entry_uuid, "bus_entries", "bus entry")
}

pub(super) fn load_native_noconnect_mutation_target(
    project: &LoadedNativeProject,
    noconnect_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, NoConnectMarker)> {
    load_native_materialized_sheet_map_target(project, noconnect_uuid, "noconnects", "no-connect")
}

fn load_native_materialized_sheet_map_target<T>(
    project: &LoadedNativeProject,
    object_uuid: Uuid,
    map_name: &str,
    object_name: &str,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, T)>
where
    T: DeserializeOwned,
{
    let model = ProjectResolver::new(&project.root)
        .resolve()
        .with_context(|| {
            format!(
                "failed to resolve native project {}",
                project.root.display()
            )
        })?;
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_value = model
            .materialized_source_shard_value_by_relative_path(&format!("schematic/{relative_path}"))
            .with_context(|| format!("failed to materialize {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get(map_name)
            .and_then(serde_json::Value::as_object)
            .and_then(|values| values.get(&object_uuid.to_string()))
        {
            let item: T = serde_json::from_value(entry.clone()).with_context(|| {
                format!("failed to parse {object_name} in {}", sheet_path.display())
            })?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, item));
        }
    }
    bail!("{object_name} not found in native project: {object_uuid}");
}
