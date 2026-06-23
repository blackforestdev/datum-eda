use super::*;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};

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
    let label_uuid = Uuid::new_v4();
    let label = NetLabel {
        uuid: label_uuid,
        kind: kind.clone(),
        name: name.clone(),
        position,
    };
    commit_schematic_operation(
        root,
        "place schematic label",
        Operation::CreateSchematicLabel {
            sheet_id: sheet_uuid,
            label_id: label_uuid,
            label: serde_json::to_value(&label).expect("native label serialization must succeed"),
        },
    )?;

    Ok(NativeProjectLabelMutationReportView {
        action: "place_label".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(relative_path)
            .display()
            .to_string(),
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
    let (sheet_uuid, sheet_path, _sheet_value, mut label) =
        load_native_label_mutation_target(&project, label_uuid)?;
    label.name = name.clone();
    commit_schematic_operation(
        root,
        "rename schematic label",
        Operation::SetSchematicLabel {
            sheet_id: sheet_uuid,
            label_id: label_uuid,
            label: serde_json::to_value(&label).expect("native label serialization must succeed"),
        },
    )?;

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
    let (sheet_uuid, sheet_path, _sheet_value, label) =
        load_native_label_mutation_target(&project, label_uuid)?;
    commit_schematic_operation(
        root,
        "delete schematic label",
        Operation::DeleteSchematicLabel {
            sheet_id: sheet_uuid,
            label_id: label_uuid,
            label: serde_json::to_value(&label).expect("native label serialization must succeed"),
        },
    )?;

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
    let wire_uuid = Uuid::new_v4();
    let wire = SchematicWire {
        uuid: wire_uuid,
        from,
        to,
    };
    commit_schematic_operation(
        root,
        "draw schematic wire",
        Operation::CreateSchematicWire {
            sheet_id: sheet_uuid,
            wire_id: wire_uuid,
            wire: serde_json::to_value(&wire).expect("native wire serialization must succeed"),
        },
    )?;

    Ok(NativeProjectWireMutationReportView {
        action: "draw_wire".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(relative_path)
            .display()
            .to_string(),
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
    let (sheet_uuid, sheet_path, _sheet_value, wire) =
        load_native_wire_mutation_target(&project, wire_uuid)?;
    commit_schematic_operation(
        root,
        "delete schematic wire",
        Operation::DeleteSchematicWire {
            sheet_id: sheet_uuid,
            wire_id: wire_uuid,
            wire: serde_json::to_value(&wire).expect("native wire serialization must succeed"),
        },
    )?;

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

pub(super) fn commit_schematic_operation(
    root: &Path,
    reason: &str,
    operation: Operation,
) -> Result<()> {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "datum-eda-cli".to_string(),
                    source: CommitSource::Cli,
                    reason: reason.to_string(),
                },
                operations: vec![operation],
            },
        )
        .map(|_| ())
        .map_err(Into::into)
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
    let junction_uuid = Uuid::new_v4();
    let junction = Junction {
        uuid: junction_uuid,
        position,
    };
    commit_schematic_operation(
        root,
        "place schematic junction",
        Operation::CreateSchematicJunction {
            sheet_id: sheet_uuid,
            junction_id: junction_uuid,
            junction: serde_json::to_value(&junction)
                .expect("native junction serialization must succeed"),
        },
    )?;

    Ok(NativeProjectJunctionMutationReportView {
        action: "place_junction".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(relative_path)
            .display()
            .to_string(),
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
    let (sheet_uuid, sheet_path, _sheet_value, junction) =
        load_native_junction_mutation_target(&project, junction_uuid)?;
    commit_schematic_operation(
        root,
        "delete schematic junction",
        Operation::DeleteSchematicJunction {
            sheet_id: sheet_uuid,
            junction_id: junction_uuid,
            junction: serde_json::to_value(&junction)
                .expect("native junction serialization must succeed"),
        },
    )?;

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
    let port_uuid = Uuid::new_v4();
    let port = HierarchicalPort {
        uuid: port_uuid,
        name: name.clone(),
        direction: direction.clone(),
        position,
    };
    commit_schematic_operation(
        root,
        "place schematic port",
        Operation::CreateSchematicPort {
            sheet_id: sheet_uuid,
            port_id: port_uuid,
            port: serde_json::to_value(&port).expect("native port serialization must succeed"),
        },
    )?;

    Ok(NativeProjectPortMutationReportView {
        action: "place_port".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(relative_path)
            .display()
            .to_string(),
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
    let (sheet_uuid, sheet_path, _sheet_value, mut port) =
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
    commit_schematic_operation(
        root,
        "edit schematic port",
        Operation::SetSchematicPort {
            sheet_id: sheet_uuid,
            port_id: port_uuid,
            port: serde_json::to_value(&port).expect("native port serialization must succeed"),
        },
    )?;

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
    let (sheet_uuid, sheet_path, _sheet_value, port) =
        load_native_port_mutation_target(&project, port_uuid)?;
    commit_schematic_operation(
        root,
        "delete schematic port",
        Operation::DeleteSchematicPort {
            sheet_id: sheet_uuid,
            port_id: port_uuid,
            port: serde_json::to_value(&port).expect("native port serialization must succeed"),
        },
    )?;

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
    let bus_uuid = Uuid::new_v4();
    let bus = Bus {
        uuid: bus_uuid,
        name: name.clone(),
        members: members.clone(),
    };
    commit_schematic_operation(
        root,
        "create schematic bus",
        Operation::CreateSchematicBus {
            sheet_id: sheet_uuid,
            bus_id: bus_uuid,
            bus: serde_json::to_value(&bus).expect("native bus serialization must succeed"),
        },
    )?;

    Ok(NativeProjectBusMutationReportView {
        action: "create_bus".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(relative_path)
            .display()
            .to_string(),
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
    let (sheet_uuid, sheet_path, _sheet_value, mut bus) =
        load_native_bus_mutation_target(&project, bus_uuid)?;
    bus.members = members.clone();
    commit_schematic_operation(
        root,
        "edit schematic bus members",
        Operation::SetSchematicBus {
            sheet_id: sheet_uuid,
            bus_id: bus_uuid,
            bus: serde_json::to_value(&bus).expect("native bus serialization must succeed"),
        },
    )?;

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

pub(crate) fn delete_native_project_bus(
    root: &Path,
    bus_uuid: Uuid,
) -> Result<NativeProjectBusMutationReportView> {
    let project = load_native_project(root)?;
    let (sheet_uuid, sheet_path, sheet_value, bus) =
        load_native_bus_mutation_target(&project, bus_uuid)?;
    let bus_key = bus_uuid.to_string();
    if let Some(entries) = sheet_value
        .get("bus_entries")
        .and_then(serde_json::Value::as_object)
    {
        if let Some((entry_uuid, _)) = entries.iter().find(|(_, value)| {
            value.get("bus").and_then(serde_json::Value::as_str) == Some(bus_key.as_str())
        }) {
            bail!("bus {bus_uuid} is still referenced by bus entry {entry_uuid}");
        }
    }
    commit_schematic_operation(
        root,
        "delete schematic bus",
        Operation::DeleteSchematicBus {
            sheet_id: sheet_uuid,
            bus_id: bus_uuid,
            bus: serde_json::to_value(&bus).expect("native bus serialization must succeed"),
        },
    )?;

    Ok(NativeProjectBusMutationReportView {
        action: "delete_bus".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        bus_uuid: bus.uuid.to_string(),
        name: bus.name,
        members: bus.members,
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
    let (bus_sheet_uuid, _, _, _) = load_native_bus_mutation_target(&project, bus_uuid)?;
    if bus_sheet_uuid != sheet_uuid {
        bail!("bus {bus_uuid} belongs to sheet {bus_sheet_uuid}, not {sheet_uuid}");
    }
    if let Some(wire_uuid) = wire_uuid {
        let (wire_sheet_uuid, _, _, _) = load_native_wire_mutation_target(&project, wire_uuid)?;
        if wire_sheet_uuid != sheet_uuid {
            bail!("wire {wire_uuid} belongs to sheet {wire_sheet_uuid}, not {sheet_uuid}");
        }
    }

    let bus_entry_uuid = Uuid::new_v4();
    let bus_entry = BusEntry {
        uuid: bus_entry_uuid,
        bus: bus_uuid,
        wire: wire_uuid,
        position,
    };
    commit_schematic_operation(
        root,
        "place schematic bus entry",
        Operation::CreateSchematicBusEntry {
            sheet_id: sheet_uuid,
            bus_entry_id: bus_entry_uuid,
            bus_entry: serde_json::to_value(&bus_entry)
                .expect("native bus entry serialization must succeed"),
        },
    )?;

    Ok(NativeProjectBusEntryMutationReportView {
        action: "place_bus_entry".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(relative_path)
            .display()
            .to_string(),
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
    let (sheet_uuid, sheet_path, _sheet_value, bus_entry) =
        load_native_bus_entry_mutation_target(&project, bus_entry_uuid)?;
    commit_schematic_operation(
        root,
        "delete schematic bus entry",
        Operation::DeleteSchematicBusEntry {
            sheet_id: sheet_uuid,
            bus_entry_id: bus_entry_uuid,
            bus_entry: serde_json::to_value(&bus_entry)
                .expect("native bus entry serialization must succeed"),
        },
    )?;

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
    let noconnect_uuid = Uuid::new_v4();
    let noconnect = NoConnectMarker {
        uuid: noconnect_uuid,
        symbol: symbol_uuid,
        pin: pin_uuid,
        position,
    };
    commit_schematic_operation(
        root,
        "place schematic noconnect",
        Operation::CreateSchematicNoConnect {
            sheet_id: sheet_uuid,
            noconnect_id: noconnect_uuid,
            noconnect: serde_json::to_value(&noconnect)
                .expect("native no-connect serialization must succeed"),
        },
    )?;

    Ok(NativeProjectNoConnectMutationReportView {
        action: "place_noconnect".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: project
            .root
            .join("schematic")
            .join(relative_path)
            .display()
            .to_string(),
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
    let (sheet_uuid, sheet_path, _sheet_value, marker) =
        load_native_noconnect_mutation_target(&project, noconnect_uuid)?;
    commit_schematic_operation(
        root,
        "delete schematic noconnect",
        Operation::DeleteSchematicNoConnect {
            sheet_id: sheet_uuid,
            noconnect_id: noconnect_uuid,
            noconnect: serde_json::to_value(&marker)
                .expect("native no-connect serialization must succeed"),
        },
    )?;

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
