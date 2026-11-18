//! Form and lifecycle controls; these are not native GUI acceptance evidence.
use super::*;
use eda_engine::{
    api::native_write::genesis::{GenesisSpec, bootstrap_native_project},
    substrate::ProjectResolver,
};
use std::sync::Arc;

fn record() -> Value {
    json!({"name":"Resistor footprint", "revision":{"asset_id":Uuid::new_v4(),"revision_id":"test-base"},
        "provenance":{"schema_version":1,"authors":[{"recorded_id":"earlier-author"}],
            "created_at":"2020-01-01T00:00:00Z","origin_kind":"import","source_assets":[],
            "change_actor":null,"changed_at":null,"change_summary":null,
            "rights":{"license_expression":"LicenseRef-Original","attribution":["First credit","Second credit"],
                "restrictions":["Preserve notice"],"evidence":[{"title":"Original license evidence"}]},
            "sources":[{"title":"Retained source","issuer":"Original issuer","document_revision":"v1",
                "locator":"source/path","access_note":"Exact source","retained_blob":"sha256:retained"},
                {"title":"Linked source","issuer":null,"document_revision":null,"locator":"https://example.invalid/source",
                "access_note":null,"retained_blob":null}],
            "evidence":[{"title":"Other evidence"}],"extensions":{"vendor":{"keep":true}}}})
}

fn fixture() -> (PathBuf, LibraryBrowser, LibraryRevisionWorkflow) {
    let root = std::env::temp_dir().join(format!("datum-provenance-controls-{}", Uuid::new_v4()));
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Provenance controls".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let mut model = ProjectResolver::new(&root).resolve().unwrap();
    model.project.schema_version = Some(2);
    let description = json!({"project_id":model.project.project_id,"model_revision":model.model_revision,
        "transaction_tip":eda_engine::revision::transaction_tip(&model.journal),
        "library":{"project_id":model.project.project_id,"model_revision":model.model_revision,
            "managed":true,"available_assets":[],"catalog_manifest":"test-catalog"}});
    let mut browser = LibraryBrowser::capture(Arc::new(model));
    browser.set_revision_inspection(description).unwrap();
    let mut workflow = LibraryRevisionWorkflow::capture(&browser, &root, Value::Null).unwrap();
    let record = record();
    workflow.request = workflow
        .candidate_request(json!({"action":"revise","base":record["revision"]}))
        .unwrap();
    workflow.provenance = Some(ProvenanceForm::new(&record).unwrap());
    workflow
        .set_purpose("Record supplied declarations".into())
        .unwrap();
    (root, browser, workflow)
}

#[test]
fn repeated_rows_preserve_hidden_evidence_and_retained_references() {
    let original = record();
    let mut form = ProvenanceForm::new(&original).unwrap();
    let retained = form.citations[0].id;
    assert!(
        form.edit_list(ProvenanceAction::RemoveCitation(retained))
            .is_err()
    );
    form.license.text.clear();
    form.attributions[1].field.text = "Updated distributor credit".into();
    form.citations[0].fields[1].text = "Recorded issuer correction".into();
    let removed = form.citations[1].id;
    form.edit_list(ProvenanceAction::RemoveCitation(removed))
        .unwrap();
    form.edit_list(ProvenanceAction::AddCitation).unwrap();
    assert!(form.value().unwrap_err().contains("needs a title"));
    form.citations[1].fields[0].text = "New source citation".into();
    let changed = form.value().unwrap();
    assert_eq!(changed["rights"]["license_expression"], Value::Null);
    assert_eq!(
        changed["rights"]["attribution"],
        json!(["First credit", "Updated distributor credit"])
    );
    assert_eq!(
        changed["sources"][0]["retained_blob"],
        original["provenance"]["sources"][0]["retained_blob"]
    );
    assert_eq!(changed["sources"][1]["retained_blob"], Value::Null);
    for key in [
        "authors",
        "created_at",
        "origin_kind",
        "source_assets",
        "evidence",
        "extensions",
    ] {
        assert_eq!(changed[key], original["provenance"][key], "{key}");
    }
    for key in ["restrictions", "evidence"] {
        assert_eq!(
            changed["rights"][key],
            original["provenance"]["rights"][key]
        );
    }
    assert!(
        form.field(ProvenanceField {
            row: removed,
            index: 0
        })
        .is_none()
    );
    assert!(
        form.edit_list(ProvenanceAction::RemoveCitation(removed))
            .is_err()
    );
}

