//! Recorded annotation family and immutable preimages for shared Proposal recovery.
use super::*;
struct Recorded<'a> {
    sheet: Uuid,
    symbols: BTreeMap<Uuid, &'a Value>,
    symbol_count: usize,
}
fn recorded(proposal: &Proposal) -> Result<Recorded<'_>, EngineError> {
    let mut sheet = None;
    let mut symbols = BTreeMap::new();
    let mut guards = BTreeSet::new();
    let mut components = BTreeSet::new();
    let mut owners = BTreeSet::new();
    for op in &proposal.batch.operations {
        match op {
            Operation::GuardObjectRevision { object_id, .. } => {
                if !guards.insert(*object_id) {
                    return Err(invalid("Annotation contains duplicate guards"));
                }
            }
            Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                symbol,
            } => {
                let typed: PlacedSymbol = serde_json::from_value(symbol.clone())?;
                prefix(&typed.reference)?;
                if typed.uuid != *symbol_id
                    || symbols.insert(*symbol_id, symbol).is_some()
                    || sheet.replace(*sheet_id).is_some_and(|old| old != *sheet_id)
                {
                    return Err(invalid("Annotation changes duplicate or unrelated Symbols"));
                }
            }
            Operation::SetComponentInstance {
                component_instance_id,
                previous_component_instance: old,
                component_instance: next,
            } => {
                let refs = old["placed_symbol_refs"]
                    .as_array()
                    .ok_or_else(|| invalid("Annotation binding references unavailable"))?;
                if refs.len() != 1
                    || old["placed_package_refs"]
                        .as_array()
                        .is_none_or(|v| !v.is_empty())
                {
                    return Err(invalid("Annotation binding is not schematic-only"));
                }
                let symbol: Uuid = serde_json::from_value(refs[0]["object_id"].clone())?;
                let mut expected = old.clone();
                expected["object_revision"] = json!(
                    old["object_revision"]
                        .as_u64()
                        .and_then(|v| v.checked_add(1))
                        .ok_or_else(|| invalid("Annotation component epoch unavailable"))?
                );
                expected["placed_symbol_refs"][0]["object_revision"] = json!(
                    refs[0]["object_revision"]
                        .as_u64()
                        .and_then(|v| v.checked_add(1))
                        .ok_or_else(|| invalid("Annotation Symbol epoch unavailable"))?
                );
                if old["uuid"] != json!(component_instance_id)
                    || expected != *next
                    || !components.insert(*component_instance_id)
                    || !owners.insert(symbol)
                {
                    return Err(invalid("Annotation changes unrelated binding data"));
                }
            }
            _ => return Err(invalid("Annotation contains unrelated operations")),
        }
    }
    if !is_annotation(proposal)
        || symbols.is_empty()
        || !owners.iter().all(|id| symbols.contains_key(id))
        || !symbols
            .keys()
            .chain(&components)
            .all(|id| guards.contains(id))
        || !components.is_disjoint(&symbols.keys().copied().collect())
    {
        return Err(invalid(
            "Annotation family or complete source guards unavailable",
        ));
    }
    Ok(Recorded {
        sheet: sheet.unwrap(),
        symbols,
        symbol_count: guards.len() - components.len(),
    })
}
pub(in crate::api::native_write) fn validate_recorded(
    proposal: &Proposal,
) -> Result<(), EngineError> {
    recorded(proposal).map(|_| ())
}
pub(in crate::api::native_write) fn summary(
    model: &DesignModel,
    proposal: &Proposal,
    current: bool,
) -> Result<AnnotationSummary, EngineError> {
    let recorded = recorded(proposal)?;
    if current {
        if proposal.prepared_against != model.model_revision
            || proposal.batch.expected_model_revision.as_ref() != Some(&model.model_revision)
        {
            return Err(invalid("Design changed; prepare sheet annotation again"));
        }
        let rebuilt = build(model, recorded.sheet, &[])?;
        if rebuilt.batch.operations != proposal.batch.operations {
            return Err(invalid(
                "Annotation differs from the complete current sheet",
            ));
        }
    }
    let old_payloads: BTreeMap<_, _> = if let Some(id) = proposal.applied_transaction_id {
        let tx = model
            .journal
            .iter()
            .find(|tx| tx.transaction_id == id)
            .ok_or_else(|| invalid("Annotation transaction unavailable"))?;
        if !proposal
            .batch
            .operations
            .iter()
            .filter(|op| !matches!(op, Operation::GuardObjectRevision { .. }))
            .all(|op| tx.operations.contains(op))
        {
            return Err(invalid(
                "Annotation transaction does not match its proposal",
            ));
        }
        let mut old = BTreeMap::new();
        for op in &tx.inverse_operations {
            if let Operation::SetSchematicSymbol {
                sheet_id,
                symbol_id,
                symbol,
            } = op
            {
                if *sheet_id != recorded.sheet
                    || !recorded.symbols.contains_key(symbol_id)
                    || old.insert(*symbol_id, symbol.clone()).is_some()
                {
                    return Err(invalid("Annotation inverse source does not match"));
                }
            }
        }
        old
    } else {
        if proposal.prepared_against != model.model_revision {
            return Err(invalid("Annotation source revision unavailable"));
        }
        let source = source_sheet_payload(
            model,
            recorded.sheet,
            *recorded.symbols.keys().next().unwrap(),
        )?;
        recorded
            .symbols
            .keys()
            .map(|id| (*id, source["symbols"][id.to_string()].clone()))
            .collect()
    };
    let mut changes = vec![];
    for (id, next) in recorded.symbols {
        let old = old_payloads
            .get(&id)
            .ok_or_else(|| invalid("Annotation preimage unavailable"))?;
        let typed: PlacedSymbol = serde_json::from_value(old.clone())?;
        let reference = next["reference"]
            .as_str()
            .ok_or_else(|| invalid("Annotation reference unavailable"))?;
        let mut expected = old.clone();
        expected["reference"] = json!(reference);
        if typed.uuid != id || typed.reference == reference || expected != *next {
            return Err(invalid("Annotation changes unrelated Symbol source"));
        }
        changes.push(AnnotationChange {
            symbol: id,
            previous_reference: typed.reference,
            reference: reference.into(),
        });
    }
    Ok(AnnotationSummary {
        sheet: recorded.sheet,
        instance_path: vec![],
        symbol_count: recorded.symbol_count,
        changes,
    })
}
