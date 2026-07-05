use super::connectivity_mutations::commit_schematic_write;
use super::symbol_component_instance::part_binding_for_pool_symbol;
use super::symbol_library_materialization::{
    materialize_pool_symbol_pins, resolve_pool_symbol_component_binding,
};
use super::symbol_reports::{
    binding_evidence_for_pool_symbol, component_instance_uuid_for_pool_symbol,
    symbol_mutation_report, symbol_mutation_report_with_binding,
};
use super::*;
use eda_engine::api::native_write::schematic_symbols::{
    build_delete_schematic_symbol, build_place_schematic_symbol, build_set_schematic_symbol,
};

fn schematic_sheet_path(project: &LoadedNativeProject, sheet_uuid: Uuid) -> Result<PathBuf> {
    let sheet_key = sheet_uuid.to_string();
    let relative_path =
        project.schematic.sheets.get(&sheet_key).ok_or_else(|| {
            anyhow::anyhow!("sheet not found in native schematic root: {sheet_uuid}")
        })?;
    Ok(project.root.join("schematic").join(relative_path))
}

fn commit_symbol_update(
    root: &Path,
    reason: &str,
    sheet_uuid: Uuid,
    symbol: &PlacedSymbol,
) -> Result<()> {
    commit_schematic_write(root, reason, |model, provenance| {
        build_set_schematic_symbol(model, provenance, sheet_uuid, symbol)
    })?;
    Ok(())
}

pub(crate) fn place_native_project_symbol(
    root: &Path,
    sheet_uuid: Uuid,
    reference: String,
    value: String,
    lib_id: Option<String>,
    position: Point,
    rotation_deg: i32,
    mirrored: bool,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let sheet_path = schematic_sheet_path(&project, sheet_uuid)?;
    let pins = materialize_pool_symbol_pins(root, lib_id.as_deref())?;
    let binding_resolution = resolve_pool_symbol_component_binding(root, lib_id.as_deref())?;
    let binding = binding_resolution.binding.clone();

    let symbol_uuid = Uuid::new_v4();
    let symbol = PlacedSymbol {
        uuid: symbol_uuid,
        part: binding
            .as_ref()
            .and_then(|binding| binding.part.as_ref().map(|part| part.part_id)),
        entity: binding.as_ref().map(|binding| binding.entity_id),
        gate: binding.as_ref().map(|binding| binding.gate_id),
        lib_id,
        reference,
        value,
        fields: Vec::<SymbolField>::new(),
        pins,
        position,
        rotation: rotation_deg,
        mirrored,
        unit_selection: None,
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: Vec::<PinDisplayOverride>::new(),
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    };
    let part_binding = binding.as_ref().and_then(part_binding_for_pool_symbol);
    let component_instance_uuid = binding.as_ref().and_then(|binding| {
        binding
            .part
            .as_ref()
            .map(|_| component_instance_uuid_for_pool_symbol(&project, symbol_uuid, binding))
    });
    let binding_evidence = binding.as_ref().map(|binding| {
        binding_evidence_for_pool_symbol(binding, symbol_uuid, component_instance_uuid)
    });
    commit_schematic_write(root, "place schematic symbol", |model, provenance| {
        build_place_schematic_symbol(
            model,
            provenance,
            sheet_uuid,
            &symbol,
            part_binding.as_ref(),
        )
    })?;
    Ok(symbol_mutation_report_with_binding(
        "place_symbol",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
        binding_resolution.status.to_string(),
        binding_resolution.diagnostics,
        binding_evidence,
        component_instance_uuid,
    ))
}

