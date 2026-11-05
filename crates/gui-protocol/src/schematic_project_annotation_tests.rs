//! Pure table/receipt projection checks supplement the canonical engine and GUI proofs.
use super::*;
use crate::schematic_authoring::{PlacementField, field_table::FieldTableDraft};
use crate::single_line_editor::SingleLineEditor;
use eda_engine::api::native_write::{
    schematic_annotation::AnnotationChange,
    schematic_project_annotation::{ProjectAnnotationSheet, ProjectAnnotationSummary},
};

#[test]
fn project_annotation_table_covers_offscreen_rows_and_refuses_partial_receipts() {
    let fixture = isolated_fixture();
    let model = eda_engine::substrate::ProjectResolver::new(&fixture.0)
        .resolve()
        .unwrap();
    let state = workspace(&model, &fixture.0);
    let mut review = DeleteReview::capture(&state).unwrap();
    let mut source = state.schematic_authoring.source.clone().unwrap();
    // A second typed projection exercises duplicate sheet labels and offscreen
    // rows. Native multi-sheet creation/eligibility is proven by engine fixtures.
    let mut second = review.source.clone();
    second.uuid = Uuid::new_v4();
    second.symbols = second
        .symbols
        .into_values()
        .map(|mut s| {
            s.uuid = Uuid::new_v4();
            (s.uuid, s)
        })
        .collect();
    source.sheets.insert(second.uuid, second);
    let mut sheets: Vec<_> = source
        .sheets
        .values()
        .map(|s| ProjectAnnotationSheet {
            sheet: s.uuid,
            name: s.name.clone(),
            symbol_count: s.symbols.len(),
            changes: s
                .symbols
                .values()
                .map(|symbol| AnnotationChange {
                    symbol: symbol.uuid,
                    previous_reference: symbol.reference.clone(),
                    reference: format!("{}9", symbol.reference),
                })
                .collect(),
        })
        .collect();
    sheets.sort_by_key(|s| s.sheet);
    let summary = ProjectAnnotationSummary { sheets };
    let mut table = FieldTableDraft::for_project_annotation(summary.clone(), &source).unwrap();
    assert_eq!(table.rows.len(), summary.change_count());
    assert!(!table.editable());
    let last = table.rows.len() - 1;
    table.focus(last, PlacementField::Value, 1);
    assert_eq!(table.offset, last);
    let label = table.cell(last, PlacementField::Reference).unwrap().0;
    assert!(
        label.contains('('),
        "duplicate names expose qualified source identity"
    );
    let value = table.cell(last, PlacementField::Value).unwrap().0;
    assert!(value.contains(" → "));
    let frozen = table.rows.clone();
    table.store(
        last,
        PlacementField::Value,
        "unexpected".into(),
        SingleLineEditor::default(),
    );
    assert_eq!(table.rows, frozen);
    let mut incomplete = summary.clone();
    incomplete.sheets.pop();
    assert!(FieldTableDraft::for_project_annotation(incomplete.clone(), &source).is_err());
    review.targets.clear();
    review.pending = None;
    review.fields = Some(table);
    let mut receipt = response(&review);
    receipt.project_annotation = Some(summary);
    let mut bad = receipt.clone();
    bad.project_annotation = Some(incomplete);
    assert!(review.observe(bad).is_err());
    assert!(review.response.is_none());
    review.observe(receipt).unwrap();
    assert!(review.ready());
}

#[test]
fn project_annotation_accepts_navigated_root_without_enabling_sheet_local_edits() {
    use super::gate_placement::{selected, setup};
    use crate::AuthoringViewBasis;
    use crate::schematic_authoring::annotation::AnnotationEvidence;
    use eda_engine::api::native_write::{
        commit_prepared,
        context::WriteProvenance,
        registry::{NativeWriteContext, find_native_write_verb},
    };
    let (fixture, mut model, first) = setup();
    let provenance = WriteProvenance::new(
        "test",
        eda_engine::substrate::CommitSource::Test,
        "second root navigation",
    );
    let write = (find_native_write_verb("datum.schematic.create_sheet")
        .unwrap()
        .build)(
        &NativeWriteContext {
            model: &model,
            project_root: &fixture.0,
        },
        provenance,
        serde_json::json!({"sheet":Uuid::new_v4(),"name":"Z auxiliary"}),
    )
    .unwrap();
    let second = write.primary_object_id.unwrap();
    commit_prepared(&mut model, &fixture.0, write).unwrap();
    let mut state = selected(&model, &fixture.0, first);
    let primary = state.schematic_authoring.sheet;
    assert_ne!(primary, Some(second));
    let document = state
        .schematic_documents
        .rows
        .iter()
        .find(|r| r.key.source_sheet == second)
        .unwrap()
        .key
        .clone();
    state
        .schematic_documents
        .prepare_scene(&document, &fixture.0, &model)
        .unwrap();
    let pane = state
        .schematic_documents
        .open_beside(&mut state.ui.layout, &document)
        .unwrap();
    assert_eq!(state.ui.layout.focused, pane);
    assert_eq!(state.schematic_authoring.sheet, primary);
    assert!(AuthoringViewBasis::capture(&state, pane).is_err());
    assert!(AnnotationEvidence::admit(&state).is_err());
    assert!(AnnotationEvidence::admit_project(&state).is_ok());
    let basis = AuthoringViewBasis::capture_root_document(&state, pane).unwrap();
    assert_eq!(basis.source, second);
    assert_eq!(basis.document.as_ref(), Some(&document));
    let captured = DeleteReview::capture_source_basis(&state, basis).unwrap();
    assert!(captured.current(&state));
    assert!(DeleteReview::capture_project_annotation(&state).is_ok());
    let menu = crate::gui_menu_model::default_gui_menu_model().unwrap();
    let action = menu.marking_menus["schematic.empty"]
        .overflow
        .iter()
        .find(|item| item.gui_local.as_deref() == Some("schematic.annotation.project"))
        .unwrap();
    assert!(action.unavailable_reason(&state).is_none());
    for mode in [
        "revision",
        "project",
        "document",
        "occurrence",
        "focus",
        "scene",
    ] {
        let mut wrong = state.clone();
        match mode {
            "revision" => wrong.scene.source_revision.push_str("-stale"),
            "project" => wrong.schematic_documents.project = Some(Uuid::new_v4()),
            "document" => {
                wrong.schematic_documents.panes.remove(&pane);
            }
            "occurrence" => wrong
                .schematic_documents
                .panes
                .get_mut(&pane)
                .unwrap()
                .instance_path
                .push(Uuid::new_v4()),
            "focus" => wrong.ui.layout.focused = PaneId(u32::MAX),
            "scene" => wrong.schematic_documents = Default::default(),
            _ => unreachable!(),
        }
        assert!(
            AuthoringViewBasis::capture_root_document(&wrong, pane).is_err(),
            "{mode}"
        );
        assert!(AnnotationEvidence::admit_project(&wrong).is_err(), "{mode}");
        assert!(
            DeleteReview::capture_project_annotation(&wrong).is_err(),
            "{mode}"
        );
    }
}
