use super::*;

fn write_proposal_sidecar(project_root: &Path, proposal: &Proposal) {
    let proposal_dir = project_root.join(".datum/proposals");
    std::fs::create_dir_all(&proposal_dir).expect("proposal dir should create");
    std::fs::write(
        proposal_dir.join(format!("{}.json", proposal.proposal_id)),
        format!("{}\n", to_json_deterministic(proposal).unwrap()),
    )
    .expect("proposal sidecar should write");
}

fn proposal_sidecar_path(project_root: &Path, proposal_id: Uuid) -> PathBuf {
    project_root
        .join(".datum/proposals")
        .join(format!("{proposal_id}.json"))
}

fn test_proposal(
    project_id: Uuid,
    prepared_against: ModelRevision,
    package_id: Uuid,
    status: ProposalStatus,
) -> Proposal {
    Proposal {
        schema_version: 1,
        proposal_id: Uuid::new_v5(&project_id, b"proposal-replay-set-value"),
        project_id,
        prepared_against: prepared_against.clone(),
        batch: OperationBatch {
            batch_id: Uuid::new_v5(&project_id, b"proposal-replay-set-value"),
            expected_model_revision: Some(prepared_against),
            provenance: CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "exercise proposal replay".to_string(),
            },
            operations: vec![Operation::SetBoardPackageValue {
                package_id,
                value: "PROPOSED".to_string(),
            }],
        },
        rationale: "exercise proposal metadata replay".to_string(),
        affected_objects: vec![package_id],
        checks_run: Vec::new(),
        finding_fingerprints: Vec::new(),
        source: ProposalSource::Tool,
        status,
        applied_transaction_id: None,
    }
}

fn seed_committed_proposal(root: &Path, name: &str) -> (Uuid, Proposal) {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    write_project_with_board_package(root, project_id, board_id, package_id);
    let mut model = ProjectResolver::new(root)
        .resolve()
        .unwrap_or_else(|error| panic!("{name}: project resolves: {error}"));
    let proposal = test_proposal(
        project_id,
        model.model_revision.clone(),
        package_id,
        ProposalStatus::Accepted,
    );
    commit_proposal_metadata_journaled(&mut model, root, proposal.clone())
        .unwrap_or_else(|error| panic!("{name}: proposal metadata should commit: {error}"));
    (package_id, proposal)
}

fn assert_materialized_proposal(model: &DesignModel, proposal: &Proposal) {
    let relative_path = format!(".datum/proposals/{}.json", proposal.proposal_id);
    let materialized = model
        .materialized_source_shard_value_by_relative_path(&relative_path)
        .expect("proposal should materialize from journal");
    assert_eq!(
        serde_json::from_value::<Proposal>(materialized).unwrap(),
        *proposal
    );
}

#[test]
fn journal_replay_recovers_missing_proposal_metadata() {
    let root = temp_project_root("proposal_replay_missing");
    let (_, proposal) = seed_committed_proposal(&root, "missing");
    std::fs::remove_file(proposal_sidecar_path(&root, proposal.proposal_id))
        .expect("promoted proposal sidecar should remove");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should recover missing proposal metadata");
    assert_eq!(
        reopened.proposals.get(&proposal.proposal_id),
        Some(&proposal)
    );
    let relative_path = format!(".datum/proposals/{}.json", proposal.proposal_id);
    assert!(reopened.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ProposalMetadata
            && shard.relative_path == relative_path
            && shard.dirty_state == SourceShardDirtyState::Missing
    }));
    assert_materialized_proposal(&reopened, &proposal);
}

#[test]
fn journal_replay_marks_stale_promoted_proposal_metadata_dirty() {
    let root = temp_project_root("proposal_replay_dirty");
    let (_, proposal) = seed_committed_proposal(&root, "dirty");
    let mut stale = proposal.clone();
    stale.status = ProposalStatus::Draft;
    write_proposal_sidecar(&root, &stale);

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should recover stale proposal metadata");
    assert_eq!(
        reopened.proposals.get(&proposal.proposal_id),
        Some(&proposal)
    );
    let relative_path = format!(".datum/proposals/{}.json", proposal.proposal_id);
    assert!(reopened.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ProposalMetadata
            && shard.relative_path == relative_path
            && shard.dirty_state == SourceShardDirtyState::Dirty
    }));
    assert_materialized_proposal(&reopened, &proposal);
}

#[test]
fn journal_replay_deleted_proposal_metadata_suppresses_stale_promoted_file() {
    let root = temp_project_root("proposal_replay_deleted");
    let (_, proposal) = seed_committed_proposal(&root, "deleted");
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before delete");
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete proposal metadata".to_string(),
                },
                operations: vec![Operation::DeleteProposalMetadata {
                    proposal_id: proposal.proposal_id,
                    relative_path: format!(".datum/proposals/{}.json", proposal.proposal_id),
                    proposal: serde_json::to_value(&proposal).unwrap(),
                }],
            },
        )
        .expect("proposal metadata delete should commit");
    write_proposal_sidecar(&root, &proposal);

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should suppress stale deleted proposal metadata");
    assert!(!reopened.proposals.contains_key(&proposal.proposal_id));
    assert!(!reopened.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ProposalMetadata
            && shard.relative_path == format!(".datum/proposals/{}.json", proposal.proposal_id)
    }));
}

#[test]
fn journal_replay_recovers_unreadable_proposal_metadata() {
    let root = temp_project_root("proposal_replay_unknown");
    let (_, proposal) = seed_committed_proposal(&root, "unknown");
    let path = proposal_sidecar_path(&root, proposal.proposal_id);
    std::fs::remove_file(&path).expect("promoted proposal sidecar should remove");
    std::fs::create_dir(&path).expect("unreadable replacement directory should create");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should recover unreadable proposal metadata");
    assert_eq!(
        reopened.proposals.get(&proposal.proposal_id),
        Some(&proposal)
    );
    let relative_path = format!(".datum/proposals/{}.json", proposal.proposal_id);
    assert!(reopened.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ProposalMetadata
            && shard.relative_path == relative_path
            && shard.dirty_state == SourceShardDirtyState::Unknown
    }));
    assert_materialized_proposal(&reopened, &proposal);
}
