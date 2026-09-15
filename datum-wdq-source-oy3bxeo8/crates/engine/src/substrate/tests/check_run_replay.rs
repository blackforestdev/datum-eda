use super::*;

#[test]
fn journal_replay_recovers_missing_check_run_generated_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let check_run_id = Uuid::new_v4();
    let root = temp_project_root("check_run_missing_promoted_shard");
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before check run");
    let run = CheckRun {
        schema_version: CHECK_RUN_SCHEMA_VERSION,
        check_run_id,
        project_id,
        model_revision: model.model_revision.clone(),
        profile_id: "native-combined".to_string(),
        status: "warning".to_string(),
        summary: serde_json::json!({
            "status": "warning",
            "warnings": 1,
            "errors": 0
        }),
        finding_count: 1,
        findings: vec![CheckFinding {
            finding_id: Uuid::new_v5(&project_id, b"journaled-check-finding"),
            index: 0,
            source: "drc".to_string(),
            code: "process_aperture".to_string(),
            severity: "warning".to_string(),
            fingerprint: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
            domain: "drc".to_string(),
            rule_id: "process_aperture".to_string(),
            standards_basis: None,
            standards_basis_detail: None,
            rule_revision: None,
            import_key: None,
            status: "active".to_string(),
            primary_target: serde_json::json!({ "kind": "pad", "id": "pad-test" }),
            related_targets: Vec::new(),
            message: "mask expansion below profile".to_string(),
            explanation: "mask expansion below profile".to_string(),
            suggested_next_action: Some(
                "Run datum-eda check repair-standards to create reviewed repair proposals."
                    .to_string(),
            ),
            evidence: Vec::new(),
            payload: serde_json::json!({ "detail": "mask expansion below profile" }),
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            waiver_refs: Vec::new(),
            deviation_refs: Vec::new(),
        }],
        proposal_refs: Vec::new(),
        proposal_links: Vec::new(),
        profile_basis: Default::default(),
        coverage: Vec::new(),
        raw_report: serde_json::json!({ "domain": "combined" }),
    };
    let before_run_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-check-run"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "record check run generated evidence".to_string(),
                },
                operations: vec![Operation::SetCheckRun {
                    check_run_id,
                    previous_check_run: None,
                    check_run: serde_json::to_value(&run).expect("run should serialize"),
                }],
            },
        )
        .expect("check run should commit");
    assert_eq!(
        model.model_revision, before_run_revision,
        "generated evidence must not mutate the authored model revision"
    );

    std::fs::remove_file(root.join(format!(".datum/check_runs/{check_run_id}.json")))
        .expect("promoted check run should remove");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project should recover check run from journal");
    assert_eq!(replayed.check_runs[&check_run_id], run);
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::CheckRun
            && shard.authority == SourceShardAuthority::GeneratedEvidence
            && shard.relative_path == format!(".datum/check_runs/{check_run_id}.json")
    }));
}

#[test]
fn journal_replay_deleted_check_run_suppresses_stale_promoted_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let check_run_id = Uuid::new_v4();
    let root = temp_project_root("check_run_deleted_stale_promoted_shard");
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before check run");
    let run = CheckRun {
        schema_version: CHECK_RUN_SCHEMA_VERSION,
        check_run_id,
        project_id,
        model_revision: model.model_revision.clone(),
        profile_id: "native-combined".to_string(),
        status: "warning".to_string(),
        summary: serde_json::json!({
            "status": "warning",
            "warnings": 1,
            "errors": 0
        }),
        finding_count: 1,
        findings: vec![CheckFinding {
            finding_id: Uuid::new_v5(&project_id, b"deleted-journaled-check-finding"),
            index: 0,
            source: "drc".to_string(),
            code: "process_aperture".to_string(),
            severity: "warning".to_string(),
            fingerprint: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
            domain: "drc".to_string(),
            rule_id: "process_aperture".to_string(),
            standards_basis: None,
            standards_basis_detail: None,
            rule_revision: None,
            import_key: None,
            status: "active".to_string(),
            primary_target: serde_json::json!({ "kind": "pad", "id": "pad-test" }),
            related_targets: Vec::new(),
            message: "mask expansion below profile".to_string(),
            explanation: "mask expansion below profile".to_string(),
            suggested_next_action: Some(
                "Run datum-eda check repair-standards to create reviewed repair proposals."
                    .to_string(),
            ),
            evidence: Vec::new(),
            payload: serde_json::json!({ "detail": "mask expansion below profile" }),
            proposal_refs: Vec::new(),
            proposal_links: Vec::new(),
            waiver_refs: Vec::new(),
            deviation_refs: Vec::new(),
        }],
        proposal_refs: Vec::new(),
        proposal_links: Vec::new(),
        profile_basis: Default::default(),
        coverage: Vec::new(),
        raw_report: serde_json::json!({ "domain": "combined" }),
    };
    let before_run_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-check-run-before-delete"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "record check run generated evidence before delete".to_string(),
                },
                operations: vec![Operation::SetCheckRun {
                    check_run_id,
                    previous_check_run: None,
                    check_run: serde_json::to_value(&run).expect("run should serialize"),
                }],
            },
        )
        .expect("check run should commit");
    assert_eq!(
        model.model_revision, before_run_revision,
        "generated evidence set must not mutate the authored model revision"
    );

    let promoted_path = root.join(format!(".datum/check_runs/{check_run_id}.json"));
    let stale_promoted_bytes =
        std::fs::read(&promoted_path).expect("promoted check run should exist before delete");
    let before_delete_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"delete-check-run"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete check run generated evidence".to_string(),
                },
                operations: vec![Operation::DeleteCheckRun {
                    check_run_id,
                    check_run: serde_json::to_value(&run).expect("run should serialize"),
                }],
            },
        )
        .expect("check run delete should commit");
    assert_eq!(
        model.model_revision, before_delete_revision,
        "generated evidence delete must not mutate the authored model revision"
    );
    assert!(
        !promoted_path.exists(),
        "delete operation should remove promoted check run shard"
    );

    std::fs::write(&promoted_path, stale_promoted_bytes)
        .expect("stale promoted check run should be restored to prove replay authority");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with stale promoted check run");
    assert!(
        !replayed.check_runs.contains_key(&check_run_id),
        "journaled delete must suppress stale promoted generated evidence"
    );
    assert!(!replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::CheckRun
            && shard.relative_path == format!(".datum/check_runs/{check_run_id}.json")
    }));
}
