use super::command_project_board_diagnostics::query_native_project_drc_with_rules;
use super::*;
use eda_engine::connectivity::schematic_hierarchy_info;
use eda_engine::drc::{DrcSeverity, DrcViolation};
use eda_engine::pool::{Symbol, SymbolPinStyle};
use eda_engine::rules::ast::RuleType;
use eda_engine::substrate::ProjectResolver;
use std::collections::HashMap;

pub(crate) fn query_native_project_hierarchy(root: &Path) -> Result<HierarchyInfo> {
    let project = load_native_project_with_resolved_board(root)?;
    Ok(schematic_hierarchy_info(&build_native_project_schematic(
        &project,
    )?))
}

pub(crate) fn query_native_project_sheets(root: &Path) -> Result<Vec<serde_json::Value>> {
    let project = load_native_project_with_resolved_board(root)?;
    let model = ProjectResolver::new(&project.root)
        .resolve()
        .with_context(|| {
            format!(
                "failed to resolve native project {}",
                project.root.display()
            )
        })?;
    let mut sheets = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let path = project.root.join("schematic").join(relative_path);
        let sheet_value = model
            .materialized_source_shard_value_by_relative_path(&format!("schematic/{relative_path}"))
            .with_context(|| format!("failed to materialize {}", path.display()))?;
        sheets.push(serde_json::json!({
            "uuid": sheet_uuid,
            "name": sheet_value.get("name").and_then(serde_json::Value::as_str).unwrap_or(""),
            "path": format!("schematic/{relative_path}"),
        }));
    }
    sheets.sort_by(|a, b| {
        a.get("uuid")
            .and_then(serde_json::Value::as_str)
            .cmp(&b.get("uuid").and_then(serde_json::Value::as_str))
    });
    Ok(sheets)
}

