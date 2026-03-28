use super::*;

pub(crate) fn place_native_project_label(
    root: &Path,
    sheet_uuid: Uuid,
    name: String,
    kind: LabelKind,
    position: Point,
) -> Result<NativeProjectLabelMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let labels = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("labels"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!("sheet labels object missing in {}", sheet_path.display())
        })?;

    let label_uuid = Uuid::new_v4();
    labels.insert(
        label_uuid.to_string(),
        serde_json::to_value(NetLabel {
            uuid: label_uuid,
            kind: kind.clone(),
            name: name.clone(),
            position,
        })
        .expect("native label serialization must succeed"),
    );

    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectLabelMutationReportView {
        action: "place_label".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        label_uuid: label_uuid.to_string(),
        name,
        kind: render_label_kind(&kind).to_string(),
        x_nm: position.x,
        y_nm: position.y,
    })
}

pub(crate) fn rename_native_project_label(
    root: &Path,
    label_uuid: Uuid,
    name: String,
) -> Result<NativeProjectLabelMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut label) =
        load_native_label_mutation_target(&project, label_uuid)?;
    label.name = name.clone();
    write_label_into_sheet(&mut sheet_value, &label)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectLabelMutationReportView {
        action: "rename_label".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        label_uuid: label.uuid.to_string(),
        name,
        kind: render_label_kind(&label.kind).to_string(),
        x_nm: label.position.x,
        y_nm: label.position.y,
    })
}

pub(crate) fn delete_native_project_label(
    root: &Path,
    label_uuid: Uuid,
) -> Result<NativeProjectLabelMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, label) =
        load_native_label_mutation_target(&project, label_uuid)?;
    let labels = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("labels"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!("sheet labels object missing in {}", sheet_path.display())
        })?;
    labels.remove(&label_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectLabelMutationReportView {
        action: "delete_label".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        label_uuid: label.uuid.to_string(),
        name: label.name,
        kind: render_label_kind(&label.kind).to_string(),
        x_nm: label.position.x,
        y_nm: label.position.y,
    })
}

pub(crate) fn draw_native_project_wire(
    root: &Path,
    sheet_uuid: Uuid,
    from: Point,
    to: Point,
) -> Result<NativeProjectWireMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let wires = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("wires"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet wires object missing in {}", sheet_path.display()))?;

    let wire_uuid = Uuid::new_v4();
    wires.insert(
        wire_uuid.to_string(),
        serde_json::to_value(SchematicWire {
            uuid: wire_uuid,
            from,
            to,
        })
        .expect("native wire serialization must succeed"),
    );

    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectWireMutationReportView {
        action: "draw_wire".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        wire_uuid: wire_uuid.to_string(),
        from_x_nm: from.x,
        from_y_nm: from.y,
        to_x_nm: to.x,
        to_y_nm: to.y,
    })
}

pub(crate) fn delete_native_project_wire(
    root: &Path,
    wire_uuid: Uuid,
) -> Result<NativeProjectWireMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, wire) =
        load_native_wire_mutation_target(&project, wire_uuid)?;
    let wires = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("wires"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet wires object missing in {}", sheet_path.display()))?;
    wires.remove(&wire_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectWireMutationReportView {
        action: "delete_wire".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        wire_uuid: wire.uuid.to_string(),
        from_x_nm: wire.from.x,
        from_y_nm: wire.from.y,
        to_x_nm: wire.to.x,
        to_y_nm: wire.to.y,
    })
}

pub(crate) fn place_native_project_junction(
    root: &Path,
    sheet_uuid: Uuid,
    position: Point,
) -> Result<NativeProjectJunctionMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let junctions = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("junctions"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!("sheet junctions object missing in {}", sheet_path.display())
        })?;

    let junction_uuid = Uuid::new_v4();
    junctions.insert(
        junction_uuid.to_string(),
        serde_json::to_value(Junction {
            uuid: junction_uuid,
            position,
        })
        .expect("native junction serialization must succeed"),
    );

    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectJunctionMutationReportView {
        action: "place_junction".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        junction_uuid: junction_uuid.to_string(),
        x_nm: position.x,
        y_nm: position.y,
    })
}

