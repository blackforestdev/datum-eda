use super::*;
use crate::substrate::artifact::persist_artifact_metadata;

#[test]
fn journal_replay_recovers_missing_artifact_run_generated_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let artifact_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
    let root = temp_project_root("artifact_run_missing_promoted_shard");
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before artifact run");
    let artifact = ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id,
        kind: ArtifactKind::Bom,
        project_id,
        model_revision: model.model_revision.clone(),
        output_job: None,
        variant: None,
        generator_version: "unit-test".to_string(),
        output_dir: None,
        files: vec![ArtifactFile {
            path: PathBuf::from("bom.csv"),
            sha256: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
        }],
        production_projections: Vec::new(),
        validation_state: ArtifactValidationState::NotValidated,
    };
    persist_artifact_metadata(&root, &artifact).expect("artifact metadata should persist");

    let run = ArtifactRun {
        schema_version: ARTIFACT_RUN_SCHEMA_VERSION,
        run_id,
        artifact_id,
        run_sequence: 1,
        project_id,
        model_revision: model.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
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
            message: "generated artifact evidence".to_string(),
        }],
    };
    let before_run_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-artifact-run"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "record artifact run generated evidence".to_string(),
                },
                operations: vec![Operation::SetArtifactRun {
                    run_id,
                    previous_artifact_run: None,
                    artifact_run: serde_json::to_value(&run).expect("run should serialize"),
                }],
            },
        )
        .expect("artifact run should commit");
    assert_eq!(
        model.model_revision, before_run_revision,
        "generated evidence must not mutate the authored model revision"
    );

    std::fs::remove_file(root.join(format!(".datum/artifact_runs/{run_id}.json")))
        .expect("promoted artifact run should remove");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project should recover artifact run from journal");
    assert_eq!(replayed.artifact_runs[&run_id], run);
    assert!(replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ArtifactRun
            && shard.authority == SourceShardAuthority::GeneratedEvidence
            && shard.relative_path == format!(".datum/artifact_runs/{run_id}.json")
            && shard.schema_version == Some(ARTIFACT_RUN_SCHEMA_VERSION)
    }));
}

#[test]
fn journal_replay_deleted_artifact_run_suppresses_stale_promoted_evidence() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let artifact_id = Uuid::new_v4();
    let run_id = Uuid::new_v4();
    let root = temp_project_root("artifact_run_deleted_stale_promoted_shard");
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before artifact run");
    let artifact = ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id,
        kind: ArtifactKind::Bom,
        project_id,
        model_revision: model.model_revision.clone(),
        output_job: None,
        variant: None,
        generator_version: "unit-test".to_string(),
        output_dir: None,
        files: vec![ArtifactFile {
            path: PathBuf::from("bom.csv"),
            sha256: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
        }],
        production_projections: Vec::new(),
        validation_state: ArtifactValidationState::NotValidated,
    };
    persist_artifact_metadata(&root, &artifact).expect("artifact metadata should persist");

    let run = ArtifactRun {
        schema_version: ARTIFACT_RUN_SCHEMA_VERSION,
        run_id,
        artifact_id,
        run_sequence: 1,
        project_id,
        model_revision: model.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
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
            message: "generated artifact evidence".to_string(),
        }],
    };
    let before_run_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-artifact-run-before-delete"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "record artifact run generated evidence before delete".to_string(),
                },
                operations: vec![Operation::SetArtifactRun {
                    run_id,
                    previous_artifact_run: None,
                    artifact_run: serde_json::to_value(&run).expect("run should serialize"),
                }],
            },
        )
        .expect("artifact run should commit");
    assert_eq!(
        model.model_revision, before_run_revision,
        "generated evidence set must not mutate the authored model revision"
    );

    let promoted_path = root.join(format!(".datum/artifact_runs/{run_id}.json"));
    let stale_promoted_bytes =
        std::fs::read(&promoted_path).expect("promoted artifact run should exist before delete");
    let before_delete_revision = model.model_revision.clone();
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"delete-artifact-run"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "delete artifact run generated evidence".to_string(),
                },
                operations: vec![Operation::DeleteArtifactRun {
                    run_id,
                    artifact_run: serde_json::to_value(&run).expect("run should serialize"),
                }],
            },
        )
        .expect("artifact run delete should commit");
    assert_eq!(
        model.model_revision, before_delete_revision,
        "generated evidence delete must not mutate the authored model revision"
    );
    assert!(
        !promoted_path.exists(),
        "delete operation should remove promoted artifact run shard"
    );

    std::fs::write(&promoted_path, stale_promoted_bytes)
        .expect("stale promoted artifact run should be restored to prove replay authority");
    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with stale promoted artifact run");
    assert!(
        !replayed.artifact_runs.contains_key(&run_id),
        "journaled delete must suppress stale promoted generated evidence"
    );
    assert!(!replayed.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ArtifactRun
            && shard.relative_path == format!(".datum/artifact_runs/{run_id}.json")
    }));
}
