use super::*;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver, Proposal,
    ProposalSource, ProposalStatus, SourceShardKind, commit_proposal_metadata_journaled,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn commit_forward_annotation_review(root: &Path) -> String {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before forward-annotation review");
    let relative_path = ".datum/forward_annotation_review/review.json".to_string();
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "cli-test".to_string(),
                    source: CommitSource::Cli,
                    reason: "record unreadable forward-annotation review sidecar".to_string(),
                },
                operations: vec![Operation::SetForwardAnnotationReview {
                    relative_path: relative_path.clone(),
                    previous_review: None,
                    review: serde_json::json!({
                        "schema_version": 1,
                        "review_id": Uuid::new_v4(),
                        "status": "pending",
                        "entries": []
                    }),
                }],
            },
        )
        .expect("forward-annotation review should commit");
    relative_path
}

fn commit_proposal_metadata(root: &Path) -> (Uuid, String) {
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves before proposal metadata");
    let proposal_id = Uuid::new_v4();
    let proposal = Proposal {
        schema_version: 1,
        proposal_id,
        project_id: model.project.project_id,
        prepared_against: model.model_revision.clone(),
        batch: OperationBatch {
            batch_id: Uuid::new_v4(),
            expected_model_revision: Some(model.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "cli-test".to_string(),
                source: CommitSource::Cli,
                reason: "empty proposal fixture".to_string(),
            },
            operations: Vec::new(),
        },
        rationale: "record unreadable proposal metadata sidecar".to_string(),
        affected_objects: Vec::new(),
        checks_run: Vec::new(),
        finding_fingerprints: Vec::new(),
        source: ProposalSource::Cli,
        status: ProposalStatus::Draft,
        applied_transaction_id: None,
    };
    commit_proposal_metadata_journaled(&mut model, root, proposal)
        .expect("proposal metadata should commit");
    (proposal_id, format!(".datum/proposals/{proposal_id}.json"))
}

#[test]
fn project_query_resolve_debug_reports_unknown_forward_annotation_review_sidecar() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-fa-review");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown Forward Annotation Review Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let relative_path = commit_forward_annotation_review(&root);
    let promoted_path = root.join(&relative_path);
    std::fs::remove_file(&promoted_path)
        .expect("promoted forward-annotation review sidecar should remove");
    std::fs::create_dir(&promoted_path)
        .expect("directory at promoted forward-annotation review sidecar path should create");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    assert!(
        report["source_shards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|shard| {
                shard["path"] == relative_path
                    && shard["kind"] == "ForwardAnnotationReview"
                    && shard["taxon"] == "ForwardAnnotationReview"
                    && shard["authority"] == "SidecarMetadata"
                    && shard["dirty_state"] == "Unknown"
            }),
        "resolve-debug should expose unreadable journal-recovered forward-annotation review as Unknown"
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should still resolve with unreadable promoted review");
    let materialized = model
        .materialized_source_shard_value(SourceShardKind::ForwardAnnotationReview)
        .expect("journal-recovered review should materialize despite unreadable promoted path");
    assert_eq!(
        materialized["schema_version"], 1,
        "materialized review should come from the retained journal write"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_resolve_debug_reports_unknown_proposal_metadata_sidecar() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-unknown-proposal");
    create_native_project(
        &root,
        Some("Resolve Debug Unknown Proposal Metadata Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let (proposal_id, relative_path) = commit_proposal_metadata(&root);
    let promoted_path = root.join(&relative_path);
    std::fs::remove_file(&promoted_path).expect("promoted proposal metadata should remove");
    std::fs::create_dir(&promoted_path)
        .expect("directory at promoted proposal metadata path should create");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    assert!(
        report["source_shards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|shard| {
                shard["path"] == relative_path
                    && shard["kind"] == "ProposalMetadata"
                    && shard["taxon"] == "ProposalMetadata"
                    && shard["authority"] == "SidecarMetadata"
                    && shard["dirty_state"] == "Unknown"
            }),
        "resolve-debug should expose unreadable journal-recovered proposal metadata as Unknown: {proposal_id}"
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should still resolve with unreadable promoted proposal");
    assert!(
        model.proposals.contains_key(&proposal_id),
        "journal-recovered proposal metadata should populate the resolved proposal map"
    );
    let materialized = model
        .materialized_source_shard_value_by_relative_path(&relative_path)
        .expect("journal-recovered proposal should materialize despite unreadable promoted path");
    assert_eq!(materialized["proposal_id"], proposal_id.to_string());

    let _ = std::fs::remove_dir_all(&root);
}
