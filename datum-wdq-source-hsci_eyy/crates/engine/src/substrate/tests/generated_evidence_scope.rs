use super::*;
use crate::ir::geometry::{Point, Polygon};

pub(super) fn evidence_batch(
    project_id: Uuid,
    operation: Operation,
    model_revision: ModelRevision,
) -> OperationBatch {
    OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(model_revision),
        provenance: CommitProvenance {
            actor: "unit-test".to_string(),
            source: CommitSource::Test,
            reason: format!("generated evidence scope test for {project_id}"),
        },
        operations: vec![operation],
    }
}

pub(super) fn stale_revision() -> ModelRevision {
    ModelRevision("stale-generated-evidence-revision".to_string())
}

pub(super) fn empty_polygon() -> Polygon {
    Polygon {
        vertices: vec![
            Point { x: 0, y: 0 },
            Point { x: 1000, y: 0 },
            Point { x: 1000, y: 1000 },
            Point { x: 0, y: 1000 },
        ],
        closed: true,
    }
}

fn evidence_log(message: &str) -> Vec<OutputJobLogEntry> {
    vec![OutputJobLogEntry {
        sequence: 1,
        level: OutputJobLogLevel::Info,
        message: message.to_string(),
    }]
}

pub(super) fn minimal_check_run(
    project_id: Uuid,
    model_revision: ModelRevision,
    check_run_id: Uuid,
) -> CheckRun {
    CheckRun {
        schema_version: CHECK_RUN_SCHEMA_VERSION,
        check_run_id,
        project_id,
        model_revision,
        profile_id: "native-combined".to_string(),
        status: "ok".to_string(),
        summary: serde_json::json!({ "status": "ok" }),
        finding_count: 0,
        findings: Vec::new(),
        proposal_refs: Vec::new(),
        proposal_links: Vec::new(),
        profile_basis: CheckRunProfileBasis::default(),
        coverage: Vec::new(),
        raw_report: serde_json::json!({ "domain": "combined" }),
    }
}

pub(super) fn minimal_artifact_metadata(
    project_id: Uuid,
    model_revision: ModelRevision,
    artifact_id: Uuid,
) -> ArtifactMetadata {
    ArtifactMetadata {
        schema_version: ARTIFACT_METADATA_SCHEMA_VERSION,
        artifact_id,
        kind: ArtifactKind::GerberSet,
        project_id,
        model_revision,
        output_job: None,
        variant: None,
        generator_version: "unit-test".to_string(),
        output_dir: Some(PathBuf::from("fab")),
        files: vec![ArtifactFile {
            path: PathBuf::from("fab/board-F_Cu.gbr"),
            sha256: "sha256:5f9fd18a00b8234c45d8e981d96bf609c0c8b3d32a4f58b8c34f277ed111cb9b"
                .to_string(),
        }],
        production_projections: Vec::new(),
        validation_state: ArtifactValidationState::NotValidated,
    }
}

pub(super) fn minimal_output_job_run(
    project_id: Uuid,
    model_revision: ModelRevision,
    run_id: Uuid,
    output_job: Uuid,
) -> OutputJobRun {
    OutputJobRun {
        schema_version: OUTPUT_JOB_RUN_SCHEMA_VERSION,
        run_id,
        output_job,
        run_sequence: 1,
        project_id,
        model_revision,
        status: OutputJobRunStatus::Succeeded,
        artifact_id: None,
        exit_code: Some(0),
        provenance: None,
        log: evidence_log("output job evidence"),
    }
}

fn minimal_output_job(output_job_id: Uuid, board_id: Uuid) -> OutputJob {
    OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: output_job_id,
        name: "Generated Evidence Link Job".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "generated-evidence".to_string(),
        output_dir: None,
        board_or_panel: board_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: ObjectRevision(0),
    }
}

pub(super) fn minimal_artifact_run(
    project_id: Uuid,
    model_revision: ModelRevision,
    run_id: Uuid,
    artifact_id: Uuid,
) -> ArtifactRun {
    ArtifactRun {
        schema_version: ARTIFACT_RUN_SCHEMA_VERSION,
        run_id,
        artifact_id,
        run_sequence: 1,
        project_id,
        model_revision,
        status: OutputJobRunStatus::Succeeded,
        exit_code: Some(0),
        provenance: None,
        log: evidence_log("artifact evidence"),
    }
}

#[test]
fn commit_journaled_rejects_output_job_run_with_missing_output_job_without_staging() {
    let root = temp_project_root("generated_evidence_missing_output_job");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let run_id = Uuid::new_v4();
    let run = minimal_output_job_run(
        project_id,
        model.model_revision.clone(),
        run_id,
        Uuid::new_v4(),
    );

    let error = model
        .commit_journaled(
            &root,
            evidence_batch(
                project_id,
                Operation::SetOutputJobRun {
                    run_id,
                    previous_output_job_run: None,
                    output_job_run: serde_json::to_value(run).expect("run should serialize"),
                },
                model.model_revision.clone(),
            ),
        )
        .expect_err("missing output job link should reject at commit time");

    assert!(
        error
            .to_string()
            .contains("output job run references missing output job")
    );
    assert!(
        !root
            .join(format!(".datum/output_job_runs/{run_id}.json"))
            .exists()
    );
}

