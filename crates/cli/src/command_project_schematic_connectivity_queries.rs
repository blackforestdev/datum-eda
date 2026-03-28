use super::*;

pub(crate) fn query_native_project_labels(root: &Path) -> Result<Vec<LabelInfo>> {
    let project = load_native_project(root)?;
    let mut labels = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value
            .get("labels")
            .and_then(serde_json::Value::as_object)
        {
            for value in entries.values() {
                let label: NetLabel = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse label in {}", path.display()))?;
                labels.push(LabelInfo {
                    uuid: label.uuid,
                    sheet: sheet_uuid,
                    kind: label.kind,
                    name: label.name,
                    position: label.position,
                });
            }
        }
    }
    labels.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(labels)
}

pub(crate) fn query_native_project_wires(root: &Path) -> Result<Vec<serde_json::Value>> {
    let project = load_native_project(root)?;
    let mut wires = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value
            .get("wires")
            .and_then(serde_json::Value::as_object)
        {
            for value in entries.values() {
                let wire: SchematicWire = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse wire in {}", path.display()))?;
                wires.push(serde_json::json!({
                    "uuid": wire.uuid,
                    "sheet": sheet_uuid,
                    "from": wire.from,
                    "to": wire.to,
                }));
            }
        }
    }
    wires.sort_by(|a, b| {
        a.get("uuid")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("uuid").and_then(serde_json::Value::as_str))
    });
    Ok(wires)
}

pub(crate) fn query_native_project_junctions(root: &Path) -> Result<Vec<serde_json::Value>> {
    let project = load_native_project(root)?;
    let mut junctions = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value
            .get("junctions")
            .and_then(serde_json::Value::as_object)
        {
            for value in entries.values() {
                let junction: Junction = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse junction in {}", path.display()))?;
                junctions.push(serde_json::json!({
                    "uuid": junction.uuid,
                    "sheet": sheet_uuid,
                    "position": junction.position,
                }));
            }
        }
    }
    junctions.sort_by(|a, b| {
        a.get("uuid")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("uuid").and_then(serde_json::Value::as_str))
    });
    Ok(junctions)
}

pub(crate) fn query_native_project_ports(root: &Path) -> Result<Vec<PortInfo>> {
    let project = load_native_project(root)?;
    let mut ports = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value
            .get("ports")
            .and_then(serde_json::Value::as_object)
        {
            for value in entries.values() {
                let port: HierarchicalPort = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse port in {}", path.display()))?;
                ports.push(PortInfo {
                    uuid: port.uuid,
                    sheet: sheet_uuid,
                    name: port.name,
                    direction: port.direction,
                    position: port.position,
                });
            }
        }
    }
    ports.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(ports)
}

pub(crate) fn query_native_project_buses(root: &Path) -> Result<Vec<BusInfo>> {
    let project = load_native_project(root)?;
    let mut buses = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value
            .get("buses")
            .and_then(serde_json::Value::as_object)
        {
            for value in entries.values() {
                let bus: Bus = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse bus in {}", path.display()))?;
                buses.push(BusInfo {
                    uuid: bus.uuid,
                    sheet: sheet_uuid,
                    name: bus.name,
                    members: bus.members,
                });
            }
        }
    }
    buses.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(buses)
}

pub(crate) fn query_native_project_bus_entries(root: &Path) -> Result<Vec<BusEntryInfo>> {
    let project = load_native_project(root)?;
    let mut entries = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(values) = sheet_value
            .get("bus_entries")
            .and_then(serde_json::Value::as_object)
        {
            for value in values.values() {
                let entry: BusEntry = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse bus entry in {}", path.display()))?;
                entries.push(BusEntryInfo {
                    uuid: entry.uuid,
                    sheet: sheet_uuid,
                    bus: entry.bus,
                    wire: entry.wire,
                    position: entry.position,
                });
            }
        }
    }
    entries.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(entries)
}

pub(crate) fn query_native_project_noconnects(root: &Path) -> Result<Vec<NoConnectInfo>> {
    let project = load_native_project(root)?;
    let mut noconnects = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(values) = sheet_value
            .get("noconnects")
            .and_then(serde_json::Value::as_object)
        {
            for value in values.values() {
                let marker: NoConnectMarker = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse no-connect in {}", path.display()))?;
                noconnects.push(NoConnectInfo {
                    uuid: marker.uuid,
                    sheet: sheet_uuid,
                    symbol: marker.symbol,
                    pin: marker.pin,
                    position: marker.position,
                });
            }
        }
    }
    noconnects.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    Ok(noconnects)
}