#[test]
fn text_list_and_source_changes_invalidate_exact_preview_and_stale_controls() {
    let (root, browser, mut workflow) = fixture();
    let target = ProvenanceField {
        row: workflow.provenance().unwrap().id,
        index: 0,
    };
    let old = workflow.provenance_control(ProvenanceAction::AddAttribution);
    let request_id = workflow.request_id;
    let original_text = workflow.provenance_field(target).unwrap().text.clone();
    workflow
        .set_provenance_field(target, original_text, SingleLineEditor::default())
        .unwrap();
    assert_eq!(
        workflow.request_id, request_id,
        "cursor movement is not a new write intent"
    );
    workflow
        .set_provenance_field(
            target,
            "LicenseRef-Supplied".into(),
            SingleLineEditor::default(),
        )
        .unwrap();
    assert!(workflow.validate_review_control(old).is_err());
    let intent = workflow.preview_intent(&browser, &root).unwrap();
    let NativeLibraryEdit::Revision { request, .. } = intent.library.as_ref().unwrap() else {
        panic!("revision");
    };
    assert!(request["request"]["change"].get("payload_json").is_none());
    assert_eq!(
        request["request"]["provenance_override"]["rights"]["license_expression"],
        "LicenseRef-Supplied"
    );
    let selected = workflow.provenance().unwrap().selected.clone();
    let preview = QualifiedLibraryPreview {
        identity: LibraryPreviewIdentity::from_intent(&intent).unwrap(),
        prepared_request: request.clone(),
        summary: json!({"action":"save_candidate","asset_changes":[{"asset_id":selected["asset_id"],
            "review":{"before":selected,"payload_bytes_changed":false,"payload_semantics_changed":false,"dependencies":[]}}]}),
    };
    let mut wrong = preview.clone();
    wrong.summary["asset_changes"][0]["review"]["payload_bytes_changed"] = json!(true);
    assert!(workflow.accept_preview(&browser, &root, wrong).is_err());
    workflow.accept_preview(&browser, &root, preview).unwrap();
    assert!(
        workflow
            .set_provenance_field(target, "Lost update".into(), SingleLineEditor::default())
            .is_err()
    );
    let edit = workflow.provenance_control(ProvenanceAction::EditFields);
    let focus = workflow.edit_provenance_list(edit).unwrap();
    assert_eq!(focus, workflow.initial_focus_control());
    workflow.validate_review_control(focus).unwrap();
    assert!(workflow.summary().is_none());
    workflow
        .edit_provenance_list(workflow.provenance_control(ProvenanceAction::AddCitation))
        .unwrap();
    assert!(
        workflow
            .preview_intent(&browser, &root)
            .unwrap_err()
            .contains("needs a title")
    );
    let stale_row = workflow.provenance().unwrap().citations.last().unwrap().id;
    workflow
        .edit_provenance_list(
            workflow.provenance_control(ProvenanceAction::RemoveCitation(stale_row)),
        )
        .unwrap();
    assert!(
        workflow
            .set_provenance_field(
                ProvenanceField {
                    row: stale_row,
                    index: 0
                },
                "Wrong row".into(),
                SingleLineEditor::default()
            )
            .is_err()
    );
    let mut changed_browser = browser.clone();
    changed_browser.revision_inspection.as_mut().unwrap()["library"]["catalog_manifest"] =
        json!("changed");
    assert!(!workflow.provenance_basis_current(&changed_browser, &root));
    assert!(workflow.preview_intent(&changed_browser, &root).is_err());
    workflow.submitted = true;
    assert!(
        workflow
            .set_provenance_field(target, "After submit".into(), SingleLineEditor::default())
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn list_actions_return_current_focus_for_new_and_surviving_rows() {
    let (root, browser, mut workflow) = fixture();
    let purpose = workflow.purpose().to_owned();
    let license = ProvenanceField {
        row: workflow.provenance().unwrap().id,
        index: 0,
    };
    assert_eq!(
        workflow.initial_focus_control(),
        workflow.provenance_control(ProvenanceAction::Field(license))
    );
    let field_target = |control| {
        let RevisionControl::Provenance {
            action: ProvenanceAction::Field(field),
            ..
        } = control
        else {
            panic!("Expected an editable declaration field");
        };
        field
    };
    for action in [
        ProvenanceAction::AddAttribution,
        ProvenanceAction::AddCitation,
    ] {
        let old = workflow.provenance_control(action);
        let focus = workflow.edit_provenance_list(old).unwrap();
        workflow.validate_review_control(focus).unwrap();
        assert!(workflow.validate_review_control(old).is_err());
        let target = field_target(focus);
        let expected = match action {
            ProvenanceAction::AddAttribution => {
                workflow
                    .provenance()
                    .unwrap()
                    .attributions
                    .last()
                    .unwrap()
                    .id
            }
            _ => workflow.provenance().unwrap().citations.last().unwrap().id,
        };
        assert_eq!(
            target,
            ProvenanceField {
                row: expected,
                index: 0
            }
        );
        workflow
            .set_provenance_field(
                target,
                "Typed into the new row".into(),
                SingleLineEditor::default(),
            )
            .unwrap();
        assert_eq!(
            workflow.provenance_field(target).unwrap().text,
            "Typed into the new row"
        );
        assert_eq!(workflow.purpose(), purpose);
        let remove = match action {
            ProvenanceAction::AddAttribution => ProvenanceAction::RemoveAttribution(target.row),
            _ => ProvenanceAction::RemoveCitation(target.row),
        };
        let next = workflow
            .edit_provenance_list(workflow.provenance_control(remove))
            .unwrap();
        workflow.validate_review_control(next).unwrap();
        assert_ne!(field_target(next).row, target.row);
        assert!(workflow.provenance_field(target).is_none());
        assert!(workflow.validate_review_control(focus).is_err());
    }
    assert!(workflow.preview_intent(&browser, &root).is_ok());
    // The last remaining list row returns focus to its Add action, never Purpose.
    while let Some(row) = workflow.provenance().unwrap().attributions.last() {
        let action = ProvenanceAction::RemoveAttribution(row.id);
        let focus = workflow
            .edit_provenance_list(workflow.provenance_control(action))
            .unwrap();
        if workflow.provenance().unwrap().attributions.is_empty() {
            assert_eq!(
                focus,
                workflow.provenance_control(ProvenanceAction::AddAttribution)
            );
        }
    }
    workflow.provenance = None;
    assert_eq!(workflow.initial_focus_control(), workflow.purpose_control());
    std::fs::remove_dir_all(root).unwrap();
}
