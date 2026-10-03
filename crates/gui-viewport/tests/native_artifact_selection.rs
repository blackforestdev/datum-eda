#[path = "../../gui-protocol/tests/support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::{
    SelectionSubject as Subject,
    selection_artifacts::{WorkspaceSelectionAuthority, build_workspace_selection_view},
    selection_resolution::{NativeSelectionResolution, SelectionAuthority},
    selection_view::InspectionValue,
};
use datum_gui_viewport::{
    selection_acquisition::{
        AcquisitionEntry, AcquisitionMode as Mode, acquire_artifact, acquire_authored,
    },
    selection_lifetime::reconcile_selection,
};
use fixture::{Fixture, id};
fn setup() -> (Fixture, datum_gui_protocol::ReviewWorkspaceState) {
    let f = Fixture::new("artifact-selection");
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.project_uuid = f.model.project.project_id.to_string();
    state.review.project_uuid = Some(state.scene.project_uuid.clone());
    state.review.artifact_path = None;
    state.review.review_source = "selected_route_proposal".into();
    state.review.producer_revision = Some(f.model.model_revision.0.clone());
    state.checks.project_id = Some(state.scene.project_uuid.clone());
    state.checks.findings = vec![datum_gui_protocol::CheckFindingSummary {
        fingerprint: "sha256:permanent-finding".into(),
        primary_target: serde_json::json!({"id":id(10),"kind":"track"}),
        status: "active".into(),
        message: "Fixture finding".into(),
        ..Default::default()
    }];
    (f, state)
}
#[test]
fn overlay_proposal_and_lane_review_are_distinct_whole_artifacts_not_authored_members() {
    let (f, state) = setup();
    let before = f.source_bytes();
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    let action = state.review.proposal_actions[0].action_id.clone();
    let proposal = Subject::Proposal(action.clone());
    let review = Subject::Review(action);
    let authority = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    for subject in [&proposal, &review] {
        assert!(authority.artifact_exists(subject).unwrap());
        assert!(authority.members(subject).unwrap().is_empty());
        let view =
            build_workspace_selection_view(&f.model, &native, subject.clone(), &state).unwrap();
        assert_eq!(view.subject, *subject);
        assert!(view.members.is_empty());
        assert!(view.related.is_empty());
        assert!(matches!(
            view.field("artifact_payload"),
            InspectionValue::Common(_)
        ));
        assert!(!view.summary().contains("0 objects"));
    }
    assert_ne!(proposal, review);
    Fixture::assert_bytes(&before);
}
#[test]
fn whole_artifact_modifiers_preserve_noops_and_refuse_mixed_compounds_atomically() {
    let (mut f, mut state) = setup();
    f.line(
        id(10),
        eda_engine::ir::geometry::Point::new(0, 0),
        eda_engine::ir::geometry::Point::new(10, 0),
    );
    state.review.producer_revision = Some(f.model.model_revision.0.clone());
    let before = f.source_bytes();
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    let authority = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    let artifact = Subject::Diagnostic("sha256:permanent-finding".into());
    let object = Subject::Object(datum_gui_protocol::AuthoredSelectionIdentity {
        class: datum_gui_protocol::AuthoredSelectionClass::Track,
        id: id(10),
        instance_path: vec![],
    });
    let replace = acquire_artifact(&object, artifact.clone(), Mode::Replace, &authority).unwrap();
    assert_eq!(replace.subject, artifact);
    let refused = acquire_artifact(&object, artifact.clone(), Mode::Add, &authority).unwrap();
    assert_eq!(refused.subject, object);
    assert!(refused.refusal.is_some());
    let noop = acquire_artifact(&object, artifact.clone(), Mode::Remove, &authority).unwrap();
    assert_eq!(noop.subject, object);
    assert!(!noop.changed);
    assert!(noop.refusal.is_none());
    let noop = acquire_artifact(&artifact, artifact.clone(), Mode::Add, &authority).unwrap();
    assert_eq!(noop.subject, artifact);
    assert!(!noop.changed);
    let removed = acquire_artifact(&artifact, artifact.clone(), Mode::Remove, &authority).unwrap();
    assert_eq!(removed.subject, Subject::None);
    let Subject::Object(id) = object.clone() else {
        unreachable!()
    };
    let candidates = std::collections::BTreeSet::from([id]);
    let refused = acquire_authored(
        &artifact,
        candidates.clone(),
        Mode::Add,
        AcquisitionEntry::Region,
        &authority,
    )
    .unwrap();
    assert_eq!(refused.subject, artifact);
    assert!(refused.refusal.is_some());
    let noop = acquire_authored(
        &artifact,
        candidates,
        Mode::Remove,
        AcquisitionEntry::Region,
        &authority,
    )
    .unwrap();
    assert_eq!(noop.subject, artifact);
    assert!(!noop.changed);
    Fixture::assert_bytes(&before);
}
#[test]
fn producer_rechecks_preserve_fingerprint_then_clear_without_journal_or_resurrection() {
    let (f, mut state) = setup();
    let before = f.source_bytes();
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = Subject::Diagnostic("sha256:permanent-finding".into());
    state.checks.check_run_id = Some("first".into());
    let a = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    assert!(
        !reconcile_selection(&subject, native.project(), &a)
            .unwrap()
            .dissolved
    );
    state.checks.check_run_id = Some("second".into());
    state.checks.findings[0].message = "Refreshed evidence".into();
    let a = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    let retained = reconcile_selection(&subject, native.project(), &a).unwrap();
    assert_eq!(retained.subject, subject);
    assert!(!retained.dissolved);
    let view = build_workspace_selection_view(&f.model, &native, subject.clone(), &state).unwrap();
    assert!(
        matches!(view.field("artifact_payload"),InspectionValue::Common(v) if v["message"]=="Refreshed evidence")
    );
    state.checks.findings.clear();
    let a = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    let cleared = reconcile_selection(&subject, native.project(), &a).unwrap();
    assert_eq!(cleared.subject, Subject::None);
    assert!(cleared.dissolved);
    state
        .checks
        .findings
        .push(datum_gui_protocol::CheckFindingSummary {
            fingerprint: "sha256:permanent-finding".into(),
            ..Default::default()
        });
    let a = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    assert_eq!(
        reconcile_selection(&cleared.subject, native.project(), &a)
            .unwrap()
            .subject,
        Subject::None
    );
    Fixture::assert_bytes(&before);
}
#[test]
fn missing_foreign_or_duplicate_artifact_authority_is_never_an_empty_success() {
    let (f, mut state) = setup();
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = Subject::Proposal(state.review.proposal_actions[0].action_id.clone());
    state.review.project_uuid = Some(id(111).to_string());
    assert!(build_workspace_selection_view(&f.model, &native, subject.clone(), &state).is_err());
    state.review.project_uuid = Some(native.project().to_string());
    state
        .review
        .proposal_actions
        .push(state.review.proposal_actions[0].clone());
    assert!(build_workspace_selection_view(&f.model, &native, subject.clone(), &state).is_err());
    state.review.proposal_actions.clear();
    let a = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    let cleared = reconcile_selection(&subject, native.project(), &a).unwrap();
    assert!(cleared.dissolved);
    assert_eq!(cleared.subject, Subject::None);
    assert!(build_workspace_selection_view(&f.model, &native, subject, &state).is_err());
}
#[test]
fn saved_action_without_current_lifecycle_binding_refuses_atomically() {
    let (f, mut state) = setup();
    let before = f.source_bytes();
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = Subject::Proposal(state.review.proposal_actions[0].action_id.clone());
    state.review.artifact_path = Some("saved-route.json".into());
    let authority = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    assert!(acquire_artifact(&Subject::None, subject, Mode::Replace, &authority).is_err());
    Fixture::assert_bytes(&before);
}

