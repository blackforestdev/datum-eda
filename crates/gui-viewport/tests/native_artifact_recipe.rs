//! Producer-native action IDs and finding fingerprints bind this focused replay.
//! Consumer negative cases never mutate the frozen design or producing artifacts.
use datum_gui_protocol::{
    LiveReviewRequest, SelectionSubject as Subject,
    selection_artifacts::{WorkspaceSelectionAuthority, build_workspace_selection_view},
    selection_resolution::{
        NativeSelectionResolution, SelectionAuthority, SelectionResolutionError,
    },
    selection_view::InspectionValue,
};
use datum_gui_viewport::{
    selection_acquisition::{AcquisitionMode, acquire_artifact},
    selection_lifetime::reconcile_selection,
};
use eda_engine::substrate::{ProjectResolver, transaction_journal_path};
use serde_json::Value;
use std::path::PathBuf;
#[test]
#[ignore = "requires frozen non-authored-channels recipe and pinned EDA_CLI_BIN; no absent-fixture pass"]
fn native_producer_artifacts_keep_whole_identities_and_refuse_ambiguous_consumers() {
    let root = PathBuf::from(
        std::env::var_os("S5A_NATIVE_ARTIFACT_FIXTURE")
            .expect("explicit frozen native artifact recipe"),
    );
    assert!(
        std::env::var_os("EDA_CLI_BIN").is_some(),
        "pinned CLI; no Cargo fallback"
    );
    let report: Value =
        serde_json::from_slice(&std::fs::read(root.join("s5a-fixture.json")).unwrap()).unwrap();
    assert!(
        !root.join(".datum/gui-terminal-context.json").exists(),
        "exercise absent-context history loading, never check execution"
    );
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert_eq!(
        model.model_revision.0,
        report["model_revision"].as_str().unwrap()
    );
    let mut before: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    for path in [
        transaction_journal_path(&root),
        root.join("route-proposal.json"),
    ] {
        before.push((path.clone(), std::fs::read(path).unwrap()));
    }
    let state = datum_gui_protocol::load_live_workspace_state(&LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: Some(root.join("route-proposal.json")),
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .unwrap();
    let native = NativeSelectionResolution::capture(&model).unwrap();
    let authority = WorkspaceSelectionAuthority {
        native: &native,
        model: &model,
        workspace: &state,
    };
    let expected = &report["producer_receipts"]["review"]["proposal_actions"];
    assert_eq!(
        state.review.proposal_actions.len(),
        expected.as_array().unwrap().len()
    );
    assert_eq!(state.review.proposal_actions.len(), 1);
    let id = expected[0]["action_id"].as_str().unwrap().to_string();
    assert_eq!(
        state.review.proposal_actions[0].action_id, id,
        "actual route producer ID, not proposal UUID or scene key"
    );
    let proposal = Subject::Proposal(id.clone());
    let review = Subject::Review(id);
    assert_ne!(proposal, review);
    for subject in [&proposal, &review] {
        let acquired = acquire_artifact(
            &Subject::None,
            subject.clone(),
            AcquisitionMode::Replace,
            &authority,
        )
        .unwrap();
        assert_eq!(acquired.subject, *subject);
        assert!(acquired.members.is_empty());
        let view =
            build_workspace_selection_view(&model, &native, subject.clone(), &state).unwrap();
        assert_eq!(view.subject, *subject);
        assert!(view.members.is_empty());
        let InspectionValue::Common(payload) = view.field("artifact_payload") else {
            panic!("whole actual action")
        };
        assert_eq!(payload["action_id"], expected[0]["action_id"]);
        assert_eq!(payload["from_anchor_pad_uuid"], report["route_anchors"][0]);
        assert_eq!(payload["to_anchor_pad_uuid"], report["route_anchors"][1]);
        let noop =
            acquire_artifact(subject, subject.clone(), AcquisitionMode::Add, &authority).unwrap();
        assert_eq!(noop.subject, *subject);
        assert!(!noop.changed);
        assert!(noop.refusal.is_none());
    }
    let mixed =
        acquire_artifact(&proposal, review.clone(), AcquisitionMode::Add, &authority).unwrap();
    assert_eq!(mixed.subject, proposal);
    assert!(mixed.refusal.is_some());
    let finding = report["producer_receipts"]["check"]["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["rule_id"] == "connectivity_cross_net_contact")
        .unwrap();
    let fingerprint = finding["fingerprint"].as_str().unwrap().to_string();
    assert!(
        state
            .checks
            .findings
            .iter()
            .any(|f| f.fingerprint == fingerprint),
        "persisted actual CheckRun finding"
    );
    let diagnostic = Subject::Diagnostic(fingerprint.clone());
    let diagnostic_view =
        build_workspace_selection_view(&model, &native, diagnostic.clone(), &state).unwrap();
    assert!(diagnostic_view.members.is_empty());
    assert_eq!(
        diagnostic_view.field("fingerprint"),
        InspectionValue::Common(serde_json::json!(fingerprint))
    );
    assert!(
        matches!(
            diagnostic_view.field("board_marker"),
            InspectionValue::Common(_)
        ),
        "source-bound actual board marker"
    );
    assert!(
        matches!(
            diagnostic_view.field("schematic_marker"),
            InspectionValue::Unavailable(_)
        ),
        "DRC marker does not acquire a schematic source"
    );
    let retained = reconcile_selection(&diagnostic, native.project(), &authority).unwrap();
    assert_eq!(retained.subject, diagnostic);
    let mut duplicate = state.clone();
    duplicate
        .review
        .proposal_actions
        .push(duplicate.review.proposal_actions[0].clone());
    let ambiguous = WorkspaceSelectionAuthority {
        native: &native,
        model: &model,
        workspace: &duplicate,
    };
    assert!(
        matches!(
            acquire_artifact(
                &diagnostic,
                proposal.clone(),
                AcquisitionMode::Replace,
                &ambiguous
            ),
            Err(SelectionResolutionError::MissingArtifactAuthority)
        ),
        "atomic duplicate producer refusal"
    );
    let mut removed = state.clone();
    removed
        .checks
        .findings
        .retain(|f| f.fingerprint != fingerprint);
    let absent = WorkspaceSelectionAuthority {
        native: &native,
        model: &model,
        workspace: &removed,
    };
    assert_eq!(
        reconcile_selection(&diagnostic, native.project(), &absent)
            .unwrap()
            .subject,
        Subject::None
    );
    assert_eq!(
        reconcile_selection(&Subject::None, native.project(), &authority)
            .unwrap()
            .subject,
        Subject::None,
        "restored producer finding does not resurrect cleared selection"
    );
    for (path, bytes) in before {
        assert!(
            std::fs::read(&path).unwrap() == bytes,
            "source, journal and producing artifact unchanged: {}",
            path.display()
        );
    }
    assert_eq!(
        ProjectResolver::new(&root)
            .resolve()
            .unwrap()
            .model_revision,
        model.model_revision
    );
}

#[test]
#[ignore = "requires frozen non-authored-channels recipe and pinned EDA_CLI_BIN"]
fn absent_context_and_checks_never_generate_a_check_transaction() {
    let root =
        PathBuf::from(std::env::var_os("S5A_NATIVE_ARTIFACT_FIXTURE").expect("frozen recipe"))
            .join("variants/no-checks");
    assert!(std::env::var_os("EDA_CLI_BIN").is_some());
    assert!(!root.join(".datum/gui-terminal-context.json").exists());
    let model = ProjectResolver::new(&root).resolve().unwrap();
    assert!(model.check_runs.is_empty());
    let mut files: Vec<_> = model
        .source_shards
        .iter()
        .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
        .collect();
    let journal = transaction_journal_path(&root);
    files.push((journal.clone(), std::fs::read(journal).unwrap()));
    let state = datum_gui_protocol::load_board_editor_workspace_state(&LiveReviewRequest {
        project_root: root.clone(),
        board_file: None,
        artifact_path: None,
        net_uuid: None,
        from_anchor_pad_uuid: None,
        to_anchor_pad_uuid: None,
        profile: None,
        kicad_board_source: None,
    })
    .unwrap();
    assert!(state.checks.findings.is_empty());
    assert!(state.checks.check_run_id.is_none());
    for (path, before) in files {
        assert!(
            std::fs::read(&path).unwrap() == before,
            "read-only load: {}",
            path.display()
        );
    }
    let reopened = ProjectResolver::new(&root).resolve().unwrap();
    assert!(reopened.check_runs.is_empty());
    assert_eq!(reopened.model_revision, model.model_revision);
}
