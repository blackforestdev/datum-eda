use super::*;
use crate::substrate::artifact::persist_artifact_metadata;

const VALID_ARTIFACT_SHA256: &str =
    "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b";

#[test]
fn generated_evidence_rejects_unsafe_artifact_paths() {
    let root = temp_project_root("generated_evidence_path_guard");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let mut artifact = ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id: Uuid::new_v5(&project_id, b"unsafe-artifact"),
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision: before.model_revision.clone(),
        output_job: None,
        variant: None,
        generator_version: "test-generator".to_string(),
        output_dir: None,
        files: vec![ArtifactFile {
            path: PathBuf::from("../escape.gbr"),
            sha256: VALID_ARTIFACT_SHA256.to_string(),
        }],
        production_projections: Vec::new(),
        validation_state: ArtifactValidationState::NotValidated,
    };

    let error = persist_artifact_metadata(&root, &artifact)
        .expect_err("unsafe artifact path should be rejected");
    assert!(error.to_string().contains("unsafe component"));

    let artifact_dir = root.join(".datum/artifacts");
    std::fs::create_dir_all(&artifact_dir).expect("artifact dir should create");
    std::fs::write(
        artifact_dir.join(format!("{}.json", artifact.artifact_id)),
        format!(
            "{}\n",
            to_json_deterministic(&artifact).expect("artifact should serialize")
        ),
    )
    .expect("unsafe artifact metadata should write");
    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with diagnostics");
    assert!(
        !resolved
            .artifact_metadata
            .contains_key(&artifact.artifact_id)
    );
    assert!(
        resolved
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "invalid_artifact_metadata"
                && diagnostic.message.contains("unsafe component"))
    );

    artifact.files[0].path = PathBuf::from("/absolute.gbr");
    let error = persist_artifact_metadata(&root, &artifact)
        .expect_err("absolute artifact path should be rejected");
    assert!(error.to_string().contains("must be relative"));
}

#[test]
fn resolver_rejects_invalid_artifact_hash_and_projection_metadata() {
    let root = temp_project_root("invalid_artifact_proof_hashes");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let artifact = ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id: Uuid::new_v5(&project_id, b"invalid-proof-artifact"),
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision: before.model_revision.clone(),
        output_job: None,
        variant: None,
        generator_version: "test-generator".to_string(),
        output_dir: None,
        files: vec![ArtifactFile {
            path: PathBuf::from("board-F_Cu.gbr"),
            sha256: "sha256:not-a-real-digest".to_string(),
        }],
        production_projections: vec![ArtifactProductionProjection {
            projection_kind: "gerber_copper_layer".to_string(),
            projection_contract: "datum.production_projection.gerber_copper_layer.v1".to_string(),
            model_revision: before.model_revision.clone(),
            byte_count: 128,
            sha256: "sha256:also-not-a-real-digest".to_string(),
        }],
        validation_state: ArtifactValidationState::NotValidated,
    };
    let artifact_dir = root.join(".datum/artifacts");
    std::fs::create_dir_all(&artifact_dir).expect("artifact dir should create");
    std::fs::write(
        artifact_dir.join(format!("{}.json", artifact.artifact_id)),
        format!(
            "{}\n",
            to_json_deterministic(&artifact).expect("artifact should serialize")
        ),
    )
    .expect("invalid artifact metadata should write");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with diagnostics");
    assert!(
        !resolved
            .artifact_metadata
            .contains_key(&artifact.artifact_id)
    );
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_artifact_metadata"
            && diagnostic
                .message
                .contains("sha256:<64 lowercase hex> value")
    }));
}

#[test]
fn resolver_rejects_artifact_metadata_with_blank_generator_version() {
    let root = temp_project_root("blank_artifact_generator_version");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let artifact = ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id: Uuid::new_v5(&project_id, b"blank-generator-artifact"),
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision: before.model_revision,
        output_job: None,
        variant: None,
        generator_version: "  ".to_string(),
        output_dir: None,
        files: vec![ArtifactFile {
            path: PathBuf::from("board-F_Cu.gbr"),
            sha256: VALID_ARTIFACT_SHA256.to_string(),
        }],
        production_projections: Vec::new(),
        validation_state: ArtifactValidationState::NotValidated,
    };

    let error = persist_artifact_metadata(&root, &artifact)
        .expect_err("blank generator_version should be rejected");
    assert!(
        error
            .to_string()
            .contains("artifact generator_version must not be blank")
    );

    let artifact_dir = root.join(".datum/artifacts");
    std::fs::create_dir_all(&artifact_dir).expect("artifact dir should create");
    std::fs::write(
        artifact_dir.join(format!("{}.json", artifact.artifact_id)),
        format!(
            "{}\n",
            to_json_deterministic(&artifact).expect("artifact should serialize")
        ),
    )
    .expect("invalid artifact metadata should write");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with diagnostics");
    assert!(
        !resolved
            .artifact_metadata
            .contains_key(&artifact.artifact_id)
    );
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_artifact_metadata"
            && diagnostic
                .message
                .contains("artifact generator_version must not be blank")
    }));
}