pub(crate) fn query_native_project_symbols(root: &Path) -> Result<Vec<SymbolInfo>> {
    let project = load_native_project_with_resolved_board(root)?;
    let model = ProjectResolver::new(&project.root)
        .resolve()
        .with_context(|| {
            format!(
                "failed to resolve native project {}",
                project.root.display()
            )
        })?;
    let mut symbols = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_value = model
            .materialized_source_shard_value_by_relative_path(&format!("schematic/{relative_path}"))
            .with_context(|| format!("failed to materialize {}", path.display()))?;
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
    let project = load_native_project_with_resolved_board(root)?;
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
    let project = load_native_project_with_resolved_board(root)?;
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
    let project = load_native_project_with_resolved_board(root)?;
    let (_, _, _, symbol) = load_native_symbol_mutation_target(&project, symbol_uuid)?;
    let anchor_styles = pool_symbol_anchor_styles(root, symbol.lib_id.as_deref())?;
    let mut pins = symbol
        .pins
        .into_iter()
        .map(|pin| {
            let pin_override = symbol
                .pin_overrides
                .iter()
                .find(|entry| entry.pin == pin.uuid);
            let anchor_style = anchor_styles.get(&pin.uuid);
            NativeProjectSymbolPinInfoView {
                symbol_uuid: symbol_uuid.to_string(),
                pin_uuid: pin.uuid.to_string(),
                number: pin.number,
                name: pin.name,
                electrical_type: format!("{:?}", pin.electrical_type),
                x_nm: pin.position.x,
                y_nm: pin.position.y,
                anchor_orientation: anchor_style.map(|style| format!("{:?}", style.orientation)),
                anchor_length_nm: anchor_style.and_then(|style| style.length_nm),
                anchor_decoration: anchor_style.map(|style| {
                    serde_json::to_value(&style.decoration)
                        .ok()
                        .and_then(|value| value.as_str().map(str::to_string))
                        .unwrap_or_else(|| format!("{:?}", style.decoration))
                }),
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

fn pool_symbol_anchor_styles(
    root: &Path,
    lib_id: Option<&str>,
) -> Result<HashMap<Uuid, SymbolPinStyle>> {
    let Some(lib_id) = lib_id else {
        return Ok(HashMap::new());
    };
    let Ok(symbol_id) = Uuid::parse_str(lib_id) else {
        return Ok(HashMap::new());
    };
    let model = ProjectResolver::new(root)
        .resolve()
        .with_context(|| format!("failed to resolve native project {}", root.display()))?;
    let Some(object) = model
        .objects
        .get(&symbol_id)
        .filter(|object| object.domain == "pool" && object.kind == "symbols")
    else {
        return Ok(HashMap::new());
    };
    let Some(shard) = model
        .source_shards
        .iter()
        .find(|shard| shard.shard_id == object.source_shard_id)
    else {
        return Ok(HashMap::new());
    };
    let symbol_value = model
        .materialized_source_shard_value_by_relative_path(&shard.relative_path)
        .with_context(|| format!("failed to materialize pool symbol {symbol_id}"))?;
    let symbol: Symbol = serde_json::from_value(symbol_value)
        .with_context(|| format!("failed to parse pool symbol {symbol_id}"))?;
    Ok(symbol
        .pin_anchors
        .into_iter()
        .map(|anchor| (anchor.pin, anchor.style))
        .collect())
}

pub(crate) fn query_native_project_texts(root: &Path) -> Result<Vec<serde_json::Value>> {
    let project = load_native_project_with_resolved_board(root)?;
    let model = ProjectResolver::new(&project.root)
        .resolve()
        .with_context(|| {
            format!(
                "failed to resolve native project {}",
                project.root.display()
            )
        })?;
    let mut texts = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_value = model
            .materialized_source_shard_value_by_relative_path(&format!("schematic/{relative_path}"))
            .with_context(|| format!("failed to materialize {}", path.display()))?;
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
    let project = load_native_project_with_resolved_board(root)?;
    let model = ProjectResolver::new(&project.root)
        .resolve()
        .with_context(|| {
            format!(
                "failed to resolve native project {}",
                project.root.display()
            )
        })?;
    let mut drawings = Vec::new();
    for (sheet_uuid, relative_path) in &project.schematic.sheets {
        let sheet_uuid = Uuid::parse_str(sheet_uuid)
            .with_context(|| format!("invalid sheet UUID key `{sheet_uuid}` in schematic root"))?;
        let path = project.root.join("schematic").join(relative_path);
        let sheet_value = model
            .materialized_source_shard_value_by_relative_path(&format!("schematic/{relative_path}"))
            .with_context(|| format!("failed to materialize {}", path.display()))?;
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
    let project = load_native_project_with_resolved_board(root)?;
    Ok(schematic_net_info(&build_native_project_schematic(
        &project,
    )?))
}

pub(crate) fn query_native_project_diagnostics(
    root: &Path,
) -> Result<Vec<ConnectivityDiagnosticInfo>> {
    let project = load_native_project_with_resolved_board(root)?;
    Ok(schematic_diagnostics(&build_native_project_schematic(
        &project,
    )?))
}

pub(crate) fn query_native_project_check(root: &Path) -> Result<CheckReport> {
    query_native_project_check_with_inputs(
        root,
        true,
        true,
        &[
            RuleType::Connectivity,
            RuleType::ClearanceCopper,
            RuleType::TrackWidth,
            RuleType::ViaHole,
            RuleType::ViaAnnularRing,
            RuleType::SilkClearance,
            RuleType::ProcessAperture,
        ],
    )
}

pub(crate) fn query_native_project_check_with_inputs(
    root: &Path,
    include_relationships: bool,
    include_erc: bool,
    drc_rules: &[RuleType],
) -> Result<CheckReport> {
    let project = load_native_project_with_resolved_board(root)?;
    let schematic = build_native_project_schematic(&project)?;
    let diagnostics = include_relationships
        .then(|| schematic_diagnostics(&schematic))
        .unwrap_or_default();
    let erc = include_erc
        .then(|| run_prechecks(&schematic))
        .unwrap_or_default();
    let drc = if drc_rules.is_empty() {
        Vec::new()
    } else {
        query_native_project_drc_with_rules(root, drc_rules)?.violations
    };
    Ok(CheckReport::Combined {
        summary: summarize_native_combined_checks(&diagnostics, &erc, &drc),
        diagnostics,
        erc,
        drc,
    })
}

fn summarize_native_combined_checks(
    diagnostics: &[ConnectivityDiagnosticInfo],
    erc: &[ErcFinding],
    drc: &[DrcViolation],
) -> CheckSummary {
    let mut summary = summarize_native_schematic_checks(diagnostics, erc);

    for violation in drc {
        if violation.waived {
            summary.waived += 1;
            continue;
        }
        match violation.severity {
            DrcSeverity::Error => summary.errors += 1,
            DrcSeverity::Warning => summary.warnings += 1,
        }
    }

    let mut drc_counts: BTreeMap<String, usize> = BTreeMap::new();
    for violation in drc {
        *drc_counts.entry(violation.code.clone()).or_default() += 1;
    }
    for (code, count) in drc_counts {
        if let Some(existing) = summary.by_code.iter_mut().find(|entry| entry.code == code) {
            existing.count += count;
        } else {
            summary.by_code.push(CheckCodeCount { code, count });
        }
    }
    summary.by_code.sort_by(|a, b| a.code.cmp(&b.code));
    summary.status = if summary.errors > 0 {
        CheckStatus::Error
    } else if summary.warnings > 0 {
        CheckStatus::Warning
    } else if summary.infos > 0 {
        CheckStatus::Info
    } else {
        CheckStatus::Ok
    };
    summary
}
