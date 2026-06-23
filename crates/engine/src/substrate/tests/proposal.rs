use super::*;

fn write_legacy_proposal_sidecar(
    project_root: &Path,
    proposal: &Proposal,
) -> Result<(), EngineError> {
    let proposal_dir = project_root.join(".datum/proposals");
    std::fs::create_dir_all(&proposal_dir)?;
    let path = proposal_dir.join(format!("{}.json", proposal.proposal_id));
    std::fs::write(path, format!("{}\n", to_json_deterministic(proposal)?))?;
    Ok(())
}

fn test_proposal(
    project_id: Uuid,
    prepared_against: ModelRevision,
    package_id: Uuid,
    status: ProposalStatus,
) -> Proposal {
    let batch = OperationBatch {
        batch_id: Uuid::new_v5(&project_id, b"proposal-set-value"),
        expected_model_revision: Some(prepared_against.clone()),
        provenance: CommitProvenance {
            actor: "unit-test".to_string(),
            source: CommitSource::Test,
            reason: "apply accepted proposal".to_string(),
        },
        operations: vec![Operation::SetBoardPackageValue {
            package_id,
            value: "PROPOSED".to_string(),
        }],
    };
    Proposal {
        schema_version: 1,
        proposal_id: Uuid::new_v5(&project_id, b"proposal-set-value"),
        project_id,
        prepared_against,
        batch,
        rationale: "exercise proposal substrate".to_string(),
        affected_objects: vec![package_id],
        checks_run: Vec::new(),
        finding_fingerprints: Vec::new(),
        source: ProposalSource::Tool,
        status,
        applied_transaction_id: None,
    }
}

fn test_check_run(
    project_id: Uuid,
    model_revision: ModelRevision,
    fingerprint: String,
) -> CheckRun {
    let check_run_id = Uuid::new_v5(&project_id, b"proposal-policy-check-run");
    CheckRun {
        check_run_id,
        project_id,
        model_revision,
        profile_id: "native-standards".to_string(),
        status: "failed".to_string(),
        summary: serde_json::json!({ "failed": 1 }),
        finding_count: 1,
        findings: vec![CheckFinding {
            finding_id: Uuid::new_v5(&project_id, b"proposal-policy-finding"),
            index: 0,
            source: "standards".to_string(),
            code: "pad_mask_expansion_missing".to_string(),
            severity: "error".to_string(),
            fingerprint,
            domain: "standards".to_string(),
            rule_id: "ipc-7351-mask-expansion".to_string(),
            standards_basis: Some("IPC-7351".to_string()),
            rule_revision: Some("fixture".to_string()),
            import_key: None,
            status: "open".to_string(),
            primary_target: serde_json::json!({ "kind": "pad", "id": "pad-test" }),
            related_targets: Vec::new(),
            message: "pad mask expansion is missing".to_string(),
            explanation: "fixture finding".to_string(),
            suggested_next_action: Some("create repair proposal".to_string()),
            evidence: Vec::new(),
            payload: serde_json::json!({}),
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            waiver_refs: Vec::new(),
            deviation_refs: Vec::new(),
        }],
        proposal_refs: Vec::new(),
        proposal_links: Vec::new(),
        profile_basis: CheckRunProfileBasis::default(),
        coverage: Vec::new(),
        raw_report: serde_json::json!({}),
    }
}

#[test]
fn resolver_discovers_proposals_without_changing_model_revision() {
    let root = temp_project_root("proposal_discovery");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal = test_proposal(
        project_id,
        before.model_revision.clone(),
        package_id,
        ProposalStatus::Accepted,
    );
    write_legacy_proposal_sidecar(&root, &proposal).expect("proposal should write");

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with proposal");
    assert_eq!(after.model_revision, before.model_revision);
    assert_eq!(after.proposals.get(&proposal.proposal_id), Some(&proposal));
    assert!(after.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ProposalMetadata
            && shard.relative_path == format!(".datum/proposals/{}.json", proposal.proposal_id)
    }));
}

