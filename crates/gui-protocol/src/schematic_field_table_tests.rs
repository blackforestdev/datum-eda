//! Projection and exact-receipt safety; engine fixtures prove native eligibility.
use super::*;
use crate::schematic_authoring::{PlacementField, field_table::FieldTableDraft};
use crate::single_line_editor::SingleLineEditor;
use eda_engine::api::native_write::schematic_field_batch::{
    FieldBatchSummary, FieldRowChange, FieldRowEdit,
};

#[test]
fn field_table_review_is_readonly_and_rejects_changed_or_partial_receipts() {
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let state = workspace(&model, &fixture.0);
    let mut review = DeleteReview::capture(&state).unwrap();
    review.targets.clear();
    review.pending = None;
    let rows: Vec<_> = review
        .source
        .symbols
        .values()
        .map(|s| FieldRowEdit {
            symbol: s.uuid,
            reference: s.reference.clone(),
            value: s.value.clone(),
        })
        .collect();
    let mut table = FieldTableDraft {
        reference_editors: vec![SingleLineEditor::default(); rows.len()],
        value_editors: vec![SingleLineEditor::default(); rows.len()],
        rows,
        focus_row: 0,
        field: PlacementField::Reference,
        offset: 0,
        summary: None,
        project_annotation: None,
    };
    let old = table.rows[0].clone();
    table.store(
        0,
        PlacementField::Value,
        "".into(),
        SingleLineEditor::default(),
    );
    assert_eq!(table.rows[0].value, "");
    let last = table.rows.len() - 1;
    table.focus(last, PlacementField::Value, 1);
    assert_eq!(table.offset, last);
    table.focus(0, PlacementField::Reference, 1);
    assert_eq!(table.offset, 0);
    let summary = FieldBatchSummary {
        sheet: review.basis.source,
        instance_path: vec![],
        changes: vec![FieldRowChange {
            symbol: old.symbol,
            previous_reference: old.reference.clone(),
            previous_value: old.value.clone(),
            reference: old.reference,
            value: "".into(),
        }],
    };
    table.summary = Some(summary.clone());
    let frozen = table.rows.clone();
    table.store(
        0,
        PlacementField::Value,
        "unexpected mutation".into(),
        SingleLineEditor::default(),
    );
    assert_eq!(table.rows, frozen);
    assert_eq!(
        table.cell(0, PlacementField::Value).unwrap().0,
        format!("{:?} → {:?}", old.value, "")
    );
    review.fields = Some(table);
    let mut receipt = response(&review);
    receipt.fields = Some(summary);
    let mut mismatch = receipt.clone();
    mismatch.fields.as_mut().unwrap().changes[0].value = "unexpected".into();
    assert!(review.observe(mismatch).is_err());
    let mut partial = receipt.clone();
    partial.fields.as_mut().unwrap().changes.clear();
    assert!(review.observe(partial).is_err());
    assert!(review.response.is_none());
    review.observe(receipt).unwrap();
    assert!(review.ready());
}
