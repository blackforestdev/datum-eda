#[path = "support/native_selection_fixture.rs"]
mod fixture;
use datum_gui_protocol::{
    SelectionSubject,
    selection_artifacts::{WorkspaceSelectionAuthority, build_workspace_selection_view},
    selection_resolution::{NativeSelectionResolution, SelectionAuthority},
};
use eda_engine::{
    ir::geometry::Point,
    substrate::{
        self, CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver, Proposal,
        ProposalCreateRequest, ProposalSource, ProposalStatus,
    },
};
use fixture::{Fixture, id};

fn proposal(f: &mut Fixture, n: u128) -> Proposal {
    let mut track = f.line(id(n), Point::new(0, 0), Point::new(10000, 0));
    track.width += 1;
    let expected = f.model.model_revision.clone();
    substrate::create_draft_proposal_from_batch(
        &mut f.model,
        &f.root,
        ProposalCreateRequest {
            proposal_id: Some(id(n + 100)),
            batch: OperationBatch {
                batch_id: id(n + 200),
                expected_model_revision: Some(expected),
                provenance: CommitProvenance {
                    actor: "native-selection-proof".into(),
                    source: CommitSource::Test,
                    reason: "whole proposal identity proof".into(),
                },
                operations: vec![Operation::SetBoardTrack {
                    track_id: track.uuid,
                    track: serde_json::to_value(track).unwrap(),
                }],
            },
            rationale: "width update".into(),
            source: ProposalSource::Manual,
            checks_run: vec![],
            finding_fingerprints: vec![],
        },
    )
    .unwrap()
}
fn workspace(f: &Fixture) -> datum_gui_protocol::ReviewWorkspaceState {
    let mut state = datum_gui_protocol::load_fixture_workspace_state();
    state.scene.project_uuid = f.model.project.project_id.to_string();
    state.scene.source_revision = f.model.model_revision.0.clone();
    state.scene.board_uuid = f.board.to_string();
    let payload =
        serde_json::json!({"proposal_count":f.model.proposals.len(),"proposals":f.model.proposals});
    state.production =
        datum_gui_protocol::production_status_from_proposals_json(&payload.to_string()).unwrap();
    state
}
fn selected(
    f: &Fixture,
    state: &datum_gui_protocol::ReviewWorkspaceState,
    subject: SelectionSubject,
) -> datum_gui_protocol::selection_view::SelectionView {
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    build_workspace_selection_view(&f.model, &native, subject, state).unwrap()
}
#[test]
fn canonical_whole_batch_selection_survives_deferred_and_stale_then_dissolves_on_rejection_or_apply()
 {
    let mut f = Fixture::new("production-selection");
    let p = proposal(&mut f, 10);
    let state = workspace(&f);
    let identity = state.production.proposals[0]
        .action_identity
        .clone()
        .unwrap();
    assert_eq!(identity.project_id, p.project_id);
    assert_eq!(identity.batch_id, p.batch.batch_id);
    let subject = SelectionSubject::Proposal(identity.selection_key());
    let before = f.source_bytes();
    let view = selected(&f, &state, subject.clone());
    assert!(view.members.is_empty());
    assert!(!view.stale);
    Fixture::assert_bytes(&before);
    substrate::review_proposal_status(
        &mut f.model,
        &f.root,
        p.proposal_id,
        ProposalStatus::Deferred,
    )
    .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(
        selected(&f, &workspace(&f), subject.clone()).subject,
        subject
    );
    f.line(id(50), Point::new(20000, 0), Point::new(30000, 0));
    let state = workspace(&f);
    let before = f.source_bytes();
    let stale = selected(&f, &state, subject.clone());
    assert!(stale.stale);
    assert_eq!(stale.subject, subject);
    Fixture::assert_bytes(&before);
    // A separate actual proposal exercises rejection; Deferred has no transition
    // back to Draft in the existing producer and selection must not invent one.
    let rejected = proposal(&mut f, 60);
    let state = workspace(&f);
    let rejected_key = state
        .production
        .proposals
        .iter()
        .find(|s| s.proposal_id == rejected.proposal_id.to_string())
        .unwrap()
        .action_identity
        .as_ref()
        .unwrap()
        .selection_key();
    substrate::review_proposal_status(
        &mut f.model,
        &f.root,
        rejected.proposal_id,
        ProposalStatus::Rejected,
    )
    .unwrap();
    let state = workspace(&f);
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    let authority = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    assert!(
        !authority
            .artifact_exists(&SelectionSubject::Proposal(rejected_key))
            .unwrap()
    );
    let applied = proposal(&mut f, 70);
    let state = workspace(&f);
    let applied_key = state
        .production
        .proposals
        .iter()
        .find(|s| s.proposal_id == applied.proposal_id.to_string())
        .unwrap()
        .action_identity
        .as_ref()
        .unwrap()
        .selection_key();
    substrate::review_proposal_status(
        &mut f.model,
        &f.root,
        applied.proposal_id,
        ProposalStatus::Accepted,
    )
    .unwrap();
    substrate::apply_accepted_proposal(&mut f.model, &f.root, applied.proposal_id).unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let state = workspace(&f);
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    let authority = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    assert!(
        !authority
            .artifact_exists(&SelectionSubject::Proposal(applied_key))
            .unwrap()
    );
}
#[test]
fn malformed_foreign_or_replaced_producer_is_unavailable_and_absent_canonical_artifact_dissolves() {
    let mut f = Fixture::new("production-guard");
    let p = proposal(&mut f, 20);
    let mut state = workspace(&f);
    let mut identity = state.production.proposals[0]
        .action_identity
        .clone()
        .unwrap();
    let before = f.source_bytes();
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    identity.batch_id = id(999);
    let authority = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    assert!(
        authority
            .artifact_exists(&SelectionSubject::Proposal(identity.selection_key()))
            .is_err()
    );
    identity.batch_id = p.batch.batch_id;
    identity.project_id = id(998);
    assert!(
        authority
            .artifact_exists(&SelectionSubject::Proposal(identity.selection_key()))
            .is_err()
    );
    let key = state.production.proposals[0]
        .action_identity
        .as_ref()
        .unwrap()
        .selection_key();
    state
        .production
        .proposals
        .push(state.production.proposals[0].clone());
    let authority = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    assert!(
        authority
            .artifact_exists(&SelectionSubject::Proposal(key.clone()))
            .is_err()
    );
    Fixture::assert_bytes(&before);
    let mut payload =
        serde_json::json!({"proposal_count":1,"proposals":{p.proposal_id.to_string():p}});
    payload["proposals"][p.proposal_id.to_string()]["batch"]["expected_model_revision"] =
        serde_json::json!("foreign");
    assert!(
        datum_gui_protocol::production_status_from_proposals_json(&payload.to_string())
            .unwrap()
            .proposals[0]
            .action_identity
            .is_none()
    );
    f.model
        .commit_journal_undo(
            &f.root,
            CommitProvenance {
                actor: "native-selection-proof".into(),
                source: CommitSource::Test,
                reason: "undo producing artifact".into(),
            },
        )
        .unwrap();
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    let state = workspace(&f);
    let native = NativeSelectionResolution::capture(&f.model).unwrap();
    let authority = WorkspaceSelectionAuthority {
        native: &native,
        model: &f.model,
        workspace: &state,
    };
    assert!(
        !authority
            .artifact_exists(&SelectionSubject::Proposal(key))
            .unwrap()
    );
}