pub(crate) fn delete_native_project_junction(
    root: &Path,
    junction_uuid: Uuid,
) -> Result<NativeProjectJunctionMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, junction) =
        load_native_junction_mutation_target(&project, junction_uuid)?;
    let junctions = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("junctions"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!("sheet junctions object missing in {}", sheet_path.display())
        })?;
    junctions.remove(&junction_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectJunctionMutationReportView {
        action: "delete_junction".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        junction_uuid: junction.uuid.to_string(),
        x_nm: junction.position.x,
        y_nm: junction.position.y,
    })
}

pub(crate) fn place_native_project_port(
    root: &Path,
    sheet_uuid: Uuid,
    name: String,
    direction: PortDirection,
    position: Point,
) -> Result<NativeProjectPortMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let ports = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("ports"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet ports object missing in {}", sheet_path.display()))?;

    let port_uuid = Uuid::new_v4();
    ports.insert(
        port_uuid.to_string(),
        serde_json::to_value(HierarchicalPort {
            uuid: port_uuid,
            name: name.clone(),
            direction: direction.clone(),
            position,
        })
        .expect("native port serialization must succeed"),
    );

    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectPortMutationReportView {
        action: "place_port".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        port_uuid: port_uuid.to_string(),
        name,
        direction: render_port_direction(&direction).to_string(),
        x_nm: position.x,
        y_nm: position.y,
    })
}

pub(crate) fn edit_native_project_port(
    root: &Path,
    port_uuid: Uuid,
    name: Option<String>,
    direction: Option<PortDirection>,
    x_nm: Option<i64>,
    y_nm: Option<i64>,
) -> Result<NativeProjectPortMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut port) =
        load_native_port_mutation_target(&project, port_uuid)?;
    if let Some(name) = name {
        port.name = name;
    }
    if let Some(direction) = direction {
        port.direction = direction;
    }
    if x_nm.is_some() || y_nm.is_some() {
        port.position = Point {
            x: x_nm.unwrap_or(port.position.x),
            y: y_nm.unwrap_or(port.position.y),
        };
    }
    write_port_into_sheet(&mut sheet_value, &port)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectPortMutationReportView {
        action: "edit_port".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        port_uuid: port.uuid.to_string(),
        name: port.name,
        direction: render_port_direction(&port.direction).to_string(),
        x_nm: port.position.x,
        y_nm: port.position.y,
    })
}

pub(crate) fn delete_native_project_port(
    root: &Path,
    port_uuid: Uuid,
) -> Result<NativeProjectPortMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, port) =
        load_native_port_mutation_target(&project, port_uuid)?;
    let ports = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("ports"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet ports object missing in {}", sheet_path.display()))?;
    ports.remove(&port_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectPortMutationReportView {
        action: "delete_port".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        port_uuid: port.uuid.to_string(),
        name: port.name,
        direction: render_port_direction(&port.direction).to_string(),
        x_nm: port.position.x,
        y_nm: port.position.y,
    })
}

pub(crate) fn create_native_project_bus(
    root: &Path,
    sheet_uuid: Uuid,
    name: String,
    members: Vec<String>,
) -> Result<NativeProjectBusMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let buses = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("buses"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| anyhow::anyhow!("sheet buses object missing in {}", sheet_path.display()))?;

    let bus_uuid = Uuid::new_v4();
    buses.insert(
        bus_uuid.to_string(),
        serde_json::to_value(Bus {
            uuid: bus_uuid,
            name: name.clone(),
            members: members.clone(),
        })
        .expect("native bus serialization must succeed"),
    );
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectBusMutationReportView {
        action: "create_bus".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        bus_uuid: bus_uuid.to_string(),
        name,
        members,
    })
}

pub(crate) fn edit_native_project_bus_members(
    root: &Path,
    bus_uuid: Uuid,
    members: Vec<String>,
) -> Result<NativeProjectBusMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, mut bus) =
        load_native_bus_mutation_target(&project, bus_uuid)?;
    bus.members = members.clone();
    write_bus_into_sheet(&mut sheet_value, &bus)?;
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectBusMutationReportView {
        action: "edit_bus_members".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        bus_uuid: bus.uuid.to_string(),
        name: bus.name,
        members,
    })
}

