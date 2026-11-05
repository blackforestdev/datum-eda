//! Shared census, deterministic numbering and operation planning for annotation scopes.
use super::*;

pub(in crate::api::native_write) struct QualifiedSheet {
    pub sheet: Uuid,
    pub name: String,
    pub symbols: Vec<PlacedSymbol>,
    payloads: BTreeMap<Uuid, Value>,
}

pub(in crate::api::native_write) fn qualify(
    model: &DesignModel,
    selected: &SelectedSchematicSource,
    sheet: Uuid,
) -> Result<QualifiedSheet, EngineError> {
    let source = selected
        .source
        .sheets
        .get(&sheet)
        .filter(|s| s.uuid == sheet)
        .ok_or_else(|| invalid("Annotation sheet is unavailable"))?;
    if source.symbols.is_empty() {
        return Ok(QualifiedSheet {
            sheet,
            name: source.name.clone(),
            symbols: vec![],
            payloads: BTreeMap::new(),
        });
    }
    super::super::schematic_assignment::component::current_set(
        model,
        &source.symbols.keys().copied().collect(),
    )?;
    let mut symbols: Vec<_> = source.symbols.values().cloned().collect();
    symbols.sort_by_key(|s| s.uuid);
    let raw = symbols
        .first()
        .map(|s| source_sheet_payload(model, sheet, s.uuid))
        .transpose()?;
    let mut payloads = BTreeMap::new();
    for symbol in &symbols {
        prefix(&symbol.reference)?;
        if selected.sheet("symbols", symbol.uuid, &[])? != sheet {
            return Err(invalid("Annotation Symbol belongs to a different sheet"));
        }
        let payload = member_payload(raw.as_ref().unwrap(), "symbols", symbol.uuid, symbol)?;
        if payload.get("locked").and_then(Value::as_bool) == Some(true) {
            return Err(invalid(
                "Annotation cannot change a sheet containing locked Symbols",
            ));
        }
        payloads.insert(symbol.uuid, payload);
    }
    Ok(QualifiedSheet {
        sheet,
        name: source.name.clone(),
        symbols,
        payloads,
    })
}

pub(in crate::api::native_write) fn references(
    sheets: &[QualifiedSheet],
    reserved: &BTreeSet<String>,
) -> Result<BTreeMap<Uuid, Vec<AnnotationChange>>, EngineError> {
    let mut groups: BTreeMap<&str, Vec<(Uuid, &PlacedSymbol)>> = BTreeMap::new();
    let mut changes: BTreeMap<_, Vec<_>> = sheets.iter().map(|s| (s.sheet, vec![])).collect();
    for sheet in sheets {
        for symbol in &sheet.symbols {
            groups
                .entry(prefix(&symbol.reference)?)
                .or_default()
                .push((sheet.sheet, symbol));
        }
    }
    let mut final_refs = reserved.clone();
    for (prefix, mut symbols) in groups {
        symbols.sort_by_key(|(sheet, s)| (*sheet, s.position.y, s.position.x, s.uuid));
        let mut number = 1u64;
        for (sheet, symbol) in symbols {
            let reference = loop {
                let candidate = format!("{prefix}{number}");
                number = number
                    .checked_add(1)
                    .ok_or_else(|| invalid("Annotation number overflow"))?;
                if !reserved.contains(&candidate) {
                    break candidate;
                }
            };
            if !final_refs.insert(reference.clone()) {
                return Err(invalid("Annotation final reference set is not unique"));
            }
            if reference != symbol.reference {
                changes.get_mut(&sheet).unwrap().push(AnnotationChange {
                    symbol: symbol.uuid,
                    previous_reference: symbol.reference.clone(),
                    reference,
                });
            }
        }
    }
    for changes in changes.values_mut() {
        changes.sort_by_key(|c| c.symbol);
    }
    Ok(changes)
}

pub(in crate::api::native_write) fn operations(
    model: &DesignModel,
    sheet: &QualifiedSheet,
    changes: &[AnnotationChange],
) -> Result<Vec<Operation>, EngineError> {
    let mut operations = Vec::new();
    for symbol in &sheet.symbols {
        operations.push(Operation::GuardObjectRevision {
            object_id: symbol.uuid,
            expected_object_revision: model
                .objects
                .get(&symbol.uuid)
                .ok_or_else(|| invalid("Annotation Symbol object is unavailable"))?
                .object_revision,
        });
    }
    for change in changes {
        let mut payload = sheet.payloads[&change.symbol].clone();
        payload["reference"] = json!(change.reference);
        operations.push(Operation::SetSchematicSymbol {
            sheet_id: sheet.sheet,
            symbol_id: change.symbol,
            symbol: payload,
        });
    }
    let changed: BTreeSet<_> = changes.iter().map(|c| c.symbol).collect();
    let symbols: Vec<_> = sheet
        .symbols
        .iter()
        .filter(|s| changed.contains(&s.uuid))
        .cloned()
        .collect();
    operations.extend(repin_symbols(model, &symbols)?);
    Ok(operations)
}
