use super::*;

#[test]
fn resolver_rejects_unsupported_production_shard_schema_versions() {
    let root = temp_project_root("production_unsupported_schema_versions");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let plan_id = Uuid::new_v4();
    let panel_id = Uuid::new_v4();
    let output_job_id = Uuid::new_v4();
    write_json(
        &root.join(format!(".datum/manufacturing_plans/{plan_id}.json")),
        serde_json::json!({ "schema_version": 2 }),
    );
    write_json(
        &root.join(format!(".datum/panel_projections/{panel_id}.json")),
        serde_json::json!({ "schema_version": 2 }),
    );
    write_json(
        &root.join(format!(".datum/output_jobs/{output_job_id}.json")),
        serde_json::json!({ "schema_version": 2 }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with production schema diagnostics");

    assert!(model.manufacturing_plans.is_empty());
    assert!(model.panel_projections.is_empty());
    assert!(model.output_jobs.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_manufacturing_plan"
            && diagnostic
                .message
                .contains("unsupported ManufacturingPlan schema_version 2")
    }));
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_panel_projection"
            && diagnostic
                .message
                .contains("unsupported PanelProjection schema_version 2")
    }));
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_output_job"
            && diagnostic
                .message
                .contains("unsupported OutputJob schema_version 2")
    }));
}

#[test]
fn resolver_rejects_unsupported_artifact_metadata_schema_version() {
    let root = temp_project_root("artifact_metadata_unsupported_schema_version");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let artifact_id = Uuid::new_v4();
    write_json(
        &root.join(format!(".datum/artifacts/{artifact_id}.json")),
        serde_json::json!({ "schema_version": 2 }),
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves with artifact metadata diagnostic");

    assert!(model.artifact_metadata.is_empty());
    assert!(model.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_artifact_metadata"
            && diagnostic
                .message
                .contains("unsupported ArtifactMetadata schema_version 2")
    }));
}

#[test]
fn production_operations_reject_unsupported_payload_schema_versions() {
    let root = temp_project_root("production_operation_unsupported_payload_schema");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve");

    let panel_id = Uuid::new_v5(&project_id, b"unsupported-panel-payload-schema");
    let mut panel = PanelProjection {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION + 1,
        id: panel_id,
        name: "Unsupported panel".to_string(),
        board_instances: vec![PanelBoardInstance {
            board: board_id,
            x_nm: 0,
            y_nm: 0,
            rotation_deg: 0,
        }],
        object_revision: ObjectRevision(0),
    };
    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"bad-panel-schema"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "reject unsupported panel schema".to_string(),
                },
                operations: vec![Operation::CreatePanelProjection {
                    panel_projection_id: panel_id,
                    panel_projection: serde_json::to_value(&panel).expect("panel should serialize"),
                }],
            },
        )
        .expect_err("unsupported panel projection schema should be rejected");
    assert!(
        error
            .to_string()
            .contains("unsupported PanelProjection schema_version 2")
    );

    panel.schema_version = PRODUCTION_RECORD_SCHEMA_VERSION;
    let plan_id = Uuid::new_v5(&project_id, b"unsupported-plan-payload-schema");
    let plan = ManufacturingPlan {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION + 1,
        id: plan_id,
        name: "Unsupported plan".to_string(),
        board_or_panel: board_id,
        variant: None,
        prefix: "unsupported-plan".to_string(),
        object_revision: ObjectRevision(0),
    };
    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"bad-plan-schema"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "reject unsupported plan schema".to_string(),
                },
                operations: vec![Operation::CreateManufacturingPlan {
                    manufacturing_plan_id: plan_id,
                    manufacturing_plan: serde_json::to_value(&plan).expect("plan should serialize"),
                }],
            },
        )
        .expect_err("unsupported manufacturing plan schema should be rejected");
    assert!(
        error
            .to_string()
            .contains("unsupported ManufacturingPlan schema_version 2")
    );

    let job_id = Uuid::new_v5(&project_id, b"unsupported-job-payload-schema");
    let job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION + 1,
        id: job_id,
        name: "Unsupported job".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "unsupported-job".to_string(),
        output_dir: None,
        board_or_panel: board_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: ObjectRevision(0),
    };
    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"bad-job-schema"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "reject unsupported job schema".to_string(),
                },
                operations: vec![Operation::CreateOutputJob {
                    output_job_id: job_id,
                    output_job: serde_json::to_value(&job).expect("job should serialize"),
                }],
            },
        )
        .expect_err("unsupported output job schema should be rejected");
    assert!(
        error
            .to_string()
            .contains("unsupported OutputJob schema_version 2")
    );
}

#[test]
fn resolver_defaults_legacy_production_record_payload_schema_versions() {
    let root = temp_project_root("legacy_production_record_payload_schema");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let plan_id = Uuid::new_v5(&project_id, b"legacy-plan-schema");
    let panel_id = Uuid::new_v5(&project_id, b"legacy-panel-schema");
    let job_id = Uuid::new_v5(&project_id, b"legacy-job-schema");
    write_json(
        &root.join(format!(".datum/manufacturing_plans/{plan_id}.json")),
        serde_json::json!({
            "id": plan_id,
            "name": "Legacy plan",
            "board_or_panel": board_id,
            "variant": null,
            "prefix": "legacy-plan",
            "object_revision": 0
        }),
    );
    write_json(
        &root.join(format!(".datum/panel_projections/{panel_id}.json")),
        serde_json::json!({
            "id": panel_id,
            "name": "Legacy panel",
            "board_instances": [{
                "board": board_id,
                "x_nm": 0,
                "y_nm": 0,
                "rotation_deg": 0
            }],
            "object_revision": 0
        }),
    );
    write_json(
        &root.join(format!(".datum/output_jobs/{job_id}.json")),
        serde_json::json!({
            "id": job_id,
            "name": "Legacy output job",
            "include": ["gerber_set"],
            "prefix": "legacy-job",
            "output_dir": null,
            "board_or_panel": board_id,
            "variant": null,
            "manufacturing_plan": null,
            "object_revision": 0
        }),
    );

    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve legacy production records");

    assert_eq!(
        resolved.manufacturing_plans[&plan_id].schema_version,
        PRODUCTION_RECORD_SCHEMA_VERSION
    );
    assert_eq!(
        resolved.panel_projections[&panel_id].schema_version,
        PRODUCTION_RECORD_SCHEMA_VERSION
    );
    assert_eq!(
        resolved.output_jobs[&job_id].schema_version,
        PRODUCTION_RECORD_SCHEMA_VERSION
    );
}
