//! Actual GUI ERC producer evidence through the native manual Proposal lifecycle.
use super::main_tests_project_check::{build_native_check_fixture, unique_project_root};
use super::*;
use eda_engine::{
    api::native_write::{
        WriteProvenance,
        schematic_delete_proposal::{
            self as proposal, DispositionKind, NativeProposalAction, NativeProposalRequest,
            NativeProposalResponse,
        },
    },
    substrate::{CommitSource, DesignModel, ProjectResolver, ProposalStatus, SourceShardKind},
};

fn model(root: &Path) -> DesignModel {
    ProjectResolver::new(root).resolve().unwrap()
}
fn source(model: &DesignModel) -> serde_json::Value {
    model
        .materialized_source_shard_value(SourceShardKind::SchematicRoot)
        .unwrap()
}
fn request(
    model: &DesignModel,
    root: &Path,
    action: NativeProposalAction,
) -> NativeProposalRequest {
    NativeProposalRequest {
        project_root: root.into(),
        expected_model_revision: model.model_revision.0.clone(),
        expected_transaction_tip: eda_engine::revision::transaction_tip(&model.journal),
        action,
    }
}
fn act(
    model: &mut DesignModel,
    root: &Path,
    action: NativeProposalAction,
) -> NativeProposalResponse {
    let request = request(model, root, action);
    proposal::execute(model, &request).unwrap()
}
fn create(id: Uuid, run: Uuid, fingerprint: &str, kind: DispositionKind) -> NativeProposalAction {
    NativeProposalAction::CreateDisposition {
        proposal_id: id,
        check_run_id: run,
        fingerprint: fingerprint.into(),
        disposition: kind,
        rationale: "Intentional fixture exception, reviewed by designer".into(),
    }
}
fn fixture(name: &str) -> (PathBuf, DesignModel, Uuid, String) {
    let root = unique_project_root(name);
    create_native_project(&root, Some("ERC disposition".into())).unwrap();
    build_native_check_fixture(&root);
    let before = model(&root);
    // This is the guarded persisted producer used by the ordinary GUI ERC action.
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "run",
            root.to_str().unwrap(),
            "--profile",
            "erc",
            "--expected-project",
            &before.project.project_id.to_string(),
            "--expected-revision",
            &before.model_revision.0,
            "--progress-job",
            &Uuid::new_v4().to_string(),
        ])
        .unwrap(),
    )
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let run = Uuid::parse_str(report["check_run_id"].as_str().unwrap()).unwrap();
    let model = model(&root);
    let check = &model.check_runs[&run];
    assert_eq!(check.model_revision, before.model_revision);
    assert_eq!(check.raw_report, report["raw_report"]);
    assert_eq!(check.profile_id, "erc");
    let finding = check
        .findings
        .iter()
        .find(|f| f.domain == "erc" && f.status == "active")
        .unwrap();
    assert!(!finding.fingerprint.is_empty());
    let fingerprint = finding.fingerprint.clone();
    (root, model, run, fingerprint)
}
fn assert_report(root: &Path, fingerprint: &str, status: &str) {
    let before = model(root);
    let report = query_native_project_check_run_with_profile(root, Some("erc")).unwrap();
    assert!(!report.persisted);
    assert_eq!(
        report
            .findings
            .iter()
            .find(|f| f.fingerprint == fingerprint)
            .unwrap()
            .status,
        status
    );
    let after = model(root);
    assert_eq!(after.model_revision, before.model_revision);
    assert_eq!(after.journal.len(), before.journal.len());
}

