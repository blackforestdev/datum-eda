use super::*;

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
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("labels")
            .and_then(serde_json::Value::as_object)
            .and_then(|labels| labels.get(&label_uuid.to_string()))
        {
            let label: NetLabel = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse label in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, label));
        }
    }

    bail!("label not found in native project: {label_uuid}");
}

pub(super) fn load_native_symbol_mutation_target(
    project: &LoadedNativeProject,
    symbol_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, PlacedSymbol)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("symbols")
            .and_then(serde_json::Value::as_object)
            .and_then(|symbols| symbols.get(&symbol_uuid.to_string()))
        {
            let symbol: PlacedSymbol = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse symbol in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, symbol));
        }
    }

    bail!("symbol not found in native project: {symbol_uuid}");
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
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
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
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("texts")
            .and_then(serde_json::Value::as_object)
            .and_then(|texts| texts.get(&text_uuid.to_string()))
        {
            let text: SchematicText = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse text in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, text));
        }
    }

    bail!("text not found in native project: {text_uuid}");
}

pub(super) fn load_native_sheet_for_insert(
    project: &LoadedNativeProject,
    sheet_uuid: Uuid,
) -> Result<(std::path::PathBuf, serde_json::Value)> {
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    Ok((sheet_path, sheet_value))
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
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("drawings")
            .and_then(serde_json::Value::as_object)
            .and_then(|drawings| drawings.get(&drawing_uuid.to_string()))
        {
            let drawing: SchematicPrimitive = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse drawing in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, drawing));
        }
    }

    bail!("drawing not found in native project: {drawing_uuid}");
}

pub(super) fn write_symbol_into_sheet(
    sheet_value: &mut serde_json::Value,
    symbol: &PlacedSymbol,
) -> Result<()> {
    let symbols = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("symbols"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet symbols object missing during symbol mutation"))?;
    symbols.insert(
        symbol.uuid.to_string(),
        serde_json::to_value(symbol).expect("native symbol serialization must succeed"),
    );
    Ok(())
}

pub(super) fn write_text_into_sheet(
    sheet_value: &mut serde_json::Value,
    text: &SchematicText,
) -> Result<()> {
    let texts = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("texts"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet texts object missing during text mutation"))?;
    texts.insert(
        text.uuid.to_string(),
        serde_json::to_value(text).expect("native text serialization must succeed"),
    );
    Ok(())
}

pub(super) fn write_drawing_into_sheet(
    sheet_value: &mut serde_json::Value,
    drawing: &SchematicPrimitive,
) -> Result<()> {
    let drawings = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("drawings"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet drawings object missing during drawing mutation"))?;
    let uuid = drawing_uuid(drawing);
    drawings.insert(
        uuid.to_string(),
        serde_json::to_value(drawing).expect("native drawing serialization must succeed"),
    );
    Ok(())
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

pub(super) fn write_label_into_sheet(
    sheet_value: &mut serde_json::Value,
    label: &NetLabel,
) -> Result<()> {
    let labels = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("labels"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet labels object missing during label mutation"))?;
    labels.insert(
        label.uuid.to_string(),
        serde_json::to_value(label).expect("native label serialization must succeed"),
    );
    Ok(())
}

pub(super) fn load_native_wire_mutation_target(
    project: &LoadedNativeProject,
    wire_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, SchematicWire)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("wires")
            .and_then(serde_json::Value::as_object)
            .and_then(|wires| wires.get(&wire_uuid.to_string()))
        {
            let wire: SchematicWire = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse wire in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, wire));
        }
    }

    bail!("wire not found in native project: {wire_uuid}");
}

pub(super) fn load_native_junction_mutation_target(
    project: &LoadedNativeProject,
    junction_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, Junction)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("junctions")
            .and_then(serde_json::Value::as_object)
            .and_then(|junctions| junctions.get(&junction_uuid.to_string()))
        {
            let junction: Junction = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse junction in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, junction));
        }
    }

    bail!("junction not found in native project: {junction_uuid}");
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
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("ports")
            .and_then(serde_json::Value::as_object)
            .and_then(|ports| ports.get(&port_uuid.to_string()))
        {
            let port: HierarchicalPort = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse port in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, port));
        }
    }

    bail!("port not found in native project: {port_uuid}");
}

pub(super) fn write_port_into_sheet(
    sheet_value: &mut serde_json::Value,
    port: &HierarchicalPort,
) -> Result<()> {
    let ports = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("ports"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet ports object missing during port mutation"))?;
    ports.insert(
        port.uuid.to_string(),
        serde_json::to_value(port).expect("native port serialization must succeed"),
    );
    Ok(())
}

pub(super) fn load_native_bus_mutation_target(
    project: &LoadedNativeProject,
    bus_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, Bus)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("buses")
            .and_then(serde_json::Value::as_object)
            .and_then(|buses| buses.get(&bus_uuid.to_string()))
        {
            let bus: Bus = serde_json::from_value(entry.clone())
                .with_context(|| format!("failed to parse bus in {}", sheet_path.display()))?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, bus));
        }
    }

    bail!("bus not found in native project: {bus_uuid}");
}

pub(super) fn write_bus_into_sheet(sheet_value: &mut serde_json::Value, bus: &Bus) -> Result<()> {
    let buses = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("buses"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet buses object missing during bus mutation"))?;
    buses.insert(
        bus.uuid.to_string(),
        serde_json::to_value(bus).expect("native bus serialization must succeed"),
    );
    Ok(())
}

pub(super) fn load_native_bus_entry_mutation_target(
    project: &LoadedNativeProject,
    bus_entry_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, BusEntry)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("bus_entries")
            .and_then(serde_json::Value::as_object)
            .and_then(|entries| entries.get(&bus_entry_uuid.to_string()))
        {
            let bus_entry: BusEntry = serde_json::from_value(entry.clone()).with_context(|| {
                format!("failed to parse bus entry in {}", sheet_path.display())
            })?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, bus_entry));
        }
    }

    bail!("bus entry not found in native project: {bus_entry_uuid}");
}

pub(super) fn load_native_noconnect_mutation_target(
    project: &LoadedNativeProject,
    noconnect_uuid: Uuid,
) -> Result<(Uuid, std::path::PathBuf, serde_json::Value, NoConnectMarker)> {
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let parsed_sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let sheet_path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&sheet_path)
            .with_context(|| format!("failed to read {}", sheet_path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
        if let Some(entry) = sheet_value
            .get("noconnects")
            .and_then(serde_json::Value::as_object)
            .and_then(|markers| markers.get(&noconnect_uuid.to_string()))
        {
            let marker: NoConnectMarker =
                serde_json::from_value(entry.clone()).with_context(|| {
                    format!("failed to parse no-connect in {}", sheet_path.display())
                })?;
            return Ok((parsed_sheet_uuid, sheet_path, sheet_value, marker));
        }
    }

    bail!("no-connect not found in native project: {noconnect_uuid}");
}
