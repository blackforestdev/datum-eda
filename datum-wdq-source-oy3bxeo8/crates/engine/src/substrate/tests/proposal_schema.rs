use super::*;

fn schema_test_proposal(
    project_id: Uuid,
    prepared_against: ModelRevision,
    proposal_id: Uuid,
    schema_version: u64,
) -> Proposal {
    Proposal {
        schema_version,
        proposal_id,
        project_id,
        prepared_against: prepared_against.clone(),
        batch: OperationBatch {
            batch_id: Uuid::new_v5(&proposal_id, b"schema-test-batch"),
            expected_model_revision: Some(prepared_against),
            provenance: CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "proposal schema regression".to_string(),
            },
            operations: Vec::new(),
        },
        rationale: "proposal schema regression".to_string(),
        affected_objects: Vec::new(),
        checks_run: Vec::new(),
        finding_fingerprints: Vec::new(),
        source: ProposalSource::Tool,
        status: ProposalStatus::Draft,
        applied_transaction_id: None,
    }
}

#[test]
fn resolver_rejects_unsupported_proposal_metadata_schema_version() {
    let root = temp_project_root("proposal_metadata_unsupported_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let proposal_id = Uuid::new_v4();
    write_json(
        &root.join(format!(".datum/proposals/{proposal_id}.json")),
        serde_json::json!({
            "schema_version": 2,
            "proposal_id": proposal_id
        }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with proposal metadata diagnostic");

    assert!(model.proposals.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_proposal_metadata"
            && diagnostic
                .message
                .contains("unsupported ProposalMetadata schema_version 2")
    }));
}

#[test]
fn proposal_operations_reject_unsupported_payload_schema_versions() {
    let root = temp_project_root("proposal_operation_unsupported_payload_schema");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");

    let proposal_id = Uuid::new_v5(&project_id, b"unsupported-proposal-payload-schema");
    let proposal = schema_test_proposal(
        project_id,
        model.model_revision.clone(),
        proposal_id,
        PROPOSAL_SCHEMA_VERSION + 1,
    );
    let error = super::super::proposal_journal_ops::proposal_from_value(
        &serde_json::to_value(&proposal).expect("proposal should serialize"),
    )
    .expect_err("unsupported proposal schema should be rejected");
    assert!(
        error
            .to_string()
            .contains("unsupported Proposal schema_version 2")
    );
}

#[test]
fn resolver_defaults_legacy_proposal_payload_schema_version() {
    let root = temp_project_root("legacy_proposal_payload_schema");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let proposal_id = Uuid::new_v5(&project_id, b"legacy-proposal-payload-schema");
    let mut proposal = serde_json::to_value(schema_test_proposal(
        project_id,
        before.model_revision,
        proposal_id,
        PROPOSAL_SCHEMA_VERSION,
    ))
    .expect("proposal should serialize");
    proposal
        .as_object_mut()
        .expect("proposal should be object")
        .remove("schema_version");
    write_json(
        &root.join(format!(".datum/proposals/{proposal_id}.json")),
        proposal,
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with legacy proposal");
    let proposal = model
        .proposals
        .get(&proposal_id)
        .expect("proposal should resolve");
    assert_eq!(proposal.schema_version, PROPOSAL_SCHEMA_VERSION);
}
