use super::*;
use crate::substrate::artifact::persist_artifact_metadata;

const VALID_ARTIFACT_SHA256: &str =
    "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b";

fn write_project_output_job(
    root: &Path,
    model: &mut DesignModel,
    output_job_id: Uuid,
    board_id: Uuid,
) {
    let output_job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: output_job_id,
        name: "Schema Run Job".to_string(),
        include: vec![ArtifactKind::Bom],
        prefix: "schema".to_string(),
        output_dir: None,
        board_or_panel: board_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: ObjectRevision(0),
    };
    model
        .commit_journaled(
            root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create output job for run schema test".to_string(),
                },
                operations: vec![Operation::CreateOutputJob {
                    output_job_id,
                    output_job: serde_json::to_value(output_job)
                        .expect("output job should serialize"),
                }],
            },
        )
        .expect("output job should commit");
}

fn artifact_metadata(project_id: Uuid, model_revision: ModelRevision) -> ArtifactMetadata {
    ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id: Uuid::new_v5(&project_id, b"run-schema-artifact"),
        kind: ArtifactKind::Bom,
        project_id,
        model_revision,
        output_job: None,
        variant: None,
        generator_version: "unit-test".to_string(),
        output_dir: None,
        files: vec![ArtifactFile {
            path: PathBuf::from("bom.csv"),
            sha256: VALID_ARTIFACT_SHA256.to_string(),
        }],
        production_projections: Vec::new(),
        validation_state: ArtifactValidationState::NotValidated,
    }
}

fn output_job_run(
    project_id: Uuid,
    output_job_id: Uuid,
    artifact_id: Uuid,
    model_revision: ModelRevision,
) -> OutputJobRun {
    OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id: Uuid::new_v5(&project_id, b"run-schema-output-run"),
        output_job: output_job_id,
        run_sequence: 1,
        project_id,
        model_revision,
        status: OutputJobRunStatus::Succeeded,
        artifact_id: Some(artifact_id),
        exit_code: Some(0),
        provenance: None,
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "schema output run".to_string(),
        }],
    }
}

fn artifact_run(project_id: Uuid, artifact_id: Uuid, model_revision: ModelRevision) -> ArtifactRun {
    ArtifactRun {
        schema_version: ARTIFACT_RUN_SCHEMA_VERSION,
        run_id: Uuid::new_v5(&project_id, b"run-schema-artifact-run"),
        artifact_id,
        run_sequence: 1,
        project_id,
        model_revision,
        status: OutputJobRunStatus::Succeeded,
        exit_code: Some(0),
        provenance: None,
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "schema artifact run".to_string(),
        }],
    }
}

#[test]
fn resolver_accepts_legacy_run_evidence_without_schema_version() {
    let root = temp_project_root("run_schema_legacy");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let output_job_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    write_project_output_job(&root, &mut model, output_job_id, board_id);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("output job resolves");
    let artifact = artifact_metadata(project_id, model.model_revision.clone());
    persist_artifact_metadata(&root, &artifact).expect("artifact should persist");
    let mut output_run = serde_json::to_value(output_job_run(
        project_id,
        output_job_id,
        artifact.artifact_id,
        model.model_revision.clone(),
    ))
    .expect("output run should serialize");
    output_run
        .as_object_mut()
        .expect("output run should be object")
        .remove("schema_version");
    let mut artifact_run = serde_json::to_value(artifact_run(
        project_id,
        artifact.artifact_id,
        model.model_revision,
    ))
    .expect("artifact run should serialize");
    artifact_run
        .as_object_mut()
        .expect("artifact run should be object")
        .remove("schema_version");
    write_json(
        &root.join(format!(
            ".datum/output_job_runs/{}.json",
            output_run["run_id"].as_str().expect("run id")
        )),
        output_run,
    );
    write_json(
        &root.join(format!(
            ".datum/artifact_runs/{}.json",
            artifact_run["run_id"].as_str().expect("run id")
        )),
        artifact_run,
    );

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("legacy run evidence resolves");
    let output_run = resolved
        .output_job_runs
        .values()
        .next()
        .expect("output run should load");
    let artifact_run = resolved
        .artifact_runs
        .values()
        .next()
        .expect("artifact run should load");
    assert_eq!(output_run.schema_version, OUTPUT_JOB_RUN_SCHEMA_VERSION);
    assert_eq!(artifact_run.schema_version, ARTIFACT_RUN_SCHEMA_VERSION);
    assert!(resolved.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::OutputJobRun && shard.schema_version.is_none()
    }));
    assert!(resolved.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ArtifactRun && shard.schema_version.is_none()
    }));
}

#[test]
fn resolver_rejects_unsupported_run_evidence_schema_versions() {
    let root = temp_project_root("run_schema_unsupported");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let output_job_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    write_project_output_job(&root, &mut model, output_job_id, board_id);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("output job resolves");
    let artifact = artifact_metadata(project_id, model.model_revision.clone());
    persist_artifact_metadata(&root, &artifact).expect("artifact should persist");
    let mut output_run = serde_json::to_value(output_job_run(
        project_id,
        output_job_id,
        artifact.artifact_id,
        model.model_revision.clone(),
    ))
    .expect("output run should serialize");
    output_run["schema_version"] = serde_json::json!(OUTPUT_JOB_RUN_SCHEMA_VERSION + 1);
    let mut artifact_run = serde_json::to_value(artifact_run(
        project_id,
        artifact.artifact_id,
        model.model_revision,
    ))
    .expect("artifact run should serialize");
    artifact_run["schema_version"] = serde_json::json!(ARTIFACT_RUN_SCHEMA_VERSION + 1);
    write_json(
        &root.join(format!(
            ".datum/output_job_runs/{}.json",
            output_run["run_id"].as_str().expect("run id")
        )),
        output_run,
    );
    write_json(
        &root.join(format!(
            ".datum/artifact_runs/{}.json",
            artifact_run["run_id"].as_str().expect("run id")
        )),
        artifact_run,
    );

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("invalid run evidence reports diagnostics");
    assert!(resolved.output_job_runs.is_empty());
    assert!(resolved.artifact_runs.is_empty());
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_output_job_run"
            && diagnostic
                .message
                .contains("unsupported OutputJobRun schema_version")
    }));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_artifact_run"
            && diagnostic
                .message
                .contains("unsupported ArtifactRun schema_version")
    }));
}
