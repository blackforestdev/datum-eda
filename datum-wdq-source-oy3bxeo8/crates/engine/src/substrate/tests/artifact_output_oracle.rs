use super::*;

#[test]
fn generated_output_artifact_graph_replays_without_model_revision_mutation() {
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let output_job_id = Uuid::new_v4();
    let artifact_id = Uuid::new_v4();
    let output_run_id = Uuid::new_v4();
    let artifact_run_id = Uuid::new_v4();
    let root = temp_project_root("generated_output_artifact_graph_revision_oracle");
    write_minimal_project(&root, project_id, board_id);

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves before output job");
    let output_job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: output_job_id,
        name: "Linked production output".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "linked".to_string(),
        output_dir: Some(PathBuf::from("fab")),
        board_or_panel: board_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: ObjectRevision(0),
    };
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"create-linked-output-job"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create linked output job".to_string(),
                },
                operations: vec![Operation::CreateOutputJob {
                    output_job_id,
                    output_job: serde_json::to_value(&output_job)
                        .expect("output job should serialize"),
                }],
            },
        )
        .expect("output job should commit");

    let authored_revision = model.model_revision.clone();
    let artifact = ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id,
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision: authored_revision.clone(),
        output_job: Some(output_job_id),
        variant: None,
        generator_version: "unit-test".to_string(),
        output_dir: Some(PathBuf::from("fab")),
        files: vec![ArtifactFile {
            path: PathBuf::from("fab/linked-F_Cu.gbr"),
            sha256: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
        }],
        production_projections: vec![ArtifactProductionProjection {
            projection_kind: "gerber_copper_layer".to_string(),
            projection_contract: "datum.production_projection.gerber_copper_layer.v1".to_string(),
            model_revision: authored_revision.clone(),
            byte_count: 128,
            sha256: "sha256:28b3adfae87a0db63bb3e0f8bb9ea8f7c6f1f9955b5f7f4188c5bb47a0f5f3f6"
                .to_string(),
        }],
        validation_state: ArtifactValidationState::NotValidated,
    };
    let output_run = OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id: output_run_id,
        output_job: output_job_id,
        run_sequence: 1,
        project_id,
        model_revision: authored_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: Some(artifact_id),
        exit_code: Some(0),
        provenance: Some(OutputJobRunProvenance {
            launcher: OutputJobRunLauncher::GuiTerminal,
            terminal_session_id: Some("terminal-linked".to_string()),
            terminal_context_path: Some(PathBuf::from(".datum/terminal-context.json")),
            project_root: Some(root.clone()),
            source_revision: Some(authored_revision.0.clone()),
        }),
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "linked output evidence".to_string(),
        }],
    };
    let artifact_run = ArtifactRun {
        schema_version: ARTIFACT_RUN_SCHEMA_VERSION,
        run_id: artifact_run_id,
        artifact_id,
        run_sequence: 1,
        project_id,
        model_revision: authored_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        exit_code: Some(0),
        provenance: Some(OutputJobRunProvenance {
            launcher: OutputJobRunLauncher::GuiTerminal,
            terminal_session_id: Some("terminal-linked".to_string()),
            terminal_context_path: Some(PathBuf::from(".datum/terminal-context.json")),
            project_root: Some(root.clone()),
            source_revision: Some(authored_revision.0.clone()),
        }),
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "linked artifact evidence".to_string(),
        }],
    };

    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"set-linked-generated-output-evidence"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "record linked generated output evidence".to_string(),
                },
                operations: vec![
                    Operation::SetArtifactMetadata {
                        artifact_id,
                        previous_artifact_metadata: None,
                        artifact_metadata: serde_json::to_value(&artifact)
                            .expect("artifact should serialize"),
                    },
                    Operation::SetOutputJobRun {
                        run_id: output_run_id,
                        previous_output_job_run: None,
                        output_job_run: serde_json::to_value(&output_run)
                            .expect("output run should serialize"),
                    },
                    Operation::SetArtifactRun {
                        run_id: artifact_run_id,
                        previous_artifact_run: None,
                        artifact_run: serde_json::to_value(&artifact_run)
                            .expect("artifact run should serialize"),
                    },
                ],
            },
        )
        .expect("linked generated evidence should commit");
    assert_eq!(
        model.model_revision, authored_revision,
        "linked generated evidence must not mutate authored model revision"
    );

    std::fs::remove_file(root.join(format!(".datum/artifacts/{artifact_id}.json")))
        .expect("promoted artifact metadata should remove");
    std::fs::remove_file(root.join(format!(".datum/output_job_runs/{output_run_id}.json")))
        .expect("promoted output job run should remove");
    std::fs::remove_file(root.join(format!(".datum/artifact_runs/{artifact_run_id}.json")))
        .expect("promoted artifact run should remove");

    let replayed = ProjectResolver::new(&root)
        .resolve()
        .expect("project should recover linked generated evidence from journal");
    assert_eq!(replayed.model_revision, authored_revision);
    assert_eq!(replayed.artifact_metadata[&artifact_id], artifact);
    assert_eq!(replayed.output_job_runs[&output_run_id], output_run);
    assert_eq!(replayed.artifact_runs[&artifact_run_id], artifact_run);
    assert_eq!(
        replayed.output_job_runs[&output_run_id].artifact_id,
        Some(artifact_id)
    );
    assert_eq!(
        replayed.artifact_runs[&artifact_run_id].artifact_id,
        artifact_id
    );
    assert_missing_generated_evidence_shard(
        &replayed,
        SourceShardKind::ArtifactMetadata,
        format!(".datum/artifacts/{artifact_id}.json"),
    );
    assert_missing_generated_evidence_shard(
        &replayed,
        SourceShardKind::OutputJobRun,
        format!(".datum/output_job_runs/{output_run_id}.json"),
    );
    assert_missing_generated_evidence_shard(
        &replayed,
        SourceShardKind::ArtifactRun,
        format!(".datum/artifact_runs/{artifact_run_id}.json"),
    );
}

fn assert_missing_generated_evidence_shard(
    model: &DesignModel,
    kind: SourceShardKind,
    relative_path: String,
) {
    assert!(model.source_shards.iter().any(|shard| {
        shard.kind == kind
            && shard.authority == SourceShardAuthority::GeneratedEvidence
            && shard.dirty_state == SourceShardDirtyState::Missing
            && shard.relative_path == relative_path
    }));
}