pub(crate) fn place_native_project_bus_entry(
    root: &Path,
    sheet_uuid: Uuid,
    bus_uuid: Uuid,
    wire_uuid: Option<Uuid>,
    position: Point,
) -> Result<NativeProjectBusEntryMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let bus_entries = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("bus_entries"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "sheet bus_entries object missing in {}",
                sheet_path.display()
            )
        })?;

    let bus_entry_uuid = Uuid::new_v4();
    bus_entries.insert(
        bus_entry_uuid.to_string(),
        serde_json::to_value(BusEntry {
            uuid: bus_entry_uuid,
            bus: bus_uuid,
            wire: wire_uuid,
            position,
        })
        .expect("native bus entry serialization must succeed"),
    );
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectBusEntryMutationReportView {
        action: "place_bus_entry".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        bus_entry_uuid: bus_entry_uuid.to_string(),
        bus_uuid: bus_uuid.to_string(),
        wire_uuid: wire_uuid.map(|uuid| uuid.to_string()),
        x_nm: position.x,
        y_nm: position.y,
    })
}

pub(crate) fn delete_native_project_bus_entry(
    root: &Path,
    bus_entry_uuid: Uuid,
) -> Result<NativeProjectBusEntryMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, bus_entry) =
        load_native_bus_entry_mutation_target(&project, bus_entry_uuid)?;
    let bus_entries = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("bus_entries"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "sheet bus_entries object missing in {}",
                sheet_path.display()
            )
        })?;
    bus_entries.remove(&bus_entry_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectBusEntryMutationReportView {
        action: "delete_bus_entry".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        bus_entry_uuid: bus_entry.uuid.to_string(),
        bus_uuid: bus_entry.bus.to_string(),
        wire_uuid: bus_entry.wire.map(|uuid| uuid.to_string()),
        x_nm: bus_entry.position.x,
        y_nm: bus_entry.position.y,
    })
}

pub(crate) fn place_native_project_noconnect(
    root: &Path,
    sheet_uuid: Uuid,
    symbol_uuid: Uuid,
    pin_uuid: Uuid,
    position: Point,
) -> Result<NativeProjectNoConnectMutationReportView> {
    let project = load_native_project(root)?;
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    let sheet_path = project.root.join("schematic").join(relative_path);
    let sheet_text = std::fs::read_to_string(&sheet_path)
        .with_context(|| format!("failed to read {}", sheet_path.display()))?;
    let mut sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
        .with_context(|| format!("failed to parse {}", sheet_path.display()))?;
    let noconnects = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("noconnects"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "sheet noconnects object missing in {}",
                sheet_path.display()
            )
        })?;

    let noconnect_uuid = Uuid::new_v4();
    noconnects.insert(
        noconnect_uuid.to_string(),
        serde_json::to_value(NoConnectMarker {
            uuid: noconnect_uuid,
            symbol: symbol_uuid,
            pin: pin_uuid,
            position,
        })
        .expect("native no-connect serialization must succeed"),
    );
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectNoConnectMutationReportView {
        action: "place_noconnect".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        noconnect_uuid: noconnect_uuid.to_string(),
        symbol_uuid: symbol_uuid.to_string(),
        pin_uuid: pin_uuid.to_string(),
        x_nm: position.x,
        y_nm: position.y,
    })
}

pub(crate) fn delete_native_project_noconnect(
    root: &Path,
    noconnect_uuid: Uuid,
) -> Result<NativeProjectNoConnectMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, mut sheet_value, marker) =
        load_native_noconnect_mutation_target(&project, noconnect_uuid)?;
    let noconnects = sheet_value
        .as_object_mut()
        .and_then(|object| object.get_mut("noconnects"))
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "sheet noconnects object missing in {}",
                sheet_path.display()
            )
        })?;
    noconnects.remove(&noconnect_uuid.to_string());
    write_canonical_json(&sheet_path, &sheet_value)?;

    Ok(NativeProjectNoConnectMutationReportView {
        action: "delete_noconnect".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        noconnect_uuid: marker.uuid.to_string(),
        symbol_uuid: marker.symbol.to_string(),
        pin_uuid: marker.pin.to_string(),
        x_nm: marker.position.x,
        y_nm: marker.position.y,
    })
}