pub(crate) fn move_native_project_symbol(
    root: &Path,
    symbol_uuid: Uuid,
    position: Point,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.position = position;
    commit_symbol_update(root, "move schematic symbol", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "move_symbol",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn rotate_native_project_symbol(
    root: &Path,
    symbol_uuid: Uuid,
    rotation_deg: i32,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.rotation = rotation_deg;
    commit_symbol_update(root, "rotate schematic symbol", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "rotate_symbol",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn mirror_native_project_symbol(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.mirrored = !symbol.mirrored;
    commit_symbol_update(root, "mirror schematic symbol", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "mirror_symbol",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn delete_native_project_symbol(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    commit_schematic_write(root, "delete schematic symbol", |model, provenance| {
        build_delete_schematic_symbol(model, provenance, sheet_uuid, &symbol)
    })?;
    Ok(symbol_mutation_report(
        "delete_symbol",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn set_native_project_symbol_reference(
    root: &Path,
    symbol_uuid: Uuid,
    reference: String,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.reference = reference;
    commit_symbol_update(root, "set schematic symbol reference", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "set_symbol_reference",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn set_native_project_symbol_value(
    root: &Path,
    symbol_uuid: Uuid,
    value: String,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.value = value;
    commit_symbol_update(root, "set schematic symbol value", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "set_symbol_value",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn set_native_project_symbol_lib_id(
    root: &Path,
    symbol_uuid: Uuid,
    lib_id: String,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.lib_id = Some(lib_id);
    commit_symbol_update(root, "set schematic symbol lib id", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "set_symbol_lib_id",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn clear_native_project_symbol_lib_id(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.lib_id = None;
    commit_symbol_update(root, "clear schematic symbol lib id", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "clear_symbol_lib_id",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn set_native_project_symbol_entity(
    root: &Path,
    symbol_uuid: Uuid,
    entity_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.entity = Some(entity_uuid);
    symbol.part = None;
    commit_symbol_update(root, "set schematic symbol entity", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "set_symbol_entity",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn clear_native_project_symbol_entity(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.entity = None;
    commit_symbol_update(root, "clear schematic symbol entity", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "clear_symbol_entity",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn set_native_project_symbol_part(
    root: &Path,
    symbol_uuid: Uuid,
    part_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.part = Some(part_uuid);
    symbol.entity = None;
    commit_symbol_update(root, "set schematic symbol part", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "set_symbol_part",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn clear_native_project_symbol_part(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.part = None;
    commit_symbol_update(root, "clear schematic symbol part", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "clear_symbol_part",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn set_native_project_symbol_unit(
    root: &Path,
    symbol_uuid: Uuid,
    unit_selection: String,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.unit_selection = Some(unit_selection);
    commit_symbol_update(root, "set schematic symbol unit", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "set_symbol_unit",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn clear_native_project_symbol_unit(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.unit_selection = None;
    commit_symbol_update(root, "clear schematic symbol unit", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "clear_symbol_unit",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn set_native_project_symbol_gate(
    root: &Path,
    symbol_uuid: Uuid,
    gate_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.gate = Some(gate_uuid);
    commit_symbol_update(root, "set schematic symbol gate", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "set_symbol_gate",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn clear_native_project_symbol_gate(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.gate = None;
    commit_symbol_update(root, "clear schematic symbol gate", sheet_uuid, &symbol)?;
    Ok(symbol_mutation_report(
        "clear_symbol_gate",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn set_native_project_symbol_display_mode(
    root: &Path,
    symbol_uuid: Uuid,
    display_mode: SymbolDisplayMode,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.display_mode = display_mode;
    commit_symbol_update(
        root,
        "set schematic symbol display mode",
        sheet_uuid,
        &symbol,
    )?;
    Ok(symbol_mutation_report(
        "set_symbol_display_mode",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn set_native_project_symbol_hidden_power_behavior(
    root: &Path,
    symbol_uuid: Uuid,
    hidden_power_behavior: HiddenPowerBehavior,
) -> Result<NativeProjectSymbolMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    symbol.hidden_power_behavior = hidden_power_behavior;
    commit_symbol_update(
        root,
        "set schematic symbol hidden power behavior",
        sheet_uuid,
        &symbol,
    )?;
    Ok(symbol_mutation_report(
        "set_symbol_hidden_power_behavior",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
    ))
}

pub(crate) fn set_native_project_symbol_pin_override(
    root: &Path,
    symbol_uuid: Uuid,
    pin_uuid: Uuid,
    visible: bool,
    position: Option<Point>,
) -> Result<NativeProjectPinOverrideMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    if !symbol.pins.iter().any(|pin| pin.uuid == pin_uuid) {
        bail!("pin not found on native symbol: {pin_uuid}");
    }
    if let Some(entry) = symbol
        .pin_overrides
        .iter_mut()
        .find(|entry| entry.pin == pin_uuid)
    {
        entry.visible = visible;
        entry.position = position;
    } else {
        symbol.pin_overrides.push(PinDisplayOverride {
            pin: pin_uuid,
            visible,
            position,
        });
    }
    commit_symbol_update(
        root,
        "set schematic symbol pin override",
        sheet_uuid,
        &symbol,
    )?;

    Ok(NativeProjectPinOverrideMutationReportView {
        action: "set_pin_override".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        pin_uuid: pin_uuid.to_string(),
        visible: Some(visible),
        x_nm: position.map(|point| point.x),
        y_nm: position.map(|point| point.y),
    })
}

pub(crate) fn clear_native_project_symbol_pin_override(
    root: &Path,
    symbol_uuid: Uuid,
    pin_uuid: Uuid,
) -> Result<NativeProjectPinOverrideMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    let before = symbol.pin_overrides.len();
    symbol.pin_overrides.retain(|entry| entry.pin != pin_uuid);
    if symbol.pin_overrides.len() == before {
        bail!("pin override not found on native symbol: {pin_uuid}");
    }
    commit_symbol_update(
        root,
        "clear schematic symbol pin override",
        sheet_uuid,
        &symbol,
    )?;

    Ok(NativeProjectPinOverrideMutationReportView {
        action: "clear_pin_override".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        pin_uuid: pin_uuid.to_string(),
        visible: None,
        x_nm: None,
        y_nm: None,
    })
}

pub(crate) fn add_native_project_symbol_field(
    root: &Path,
    symbol_uuid: Uuid,
    key: String,
    value: String,
    visible: bool,
    position: Option<Point>,
) -> Result<NativeProjectSymbolFieldMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, mut symbol) =
        load_native_symbol_mutation_target(&project, symbol_uuid)?;
    let field_uuid = Uuid::new_v4();
    symbol.fields.push(SymbolField {
        uuid: field_uuid,
        key: key.clone(),
        value: value.clone(),
        position,
        visible,
    });
    commit_symbol_update(root, "add schematic symbol field", sheet_uuid, &symbol)?;

    Ok(NativeProjectSymbolFieldMutationReportView {
        action: "add_symbol_field".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        field_uuid: field_uuid.to_string(),
        key,
        value,
        visible,
        x_nm: position.map(|point| point.x),
        y_nm: position.map(|point| point.y),
    })
}

pub(crate) fn edit_native_project_symbol_field(
    root: &Path,
    field_uuid: Uuid,
    key: Option<String>,
    value: Option<String>,
    visible: Option<bool>,
    position: Option<Point>,
) -> Result<NativeProjectSymbolFieldMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, symbol_uuid, mut symbol, mut field) =
        load_native_field_mutation_target(&project, field_uuid)?;
    if let Some(key) = key {
        field.key = key;
    }
    if let Some(value) = value {
        field.value = value;
    }
    if let Some(visible) = visible {
        field.visible = visible;
    }
    if let Some(position) = position {
        field.position = Some(position);
    }
    for existing in &mut symbol.fields {
        if existing.uuid == field_uuid {
            *existing = field.clone();
            break;
        }
    }
    commit_symbol_update(root, "edit schematic symbol field", sheet_uuid, &symbol)?;

    Ok(NativeProjectSymbolFieldMutationReportView {
        action: "edit_symbol_field".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol_uuid.to_string(),
        field_uuid: field.uuid.to_string(),
        key: field.key,
        value: field.value,
        visible: field.visible,
        x_nm: field.position.map(|point| point.x),
        y_nm: field.position.map(|point| point.y),
    })
}

pub(crate) fn delete_native_project_symbol_field(
    root: &Path,
    field_uuid: Uuid,
) -> Result<NativeProjectSymbolFieldMutationReportView> {
    let project = load_native_project_with_resolved_board(root)?;
    let (sheet_uuid, sheet_path, _sheet_value, symbol_uuid, mut symbol, field) =
        load_native_field_mutation_target(&project, field_uuid)?;
    symbol.fields.retain(|existing| existing.uuid != field_uuid);
    commit_symbol_update(root, "delete schematic symbol field", sheet_uuid, &symbol)?;

    Ok(NativeProjectSymbolFieldMutationReportView {
        action: "delete_symbol_field".to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol_uuid.to_string(),
        field_uuid: field.uuid.to_string(),
        key: field.key,
        value: field.value,
        visible: field.visible,
        x_nm: field.position.map(|point| point.x),
        y_nm: field.position.map(|point| point.y),
    })
}

// Phase 5: exec-layer dissolution — variant run() impls (the former
// command_exec destructure-and-forward glue, now inherent methods on the
// clap args structs).

impl ProjectPlaceSymbolArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            reference,
            value,
            lib_id,
            x_nm,
            y_nm,
            rotation_deg,
            mirrored,
        } = self;
        let report = place_native_project_symbol(
            &path,
            sheet,
            reference,
            value,
            lib_id,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
            rotation_deg,
            mirrored,
        )?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectMoveSymbolArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            x_nm,
            y_nm,
        } = self;
        let report = move_native_project_symbol(
            &path,
            symbol,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
        )?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectRotateSymbolArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            rotation_deg,
        } = self;
        let report = rotate_native_project_symbol(&path, symbol, rotation_deg)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectMirrorSymbolArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, symbol } = self;
        let report = mirror_native_project_symbol(&path, symbol)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeleteSymbolArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, symbol } = self;
        let report = delete_native_project_symbol(&path, symbol)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectSetSymbolReferenceArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            reference,
        } = self;
        let report = set_native_project_symbol_reference(&path, symbol, reference)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectSetSymbolValueArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            value,
        } = self;
        let report = set_native_project_symbol_value(&path, symbol, value)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectSetSymbolLibIdArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol_uuid,
            lib_id,
        } = self;
        let report = set_native_project_symbol_lib_id(&path, symbol_uuid, lib_id)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectClearSymbolLibIdArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, symbol_uuid } = self;
        let report = clear_native_project_symbol_lib_id(&path, symbol_uuid)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectSetSymbolEntityArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            entity_uuid,
        } = self;
        let report = set_native_project_symbol_entity(&path, symbol, entity_uuid)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectClearSymbolEntityArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, symbol } = self;
        let report = clear_native_project_symbol_entity(&path, symbol)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectSetSymbolPartArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            part_uuid,
        } = self;
        let report = set_native_project_symbol_part(&path, symbol, part_uuid)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectClearSymbolPartArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, symbol } = self;
        let report = clear_native_project_symbol_part(&path, symbol)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectSetSymbolUnitArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            unit_selection,
        } = self;
        let report = set_native_project_symbol_unit(&path, symbol, unit_selection)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectClearSymbolUnitArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, symbol } = self;
        let report = clear_native_project_symbol_unit(&path, symbol)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectSetSymbolGateArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            gate_uuid,
        } = self;
        let report = set_native_project_symbol_gate(&path, symbol, gate_uuid)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectClearSymbolGateArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, symbol } = self;
        let report = clear_native_project_symbol_gate(&path, symbol)?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectSetSymbolDisplayModeArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            display_mode,
        } = self;
        let report = set_native_project_symbol_display_mode(
            &path,
            symbol,
            parse_native_symbol_display_mode(display_mode),
        )?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectSetPinOverrideArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            pin_uuid,
            visible,
            x_nm,
            y_nm,
        } = self;
        let position = parse_native_field_position(x_nm, y_nm)?;
        let report =
            set_native_project_symbol_pin_override(&path, symbol, pin_uuid, visible, position)?;
        let output = render_report(
            format,
            &report,
            render_native_project_pin_override_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectSetSymbolHiddenPowerBehaviorArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            hidden_power_behavior,
        } = self;
        let report = set_native_project_symbol_hidden_power_behavior(
            &path,
            symbol,
            parse_native_hidden_power_behavior(hidden_power_behavior),
        )?;
        let output = render_report(format, &report, render_native_project_symbol_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectClearPinOverrideArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            pin_uuid,
        } = self;
        let report = clear_native_project_symbol_pin_override(&path, symbol, pin_uuid)?;
        let output = render_report(
            format,
            &report,
            render_native_project_pin_override_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectAddSymbolFieldArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            symbol,
            key,
            value,
            hidden,
            x_nm,
            y_nm,
        } = self;
        let report = add_native_project_symbol_field(
            &path,
            symbol,
            key,
            value,
            !hidden,
            parse_native_field_position(x_nm, y_nm)?,
        )?;
        let output = render_report(
            format,
            &report,
            render_native_project_symbol_field_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectEditSymbolFieldArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            field,
            key,
            value,
            visible,
            x_nm,
            y_nm,
            ..
        } = self;
        let report = edit_native_project_symbol_field(
            &path,
            field,
            key,
            value,
            visible,
            parse_native_field_position(x_nm, y_nm)?,
        )?;
        let output = render_report(
            format,
            &report,
            render_native_project_symbol_field_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectDeleteSymbolFieldArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, field } = self;
        let report = delete_native_project_symbol_field(&path, field)?;
        let output = render_report(
            format,
            &report,
            render_native_project_symbol_field_mutation_text,
        );
        Ok((output, 0))
    }
}

impl ProjectPlaceTextArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            text,
            x_nm,
            y_nm,
            rotation_deg,
        } = self;
        let report = place_native_project_text(
            &path,
            sheet,
            text,
            eda_engine::ir::geometry::Point { x: x_nm, y: y_nm },
            rotation_deg,
        )?;
        let output = render_report(format, &report, render_native_project_text_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectEditTextArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            text,
            value,
            x_nm,
            y_nm,
            rotation_deg,
        } = self;
        let position = match (x_nm, y_nm) {
            (None, None) => None,
            (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
            _ => bail!("text position requires both --x-nm and --y-nm"),
        };
        let report = edit_native_project_text(&path, text, value, position, rotation_deg)?;
        let output = render_report(format, &report, render_native_project_text_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeleteTextArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, text } = self;
        let report = delete_native_project_text(&path, text)?;
        let output = render_report(format, &report, render_native_project_text_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectPlaceDrawingLineArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
        } = self;
        let report = place_native_project_drawing_line(
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
        let output = render_report(format, &report, render_native_project_drawing_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectPlaceDrawingRectArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            min_x_nm,
            min_y_nm,
            max_x_nm,
            max_y_nm,
        } = self;
        let report = place_native_project_drawing_rect(
            &path,
            sheet,
            eda_engine::ir::geometry::Point {
                x: min_x_nm,
                y: min_y_nm,
            },
            eda_engine::ir::geometry::Point {
                x: max_x_nm,
                y: max_y_nm,
            },
        )?;
        let output = render_report(format, &report, render_native_project_drawing_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectPlaceDrawingCircleArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            center_x_nm,
            center_y_nm,
            radius_nm,
        } = self;
        let report = place_native_project_drawing_circle(
            &path,
            sheet,
            eda_engine::ir::geometry::Point {
                x: center_x_nm,
                y: center_y_nm,
            },
            radius_nm,
        )?;
        let output = render_report(format, &report, render_native_project_drawing_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectPlaceDrawingArcArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            sheet,
            center_x_nm,
            center_y_nm,
            radius_nm,
            start_angle_mdeg,
            end_angle_mdeg,
        } = self;
        let report = place_native_project_drawing_arc(
            &path,
            sheet,
            eda_engine::ir::geometry::Arc {
                center: eda_engine::ir::geometry::Point {
                    x: center_x_nm,
                    y: center_y_nm,
                },
                radius: radius_nm,
                start_angle: start_angle_mdeg,
                end_angle: end_angle_mdeg,
            },
        )?;
        let output = render_report(format, &report, render_native_project_drawing_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectEditDrawingLineArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            drawing,
            from_x_nm,
            from_y_nm,
            to_x_nm,
            to_y_nm,
        } = self;
        let from = match (from_x_nm, from_y_nm) {
            (None, None) => None,
            (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
            _ => bail!("drawing start requires both --from-x-nm and --from-y-nm"),
        };
        let to = match (to_x_nm, to_y_nm) {
            (None, None) => None,
            (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
            _ => bail!("drawing end requires both --to-x-nm and --to-y-nm"),
        };
        let report = edit_native_project_drawing_line(&path, drawing, from, to)?;
        let output = render_report(format, &report, render_native_project_drawing_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectEditDrawingRectArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            drawing,
            min_x_nm,
            min_y_nm,
            max_x_nm,
            max_y_nm,
        } = self;
        let min = match (min_x_nm, min_y_nm) {
            (None, None) => None,
            (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
            _ => bail!("rect min requires both --min-x-nm and --min-y-nm"),
        };
        let max = match (max_x_nm, max_y_nm) {
            (None, None) => None,
            (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
            _ => bail!("rect max requires both --max-x-nm and --max-y-nm"),
        };
        let report = edit_native_project_drawing_rect(&path, drawing, min, max)?;
        let output = render_report(format, &report, render_native_project_drawing_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectEditDrawingCircleArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            drawing,
            center_x_nm,
            center_y_nm,
            radius_nm,
        } = self;
        let center = match (center_x_nm, center_y_nm) {
            (None, None) => None,
            (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
            _ => bail!("circle center requires both --center-x-nm and --center-y-nm"),
        };
        let report = edit_native_project_drawing_circle(&path, drawing, center, radius_nm)?;
        let output = render_report(format, &report, render_native_project_drawing_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectEditDrawingArcArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self {
            path,
            drawing,
            center_x_nm,
            center_y_nm,
            radius_nm,
            start_angle_mdeg,
            end_angle_mdeg,
        } = self;
        let center = match (center_x_nm, center_y_nm) {
            (None, None) => None,
            (Some(x), Some(y)) => Some(eda_engine::ir::geometry::Point { x, y }),
            _ => bail!("arc center requires both --center-x-nm and --center-y-nm"),
        };
        let report = edit_native_project_drawing_arc(
            &path,
            drawing,
            center,
            radius_nm,
            start_angle_mdeg,
            end_angle_mdeg,
        )?;
        let output = render_report(format, &report, render_native_project_drawing_mutation_text);
        Ok((output, 0))
    }
}

impl ProjectDeleteDrawingArgs {
    pub(crate) fn run(self, format: &OutputFormat) -> Result<(String, i32)> {
        let Self { path, drawing } = self;
        let report = delete_native_project_drawing(&path, drawing)?;
        let output = render_report(format, &report, render_native_project_drawing_mutation_text);
        Ok((output, 0))
    }
}

// Phase 5: display-mode/hidden-power parse helpers absorbed from the
// dissolved command_exec_native_support.rs.
fn parse_native_symbol_display_mode(value: NativeSymbolDisplayModeArg) -> SymbolDisplayMode {
    match value {
        NativeSymbolDisplayModeArg::LibraryDefault => SymbolDisplayMode::LibraryDefault,
        NativeSymbolDisplayModeArg::ShowHiddenPins => SymbolDisplayMode::ShowHiddenPins,
        NativeSymbolDisplayModeArg::HideOptionalPins => SymbolDisplayMode::HideOptionalPins,
    }
}

fn parse_native_hidden_power_behavior(value: NativeHiddenPowerBehaviorArg) -> HiddenPowerBehavior {
    match value {
        NativeHiddenPowerBehaviorArg::SourceDefinedImplicit => {
            HiddenPowerBehavior::SourceDefinedImplicit
        }
        NativeHiddenPowerBehaviorArg::ExplicitPowerObject => {
            HiddenPowerBehavior::ExplicitPowerObject
        }
        NativeHiddenPowerBehaviorArg::PreservedAsImportedMetadata => {
            HiddenPowerBehavior::PreservedAsImportedMetadata
        }
    }
}
