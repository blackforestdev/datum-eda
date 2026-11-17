use super::*;
use eda_engine::api::native_write::genesis::{GenesisSpec, bootstrap_native_project};
use eda_engine::substrate::ProjectResolver;
use std::sync::Arc;

#[test]
fn recorded_details_never_change_write_identity_or_survive_a_new_review() {
    let (root, browser, mut workflow) = fixture();
    let intent = workflow.preview_intent(&browser, &root).unwrap();
    // Presentation controls only; this token does not claim a service response
    // or managed-CAD acceptance (the socket owner qualifies actual responses).
    let mut preview = token(&intent);
    preview.summary["asset_changes"] = json!([{"asset_id":"record"}]);
    workflow
        .accept_preview(&browser, &root, preview.clone())
        .unwrap();
    let details = |workflow: &LibraryRevisionWorkflow| {
        workflow
            .review_controls()
            .into_iter()
            .find(|control| matches!(control, RevisionControl::RecordDetails { .. }))
            .unwrap()
    };
    let show = details(&workflow);
    workflow.set_record_details(show).unwrap();
    assert!(workflow.set_record_details(show).is_err());
    assert_eq!(
        LibraryPreviewIdentity::from_intent(&intent),
        LibraryPreviewIdentity::from_intent(&workflow.preview_intent(&browser, &root).unwrap())
    );
    let hide = details(&workflow);
    assert!(matches!(
        hide,
        RevisionControl::RecordDetails { show: false, .. }
    ));
    workflow.accept_preview(&browser, &root, preview).unwrap();
    assert!(matches!(
        details(&workflow),
        RevisionControl::RecordDetails { show: true, .. }
    ));
    assert!(workflow.set_record_details(hide).is_err());
    workflow
        .set_purpose("Different change purpose".into())
        .unwrap();
    assert!(workflow.set_record_details(show).is_err());
    let current = workflow.preview_intent(&browser, &root).unwrap();
    let mut preview = token(&current);
    preview.summary["asset_changes"] = json!([{"asset_id":"record"}]);
    workflow.accept_preview(&browser, &root, preview).unwrap();
    let current_show = details(&workflow);
    assert!(workflow.set_record_details(show).is_err());
    workflow.submit(&browser, &root).unwrap();
    assert!(workflow.set_record_details(current_show).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

fn fixture() -> (PathBuf, LibraryBrowser, LibraryRevisionWorkflow) {
    let root = std::env::temp_dir().join(format!("datum-revision-review-{}", Uuid::new_v4()));
    bootstrap_native_project(
        &root,
        GenesisSpec {
            project_name: "Revision review controls".into(),
            existing_ids: None,
        },
    )
    .unwrap();
    let model = ProjectResolver::new(&root).resolve().unwrap();
    let description = json!({"project_id":model.project.project_id,
        "model_revision":model.model_revision,
        "transaction_tip":eda_engine::revision::transaction_tip(&model.journal),
        "library":{"project_id":model.project.project_id,
            "model_revision":model.model_revision,"managed":false,"available_assets":[]}});
    let mut browser = LibraryBrowser::capture(Arc::new(model));
    browser.set_revision_inspection(description).unwrap();
    let mut workflow = LibraryRevisionWorkflow::adopt_profile(&browser, &root).unwrap();
    workflow
        .set_purpose("Record the separate empty library baseline".into())
        .unwrap();
    (root, browser, workflow)
}

// Controller controls only. Actual response qualification belongs to the
// client's socket controls; these tokens are not runtime write evidence.
fn token(intent: &NativeEditIntent) -> QualifiedLibraryPreview {
    QualifiedLibraryPreview {
        identity: LibraryPreviewIdentity::from_intent(intent).unwrap(),
        summary: json!({"action":"adopt_profile"}),
        prepared_request: match intent.library.as_ref().unwrap() {
            NativeLibraryEdit::Revision { request, .. } => request.clone(),
            _ => panic!("Revision intent required"),
        },
    }
}

#[test]
fn qualified_geometry_preview_submits_exactly_once_without_retargeting_selection() {
    let (root, browser, mut workflow) = fixture();
    // Controller inputs only; this does not claim a physical CAD workflow.
    workflow.request = json!({"action":"adopt_selection","request":{
        "expected_catalog":"catalog","selected":{"asset_id":Uuid::new_v4(),
            "revision_id":"exact"}}});
    let intent = workflow.preview_intent(&browser, &root).unwrap();
    let mut preview = token(&intent);
    preview.summary = json!({"action":"adopt_selection"});
    preview.prepared_request["request"]["physical_geometry"] =
        json!({"placements":[{"placed_package":Uuid::new_v4(),"pads":[]}]});
    let expected = preview.prepared_request.clone();
    let mut invalid = vec![json!(null), json!([])];
    for field in ["selected", "expected_catalog"] {
        let mut changed = expected.clone();
        changed["request"][field] = json!("different");
        invalid.push(changed);
    }
    let mut changed = expected.clone();
    changed["action"] = json!("adopt_profile");
    invalid.push(changed);
    let mut changed = expected.clone();
    changed["request"]["physical_geometry"]["placements"] = json!([]);
    invalid.push(changed);
    for request in invalid {
        let mut forged = preview.clone();
        forged.prepared_request = request;
        assert!(workflow.accept_preview(&browser, &root, forged).is_err());
        assert!(workflow.submit(&browser, &root).is_err());
    }
    workflow.accept_preview(&browser, &root, preview).unwrap();
    assert!(
        workflow.request["request"]
            .get("physical_geometry")
            .is_none()
    );
    let submitted = workflow.submit(&browser, &root).unwrap();
    assert_eq!(submitted.request_id, intent.request_id);
    assert_eq!(submitted.expected_revision, intent.expected_revision);
    assert_eq!(submitted.expected_tip, intent.expected_tip);
    let NativeLibraryEdit::Revision { request, reason } = submitted.library.unwrap() else {
        panic!("Revision intent required");
    };
    assert_eq!(request, expected);
    assert_eq!(reason, workflow.purpose.trim());
    assert!(workflow.submit(&browser, &root).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn same_id_preview_cannot_authorize_different_inputs() {
    let (root, browser, mut workflow) = fixture();
    let original = workflow.preview_intent(&browser, &root).unwrap();
    for change in 0..6 {
        let mut altered = original.clone();
        match change {
            0 => {
                altered.library = Some(NativeLibraryEdit::Revision {
                    request: json!({"action":"save_candidate"}),
                    reason: workflow.purpose().into(),
                })
            }
            1 => {
                altered.library = Some(NativeLibraryEdit::Revision {
                    request: json!({"action":"adopt_profile"}),
                    reason: "Different purpose".into(),
                })
            }
            2 => altered.project_root = root.join("different-project"),
            3 => altered.project_id = Uuid::new_v4(),
            4 => altered.expected_revision.push_str("-changed"),
            _ => altered.expected_tip = Some(Uuid::new_v4()),
        }
        assert_eq!(altered.request_id, original.request_id);
        assert!(
            workflow
                .accept_preview(&browser, &root, token(&altered))
                .is_err()
        );
        assert!(workflow.submit(&browser, &root).is_err());
    }
    workflow
        .accept_preview(&browser, &root, token(&original))
        .unwrap();
    let submitted = workflow.submit(&browser, &root).unwrap();
    assert_eq!(
        LibraryPreviewIdentity::from_intent(&submitted),
        LibraryPreviewIdentity::from_intent(&original)
    );
    assert!(workflow.submit(&browser, &root).is_err());
    assert!(
        workflow
            .set_purpose("Retry without reconciliation".into())
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn purpose_change_and_stale_project_invalidate_review() {
    let (root, mut browser, mut workflow) = fixture();
    let old = workflow.preview_intent(&browser, &root).unwrap();
    workflow
        .accept_preview(&browser, &root, token(&old))
        .unwrap();
    workflow
        .set_purpose("A newly authored purpose".into())
        .unwrap();
    assert!(workflow.summary().is_none());
    assert!(
        workflow
            .accept_preview(&browser, &root, token(&old))
            .is_err()
    );
    let current = workflow.preview_intent(&browser, &root).unwrap();
    workflow
        .accept_preview(&browser, &root, token(&current))
        .unwrap();
    browser.revision_inspection = None;
    assert!(workflow.submit(&browser, &root).is_err());
    assert!(!workflow.is_submitted());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn definite_dispatch_failure_discards_preview_without_retrying() {
    let (root, browser, mut workflow) = fixture();
    let original = workflow.preview_intent(&browser, &root).unwrap();
    workflow
        .accept_preview(&browser, &root, token(&original))
        .unwrap();
    let submitted = workflow.submit(&browser, &root).unwrap();
    assert!(!workflow.reject_unqueued_submission(Uuid::new_v4()));
    assert!(workflow.is_submitted());
    assert!(workflow.reject_unqueued_submission(submitted.request_id));
    assert!(!workflow.is_submitted());
    assert!(workflow.summary().is_none());
    assert!(workflow.submit(&browser, &root).is_err());
    assert!(
        workflow
            .accept_preview(&browser, &root, token(&original))
            .is_err()
    );
    let new = workflow.preview_intent(&browser, &root).unwrap();
    assert_ne!(original.request_id, new.request_id);
    workflow
        .accept_preview(&browser, &root, token(&new))
        .unwrap();
    assert_eq!(
        workflow.submit(&browser, &root).unwrap().request_id,
        new.request_id
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn displayed_revision_entries_refuse_refresh_selection_and_project_replacement() {
    let (root, mut browser, _) = fixture();
    let asset = Uuid::new_v4();
    let entry = super::super::LibraryEntry {
        id: asset,
        kind: "parts".into(),
        name: "Projection-only Part".into(),
        source_path: format!("pool/parts/{asset}.json"),
        revision: None,
        raw: json!({}),
        warning: None,
    };
    let selection = crate::SelectionTarget::AuthoredObject(entry.key());
    browser.entries.push(entry);
    // Controlled view projection only: no managed CAD fixture or write proof is
    // manufactured here. The existing native project on disk stays unchanged.
    std::sync::Arc::make_mut(browser.basis.as_mut().unwrap())
        .project
        .schema_version = Some(2);
    let first = json!({"asset_id":asset,"revision_id":format!("sha256:{}", "1".repeat(64))});
    let second = json!({"asset_id":asset,"revision_id":format!("sha256:{}", "2".repeat(64))});
    let mut description = browser.revision_inspection.clone().unwrap();
    description["library"]["managed"] = json!(true);
    description["library"]["catalog_manifest"] = json!(format!("sha256:{}", "a".repeat(64)));
    description["library"]["available_assets"] = json!([
        {"revision":first,"active":false},{"revision":second,"active":false}
    ]);
    browser
        .set_revision_inspection(description.clone())
        .unwrap();
    let old = browser.revision_entry_controls(&selection)[0].1;
    assert!(browser.validate_revision_entry(&selection, old).is_ok());
    let view_id = browser.revision_inspection_id;
    browser
        .set_revision_inspection(description.clone())
        .unwrap();
    assert_eq!(browser.revision_inspection_id, view_id);
    assert!(browser.validate_revision_entry(&selection, old).is_ok());
    // An old rendered row zero must not be interpreted as the new row zero.
    description["library"]["available_assets"]
        .as_array_mut()
        .unwrap()
        .reverse();
    browser
        .set_revision_inspection(description.clone())
        .unwrap();
    assert!(browser.validate_revision_entry(&selection, old).is_err());
    let current = browser.revision_entry_controls(&selection)[0].1;
    assert!(browser.validate_revision_entry(&selection, current).is_ok());
    assert!(
        browser
            .validate_revision_entry(&crate::SelectionTarget::None, current)
            .is_err()
    );
    let mut reopened = LibraryBrowser::capture(browser.basis.clone().unwrap());
    reopened.entries = browser.entries.clone();
    reopened.set_revision_inspection(description).unwrap();
    assert!(
        reopened
            .validate_revision_entry(&selection, current)
            .is_err()
    );
    let new = reopened.revision_entry_controls(&selection)[0].1;
    assert!(reopened.validate_revision_entry(&selection, new).is_ok());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn displayed_apply_cannot_act_on_new_inputs_or_a_new_review() {
    let (root, browser, mut workflow) = fixture();
    let intent = workflow.preview_intent(&browser, &root).unwrap();
    workflow
        .accept_preview(&browser, &root, token(&intent))
        .unwrap();
    let old_apply = workflow
        .review_controls()
        .into_iter()
        .find(|control| matches!(control, RevisionControl::Apply { .. }))
        .unwrap();
    let old_cancel = RevisionControl::Cancel(workflow.session_id());
    let purpose = workflow.purpose_control();
    assert!(workflow.validate_review_control(old_apply).is_ok());
    workflow
        .set_purpose("Changed while an older frame remains visible".into())
        .unwrap();
    let changed = workflow.preview_intent(&browser, &root).unwrap();
    workflow
        .accept_preview(&browser, &root, token(&changed))
        .unwrap();
    assert!(workflow.summary().is_some());
    assert!(workflow.validate_review_control(old_apply).is_err());
    assert!(workflow.validate_review_control(purpose).is_ok());
    let current_apply = workflow
        .review_controls()
        .into_iter()
        .find(|control| matches!(control, RevisionControl::Apply { .. }))
        .unwrap();
    assert!(workflow.validate_review_control(current_apply).is_ok());
    let next = LibraryRevisionWorkflow::adopt_profile(&browser, &root).unwrap();
    assert!(next.validate_review_control(current_apply).is_err());
    assert!(next.validate_review_control(old_cancel).is_err());
    assert!(next.validate_review_control(purpose).is_err());
    assert!(!workflow.is_submitted());
    std::fs::remove_dir_all(root).unwrap();
}
