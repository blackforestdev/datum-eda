use super::generated_evidence_scope::{
    empty_polygon, evidence_batch, minimal_artifact_metadata, minimal_artifact_run,
    minimal_check_run, minimal_output_job_run, stale_revision,
};
use super::*;

fn minimal_zone_fill(zone_id: Uuid, model_revision: ModelRevision) -> ZoneFill {
    ZoneFill {
        schema_version: ZONE_FILL_SCHEMA_VERSION,
        zone_id,
        state: ZoneFillState::Filled,
        source_zone_revision: ObjectRevision(0),
        model_revision,
        islands: vec![empty_polygon()],
        provenance: Some("unit-test".to_string()),
    }
}

#[test]
fn commit_journaled_rejects_stale_generated_evidence_model_revision() {
    let root = temp_project_root("generated_evidence_stale_model_revision");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");

    let output_job_run_id = Uuid::new_v4();
    let artifact_run_id = Uuid::new_v4();
    let check_run_id = Uuid::new_v4();
    let artifact_id = Uuid::new_v4();
    let zone_id = Uuid::new_v4();
    let cases = vec![
        (
            "output job run",
            Operation::SetOutputJobRun {
                run_id: output_job_run_id,
                previous_output_job_run: None,
                output_job_run: serde_json::to_value(minimal_output_job_run(
                    project_id,
                    stale_revision(),
                    output_job_run_id,
                    Uuid::new_v4(),
                ))
                .expect("output job run should serialize"),
            },
        ),
        (
            "artifact run",
            Operation::SetArtifactRun {
                run_id: artifact_run_id,
                previous_artifact_run: None,
                artifact_run: serde_json::to_value(minimal_artifact_run(
                    project_id,
                    stale_revision(),
                    artifact_run_id,
                    artifact_id,
                ))
                .expect("artifact run should serialize"),
            },
        ),
        (
            "check run",
            Operation::SetCheckRun {
                check_run_id,
                previous_check_run: None,
                check_run: serde_json::to_value(minimal_check_run(
                    project_id,
                    stale_revision(),
                    check_run_id,
                ))
                .expect("check run should serialize"),
            },
        ),
        (
            "artifact metadata",
            Operation::SetArtifactMetadata {
                artifact_id,
                previous_artifact_metadata: None,
                artifact_metadata: serde_json::to_value(minimal_artifact_metadata(
                    project_id,
                    stale_revision(),
                    artifact_id,
                ))
                .expect("artifact metadata should serialize"),
            },
        ),
        (
            "zone fill",
            Operation::SetZoneFill {
                zone_id,
                previous_zone_fill: None,
                zone_fill: serde_json::to_value(minimal_zone_fill(zone_id, stale_revision()))
                    .expect("zone fill should serialize"),
            },
        ),
    ];

    for (label, operation) in cases {
        let mut candidate = model.clone();
        let error = candidate
            .commit_journaled(
                &root,
                evidence_batch(project_id, operation, model.model_revision.clone()),
            )
            .expect_err("stale generated evidence must be rejected");
        assert!(
            error
                .to_string()
                .contains("model_revision stale-generated-evidence-revision does not match current model_revision"),
            "{label} should reject stale model revision, got {error}"
        );
        assert_eq!(candidate.model_revision, model.model_revision);
    }
}

#[test]
fn commit_journaled_rejects_generated_evidence_for_wrong_project() {
    let root = temp_project_root("generated_evidence_wrong_project");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let run_id = Uuid::new_v4();
    let wrong_project_id = Uuid::new_v4();

    let error = model
        .commit_journaled(
            &root,
            evidence_batch(
                project_id,
                Operation::SetOutputJobRun {
                    run_id,
                    previous_output_job_run: None,
                    output_job_run: serde_json::to_value(minimal_output_job_run(
                        wrong_project_id,
                        model.model_revision.clone(),
                        run_id,
                        Uuid::new_v4(),
                    ))
                    .expect("output job run should serialize"),
                },
                model.model_revision.clone(),
            ),
        )
        .expect_err("wrong-project generated evidence must be rejected");

    assert!(
        error.to_string().contains("project_id")
            && error
                .to_string()
                .contains("does not match current project_id")
    );
    assert!(
        !root
            .join(format!(".datum/output_job_runs/{run_id}.json"))
            .exists()
    );
}

#[test]
fn commit_journaled_rejects_non_output_generated_evidence_for_wrong_project() {
    let root = temp_project_root("generated_evidence_wrong_project_matrix");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");
    let wrong_project_id = Uuid::new_v4();
    let artifact_run_id = Uuid::new_v4();
    let check_run_id = Uuid::new_v4();
    let artifact_id = Uuid::new_v4();
    let cases = vec![
        (
            "artifact run",
            Operation::SetArtifactRun {
                run_id: artifact_run_id,
                previous_artifact_run: None,
                artifact_run: serde_json::to_value(minimal_artifact_run(
                    wrong_project_id,
                    model.model_revision.clone(),
                    artifact_run_id,
                    Uuid::new_v4(),
                ))
                .expect("artifact run should serialize"),
            },
            format!(".datum/artifact_runs/{artifact_run_id}.json"),
        ),
        (
            "check run",
            Operation::SetCheckRun {
                check_run_id,
                previous_check_run: None,
                check_run: serde_json::to_value(minimal_check_run(
                    wrong_project_id,
                    model.model_revision.clone(),
                    check_run_id,
                ))
                .expect("check run should serialize"),
            },
            format!(".datum/check_runs/{check_run_id}.json"),
        ),
        (
            "artifact metadata",
            Operation::SetArtifactMetadata {
                artifact_id,
                previous_artifact_metadata: None,
                artifact_metadata: serde_json::to_value(minimal_artifact_metadata(
                    wrong_project_id,
                    model.model_revision.clone(),
                    artifact_id,
                ))
                .expect("artifact metadata should serialize"),
            },
            format!(".datum/artifacts/{artifact_id}.json"),
        ),
    ];

    for (label, operation, relative_path) in cases {
        let mut candidate = model.clone();
        let error = candidate
            .commit_journaled(
                &root,
                evidence_batch(project_id, operation, model.model_revision.clone()),
            )
            .expect_err("wrong-project generated evidence must be rejected");

        assert!(
            error.to_string().contains("project_id")
                && error
                    .to_string()
                    .contains("does not match current project_id"),
            "{label} should reject wrong project id, got {error}"
        );
        assert!(!root.join(relative_path).exists());
    }
}