#[test]
fn commit_journaled_rejects_artifact_run_with_missing_artifact_without_staging() {
    let root = temp_project_root("generated_evidence_missing_artifact");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let run_id = Uuid::new_v4();
    let run = minimal_artifact_run(
        project_id,
        model.model_revision.clone(),
        run_id,
        Uuid::new_v4(),
    );

    let error = model
        .commit_journaled(
            &root,
            evidence_batch(
                project_id,
                Operation::SetArtifactRun {
                    run_id,
                    previous_artifact_run: None,
                    artifact_run: serde_json::to_value(run).expect("run should serialize"),
                },
                model.model_revision.clone(),
            ),
        )
        .expect_err("missing artifact metadata link should reject at commit time");

    assert!(
        error
            .to_string()
            .contains("artifact run references missing artifact metadata")
    );
    assert!(
        !root
            .join(format!(".datum/artifact_runs/{run_id}.json"))
            .exists()
    );
}

#[test]
fn commit_journaled_rejects_output_job_run_with_missing_artifact_without_staging() {
    let root = temp_project_root("generated_evidence_output_missing_artifact");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let output_job_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let run_id = Uuid::new_v4();
    let mut run = minimal_output_job_run(
        project_id,
        model.model_revision.clone(),
        run_id,
        output_job_id,
    );
    run.artifact_id = Some(Uuid::new_v4());

    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove output run missing artifact guard".to_string(),
                },
                operations: vec![
                    Operation::CreateOutputJob {
                        output_job_id,
                        output_job: serde_json::to_value(minimal_output_job(
                            output_job_id,
                            board_id,
                        ))
                        .expect("output job should serialize"),
                    },
                    Operation::SetOutputJobRun {
                        run_id,
                        previous_output_job_run: None,
                        output_job_run: serde_json::to_value(run).expect("run should serialize"),
                    },
                ],
            },
        )
        .expect_err("missing output run artifact metadata link should reject at commit time");

    assert!(
        error
            .to_string()
            .contains("output job run references missing artifact metadata")
    );
    assert!(
        !root
            .join(format!(".datum/output_job_runs/{run_id}.json"))
            .exists()
    );
}

#[test]
fn commit_journaled_rejects_output_job_run_with_mismatched_artifact_metadata_without_staging() {
    let root = temp_project_root("generated_evidence_output_bad_artifact");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let output_job_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");

    for (label, artifact_project_id, artifact_model_revision, expected_message) in [
        (
            "wrong project",
            Uuid::new_v4(),
            model.model_revision.clone(),
            "output job run artifact project_id does not match run",
        ),
        (
            "wrong model revision",
            project_id,
            stale_revision(),
            "output job run artifact model_revision does not match run",
        ),
    ] {
        let mut candidate = model.clone();
        let artifact_id = Uuid::new_v4();
        let run_id = Uuid::new_v4();
        candidate.artifact_metadata.insert(
            artifact_id,
            minimal_artifact_metadata(artifact_project_id, artifact_model_revision, artifact_id),
        );
        let mut run = minimal_output_job_run(
            project_id,
            candidate.model_revision.clone(),
            run_id,
            output_job_id,
        );
        run.artifact_id = Some(artifact_id);

        let error = candidate
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v4(),
                    expected_model_revision: Some(candidate.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "unit-test".to_string(),
                        source: CommitSource::Test,
                        reason: format!("prove output run artifact link guard: {label}"),
                    },
                    operations: vec![
                        Operation::CreateOutputJob {
                            output_job_id,
                            output_job: serde_json::to_value(minimal_output_job(
                                output_job_id,
                                board_id,
                            ))
                            .expect("output job should serialize"),
                        },
                        Operation::SetOutputJobRun {
                            run_id,
                            previous_output_job_run: None,
                            output_job_run: serde_json::to_value(run)
                                .expect("run should serialize"),
                        },
                    ],
                },
            )
            .expect_err("mismatched output run artifact metadata link should reject");

        assert!(
            error.to_string().contains(expected_message),
            "{label} should reject with {expected_message}, got {error}"
        );
        assert!(
            !root
                .join(format!(".datum/output_job_runs/{run_id}.json"))
                .exists()
        );
    }
}