#[test]
fn resolver_rejects_invalid_output_job_and_artifact_run_evidence() {
    let root = temp_project_root("invalid_run_evidence");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let output_job_run = OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id: Uuid::new_v5(&project_id, b"invalid-output-job-run"),
        output_job: Uuid::new_v5(&project_id, b"output-job"),
        run_sequence: 0,
        project_id,
        model_revision: before.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: None,
        exit_code: Some(1),
        provenance: None,
        log: Vec::new(),
    };
    let artifact_run = ArtifactRun {
        schema_version: ARTIFACT_RUN_SCHEMA_VERSION,
        run_id: Uuid::new_v5(&project_id, b"invalid-artifact-run"),
        artifact_id: Uuid::new_v5(&project_id, b"artifact"),
        run_sequence: 1,
        project_id,
        model_revision: before.model_revision.clone(),
        status: OutputJobRunStatus::Running,
        exit_code: Some(0),
        provenance: None,
        log: vec![OutputJobLogEntry {
            sequence: 2,
            level: OutputJobLogLevel::Info,
            message: "sequence starts at two".to_string(),
        }],
    };
    let output_run_dir = root.join(".datum/output_job_runs");
    std::fs::create_dir_all(&output_run_dir).expect("output run dir should create");
    std::fs::write(
        output_run_dir.join(format!("{}.json", output_job_run.run_id)),
        format!(
            "{}\n",
            to_json_deterministic(&output_job_run).expect("output job run should serialize")
        ),
    )
    .expect("invalid output job run should write");
    let artifact_run_dir = root.join(".datum/artifact_runs");
    std::fs::create_dir_all(&artifact_run_dir).expect("artifact run dir should create");
    std::fs::write(
        artifact_run_dir.join(format!("{}.json", artifact_run.run_id)),
        format!(
            "{}\n",
            to_json_deterministic(&artifact_run).expect("artifact run should serialize")
        ),
    )
    .expect("invalid artifact run should write");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with diagnostics");
    assert!(
        !resolved
            .output_job_runs
            .contains_key(&output_job_run.run_id)
    );
    assert!(!resolved.artifact_runs.contains_key(&artifact_run.run_id));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_output_job_run"
            && diagnostic.message.contains("run_sequence must be greater")
    }));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_artifact_run"
            && diagnostic
                .message
                .contains("running run evidence must not have an exit_code")
    }));
}

#[test]
fn resolver_rejects_run_evidence_with_blank_provenance_fields() {
    let root = temp_project_root("invalid_run_evidence_provenance");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let output_job_run = OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id: Uuid::new_v5(&project_id, b"invalid-output-job-run-provenance"),
        output_job: Uuid::new_v5(&project_id, b"output-job"),
        run_sequence: 1,
        project_id,
        model_revision: before.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: None,
        exit_code: Some(0),
        provenance: Some(OutputJobRunProvenance {
            launcher: OutputJobRunLauncher::GuiTerminal,
            terminal_session_id: Some("  ".to_string()),
            terminal_context_path: None,
            project_root: None,
            source_revision: None,
        }),
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "blank provenance".to_string(),
        }],
    };
    let artifact_run = ArtifactRun {
        schema_version: ARTIFACT_RUN_SCHEMA_VERSION,
        run_id: Uuid::new_v5(&project_id, b"invalid-artifact-run-provenance"),
        artifact_id: Uuid::new_v5(&project_id, b"artifact"),
        run_sequence: 1,
        project_id,
        model_revision: before.model_revision,
        status: OutputJobRunStatus::Succeeded,
        exit_code: Some(0),
        provenance: Some(OutputJobRunProvenance {
            launcher: OutputJobRunLauncher::GuiTerminal,
            terminal_session_id: None,
            terminal_context_path: Some(PathBuf::new()),
            project_root: None,
            source_revision: Some("".to_string()),
        }),
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "empty provenance path".to_string(),
        }],
    };
    let output_run_dir = root.join(".datum/output_job_runs");
    std::fs::create_dir_all(&output_run_dir).expect("output run dir should create");
    std::fs::write(
        output_run_dir.join(format!("{}.json", output_job_run.run_id)),
        format!(
            "{}\n",
            to_json_deterministic(&output_job_run).expect("output job run should serialize")
        ),
    )
    .expect("invalid output job run should write");
    let artifact_run_dir = root.join(".datum/artifact_runs");
    std::fs::create_dir_all(&artifact_run_dir).expect("artifact run dir should create");
    std::fs::write(
        artifact_run_dir.join(format!("{}.json", artifact_run.run_id)),
        format!(
            "{}\n",
            to_json_deterministic(&artifact_run).expect("artifact run should serialize")
        ),
    )
    .expect("invalid artifact run should write");

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with diagnostics");
    assert!(
        !resolved
            .output_job_runs
            .contains_key(&output_job_run.run_id)
    );
    assert!(!resolved.artifact_runs.contains_key(&artifact_run.run_id));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_output_job_run"
            && diagnostic
                .message
                .contains("terminal_session_id must not be blank")
    }));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_artifact_run"
            && diagnostic
                .message
                .contains("terminal_context_path must not be empty")
    }));
}

