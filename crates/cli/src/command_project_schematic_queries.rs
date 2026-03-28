use super::*;

pub(crate) fn query_native_project_symbols(root: &Path) -> Result<Vec<SymbolInfo>> {
    let project = load_native_project(root)?;
    let mut symbols = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value
            .get("symbols")
            .and_then(serde_json::Value::as_object)
        {
            for value in entries.values() {
                let symbol: PlacedSymbol = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse symbol in {}", path.display()))?;
                symbols.push(SymbolInfo {
                    uuid: symbol.uuid,
                    sheet: sheet_uuid,
                    reference: symbol.reference,
                    value: symbol.value,
                    lib_id: symbol.lib_id,
                    position: symbol.position,
                    rotation: symbol.rotation,
                    mirrored: symbol.mirrored,
                    part_uuid: symbol.part,
                    entity_uuid: symbol.entity,
                    gate_uuid: symbol.gate,
                });
            }
        }
    }
    symbols.sort_by(|a, b| {
        a.reference
            .cmp(&b.reference)
            .then_with(|| a.uuid.cmp(&b.uuid))
    });
    Ok(symbols)
}

pub(crate) fn query_native_project_symbol_fields(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<Vec<SymbolFieldInfo>> {
    let project = load_native_project(root)?;
    let (_, _, _, symbol) = load_native_symbol_mutation_target(&project, symbol_uuid)?;
    let mut fields = symbol
        .fields
        .into_iter()
        .map(|field| SymbolFieldInfo {
            uuid: field.uuid,
            symbol: symbol_uuid,
            key: field.key,
            value: field.value,
            visible: field.visible,
            position: field.position,
        })
        .collect::<Vec<_>>();
    fields.sort_by(|a, b| a.key.cmp(&b.key).then_with(|| a.uuid.cmp(&b.uuid)));
    Ok(fields)
}

pub(crate) fn query_native_project_symbol_semantics(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<NativeProjectSymbolSemanticsView> {
    let project = load_native_project(root)?;
    let (_, _, _, symbol) = load_native_symbol_mutation_target(&project, symbol_uuid)?;
    Ok(NativeProjectSymbolSemanticsView {
        symbol_uuid: symbol.uuid.to_string(),
        gate_uuid: symbol.gate.map(|uuid| uuid.to_string()),
        unit_selection: symbol.unit_selection,
        hidden_power_behavior: render_hidden_power_behavior(&symbol.hidden_power_behavior),
    })
}

pub(crate) fn query_native_project_symbol_pins(
    root: &Path,
    symbol_uuid: Uuid,
) -> Result<Vec<NativeProjectSymbolPinInfoView>> {
    let project = load_native_project(root)?;
    let (_, _, _, symbol) = load_native_symbol_mutation_target(&project, symbol_uuid)?;
    let mut pins = symbol
        .pins
        .into_iter()
        .map(|pin| {
            let pin_override = symbol
                .pin_overrides
                .iter()
                .find(|entry| entry.pin == pin.uuid);
            NativeProjectSymbolPinInfoView {
                symbol_uuid: symbol_uuid.to_string(),
                pin_uuid: pin.uuid.to_string(),
                number: pin.number,
                name: pin.name,
                electrical_type: format!("{:?}", pin.electrical_type),
                x_nm: pin.position.x,
                y_nm: pin.position.y,
                visible_override: pin_override.map(|entry| entry.visible),
                override_x_nm: pin_override.and_then(|entry| entry.position.map(|p| p.x)),
                override_y_nm: pin_override.and_then(|entry| entry.position.map(|p| p.y)),
            }
        })
        .collect::<Vec<_>>();
    pins.sort_by(|a, b| {
        a.number
            .cmp(&b.number)
            .then_with(|| a.pin_uuid.cmp(&b.pin_uuid))
    });
    Ok(pins)
}

pub(crate) fn query_native_project_texts(root: &Path) -> Result<Vec<serde_json::Value>> {
    let project = load_native_project(root)?;
    let mut texts = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value
            .get("texts")
            .and_then(serde_json::Value::as_object)
        {
            for value in entries.values() {
                let text: SchematicText = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse text in {}", path.display()))?;
                texts.push(serde_json::json!({
                    "uuid": text.uuid,
                    "sheet": sheet_uuid,
                    "text": text.text,
                    "position": text.position,
                    "rotation": text.rotation,
                }));
            }
        }
    }
    texts.sort_by(|a, b| {
        a.get("uuid")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("uuid").and_then(serde_json::Value::as_str))
    });
    Ok(texts)
}

pub(crate) fn query_native_project_drawings(root: &Path) -> Result<Vec<serde_json::Value>> {
    let project = load_native_project(root)?;
    let mut drawings = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_text = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let sheet_value: serde_json::Value = serde_json::from_str(&sheet_text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(entries) = sheet_value
            .get("drawings")
            .and_then(serde_json::Value::as_object)
        {
            for value in entries.values() {
                let primitive: SchematicPrimitive = serde_json::from_value(value.clone())
                    .with_context(|| format!("failed to parse drawing in {}", path.display()))?;
                if let Some(view) = render_drawing_query_view(sheet_uuid, primitive) {
                    drawings.push(view);
                }
            }
        }
    }
    drawings.sort_by(|a, b| {
        a.get("uuid")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("uuid").and_then(serde_json::Value::as_str))
    });
    Ok(drawings)
}

pub(crate) fn query_native_project_nets(root: &Path) -> Result<Vec<SchematicNetInfo>> {
    let project = load_native_project(root)?;
    Ok(schematic_net_info(&build_native_project_schematic(
        &project,
    )?))
}

pub(crate) fn query_native_project_diagnostics(
    root: &Path,
) -> Result<Vec<ConnectivityDiagnosticInfo>> {
    let project = load_native_project(root)?;
    Ok(schematic_diagnostics(&build_native_project_schematic(
        &project,
    )?))
}

pub(crate) fn query_native_project_erc(root: &Path) -> Result<Vec<ErcFinding>> {
    let project = load_native_project(root)?;
    Ok(run_prechecks(&build_native_project_schematic(&project)?))
}

pub(crate) fn query_native_project_check(root: &Path) -> Result<CheckReport> {
    let project = load_native_project(root)?;
    let schematic = build_native_project_schematic(&project)?;
    let diagnostics = schematic_diagnostics(&schematic);
    let erc = run_prechecks(&schematic);
    Ok(CheckReport::Schematic {
        summary: summarize_native_schematic_checks(&diagnostics, &erc),
        diagnostics,
        erc,
    })
}