#[test]
fn commit_journaled_rejects_duplicate_output_job_run_sequence_without_staging() {
    let root = temp_project_root("generated_evidence_duplicate_output_sequence");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let output_job_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let run_id = Uuid::new_v4();
    let duplicate_run_id = Uuid::new_v4();
    let run = minimal_output_job_run(
        project_id,
        model.model_revision.clone(),
        run_id,
        output_job_id,
    );
    let duplicate_run = minimal_output_job_run(
        project_id,
        model.model_revision.clone(),
        duplicate_run_id,
        output_job_id,
    );

    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove duplicate generated run sequence guard".to_string(),
                },
                operations: vec![
                    Operation::CreateOutputJob {
                        output_job_id,
                        output_job: serde_json::to_value(minimal_output_job(
                            output_job_id,
                            board_id,
                        ))
                        .expect("output job should serialize"),
                    },
                    Operation::SetOutputJobRun {
                        run_id,
                        previous_output_job_run: None,
                        output_job_run: serde_json::to_value(run).expect("run should serialize"),
                    },
                    Operation::SetOutputJobRun {
                        run_id: duplicate_run_id,
                        previous_output_job_run: None,
                        output_job_run: serde_json::to_value(duplicate_run)
                            .expect("duplicate run should serialize"),
                    },
                ],
            },
        )
        .expect_err("duplicate output run sequence should reject at commit time");

    assert!(
        error
            .to_string()
            .contains("duplicate output job run_sequence for output job")
    );
    assert!(
        !root
            .join(format!(".datum/output_job_runs/{run_id}.json"))
            .exists()
    );
    assert!(
        !root
            .join(format!(".datum/output_job_runs/{duplicate_run_id}.json"))
            .exists()
    );
}

#[test]
fn commit_journaled_rejects_artifact_run_with_mismatched_artifact_metadata_without_staging() {
    let root = temp_project_root("generated_evidence_artifact_bad_artifact");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");

    for (label, artifact_project_id, artifact_model_revision, expected_message) in [
        (
            "wrong project",
            Uuid::new_v4(),
            model.model_revision.clone(),
            "artifact run artifact project_id does not match run",
        ),
        (
            "wrong model revision",
            project_id,
            stale_revision(),
            "artifact run artifact model_revision does not match run",
        ),
    ] {
        let mut candidate = model.clone();
        let artifact_id = Uuid::new_v4();
        let run_id = Uuid::new_v4();
        candidate.artifact_metadata.insert(
            artifact_id,
            minimal_artifact_metadata(artifact_project_id, artifact_model_revision, artifact_id),
        );
        let run = minimal_artifact_run(
            project_id,
            candidate.model_revision.clone(),
            run_id,
            artifact_id,
        );

        let error = candidate
            .commit_journaled(
                &root,
                evidence_batch(
                    project_id,
                    Operation::SetArtifactRun {
                        run_id,
                        previous_artifact_run: None,
                        artifact_run: serde_json::to_value(run).expect("run should serialize"),
                    },
                    candidate.model_revision.clone(),
                ),
            )
            .expect_err("mismatched artifact run metadata link should reject");

        assert!(
            error.to_string().contains(expected_message),
            "{label} should reject with {expected_message}, got {error}"
        );
        assert!(
            !root
                .join(format!(".datum/artifact_runs/{run_id}.json"))
                .exists()
        );
    }
}

#[test]
fn commit_journaled_rejects_duplicate_artifact_run_sequence_without_staging() {
    let root = temp_project_root("generated_evidence_duplicate_artifact_sequence");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let artifact_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let run_id = Uuid::new_v4();
    let duplicate_run_id = Uuid::new_v4();
    let run = minimal_artifact_run(
        project_id,
        model.model_revision.clone(),
        run_id,
        artifact_id,
    );
    let duplicate_run = minimal_artifact_run(
        project_id,
        model.model_revision.clone(),
        duplicate_run_id,
        artifact_id,
    );

    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v4(),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "prove duplicate artifact run sequence guard".to_string(),
                },
                operations: vec![
                    Operation::SetArtifactMetadata {
                        artifact_id,
                        previous_artifact_metadata: None,
                        artifact_metadata: serde_json::to_value(minimal_artifact_metadata(
                            project_id,
                            model.model_revision.clone(),
                            artifact_id,
                        ))
                        .expect("artifact metadata should serialize"),
                    },
                    Operation::SetArtifactRun {
                        run_id,
                        previous_artifact_run: None,
                        artifact_run: serde_json::to_value(run).expect("run should serialize"),
                    },
                    Operation::SetArtifactRun {
                        run_id: duplicate_run_id,
                        previous_artifact_run: None,
                        artifact_run: serde_json::to_value(duplicate_run)
                            .expect("duplicate run should serialize"),
                    },
                ],
            },
        )
        .expect_err("duplicate artifact run sequence should reject at commit time");

    assert!(
        error
            .to_string()
            .contains("duplicate artifact run_sequence for artifact")
    );
    assert!(
        !root
            .join(format!(".datum/artifact_runs/{run_id}.json"))
            .exists()
    );
    assert!(
        !root
            .join(format!(".datum/artifact_runs/{duplicate_run_id}.json"))
            .exists()
    );
}
