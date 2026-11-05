//! Explicit multi-row Reference/Value edits planned against one immutable sheet.
use super::{
    context::{PreparedWrite, WriteProvenance, build_batch},
    schematic_assignment::repin_symbols,
    schematic_selected_source::SelectedSchematicSource,
    schematic_selection_edit::validate_field_values,
    schematic_symbol_payload::{member_payload, source_sheet_payload},
};
use crate::{
    error::EngineError,
    substrate::{DesignModel, ElectricalSelectionSnapshot, Operation},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub(crate) const REASON: &str = "Edit schematic symbol fields table";
#[path = "schematic_field_batch_receipt.rs"]
mod receipt;
pub(super) use receipt::{summary, validate_recorded};

pub(super) fn is_field_batch(proposal: &crate::substrate::Proposal) -> bool {
    proposal.batch.provenance.reason == REASON
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldRowEdit {
    pub symbol: Uuid,
    pub reference: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldRowChange {
    pub symbol: Uuid,
    pub previous_reference: String,
    pub previous_value: String,
    pub reference: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldBatchSummary {
    pub sheet: Uuid,
    pub instance_path: Vec<Uuid>,
    pub changes: Vec<FieldRowChange>,
}

fn invalid(message: impl Into<String>) -> EngineError {
    EngineError::Validation(message.into())
}

/// Invocation-only planning; no source is written by a draft or preview.
pub fn preview(
    model: &DesignModel,
    sheet: Uuid,
    path: &[Uuid],
    edits: &[FieldRowEdit],
) -> Result<FieldBatchSummary, EngineError> {
    Ok(plan(model, sheet, path, edits)?.0)
}

fn plan(
    model: &DesignModel,
    sheet: Uuid,
    path: &[Uuid],
    edits: &[FieldRowEdit],
) -> Result<(FieldBatchSummary, Vec<Operation>), EngineError> {
    if !path.is_empty() {
        return Err(invalid(
            "Symbol Fields Table requires the unique source-sheet view",
        ));
    }
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Symbol fields source unavailable: {e:?}")))?;
    let selected = SelectedSchematicSource::from_snapshot(&snapshot)?;
    let source = selected
        .source
        .sheets
        .get(&sheet)
        .filter(|s| s.uuid == sheet)
        .ok_or_else(|| invalid("Symbol fields sheet is unavailable"))?;
    let mut requested = BTreeMap::new();
    for edit in edits {
        validate_field_values(Some(&edit.reference), Some(&edit.value))?;
        if requested.insert(edit.symbol, edit).is_some() {
            return Err(invalid("Symbol Fields Table contains a duplicate row"));
        }
        if !source.symbols.contains_key(&edit.symbol)
            || selected.sheet("symbols", edit.symbol, path)? != sheet
        {
            return Err(invalid(
                "Symbol Fields Table row belongs to another sheet or occurrence",
            ));
        }
    }
    // Validate the simultaneous result, never intermediate per-row references.
    let mut references = BTreeSet::new();
    for (sheet_id, other_sheet) in &selected.source.sheets {
        for symbol in other_sheet.symbols.values() {
            let reference = if *sheet_id == sheet {
                requested
                    .get(&symbol.uuid)
                    .map(|e| e.reference.as_str())
                    .unwrap_or(&symbol.reference)
            } else {
                &symbol.reference
            };
            if !references.insert(reference) {
                return Err(invalid(format!(
                    "Final reference {reference} is already in use; no rows were applied"
                )));
            }
        }
    }
    let mut changes = Vec::new();
    let mut operations = Vec::new();
    let mut changed_symbols = Vec::new();
    let raw = requested
        .keys()
        .next()
        .map(|id| source_sheet_payload(model, sheet, *id))
        .transpose()?;
    for (id, edit) in requested {
        let original = &source.symbols[&id];
        if original.reference == edit.reference && original.value == edit.value {
            continue;
        }
        let mut payload = member_payload(raw.as_ref().unwrap(), "symbols", id, original)?;
        if payload.get("locked").and_then(Value::as_bool) == Some(true) {
            return Err(invalid(format!(
                "Symbol {} is locked; no rows were applied",
                original.reference
            )));
        }
        payload["reference"] = json!(edit.reference);
        payload["value"] = json!(edit.value);
        operations.push(Operation::GuardObjectRevision {
            object_id: id,
            expected_object_revision: model
                .objects
                .get(&id)
                .ok_or_else(|| invalid("Symbol object is unavailable"))?
                .object_revision,
        });
        operations.push(Operation::SetSchematicSymbol {
            sheet_id: sheet,
            symbol_id: id,
            symbol: payload,
        });
        changes.push(FieldRowChange {
            symbol: id,
            previous_reference: original.reference.clone(),
            previous_value: original.value.clone(),
            reference: edit.reference.clone(),
            value: edit.value.clone(),
        });
        changed_symbols.push(original.clone());
    }
    // Qualify and repin each owner once, retaining all unchanged sibling members.
    let bindings = repin_symbols(model, &changed_symbols)?;
    let mut guarded: BTreeSet<_> = changed_symbols.iter().map(|s| s.uuid).collect();
    for operation in &bindings {
        if let Operation::SetComponentInstance {
            component_instance_id,
            ..
        } = operation
        {
            for id in &model.component_instances[component_instance_id].placed_symbol_refs {
                if !source.symbols.contains_key(id) {
                    return Err(invalid(
                        "Symbol Fields Table requires same-sheet component members",
                    ));
                }
                if guarded.insert(*id) {
                    operations.push(Operation::GuardObjectRevision {
                        object_id: *id,
                        expected_object_revision: model.objects[id].object_revision,
                    });
                }
            }
        }
    }
    operations.extend(bindings);
    Ok((
        FieldBatchSummary {
            sheet,
            instance_path: path.to_vec(),
            changes,
        },
        operations,
    ))
}

pub(super) fn build(
    model: &DesignModel,
    sheet: Uuid,
    path: &[Uuid],
    edits: &[FieldRowEdit],
) -> Result<PreparedWrite, EngineError> {
    let (summary, operations) = plan(model, sheet, path, edits)?;
    if summary.changes.is_empty() {
        return Err(invalid("Symbol fields are unchanged"));
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
#[path = "schematic_field_batch_tests.rs"]
mod tests;
