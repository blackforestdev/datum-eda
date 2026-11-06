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
pub(super) fn prefix(reference: &str) -> Result<&str, EngineError> {
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
#[path = "schematic_annotation_plan.rs"]
pub(super) mod planning;

fn plan(
    model: &DesignModel,
    sheet: Uuid,
    path: &[Uuid],
) -> Result<(AnnotationSummary, Vec<Operation>), EngineError> {
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Annotation source unavailable: {e:?}")))?;
    let selected = SelectedSchematicSource::from_snapshot(&snapshot)?;
    let qualified = planning::qualify_at(model, &selected, sheet, path)?;
    if qualified.symbols.is_empty() {
        return Ok((
            AnnotationSummary {
                sheet,
                instance_path: path.to_vec(),
                symbol_count: 0,
                changes: vec![],
            },
            vec![],
        ));
    }
    let mut reserved = BTreeSet::new();
    for (sheet_id, source) in &selected.source.sheets {
        for symbol in source.symbols.values() {
            prefix(&symbol.reference)?;
            if *sheet_id != sheet && !reserved.insert(symbol.reference.clone()) {
                return Err(invalid(
                    "References reserved outside this sheet are ambiguous",
                ));
            }
        }
    }
    let sheets = [qualified];
    let changes = planning::references(&sheets, &reserved)?
        .remove(&sheet)
        .unwrap();
    let operations = planning::operations(model, &sheets[0], &changes)?;
    Ok((
        AnnotationSummary {
            sheet,
            instance_path: path.to_vec(),
            symbol_count: sheets[0].symbols.len(),
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
#[path = "schematic_annotation_multigate_tests.rs"]
mod multigate_tests;
#[cfg(test)]
#[path = "schematic_annotation_tests.rs"]
pub(crate) mod tests;