#[test]
fn native_erc_disposition_real_producer_review_history_reopen() {
    for kind in [DispositionKind::Waiver, DispositionKind::Deviation] {
        let (root, mut state, run, fingerprint) = fixture("native-erc-disposition-lifecycle");
        let before = source(&state);
        let revision = state.model_revision.clone();
        let identities = state.electrical_identities.clone();
        let id = Uuid::new_v4();
        let draft = act(&mut state, &root, create(id, run, &fingerprint, kind));
        assert_eq!(draft.status, ProposalStatus::Draft);
        assert!(draft.deleted.is_empty());
        assert_eq!(draft.disposition.as_ref().unwrap().check_run_id, run);
        assert_eq!(state.proposals[&id].checks_run, vec![run]);
        assert_eq!(
            state.proposals[&id].finding_fingerprints,
            vec![fingerprint.clone()]
        );
        assert_eq!(state.model_revision, revision); // Proposal metadata is not design mutation.
        assert_eq!(source(&state), before);
        let journal_len = state.journal.len();
        assert_eq!(
            act(
                &mut state,
                &root,
                NativeProposalAction::Preview { proposal_id: id }
            )
            .disposition,
            draft.disposition
        );
        assert_eq!(state.journal.len(), journal_len);
        assert_eq!(source(&model(&root)), before);
        let applied = act(
            &mut state,
            &root,
            NativeProposalAction::AcceptApply { proposal_id: id },
        );
        assert_eq!(applied.status, ProposalStatus::Applied);
        assert_eq!(applied.transaction_id, applied.transaction_tip);
        assert_eq!(applied.disposition, draft.disposition);
        let after = source(&state);
        assert_ne!(after, before);
        assert_eq!(state.electrical_identities, identities);
        let expected = if kind == DispositionKind::Waiver {
            "waived"
        } else {
            "accepted_deviation"
        };
        assert_report(&root, &fingerprint, expected);
        // Recovery reads after an ambiguous acknowledgement never write again.
        let len = state.journal.len();
        assert_eq!(
            act(
                &mut state,
                &root,
                NativeProposalAction::Preview { proposal_id: id }
            )
            .status,
            ProposalStatus::Applied
        );
        assert_eq!(state.journal.len(), len);
        let history =
            || WriteProvenance::new("test", CommitSource::Test, "disposition history").into();
        state.commit_journal_undo(&root, history()).unwrap();
        state = model(&root);
        assert_eq!(source(&state), before);
        assert_eq!(state.proposals[&id].status, ProposalStatus::Accepted);
        assert_report(&root, &fingerprint, "active");
        state.commit_journal_redo(&root, history()).unwrap();
        state = model(&root);
        assert_eq!(source(&state), after);
        assert_eq!(state.proposals[&id].status, ProposalStatus::Applied);
        assert_eq!(state.electrical_identities, identities);
        assert_report(&root, &fingerprint, expected);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn native_erc_disposition_cancel_stale_and_duplicate_are_write_safe() {
    let (root, mut state, run, fingerprint) = fixture("native-erc-disposition-refusals");
    let before = source(&state);
    let rejected = Uuid::new_v4();
    act(
        &mut state,
        &root,
        create(rejected, run, &fingerprint, DispositionKind::Waiver),
    );
    assert_eq!(
        act(
            &mut state,
            &root,
            NativeProposalAction::Reject {
                proposal_id: rejected
            }
        )
        .status,
        ProposalStatus::Rejected
    );
    assert_eq!(source(&state), before);
    let stale = Uuid::new_v4();
    act(
        &mut state,
        &root,
        create(stale, run, &fingerprint, DispositionKind::Waiver),
    );
    let winner = Uuid::new_v4();
    act(
        &mut state,
        &root,
        create(winner, run, &fingerprint, DispositionKind::Deviation),
    );
    act(
        &mut state,
        &root,
        NativeProposalAction::AcceptApply {
            proposal_id: winner,
        },
    );
    let after = source(&state);
    let len = state.journal.len();
    for action in [
        NativeProposalAction::Preview { proposal_id: stale },
        NativeProposalAction::AcceptApply { proposal_id: stale },
    ] {
        let request = request(&state, &root, action);
        assert!(proposal::execute(&mut state, &request).is_err());
        assert_eq!(state.journal.len(), len);
        assert_eq!(source(&state), after);
    }
    // Dismissal remains available after unrelated design changes.
    act(
        &mut state,
        &root,
        NativeProposalAction::Reject { proposal_id: stale },
    );
    assert_eq!(source(&state), after);
    // A current real check run sees the existing disposition and refuses replacement.
    let refreshed = run_native_project_check_with_profile(&root, Some("erc")).unwrap();
    state = model(&root);
    let request = request(
        &state,
        &root,
        create(
            Uuid::new_v4(),
            refreshed.check_run_id,
            &fingerprint,
            DispositionKind::Waiver,
        ),
    );
    let len = state.journal.len();
    assert!(proposal::execute(&mut state, &request).is_err());
    assert_eq!(state.journal.len(), len);
    assert_eq!(source(&state), after);
    std::fs::remove_dir_all(root).unwrap();
}
