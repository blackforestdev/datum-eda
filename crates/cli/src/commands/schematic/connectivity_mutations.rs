use super::*;
use eda_engine::api::native_write::schematic_connectivity::{
    build_create_schematic_bus, build_create_schematic_bus_entry, build_create_schematic_label,
    build_create_schematic_port, build_create_schematic_wire, build_delete_schematic_bus,
    build_delete_schematic_bus_entry, build_delete_schematic_junction, build_delete_schematic_label,
    build_delete_schematic_noconnect, build_delete_schematic_port, build_delete_schematic_wire,
    build_place_schematic_marker, build_set_schematic_bus, build_set_schematic_label,
    build_set_schematic_port,
};
use eda_engine::api::native_write::{PreparedWrite, WriteProvenance, commit_prepared};
use eda_engine::error::EngineError;
use eda_engine::substrate::{CommitReport, DesignModel, ProjectResolver, SchematicMarkerKind};

pub(crate) fn place_native_project_label(
    root: &Path,
    sheet_uuid: Uuid,
    name: String,
    kind: LabelKind,
    position: Point,
) -> Result<NativeProjectLabelMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
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
    commit_schematic_write(root, "place schematic label", |model, provenance| {
        build_create_schematic_label(model, provenance, sheet_uuid, &label)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut label) =
        load_native_label_mutation_target(&project, label_uuid)?;
    label.name = name.clone();
    commit_schematic_write(root, "rename schematic label", |model, provenance| {
        build_set_schematic_label(model, provenance, sheet_uuid, &label)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, label) =
        load_native_label_mutation_target(&project, label_uuid)?;
    commit_schematic_write(root, "delete schematic label", |model, provenance| {
        build_delete_schematic_label(model, provenance, sheet_uuid, &label)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
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
    commit_schematic_write(root, "draw schematic wire", |model, provenance| {
        build_create_schematic_wire(model, provenance, sheet_uuid, &wire)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, wire) =
        load_native_wire_mutation_target(&project, wire_uuid)?;
    commit_schematic_write(root, "delete schematic wire", |model, provenance| {
        build_delete_schematic_wire(model, provenance, sheet_uuid, &wire)
    })?;

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

/// Resolve the native project, author one write through the engine's
/// native-write facade builders, and commit it through the single journaled
/// commit path.
pub(super) fn commit_schematic_write<F>(root: &Path, reason: &str, build: F) -> Result<CommitReport>
where
    F: FnOnce(&DesignModel, WriteProvenance) -> Result<PreparedWrite, EngineError>,
{
    let mut model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let provenance = WriteProvenance::new("datum-eda-cli", cli_commit_source()?, reason);
    let prepared = build(&model, provenance)?;
    commit_prepared(&mut model, root, prepared).map_err(Into::into)
}

pub(crate) fn place_native_project_junction(
    root: &Path,
    sheet_uuid: Uuid,
    position: Point,
) -> Result<NativeProjectJunctionMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
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
    commit_schematic_write(root, "place schematic junction", |model, provenance| {
        build_place_schematic_marker(
            model,
            provenance,
            sheet_uuid,
            junction_uuid,
            SchematicMarkerKind::Junction,
            serde_json::to_value(&junction)?,
        )
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, junction) =
        load_native_junction_mutation_target(&project, junction_uuid)?;
    commit_schematic_write(root, "delete schematic junction", |model, provenance| {
        build_delete_schematic_junction(model, provenance, sheet_uuid, &junction)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
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
    commit_schematic_write(root, "place schematic port", |model, provenance| {
        build_create_schematic_port(model, provenance, sheet_uuid, &port)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
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
    commit_schematic_write(root, "edit schematic port", |model, provenance| {
        build_set_schematic_port(model, provenance, sheet_uuid, &port)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, port) =
        load_native_port_mutation_target(&project, port_uuid)?;
    commit_schematic_write(root, "delete schematic port", |model, provenance| {
        build_delete_schematic_port(model, provenance, sheet_uuid, &port)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
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
    commit_schematic_write(root, "create schematic bus", |model, provenance| {
        build_create_schematic_bus(model, provenance, sheet_uuid, &bus)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut bus) =
        load_native_bus_mutation_target(&project, bus_uuid)?;
    bus.members = members.clone();
    commit_schematic_write(root, "edit schematic bus members", |model, provenance| {
        build_set_schematic_bus(model, provenance, sheet_uuid, &bus)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
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
    commit_schematic_write(root, "delete schematic bus", |model, provenance| {
        build_delete_schematic_bus(model, provenance, sheet_uuid, &bus)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
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
    commit_schematic_write(root, "place schematic bus entry", |model, provenance| {
        build_create_schematic_bus_entry(model, provenance, sheet_uuid, &bus_entry)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, bus_entry) =
        load_native_bus_entry_mutation_target(&project, bus_entry_uuid)?;
    commit_schematic_write(root, "delete schematic bus entry", |model, provenance| {
        build_delete_schematic_bus_entry(model, provenance, sheet_uuid, &bus_entry)
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
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
    commit_schematic_write(root, "place schematic noconnect", |model, provenance| {
        build_place_schematic_marker(
            model,
            provenance,
            sheet_uuid,
            noconnect_uuid,
            SchematicMarkerKind::NoConnect,
            serde_json::to_value(&noconnect)?,
        )
    })?;

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
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, marker) =
        load_native_noconnect_mutation_target(&project, noconnect_uuid)?;
    commit_schematic_write(root, "delete schematic noconnect", |model, provenance| {
        build_delete_schematic_noconnect(model, provenance, sheet_uuid, &marker)
    })?;

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

// Phase 5: exec-layer dissolution — variant run() impls (the former
// command_exec destructure-and-forward glue, now inherent methods on the
// clap args structs).

impl ProjectCreateSheetArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, name, sheet } = self;
        let report = create_native_project_sheet(&path, name, sheet)?;
        let output = render_report(format, &report, render_native_project_sheet_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeleteSheetArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, sheet } = self;
        let report = delete_native_project_sheet(&path, sheet)?;
        let output = render_report(format, &report, render_native_project_sheet_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectRenameSheetArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, sheet, name } = self;
        let report = rename_native_project_sheet(&path, sheet, name)?;
        let output = render_report(format, &report, render_native_project_sheet_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectCreateSheetDefinitionArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            root_sheet,
            name,
            definition,
        } = self;
        let report = create_native_project_sheet_definition(&path, root_sheet, name, definition)?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_definition_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectCreateSheetInstanceArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            definition,
            parent_sheet,
            name,
            x_nm,
            y_nm,
            instance,
        } = self;
        let report = create_native_project_sheet_instance(
            &path,
            definition,
            parent_sheet,
            name,
            x_nm,
            y_nm,
            instance,
        )?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_instance_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectDeleteSheetInstanceArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, instance } = self;
        let report = delete_native_project_sheet_instance(&path, instance)?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_instance_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectMoveSheetInstanceArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            instance,
            x_nm,
            y_nm,
        } = self;
        let report = move_native_project_sheet_instance(&path, instance, x_nm, y_nm)?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_instance_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectBindSheetInstancePortArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            instance,
            port,
        } = self;
        let report = bind_native_project_sheet_instance_port(&path, instance, port)?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_instance_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectUnbindSheetInstancePortArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            instance,
            port,
        } = self;
        let report = unbind_native_project_sheet_instance_port(&path, instance, port)?;
        let output = render_report(
            format,
            &report,
            render_native_project_sheet_instance_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectPlaceLabelArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            name,
            kind,
            x_nm,
            y_nm,
        } = self;
        let kind = match kind {
            NativeLabelKindArg::Local => LabelKind::Local,
            NativeLabelKindArg::Global => LabelKind::Global,
            NativeLabelKindArg::Hierarchical => LabelKind::Hierarchical,
            NativeLabelKindArg::Power => LabelKind::Power,
        };
        let report = place_native_project_label(
            &path,
            sheet,
            name,
            kind,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
        )?;
        let output = render_report(format, &report, render_native_project_label_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectRenameLabelArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, label, name } = self;
        let report = rename_native_project_label(&path, label, name)?;
        let output = render_report(format, &report, render_native_project_label_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeleteLabelArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, label } = self;
        let report = delete_native_project_label(&path, label)?;
        let output = render_report(format, &report, render_native_project_label_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDrawWireArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
        } = self;
        let report = draw_native_project_wire(
            &path,
            sheet,
            eda_engine::ir::geometry::Point {
                x: from_x_nm,
                y: from_y_nm,
            },
            eda_engine::ir::geometry::Point {
                x: to_x_nm,
                y: to_y_nm,
            },
        )?;
        let output = render_report(format, &report, render_native_project_wire_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeleteWireArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, wire } = self;
        let report = delete_native_project_wire(&path, wire)?;
        let output = render_report(format, &report, render_native_project_wire_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectPlaceJunctionArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            x_nm,
            y_nm,
        } = self;
        let report = place_native_project_junction(
            &path,
            sheet,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
        )?;
        let output = render_report(
            format,
            &report,
            render_native_project_junction_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectDeleteJunctionArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, junction } = self;
        let report = delete_native_project_junction(&path, junction)?;
        let output = render_report(
            format,
            &report,
            render_native_project_junction_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectPlacePortArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            name,
            direction,
            x_nm,
            y_nm,
        } = self;
        let direction = match direction {
            NativePortDirectionArg::Input => PortDirection::Input,
            NativePortDirectionArg::Output => PortDirection::Output,
            NativePortDirectionArg::Bidirectional => PortDirection::Bidirectional,
            NativePortDirectionArg::Passive => PortDirection::Passive,
        };
        let report = place_native_project_port(
            &path,
            sheet,
            name,
            direction,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
        )?;
        let output = render_report(format, &report, render_native_project_port_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectEditPortArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            port,
            name,
            direction,
            x_nm,
            y_nm,
        } = self;
        let direction = direction.map(|value| match value {
            NativePortDirectionArg::Input => PortDirection::Input,
            NativePortDirectionArg::Output => PortDirection::Output,
            NativePortDirectionArg::Bidirectional => PortDirection::Bidirectional,
            NativePortDirectionArg::Passive => PortDirection::Passive,
        });
        let report = edit_native_project_port(&path, port, name, direction, x_nm, y_nm)?;
        let output = render_report(format, &report, render_native_project_port_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeletePortArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, port } = self;
        let report = delete_native_project_port(&path, port)?;
        let output = render_report(format, &report, render_native_project_port_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectCreateBusArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            name,
            members,
        } = self;
        let report = create_native_project_bus(&path, sheet, name, members)?;
        let output = render_report(format, &report, render_native_project_bus_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectEditBusMembersArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, bus, members } = self;
        let report = edit_native_project_bus_members(&path, bus, members)?;
        let output = render_report(format, &report, render_native_project_bus_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeleteBusArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, bus } = self;
        let report = delete_native_project_bus(&path, bus)?;
        let output = render_report(format, &report, render_native_project_bus_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectPlaceBusEntryArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            bus,
            wire,
            x_nm,
            y_nm,
        } = self;
        let report = place_native_project_bus_entry(
            &path,
            sheet,
            bus,
            wire,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
        )?;
        let output = render_report(
            format,
            &report,
            render_native_project_bus_entry_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectDeleteBusEntryArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, bus_entry } = self;
        let report = delete_native_project_bus_entry(&path, bus_entry)?;
        let output = render_report(
            format,
            &report,
            render_native_project_bus_entry_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectPlaceNoConnectArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            symbol,
            pin,
            x_nm,
            y_nm,
        } = self;
        let report = place_native_project_noconnect(
            &path,
            sheet,
            symbol,
            pin,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
        )?;
        let output = render_report(
            format,
            &report,
            render_native_project_noconnect_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectDeleteNoConnectArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, noconnect } = self;
        let report = delete_native_project_noconnect(&path, noconnect)?;
        let output = render_report(
            format,
            &report,
            render_native_project_noconnect_mutation_text,
        );
        Ok((output, 0))
    }
}