#[test]
fn prebuilt_proposal_metadata_can_be_committed_through_journal() {
    let root = temp_project_root("proposal_metadata_journal_create");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal = test_proposal(
        project_id,
        model.model_revision.clone(),
        package_id,
        ProposalStatus::Accepted,
    );
    let created = commit_proposal_metadata_journaled(&mut model, &root, proposal.clone())
        .expect("proposal metadata should commit through journal");

    assert_eq!(created, proposal);
    assert_eq!(model.proposals.get(&proposal.proposal_id), Some(&proposal));
    assert_eq!(model.journal.len(), 1);
    assert!(matches!(
        model.journal[0].operations.first(),
        Some(Operation::CreateProposalMetadata { proposal_id, .. }) if *proposal_id == proposal.proposal_id
    ));

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen with proposal metadata");
    assert_eq!(
        reopened.proposals.get(&proposal.proposal_id),
        Some(&proposal)
    );
}

#[test]
fn resolver_rejects_proposal_filename_mismatch() {
    let root = temp_project_root("proposal_filename_mismatch");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal = test_proposal(
        project_id,
        before.model_revision.clone(),
        package_id,
        ProposalStatus::Accepted,
    );
    let proposal_dir = root.join(".datum/proposals");
    std::fs::create_dir_all(&proposal_dir).expect("proposal dir should create");
    std::fs::write(
        proposal_dir.join(format!("{}.json", Uuid::new_v4())),
        format!(
            "{}\n",
            crate::ir::serialization::to_json_deterministic(&proposal)
                .expect("proposal should serialize")
        ),
    )
    .expect("mismatched proposal should write");

    let after = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with invalid proposal sidecar");
    assert_eq!(after.model_revision, before.model_revision);
    assert!(after.proposals.is_empty());
    assert!(after.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "proposal_filename_mismatch"
            && diagnostic
                .message
                .contains(&proposal.proposal_id.to_string())
    }));
}

#[test]
fn accepted_proposal_applies_through_journal_and_marks_applied() {
    let root = temp_project_root("proposal_apply");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal = test_proposal(
        project_id,
        before.model_revision.clone(),
        package_id,
        ProposalStatus::Accepted,
    );
    write_legacy_proposal_sidecar(&root, &proposal).expect("proposal should write");

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with proposal");
    let report = apply_accepted_proposal(&mut model, &root, proposal.proposal_id)
        .expect("accepted proposal should apply");
    assert_eq!(report.transaction.batch_id, proposal.batch.batch_id);
    assert_eq!(model.journal.len(), 1);
    assert!(matches!(
        model.journal[0].operations.last(),
        Some(Operation::SetProposalMetadata { proposal_id, .. }) if *proposal_id == proposal.proposal_id
    ));
    let applied = model
        .proposals
        .get(&proposal.proposal_id)
        .expect("proposal should remain in model");
    assert_eq!(applied.status, ProposalStatus::Applied);
    assert_eq!(
        applied.applied_transaction_id,
        Some(report.transaction.transaction_id)
    );

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(
        reopened
            .proposals
            .get(&proposal.proposal_id)
            .expect("proposal should reopen")
            .status,
        ProposalStatus::Applied
    );
    let board = reopened
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .expect("board should materialize");
    assert_eq!(
        board["packages"][package_id.to_string()]["value"],
        serde_json::json!("PROPOSED")
    );
}

#[test]
fn proposal_apply_rejects_non_accepted_or_stale_proposals() {
    let root = temp_project_root("proposal_rejects");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let draft = test_proposal(
        project_id,
        before.model_revision.clone(),
        package_id,
        ProposalStatus::Draft,
    );
    write_legacy_proposal_sidecar(&root, &draft).expect("draft proposal should write");
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with draft proposal");
    let draft_validation =
        validate_proposal_apply(&model, draft.proposal_id).expect("draft should validate");
    assert_eq!(draft_validation.blockers[0].code, "missing_acceptance");
    assert!(apply_accepted_proposal(&mut model, &root, draft.proposal_id).is_err());

    let accepted = test_proposal(
        project_id,
        before.model_revision.clone(),
        package_id,
        ProposalStatus::Accepted,
    );
    write_legacy_proposal_sidecar(&root, &accepted).expect("accepted proposal should write");
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with accepted proposal");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"intervening-change"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create stale proposal".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "INTERVENING".to_string(),
                }],
            },
        )
        .expect("intervening commit should apply");
    let stale_validation =
        validate_proposal_apply(&model, accepted.proposal_id).expect("stale should validate");
    assert_eq!(stale_validation.blockers[0].code, "stale_model_revision");
    assert!(apply_accepted_proposal(&mut model, &root, accepted.proposal_id).is_err());
}