fn saved_fixture() -> (
    Fixture,
    datum_gui_protocol::ReviewWorkspaceState,
    eda_engine::substrate::Proposal,
) {
    let (f, mut state) = setup();
    let mut payload = serde_json::to_value(&state.review.proposal_actions[0]).unwrap();
    payload["net_uuid"] = serde_json::json!(f.net);
    payload["from_anchor_pad_uuid"] = serde_json::json!(id(11));
    payload["to_anchor_pad_uuid"] = serde_json::json!(id(12));
    payload["reused_via_uuid"] = serde_json::Value::Null;
    payload["reused_via_uuids"] = serde_json::json!([]);
    payload["reused_object_uuid"] = serde_json::Value::Null;
    let mut action: eda_engine::board::route_proposal::RouteProposalAction =
        serde_json::from_value(payload).unwrap();
    action.net_uuid = f.net;
    action.layer = 1;
    action.width_nm = 2;
    action.from = eda_engine::ir::geometry::Point::new(0, 0);
    action.to = eda_engine::ir::geometry::Point::new(10, 0);
    action.proposal_action = "draw_track".into();
    let built = eda_engine::board::route_proposal::build_accepted_route_proposal(
        &f.model,
        fixture::provenance(),
        &[action.clone()],
        |id| id == f.net,
    )
    .unwrap();
    let proposal = built.proposal.unwrap();
    let path = f.root.join("saved-proposal.json");
    let source = serde_json::json!({"kind":"native_route_proposal_artifact","version":1,"project_uuid":f.model.project.project_id,"project_name":"artifact fixture","contract":action.contract,"actions":[action],"proposal":proposal});
    std::fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
    state.review.artifact_path = Some(path.display().to_string());
    state.review.review_source = "route_proposal_artifact".into();
    state.review.producer_proposal = Some(proposal.clone());
    state.review.producer_revision = Some(proposal.prepared_against.0.clone());
    // Adapter fixture transport witness; real producer byte-hash calculation is
    // independently exercised by the existing CLI review test in this unit.
    state.review.source_fingerprint = Some(format!("sha256:{}", "a".repeat(64)));
    state.review.proposal_actions =
        vec![serde_json::from_value(serde_json::to_value(action).unwrap()).unwrap()];
    (f, state, proposal)
}
#[test]
fn saved_action_apply_clears_using_canonical_status_and_undo_reopen_never_resurrects_selection() {
    use eda_engine::substrate::{
        ProposalStatus, apply_accepted_proposal, commit_proposal_metadata_journaled,
    };
    let (mut f, state, proposal) = saved_fixture();
    let subject = Subject::Proposal(state.review.proposal_actions[0].action_id.clone());
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    let selected =
        build_workspace_selection_view(&f.model, &native, subject.clone(), &state).unwrap();
    assert_eq!(selected.subject, subject);
    commit_proposal_metadata_journaled(&mut f.model, &f.root, proposal.clone()).unwrap();
    apply_accepted_proposal(&mut f.model, &f.root, proposal.proposal_id).unwrap();
    assert_eq!(
        f.model.proposals[&proposal.proposal_id].status,
        ProposalStatus::Applied
    );
    let before = f.source_bytes();
    for model in [&f.model, &f.reopen()] {
        let native = NativeSelectionResolution::capture(model).unwrap();
        let authority = WorkspaceSelectionAuthority {
            native: &native,
            model,
            workspace: &state,
        };
        let cleared = reconcile_selection(&subject, native.project(), &authority).unwrap();
        assert!(cleared.dissolved);
        assert_eq!(cleared.subject, Subject::None);
        assert!(build_workspace_selection_view(model, &native, subject.clone(), &state).is_err());
    }
    Fixture::assert_bytes(&before);
    f.undo();
    let reopened = f.reopen();
    let native = NativeSelectionResolution::capture(&reopened).unwrap();
    let before = f.source_bytes();
    let authority = WorkspaceSelectionAuthority {
        native: &native,
        model: &reopened,
        workspace: &state,
    };
    assert_eq!(
        reconcile_selection(&Subject::None, native.project(), &authority)
            .unwrap()
            .subject,
        Subject::None
    );
    Fixture::assert_bytes(&before);
}
#[test]
fn deleted_artifact_clears_and_conflicting_canonical_batch_refuses_instead_of_rebinding() {
    use eda_engine::substrate::commit_proposal_metadata_journaled;
    let (mut f, mut state, proposal) = saved_fixture();
    commit_proposal_metadata_journaled(&mut f.model, &f.root, proposal).unwrap();
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = Subject::Review(state.review.proposal_actions[0].action_id.clone());
    let before = f.source_bytes();
    state
        .review
        .producer_proposal
        .as_mut()
        .unwrap()
        .batch
        .operations
        .clear();
    assert!(build_workspace_selection_view(&f.model, &native, subject.clone(), &state).is_err());
    Fixture::assert_bytes(&before);
    state.review.producer_proposal = Some(f.model.proposals.values().next().unwrap().clone());
    std::fs::remove_file(state.review.artifact_path.as_ref().unwrap()).unwrap();
    let authority = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    let cleared = reconcile_selection(&subject, native.project(), &authority).unwrap();
    assert_eq!(cleared.subject, Subject::None);
    assert!(cleared.dissolved);
    Fixture::assert_bytes(&before);
}
#[test]
fn producer_namespace_changes_with_reused_action_string_are_explicit_not_rebound() {
    let (f, mut state, _) = saved_fixture();
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    let subject = Subject::Proposal(state.review.proposal_actions[0].action_id.clone());
    let first = build_workspace_selection_view(&f.model, &native, subject.clone(), &state).unwrap();
    state.review.producer_proposal.as_mut().unwrap().proposal_id = id(9999);
    let second = build_workspace_selection_view(&f.model, &native, subject, &state).unwrap();
    assert!(datum_gui_protocol::selection_artifacts::artifact_producer_changed(&first, &second));
}
