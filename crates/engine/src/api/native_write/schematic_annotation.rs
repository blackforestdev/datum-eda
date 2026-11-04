//! Deterministic complete-sheet reference annotation through manual Proposals.
use super::{
    context::{PreparedWrite, WriteProvenance, build_batch},
    schematic_assignment::repin_symbols,
    schematic_selected_source::SelectedSchematicSource,
    schematic_symbol_payload::{member_payload, source_sheet_payload},
};
use crate::{
    error::EngineError,
    schematic::PlacedSymbol,
    substrate::{DesignModel, ElectricalSelectionSnapshot, Operation, Proposal},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
#[path = "schematic_annotation_receipt.rs"]
mod receipt;
pub(super) use receipt::{summary, validate_recorded};
pub(super) const REASON: &str = "Annotate schematic sheet";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationChange {
    pub symbol: Uuid,
    pub previous_reference: String,
    pub reference: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationSummary {
    pub sheet: Uuid,
    pub instance_path: Vec<Uuid>,
    pub symbol_count: usize,
    pub changes: Vec<AnnotationChange>,
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}
pub(super) fn is_annotation(proposal: &Proposal) -> bool {
    proposal.batch.provenance.reason == REASON
}
/// Invocation-only: capture complete current source and ownership, never paint-time I/O.
pub fn preview(
    model: &DesignModel,
    sheet: Uuid,
    path: &[Uuid],
) -> Result<AnnotationSummary, EngineError> {
    Ok(plan(model, sheet, path)?.0)
}
fn prefix(reference: &str) -> Result<&str, EngineError> {
    if reference.is_empty()
        || reference
            .chars()
            .any(|c| c.is_whitespace() || c.is_control())
    {
        return Err(invalid(
            "References must be nonempty with no whitespace or control characters",
        ));
    }
    let prefix = reference
        .strip_suffix('?')
        .unwrap_or_else(|| reference.trim_end_matches(|c: char| c.is_ascii_digit()));
    if prefix.is_empty() || prefix.contains('?') {
        return Err(invalid(
            "Annotation requires an unambiguous nonempty reference prefix",
        ));
    }
    Ok(prefix)
}
fn plan(
    model: &DesignModel,
    sheet: Uuid,
    path: &[Uuid],
) -> Result<(AnnotationSummary, Vec<Operation>), EngineError> {
    if !path.is_empty() {
        return Err(invalid("Annotation requires the unique source-sheet view"));
    }
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Annotation source unavailable: {e:?}")))?;
    let selected = SelectedSchematicSource::from_snapshot(&snapshot)?;
    let source = selected
        .source
        .sheets
        .get(&sheet)
        .filter(|s| s.uuid == sheet)
        .ok_or_else(|| invalid("Annotation sheet is unavailable"))?;
    if source.symbols.is_empty() {
        return Ok((
            AnnotationSummary {
                sheet,
                instance_path: vec![],
                symbol_count: 0,
                changes: vec![],
            },
            vec![],
        ));
    }
    for symbol in source.symbols.keys() {
        super::schematic_assignment::component::current(model, *symbol)?;
    }
    let mut reserved = BTreeSet::new();
    for (sheet_id, s) in &selected.source.sheets {
        for symbol in s.symbols.values() {
            prefix(&symbol.reference)?;
            if *sheet_id != sheet && !reserved.insert(symbol.reference.clone()) {
                return Err(invalid(
                    "References reserved outside this sheet are ambiguous",
                ));
            }
        }
    }
    let mut originals: Vec<_> = source.symbols.values().cloned().collect();
    originals.sort_by_key(|s| s.uuid);
    let raw = source_sheet_payload(model, sheet, originals[0].uuid)?;
    let mut payloads = BTreeMap::new();
    let mut groups: BTreeMap<String, Vec<&PlacedSymbol>> = BTreeMap::new();
    for symbol in &originals {
        if selected.sheet("symbols", symbol.uuid, path)? != sheet {
            return Err(invalid("Annotation Symbol belongs to a different sheet"));
        }
        let payload = member_payload(&raw, "symbols", symbol.uuid, symbol)?;
        if payload.get("locked").and_then(Value::as_bool) == Some(true) {
            return Err(invalid(
                "Annotation cannot change a sheet containing locked Symbols",
            ));
        }
        payloads.insert(symbol.uuid, payload);
        groups
            .entry(prefix(&symbol.reference)?.into())
            .or_default()
            .push(symbol);
    }
    // Qualify every owner, even when its reference will remain unchanged.
    let component_ops = repin_symbols(model, &originals)?;
    let mut changes = vec![];
    let mut final_refs = reserved.clone();
    for (prefix, mut symbols) in groups {
        symbols.sort_by_key(|s| (s.position.y, s.position.x, s.uuid));
        let mut number = 1u64;
        for symbol in symbols {
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
                changes.push(AnnotationChange {
                    symbol: symbol.uuid,
                    previous_reference: symbol.reference.clone(),
                    reference,
                });
            }
        }
    }
    // Stable ID ordering also makes recovered applied receipts deterministic.
    changes.sort_by_key(|c| c.symbol);
    let mut operations = vec![];
    for symbol in &originals {
        operations.push(Operation::GuardObjectRevision {
            object_id: symbol.uuid,
            expected_object_revision: model
                .objects
                .get(&symbol.uuid)
                .ok_or_else(|| invalid("Annotation Symbol object is unavailable"))?
                .object_revision,
        });
    }
    let changed: BTreeSet<_> = changes.iter().map(|c| c.symbol).collect();
    for change in &changes {
        let mut payload = payloads.remove(&change.symbol).unwrap();
        payload["reference"] = json!(change.reference);
        operations.push(Operation::SetSchematicSymbol {
            sheet_id: sheet,
            symbol_id: change.symbol,
            symbol: payload,
        });
    }
    for op in component_ops {
        let Operation::SetComponentInstance {
            ref component_instance,
            ..
        } = op
        else {
            return Err(invalid(
                "Annotation ownership builder emitted an unexpected operation",
            ));
        };
        let id: Uuid = serde_json::from_value(
            component_instance["placed_symbol_refs"][0]["object_id"].clone(),
        )?;
        if changed.contains(&id) {
            operations.push(op);
        }
    }
    Ok((
        AnnotationSummary {
            sheet,
            instance_path: vec![],
            symbol_count: originals.len(),
            changes,
        },
        operations,
    ))
}
pub(super) fn build(
    model: &DesignModel,
    sheet: Uuid,
    path: &[Uuid],
) -> Result<PreparedWrite, EngineError> {
    let (summary, operations) = plan(model, sheet, path)?;
    if summary.changes.is_empty() {
        return Err(invalid("Sheet is already annotated"));
    }
    Ok(PreparedWrite {
        batch: build_batch(
            model,
            WriteProvenance::new("datum-gui", crate::substrate::CommitSource::Manual, REASON),
            operations,
        )?,
        primary_object_id: Some(sheet),
    })
}
#[cfg(test)]
#[path = "schematic_annotation_tests.rs"]
pub(crate) mod tests;