#[test]
fn proposal_apply_validation_reports_missing_revision_guard() {
    let root = temp_project_root("proposal_missing_revision_guard");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let mut proposal = test_proposal(
        project_id,
        before.model_revision.clone(),
        package_id,
        ProposalStatus::Accepted,
    );
    proposal.batch.expected_model_revision = None;
    write_legacy_proposal_sidecar(&root, &proposal).expect("proposal should write");

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with proposal");
    let validation =
        validate_proposal_apply(&model, proposal.proposal_id).expect("proposal should validate");
    assert_eq!(validation.blockers[0].code, "missing_revision_guard");
}

#[test]
fn proposal_review_rejects_stale_acceptance() {
    let root = temp_project_root("proposal_review_stale_acceptance");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal = test_proposal(
        project_id,
        before.model_revision.clone(),
        package_id,
        ProposalStatus::Draft,
    );
    write_legacy_proposal_sidecar(&root, &proposal).expect("proposal should write");
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with proposal");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"stale-before-review"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "make proposal stale before review".to_string(),
                },
                operations: vec![Operation::SetBoardPackageValue {
                    package_id,
                    value: "INTERVENING".to_string(),
                }],
            },
        )
        .expect("intervening commit should apply");

    assert!(
        review_proposal_status(
            &mut model,
            &root,
            proposal.proposal_id,
            ProposalStatus::Accepted,
        )
        .is_err()
    );
    let deferred = review_proposal_status(
        &mut model,
        &root,
        proposal.proposal_id,
        ProposalStatus::Deferred,
    )
    .expect("stale proposal should still be deferrable");
    assert_eq!(deferred.status, ProposalStatus::Deferred);
}

#[test]
fn check_source_proposal_creation_requires_check_evidence() {
    let root = temp_project_root("check_source_proposal_creation_policy");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let batch = OperationBatch {
        batch_id: Uuid::new_v5(&project_id, b"check-proposal-policy"),
        expected_model_revision: Some(model.model_revision.clone()),
        provenance: CommitProvenance {
            actor: "unit-test".to_string(),
            source: CommitSource::Test,
            reason: "check proposal policy".to_string(),
        },
        operations: vec![Operation::SetBoardPackageValue {
            package_id,
            value: "CHECKED".to_string(),
        }],
    };
    let missing = create_draft_proposal_from_batch(
        &mut model,
        &root,
        ProposalCreateRequest {
            proposal_id: None,
            batch: batch.clone(),
            rationale: "checker repair without evidence".to_string(),
            source: ProposalSource::Check,
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
        },
    )
    .expect_err("check proposal without evidence should fail");
    assert!(missing.to_string().contains("missing_check_evidence"));

    let malformed = create_draft_proposal_from_batch(
        &mut model,
        &root,
        ProposalCreateRequest {
            proposal_id: None,
            batch: batch.clone(),
            rationale: "checker repair with malformed fingerprint".to_string(),
            source: ProposalSource::Check,
            checks_run: vec![Uuid::new_v4()],
            finding_fingerprints: vec!["sha256:not-a-digest".to_string()],
        },
    )
    .expect_err("check proposal with malformed fingerprint should fail");
    assert!(
        malformed
            .to_string()
            .contains("invalid_finding_fingerprint")
    );

    let proposal = create_draft_proposal_from_batch(
        &mut model,
        &root,
        ProposalCreateRequest {
            proposal_id: None,
            batch: batch.clone(),
            rationale: "checker repair with unknown run".to_string(),
            source: ProposalSource::Check,
            checks_run: vec![Uuid::new_v4()],
            finding_fingerprints: vec![
                "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                    .to_string(),
            ],
        },
    )
    .expect_err("check proposal with unknown run should fail");
    assert!(proposal.to_string().contains("unknown_check_run"));

    let fingerprint =
        "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b".to_string();
    let check_run = test_check_run(
        project_id,
        model.model_revision.clone(),
        fingerprint.clone(),
    );
    let check_run_id = check_run.check_run_id;
    model.check_runs.insert(check_run_id, check_run);
    let unlinked = create_draft_proposal_from_batch(
        &mut model,
        &root,
        ProposalCreateRequest {
            proposal_id: None,
            batch: batch.clone(),
            rationale: "checker repair with unlinked finding".to_string(),
            source: ProposalSource::Check,
            checks_run: vec![check_run_id],
            finding_fingerprints: vec![
                "sha256:7f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                    .to_string(),
            ],
        },
    )
    .expect_err("check proposal with unlinked finding should fail");
    assert!(
        unlinked
            .to_string()
            .contains("unlinked_finding_fingerprint")
    );

    let proposal = create_draft_proposal_from_batch(
        &mut model,
        &root,
        ProposalCreateRequest {
            proposal_id: None,
            batch,
            rationale: "checker repair with linked evidence".to_string(),
            source: ProposalSource::Check,
            checks_run: vec![check_run_id],
            finding_fingerprints: vec![fingerprint],
        },
    )
    .expect("check proposal with evidence should be created");
    assert_eq!(proposal.source, ProposalSource::Check);
}

