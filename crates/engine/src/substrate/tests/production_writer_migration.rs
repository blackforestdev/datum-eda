use super::*;

#[test]
fn journaled_production_records_create_undo_and_redo() {
    let root = temp_project_root("journaled_production_records");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();
    let panel_id = Uuid::new_v4();
    let job_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let plan = ManufacturingPlan {
        id: plan_id,
        name: "Assembly A".to_string(),
        board_or_panel: panel_id,
        variant: None,
        prefix: "assy-a".to_string(),
        object_revision: ObjectRevision(0),
    };
    let panel = PanelProjection {
        id: panel_id,
        name: "Two-up".to_string(),
        board_instances: vec![PanelBoardInstance {
            board: board_id,
            x_nm: 0,
            y_nm: 0,
            rotation_deg: 0,
        }],
        object_revision: ObjectRevision(0),
    };
    let job = OutputJob {
        id: job_id,
        name: "Gerber set".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "assy-a".to_string(),
        output_dir: None,
        board_or_panel: panel_id,
        variant: None,
        manufacturing_plan: Some(plan_id),
        object_revision: ObjectRevision(0),
    };
    let plan_path = root.join(format!(".datum/manufacturing_plans/{plan_id}.json"));
    let panel_path = root.join(format!(".datum/panel_projections/{panel_id}.json"));
    let job_path = root.join(format!(".datum/output_jobs/{job_id}.json"));

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"create-production-records"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "create production records through substrate".to_string(),
                },
                operations: vec![
                    Operation::CreatePanelProjection {
                        panel_projection_id: panel_id,
                        panel_projection: serde_json::to_value(&panel)
                            .expect("panel should serialize"),
                    },
                    Operation::CreateManufacturingPlan {
                        manufacturing_plan_id: plan_id,
                        manufacturing_plan: serde_json::to_value(&plan)
                            .expect("plan should serialize"),
                    },
                    Operation::CreateOutputJob {
                        output_job_id: job_id,
                        output_job: serde_json::to_value(&job).expect("job should serialize"),
                    },
                ],
            },
        )
        .expect("journaled production create should succeed");

    assert_eq!(
        report.transaction.diff.created,
        vec![panel_id, plan_id, job_id]
    );
    assert!(plan_path.exists());
    assert!(panel_path.exists());
    assert!(job_path.exists());
    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed after create");
    assert_eq!(resolved.manufacturing_plans[&plan_id], plan);
    assert_eq!(resolved.panel_projections[&panel_id], panel);
    assert_eq!(resolved.output_jobs[&job_id], job);

    model
        .commit_journal_undo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "undo production records".to_string(),
            },
        )
        .expect("undo should remove production records");
    assert!(!plan_path.exists());
    assert!(!panel_path.exists());
    assert!(!job_path.exists());
    let undone = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed after undo");
    assert!(!undone.manufacturing_plans.contains_key(&plan_id));
    assert!(!undone.panel_projections.contains_key(&panel_id));
    assert!(!undone.output_jobs.contains_key(&job_id));

    model
        .commit_journal_redo(
            &root,
            CommitProvenance {
                actor: "unit-test".to_string(),
                source: CommitSource::Test,
                reason: "redo production records".to_string(),
            },
        )
        .expect("redo should restore production records");
    assert!(plan_path.exists());
    assert!(panel_path.exists());
    assert!(job_path.exists());
    let redone = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed after redo");
    assert_eq!(redone.manufacturing_plans[&plan_id], plan);
    assert_eq!(redone.panel_projections[&panel_id], panel);
    assert_eq!(redone.output_jobs[&job_id], job);
}

#[test]
fn journal_replay_recovers_missing_production_shards() {
    let root = temp_project_root("journal_replay_recovers_production");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();
    let panel_id = Uuid::new_v4();
    let job_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let panel = PanelProjection {
        id: panel_id,
        name: "Recovered panel".to_string(),
        board_instances: vec![PanelBoardInstance {
            board: board_id,
            x_nm: 0,
            y_nm: 0,
            rotation_deg: 0,
        }],
        object_revision: ObjectRevision(0),
    };
    let plan = ManufacturingPlan {
        id: plan_id,
        name: "Recovered manufacturing plan".to_string(),
        board_or_panel: panel_id,
        variant: None,
        prefix: "recovered".to_string(),
        object_revision: ObjectRevision(0),
    };
    let job = OutputJob {
        id: job_id,
        name: "Recovered Gerbers".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "recovered".to_string(),
        output_dir: None,
        board_or_panel: panel_id,
        variant: None,
        manufacturing_plan: Some(plan_id),
        object_revision: ObjectRevision(0),
    };

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let report = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"recover-production-records"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "recover production records from journal".to_string(),
                },
                operations: vec![
                    Operation::CreatePanelProjection {
                        panel_projection_id: panel_id,
                        panel_projection: serde_json::to_value(&panel)
                            .expect("panel should serialize"),
                    },
                    Operation::CreateManufacturingPlan {
                        manufacturing_plan_id: plan_id,
                        manufacturing_plan: serde_json::to_value(&plan)
                            .expect("plan should serialize"),
                    },
                    Operation::CreateOutputJob {
                        output_job_id: job_id,
                        output_job: serde_json::to_value(&job).expect("job should serialize"),
                    },
                ],
            },
        )
        .expect("journaled production create should succeed");

    std::fs::remove_file(root.join(format!(".datum/manufacturing_plans/{plan_id}.json")))
        .expect("plan file should remove");
    std::fs::remove_file(root.join(format!(".datum/panel_projections/{panel_id}.json")))
        .expect("panel file should remove");
    std::fs::remove_file(root.join(format!(".datum/output_jobs/{job_id}.json")))
        .expect("job file should remove");

    let recovered = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should recover production records from journal");
    assert_eq!(
        recovered.model_revision,
        report.transaction.after_model_revision
    );
    assert_eq!(recovered.manufacturing_plans[&plan_id], plan);
    assert_eq!(recovered.panel_projections[&panel_id], panel);
    assert_eq!(recovered.output_jobs[&job_id], job);
    assert!(recovered.diagnostics.is_empty());
}
