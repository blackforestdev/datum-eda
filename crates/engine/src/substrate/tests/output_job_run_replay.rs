use super::*;

#[test]
fn journal_replay_recovers_missing_output_job_run_generated_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let output_job_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
    let root = temp_project_root("output_job_run_missing_promoted_shard");
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before output job");
    let output_job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: output_job_id,
        name: "Journaled Gerbers".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "journaled".to_string(),
        output_dir: None,
        board_or_panel: board_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: ObjectRevision(0),
    };
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"create-output-job-for-run"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create output job for generated evidence run".to_string(),
                },
                operations: vec![Operation::CreateOutputJob {
                    output_job_id,
                    output_job: serde_json::to_value(&output_job)
                        .expect("output job should serialize"),
                }],
            },
        )
        .expect("output job should commit");

    let run = OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id,
        output_job: output_job_id,
        run_sequence: 1,
        project_id,
        model_revision: model.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: None,
        exit_code: Some(0),
        provenance: Some(OutputJobRunProvenance {
            launcher: OutputJobRunLauncher::GuiTerminal,
            terminal_session_id: Some("terminal-test".to_string()),
            terminal_context_path: Some(PathBuf::from(".datum/terminal-context.json")),
            project_root: Some(root.clone()),
            source_revision: None,
        }),
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "generated output evidence".to_string(),
        }],
    };
    let before_run_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-output-job-run"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "record output job run generated evidence".to_string(),
                },
                operations: vec![Operation::SetOutputJobRun {
                    run_id,
                    previous_output_job_run: None,
                    output_job_run: serde_json::to_value(&run).expect("run should serialize"),
                }],
            },
        )
        .expect("output job run should commit");
    assert_eq!(
        model.model_revision, before_run_revision,
        "generated evidence must not mutate the authored model revision"
    );

    std::fs::remove_file(root.join(format!(".datum/output_job_runs/{run_id}.json")))
        .expect("promoted output job run should remove");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project should recover output job run from journal");
    assert_eq!(replayed.output_job_runs[&run_id], run);
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::OutputJobRun
            && shard.authority == SourceShardAuthority::GeneratedEvidence
            && shard.relative_path == format!(".datum/output_job_runs/{run_id}.json")
            && shard.schema_version == Some(OUTPUT_JOB_RUN_SCHEMA_VERSION)
    }));
}

#[test]
fn journal_replay_deleted_output_job_run_suppresses_stale_promoted_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let output_job_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
    let root = temp_project_root("output_job_run_deleted_stale_promoted_shard");
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before output job");
    let output_job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: output_job_id,
        name: "Journaled Gerbers".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "journaled".to_string(),
        output_dir: None,
        board_or_panel: board_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: ObjectRevision(0),
    };
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"create-output-job-for-deleted-run"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create output job for deleted generated evidence run".to_string(),
                },
                operations: vec![Operation::CreateOutputJob {
                    output_job_id,
                    output_job: serde_json::to_value(&output_job)
                        .expect("output job should serialize"),
                }],
            },
        )
        .expect("output job should commit");

    let run = OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id,
        output_job: output_job_id,
        run_sequence: 1,
        project_id,
        model_revision: model.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: None,
        exit_code: Some(0),
        provenance: Some(OutputJobRunProvenance {
            launcher: OutputJobRunLauncher::GuiTerminal,
            terminal_session_id: Some("terminal-test".to_string()),
            terminal_context_path: Some(PathBuf::from(".datum/terminal-context.json")),
            project_root: Some(root.clone()),
            source_revision: None,
        }),
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "generated output evidence".to_string(),
        }],
    };
    let before_run_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-output-job-run-before-delete"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "record output job run generated evidence before delete".to_string(),
                },
                operations: vec![Operation::SetOutputJobRun {
                    run_id,
                    previous_output_job_run: None,
                    output_job_run: serde_json::to_value(&run).expect("run should serialize"),
                }],
            },
        )
        .expect("output job run should commit");
    assert_eq!(
        model.model_revision, before_run_revision,
        "generated evidence set must not mutate the authored model revision"
    );

    let promoted_path = root.join(format!(".datum/output_job_runs/{run_id}.json"));
    let stale_promoted_bytes =
        std::fs::read(&promoted_path).expect("promoted output job run should exist before delete");
    let before_delete_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"delete-output-job-run"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete output job run generated evidence".to_string(),
                },
                operations: vec![Operation::DeleteOutputJobRun {
                    run_id,
                    output_job_run: serde_json::to_value(&run).expect("run should serialize"),
                }],
            },
        )
        .expect("output job run delete should commit");
    assert_eq!(
        model.model_revision, before_delete_revision,
        "generated evidence delete must not mutate the authored model revision"
    );
    assert!(
        !promoted_path.exists(),
        "delete operation should remove promoted output job run shard"
    );

    std::fs::write(&promoted_path, stale_promoted_bytes)
        .expect("stale promoted output job run should be restored to prove replay authority");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with stale promoted output job run");
    assert!(
        !replayed.output_job_runs.contains_key(&run_id),
        "journaled delete must suppress stale promoted generated evidence"
    );
    assert!(!replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::OutputJobRun
            && shard.relative_path == format!(".datum/output_job_runs/{run_id}.json")
    }));
}