#[test]
fn check_source_proposal_apply_validation_reports_missing_evidence() {
    let root = temp_project_root("check_source_proposal_apply_policy");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(&root, project_id, board_id, package_id);

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let mut proposal = test_proposal(
        project_id,
        before.model_revision.clone(),
        package_id,
        ProposalStatus::Accepted,
    );
    proposal.source = ProposalSource::Check;
    write_legacy_proposal_sidecar(&root, &proposal).expect("legacy sidecar proposal should write");

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with proposal");
    let validation =
        validate_proposal_apply(&model, proposal.proposal_id).expect("proposal should validate");
    let blocker_codes = validation
        .blockers
        .iter()
        .map(|blocker| blocker.code.as_str())
        .collect::<Vec<_>>();
    assert!(blocker_codes.contains(&"missing_check_evidence"));
    assert!(blocker_codes.contains(&"missing_finding_fingerprint"));

    let fingerprint =
        "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b".to_string();
    let mut linked = test_proposal(
        project_id,
        before.model_revision.clone(),
        package_id,
        ProposalStatus::Accepted,
    );
    let check_run_id = Uuid::new_v5(&project_id, b"proposal-policy-missing-check-run");
    linked.proposal_id = Uuid::new_v5(&project_id, b"proposal-policy-linked-sidecar");
    linked.source = ProposalSource::Check;
    linked.checks_run = vec![check_run_id];
    linked.finding_fingerprints = vec![fingerprint.clone()];
    write_legacy_proposal_sidecar(&root, &linked).expect("linked sidecar proposal should write");
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with linked proposal");
    let missing_run =
        validate_proposal_apply(&model, linked.proposal_id).expect("proposal should validate");
    assert!(
        missing_run
            .blockers
            .iter()
            .any(|blocker| blocker.code == "unknown_check_run")
    );

    model.check_runs.insert(
        check_run_id,
        test_check_run(
            project_id,
            model.model_revision.clone(),
            "sha256:7f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b".to_string(),
        ),
    );
    let unlinked =
        validate_proposal_apply(&model, linked.proposal_id).expect("proposal should validate");
    assert!(
        unlinked
            .blockers
            .iter()
            .any(|blocker| blocker.code == "unlinked_finding_fingerprint")
    );

    model.check_runs.insert(
        check_run_id,
        test_check_run(project_id, model.model_revision.clone(), fingerprint),
    );
    let linked_validation =
        validate_proposal_apply(&model, linked.proposal_id).expect("proposal should validate");
    assert!(linked_validation.can_apply);
}
