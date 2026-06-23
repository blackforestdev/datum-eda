use super::*;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn execute(cli: Cli) -> Result<String> {
    let (output, _) = execute_with_exit_code(cli)?;
    Ok(output)
}

fn policy_batch(root: &Path) -> (Uuid, PathBuf) {
    let model = ProjectResolver::new(root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"proposal-policy-contract");
    let batch = OperationBatch {
        batch_id: Uuid::new_v5(&model.project.project_id, b"proposal-policy-contract-batch"),
        expected_model_revision: None,
        provenance: CommitProvenance {
            actor: "test".to_string(),
            source: CommitSource::Cli,
            reason: "exercise proposal policy contract".to_string(),
        },
        operations: vec![Operation::SetProjectName {
            project_id: model.project.project_id,
            name: "Applied Through Policy Contract".to_string(),
        }],
    };
    let batch_path = root.join("proposal-policy-batch.json");
    std::fs::write(&batch_path, serde_json::to_string_pretty(&batch).unwrap()).unwrap();
    (proposal_id, batch_path)
}

fn assert_policy_contract(value: &serde_json::Value) {
    assert_eq!(
        value["policy"],
        "accepted_revision_guarded_source_policy_v1"
    );
    assert_eq!(value["approval_path"], "draft_review_accept_then_apply");
}

fn assert_validation_policy_contract(validation: &serde_json::Value) {
    assert_policy_contract(validation);
    assert_eq!(validation["acceptance_required"], true);
    assert_eq!(validation["current_revision_required"], true);
    assert_eq!(validation["revision_guard_required"], true);
    assert_eq!(validation["check_source_evidence_required"], true);
}

#[test]
fn proposal_reports_policy_contract_on_create_validate_and_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-policy-contract");
    create_native_project(&root, Some("Proposal Policy Contract Demo".to_string()))
        .expect("initial scaffold should succeed");
    let (proposal_id, batch_path) = policy_batch(&root);

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create",
            root.to_str().unwrap(),
            "--batch",
            batch_path.to_str().unwrap(),
            "--rationale",
            "exercise proposal policy contract",
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal create should succeed");
    let create: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    assert_validation_policy_contract(&create["validation"]);
    assert_eq!(create["validation"]["can_apply"], false);

    let validate_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "validate",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal validate should succeed");
    let validate: serde_json::Value = serde_json::from_str(&validate_output).unwrap();
    assert_validation_policy_contract(&validate);
    assert_eq!(
        validate["blocker_codes"],
        serde_json::json!(["missing_acceptance"])
    );

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal accept-apply should succeed");
    let apply: serde_json::Value = serde_json::from_str(&apply_output).unwrap();
    assert_policy_contract(&apply);
    assert_validation_policy_contract(&apply["validation"]);
    assert_eq!(apply["validation"]["status"], "accepted");
    assert_eq!(apply["validation"]["can_apply"], true);
    assert_eq!(apply["status"], "applied");

    let _ = std::fs::remove_dir_all(&root);
}
