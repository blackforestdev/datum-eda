//! One complete root-sheet annotation census and one canonical Proposal batch.
use super::{
    context::{PreparedWrite, WriteProvenance, build_batch},
    schematic_annotation::{AnnotationChange, planning},
    schematic_selected_source::SelectedSchematicSource,
};
use crate::{
    error::EngineError,
    substrate::{DesignModel, ElectricalSelectionSnapshot, Operation, Proposal},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

#[path = "schematic_project_annotation_receipt.rs"]
mod receipt;
pub(super) use receipt::{summary, validate_recorded};

pub(crate) const REASON: &str = "Annotate schematic project";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectAnnotationSheet {
    pub sheet: Uuid,
    pub name: String,
    pub symbol_count: usize,
    pub changes: Vec<AnnotationChange>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectAnnotationSummary {
    pub sheets: Vec<ProjectAnnotationSheet>,
}
impl ProjectAnnotationSummary {
    pub fn change_count(&self) -> usize {
        self.sheets.iter().map(|s| s.changes.len()).sum()
    }
}
fn invalid(reason: impl Into<String>) -> EngineError {
    EngineError::Validation(reason.into())
}
pub(super) fn is_project_annotation(proposal: &Proposal) -> bool {
    proposal.batch.provenance.reason == REASON
}

pub fn preview(model: &DesignModel) -> Result<ProjectAnnotationSummary, EngineError> {
    Ok(plan(model)?.0)
}
fn plan(model: &DesignModel) -> Result<(ProjectAnnotationSummary, Vec<Operation>), EngineError> {
    let snapshot = ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(format!("Project annotation source unavailable: {e:?}")))?;
    let selected = SelectedSchematicSource::from_snapshot(&snapshot)?;
    if !selected.source.sheet_definitions.is_empty() || !selected.source.sheet_instances.is_empty()
    {
        return Err(invalid(
            "Annotate Project requires unique root sheets; hierarchy and repeated sheets are unsupported, so no sheet was changed",
        ));
    }
    if selected.source.sheets.is_empty() {
        return Err(invalid("Annotate Project requires a schematic sheet"));
    }
    let mut ids: Vec<_> = selected.source.sheets.keys().copied().collect();
    ids.sort();
    // Every sheet is qualified before numbering or operation construction. No
    // unsupported offscreen participant may silently disappear from the census.
    let sheets = ids
        .into_iter()
        .map(|id| planning::qualify(model, &selected, id))
        .collect::<Result<Vec<_>, _>>()?;
    let mut changes = planning::references(&sheets, &BTreeSet::new())?;
    let mut summaries = Vec::new();
    let mut operations = Vec::new();
    for sheet in sheets {
        let changes = changes.remove(&sheet.sheet).unwrap();
        operations.extend(planning::operations(model, &sheet, &changes)?);
        summaries.push(ProjectAnnotationSheet {
            sheet: sheet.sheet,
            name: sheet.name,
            symbol_count: sheet.symbols.len(),
            changes,
        });
    }
    Ok((ProjectAnnotationSummary { sheets: summaries }, operations))
}
pub(super) fn build(model: &DesignModel) -> Result<PreparedWrite, EngineError> {
    let (summary, operations) = plan(model)?;
    if summary.change_count() == 0 {
        return Err(invalid("Project is already annotated"));
    }
    Ok(PreparedWrite {
        batch: build_batch(
            model,
            WriteProvenance::new("datum-gui", crate::substrate::CommitSource::Manual, REASON),
            operations,
        )?,
        primary_object_id: Some(model.project.project_id),
    })
}

#[cfg(test)]
#[path = "schematic_project_annotation_tests.rs"]
mod tests;
