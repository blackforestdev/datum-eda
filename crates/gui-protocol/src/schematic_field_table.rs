//! Editable rows configure the shared text controls and frozen Proposal lifecycle.
use super::{PlacementField, delete_review::DeleteReview};
use crate::{ReviewWorkspaceState, single_line_editor::SingleLineEditor};
use eda_engine::api::native_write::{
    schematic_delete_proposal::NativeProposalAction,
    schematic_field_batch::{self, FieldBatchSummary, FieldRowEdit},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldTableControl {
    Cell(usize, PlacementField),
    Page(usize),
    Review,
    Accept,
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldTableDraft {
    pub rows: Vec<FieldRowEdit>,
    pub reference_editors: Vec<SingleLineEditor>,
    pub value_editors: Vec<SingleLineEditor>,
    pub focus_row: usize,
    pub field: PlacementField,
    pub offset: usize,
    pub summary: Option<FieldBatchSummary>,
    pub project_annotation: Option<
        eda_engine::api::native_write::schematic_project_annotation::ProjectAnnotationSummary,
    >,
}
impl FieldTableDraft {
    fn capture(review: &DeleteReview) -> Result<Self, String> {
        let mut rows: Vec<_> = review
            .source
            .symbols
            .values()
            .map(|s| FieldRowEdit {
                symbol: s.uuid,
                reference: s.reference.clone(),
                value: s.value.clone(),
            })
            .collect();
        rows.sort_by(|a, b| (&a.reference, a.symbol).cmp(&(&b.reference, b.symbol)));
        if rows.is_empty() {
            return Err("This sheet has no Symbols to edit".into());
        }
        Ok(Self {
            reference_editors: vec![SingleLineEditor::default(); rows.len()],
            value_editors: vec![SingleLineEditor::default(); rows.len()],
            rows,
            focus_row: 0,
            field: PlacementField::Reference,
            offset: 0,
            summary: None,
            project_annotation: None,
        })
    }
    pub fn editable(&self) -> bool {
        self.summary.is_none() && self.project_annotation.is_none()
    }
    pub fn focus(&mut self, row: usize, field: PlacementField, visible: usize) {
        self.focus_row = row.min(self.rows.len().saturating_sub(1));
        self.field = field;
        let visible = visible.max(1);
        if self.focus_row < self.offset {
            self.offset = self.focus_row;
        }
        if self.focus_row >= self.offset + visible {
            self.offset = self.focus_row + 1 - visible;
        }
    }
    pub fn cell(&self, row: usize, field: PlacementField) -> Option<(String, SingleLineEditor)> {
        let value = self.rows.get(row)?;
        let (mut text, editor) = match field {
            PlacementField::Reference => (value.reference.clone(), self.reference_editors[row]),
            PlacementField::Value => (value.value.clone(), self.value_editors[row]),
        };
        if let Some(change) = self
            .summary
            .as_ref()
            .and_then(|s| s.changes.iter().find(|c| c.symbol == value.symbol))
        {
            text = match field {
                PlacementField::Reference => {
                    format!("{} → {}", change.previous_reference, change.reference)
                }
                PlacementField::Value => {
                    format!("{:?} → {:?}", change.previous_value, change.value)
                }
            };
        }
        if let Some(summary) = &self.project_annotation {
            let (sheet, change) = summary.sheets.iter().find_map(|sheet| {
                sheet
                    .changes
                    .iter()
                    .find(|c| c.symbol == value.symbol)
                    .map(|c| (sheet, c))
            })?;
            text = match field {
                PlacementField::Reference => {
                    if summary
                        .sheets
                        .iter()
                        .filter(|s| s.name == sheet.name)
                        .count()
                        > 1
                    {
                        format!("{} ({})", sheet.name, sheet.sheet)
                    } else {
                        sheet.name.clone()
                    }
                }
                PlacementField::Value => {
                    format!("{} → {}", change.previous_reference, change.reference)
                }
            };
        }
        Some((text, editor))
    }
    pub fn store(
        &mut self,
        row: usize,
        field: PlacementField,
        value: String,
        editor: SingleLineEditor,
    ) {
        let editable = self.editable();
        let Some(target) = self.rows.get_mut(row) else {
            return;
        };
        match field {
            PlacementField::Reference => {
                if editable {
                    target.reference = value;
                }
                self.reference_editors[row] = editor;
            }
            PlacementField::Value => {
                if editable {
                    target.value = value;
                }
                self.value_editors[row] = editor;
            }
        }
    }
}
impl DeleteReview {
    pub fn capture_field_table(state: &ReviewWorkspaceState) -> Result<Self, String> {
        super::annotation::AnnotationEvidence::admit(state)?;
        let mut review = Self::capture_source(state)?;
        review.fields = Some(FieldTableDraft::capture(&review)?);
        Ok(review)
    }
    pub fn stage_field_table(&mut self, state: &ReviewWorkspaceState) -> Result<bool, String> {
        if !self.current(state) || self.pending.is_some() || self.response.is_some() {
            return Err(
                "Source changed or a Proposal is pending; cancel and reopen the table".into(),
            );
        }
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Current source unavailable")?;
        let table = self
            .fields
            .as_mut()
            .ok_or("Symbol Fields Table is unavailable")?;
        if !table.editable() {
            return Err("This table is already in read-only Proposal review".into());
        }
        let path = &self
            .basis
            .document
            .as_ref()
            .ok_or("Qualified sheet unavailable")?
            .instance_path;
        let summary = schematic_field_batch::preview(model, self.basis.source, path, &table.rows)
            .map_err(|e| e.to_string())?;
        if summary.changes.is_empty() {
            return Ok(false);
        }
        self.pending = Some(NativeProposalAction::CreateFieldBatch {
            proposal_id: self.proposal_id,
            sheet: self.basis.source,
            instance_path: path.clone(),
            edits: table.rows.clone(),
        });
        table.summary = Some(summary);
        table.reference_editors.fill(SingleLineEditor::default());
        table.value_editors.fill(SingleLineEditor::default());
        table.offset = 0;
        table.focus_row = 0;
        table.field = PlacementField::Reference;
        self.refusal = None;
        Ok(true)
    }
}
