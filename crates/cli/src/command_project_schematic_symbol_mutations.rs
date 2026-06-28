use super::command_project_schematic_symbol_library_materialization::materialize_pool_symbol_pins;
use super::*;
use crate::command_project::command_project_schematic_connectivity_mutations::commit_schematic_operation;
use eda_engine::substrate::Operation;

fn symbol_mutation_report(
    action: &str,
    project: &LoadedNativeProject,
    sheet_uuid: Uuid,
    sheet_path: &Path,
    symbol: &PlacedSymbol,
) -> NativeProjectSymbolMutationReportView {
    NativeProjectSymbolMutationReportView {
        action: action.to_string(),
        project_root: project.root.display().to_string(),
        sheet_uuid: sheet_uuid.to_string(),
        sheet_path: sheet_path.display().to_string(),
        symbol_uuid: symbol.uuid.to_string(),
        reference: symbol.reference.clone(),
        value: symbol.value.clone(),
        lib_id: symbol.lib_id.clone(),
        x_nm: symbol.position.x,
        y_nm: symbol.position.y,
        rotation_deg: symbol.rotation,
        mirrored: symbol.mirrored,
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection.clone(),
        display_mode: render_symbol_display_mode(&symbol.display_mode),
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    }
}

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
    commit_schematic_operation(
        root,
        reason,
        Operation::SetSchematicSymbol {
            sheet_id: sheet_uuid,
            symbol_id: symbol.uuid,
            symbol: serde_json::to_value(symbol).expect("native symbol serialization must succeed"),
        },
    )?;
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

    let symbol_uuid = Uuid::new_v4();
    let symbol = PlacedSymbol {
        uuid: symbol_uuid,
        part: None,
        entity: None,
        gate: None,
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
    commit_schematic_operation(
        root,
        "place schematic symbol",
        Operation::CreateSchematicSymbol {
            sheet_id: sheet_uuid,
            symbol_id: symbol_uuid,
            symbol: serde_json::to_value(&symbol)
                .expect("native symbol serialization must succeed"),
        },
    )?;
    Ok(symbol_mutation_report(
        "place_symbol",
        &project,
        sheet_uuid,
        &sheet_path,
        &symbol,
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
    commit_schematic_operation(
        root,
        "delete schematic symbol",
        Operation::DeleteSchematicSymbol {
            sheet_id: sheet_uuid,
            symbol_id: symbol_uuid,
            symbol: serde_json::to_value(&symbol)
                .expect("native symbol serialization must succeed"),
        },
    )?;
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