#[test]
fn resolver_rejects_run_evidence_with_broken_links_and_duplicate_sequences() {
    let root = temp_project_root("invalid_run_evidence_links");
    let project_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, Uuid::new_v4());
    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("minimal project should resolve");
    let output_job_id = Uuid::new_v5(&project_id, b"linked-output-job");
    let output_job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: output_job_id,
        name: "Linked Output Job".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "linked".to_string(),
        output_dir: None,
        board_or_panel: before.project.project_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: ObjectRevision(0),
    };
    let output_job_dir = root.join(".datum/output_jobs");
    std::fs::create_dir_all(&output_job_dir).expect("output job dir should create");
    std::fs::write(
        output_job_dir.join(format!("{}.json", output_job.id)),
        format!(
            "{}\n",
            to_json_deterministic(&output_job).expect("output job should serialize")
        ),
    )
    .expect("output job should write");

    let with_job = ProjectResolver::new(&root)
        .resolve()
        .expect("output job project should resolve");
    let artifact = ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id: Uuid::new_v5(&project_id, b"linked-artifact"),
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision: with_job.model_revision.clone(),
        output_job: Some(output_job_id),
        variant: None,
        generator_version: "test-generator".to_string(),
        output_dir: None,
        files: vec![ArtifactFile {
            path: PathBuf::from("board-F_Cu.gbr"),
            sha256: VALID_ARTIFACT_SHA256.to_string(),
        }],
        production_projections: Vec::new(),
        validation_state: ArtifactValidationState::NotValidated,
    };
    persist_artifact_metadata(&root, &artifact).expect("artifact metadata should persist");

    let valid_output_run = OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id: Uuid::new_v5(&project_id, b"valid-output-run"),
        output_job: output_job_id,
        run_sequence: 1,
        project_id,
        model_revision: artifact.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        artifact_id: Some(artifact.artifact_id),
        exit_code: Some(0),
        provenance: None,
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "valid output job run".to_string(),
        }],
    };
    let duplicate_output_run = OutputJobRun {
        run_id: Uuid::new_v5(&project_id, b"duplicate-output-run"),
        ..valid_output_run.clone()
    };
    let missing_output_job_run = OutputJobRun {
        run_id: Uuid::new_v5(&project_id, b"missing-output-job-run"),
        output_job: Uuid::new_v5(&project_id, b"missing-output-job"),
        run_sequence: 2,
        ..valid_output_run.clone()
    };
    let output_run_dir = root.join(".datum/output_job_runs");
    std::fs::create_dir_all(&output_run_dir).expect("output run dir should create");
    for run in [
        &valid_output_run,
        &duplicate_output_run,
        &missing_output_job_run,
    ] {
        std::fs::write(
            output_run_dir.join(format!("{}.json", run.run_id)),
            format!(
                "{}\n",
                to_json_deterministic(run).expect("output job run should serialize")
            ),
        )
        .expect("output job run should write");
    }

    let valid_artifact_run = ArtifactRun {
        schema_version: ARTIFACT_RUN_SCHEMA_VERSION,
        run_id: Uuid::new_v5(&project_id, b"valid-artifact-run"),
        artifact_id: artifact.artifact_id,
        run_sequence: 1,
        project_id,
        model_revision: artifact.model_revision.clone(),
        status: OutputJobRunStatus::Succeeded,
        exit_code: Some(0),
        provenance: None,
        log: vec![OutputJobLogEntry {
            sequence: 1,
            level: OutputJobLogLevel::Info,
            message: "valid artifact run".to_string(),
        }],
    };
    let duplicate_artifact_run = ArtifactRun {
        run_id: Uuid::new_v5(&project_id, b"duplicate-artifact-run"),
        ..valid_artifact_run.clone()
    };
    let missing_artifact_run = ArtifactRun {
        run_id: Uuid::new_v5(&project_id, b"missing-artifact-run"),
        artifact_id: Uuid::new_v5(&project_id, b"missing-artifact"),
        run_sequence: 2,
        ..valid_artifact_run.clone()
    };
    let artifact_run_dir = root.join(".datum/artifact_runs");
    std::fs::create_dir_all(&artifact_run_dir).expect("artifact run dir should create");
    for run in [
        &valid_artifact_run,
        &duplicate_artifact_run,
        &missing_artifact_run,
    ] {
        std::fs::write(
            artifact_run_dir.join(format!("{}.json", run.run_id)),
            format!(
                "{}\n",
                to_json_deterministic(run).expect("artifact run should serialize")
            ),
        )
        .expect("artifact run should write");
    }

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with run evidence diagnostics");
    assert!(resolved.output_job_runs.is_empty());
    assert!(resolved.artifact_runs.is_empty());
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_output_job_run"
            && diagnostic.message.contains("references missing output job")
    }));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_output_job_run"
            && diagnostic
                .message
                .contains("duplicate output job run_sequence")
    }));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_artifact_run"
            && diagnostic
                .message
                .contains("references missing artifact metadata")
    }));
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_artifact_run"
            && diagnostic
                .message
                .contains("duplicate artifact run_sequence")
    }));
}
