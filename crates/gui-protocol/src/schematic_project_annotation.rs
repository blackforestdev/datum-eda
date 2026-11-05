//! Project annotation uses the existing read-only fields table and Proposal lifecycle.
use super::{
    PlacementField, annotation::AnnotationEvidence, delete_review::DeleteReview,
    field_table::FieldTableDraft,
};
use crate::{ReviewWorkspaceState, single_line_editor::SingleLineEditor};
use eda_engine::{
    api::native_write::{
        schematic_delete_proposal::NativeProposalAction,
        schematic_field_batch::FieldRowEdit,
        schematic_project_annotation::{self, ProjectAnnotationSummary},
    },
    schematic::Schematic,
};
use std::collections::BTreeSet;

impl FieldTableDraft {
    pub(crate) fn for_project_annotation(
        summary: ProjectAnnotationSummary,
        source: &Schematic,
    ) -> Result<Self, String> {
        let mut rows = Vec::new();
        let mut sheets = BTreeSet::new();
        let mut symbols = BTreeSet::new();
        for sheet in &summary.sheets {
            let original = source
                .sheets
                .get(&sheet.sheet)
                .ok_or("Project annotation sheet unavailable")?;
            if !sheets.insert(sheet.sheet)
                || sheet.name != original.name
                || sheet.symbol_count != original.symbols.len()
            {
                return Err("Project annotation sheet identity or census changed".into());
            }
            for change in &sheet.changes {
                let symbol = original
                    .symbols
                    .get(&change.symbol)
                    .ok_or("Project annotation Symbol unavailable")?;
                if !symbols.insert(change.symbol)
                    || symbol.reference != change.previous_reference
                    || change.previous_reference == change.reference
                {
                    return Err("Project annotation before/after reference changed".into());
                }
                rows.push(FieldRowEdit {
                    symbol: change.symbol,
                    reference: change.reference.clone(),
                    value: symbol.value.clone(),
                });
            }
        }
        if sheets != source.sheets.keys().copied().collect() || rows.is_empty() {
            return Err("Project annotation requires a complete nonempty change review".into());
        }
        Ok(Self {
            reference_editors: vec![SingleLineEditor::default(); rows.len()],
            value_editors: vec![SingleLineEditor::default(); rows.len()],
            rows,
            focus_row: 0,
            field: PlacementField::Value,
            offset: 0,
            summary: None,
            project_annotation: Some(summary),
        })
    }
}
impl DeleteReview {
    pub fn capture_project_annotation(
        state: &ReviewWorkspaceState,
    ) -> Result<Option<Self>, String> {
        AnnotationEvidence::admit_project(state)?;
        let basis =
            crate::AuthoringViewBasis::capture_root_document(state, state.ui.layout.focused)?;
        let mut review = Self::capture_source_basis(state, basis)?;
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Current source unavailable")?;
        if model.project.project_id != review.basis.project
            || model.model_revision.0 != review.basis.revision
        {
            return Err("Project annotation source does not match the current review basis".into());
        }
        let summary = schematic_project_annotation::preview(model).map_err(|e| e.to_string())?;
        if summary.change_count() == 0 {
            return Ok(None);
        }
        let source = state
            .schematic_authoring
            .source
            .as_ref()
            .ok_or("Complete schematic source unavailable")?;
        review.fields = Some(FieldTableDraft::for_project_annotation(summary, source)?);
        review.pending = Some(NativeProposalAction::CreateProjectAnnotation {
            proposal_id: review.proposal_id,
        });
        Ok(Some(review))
    }
}
