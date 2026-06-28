use super::*;

fn seed_production_records(
    root: &Path,
    project_id: Uuid,
    board_id: Uuid,
) -> (
    Uuid,
    Uuid,
    Uuid,
    ManufacturingPlan,
    PanelProjection,
    OutputJob,
) {
    let plan_id = Uuid::new_v4();
    let panel_id = Uuid::new_v4();
    let job_id = Uuid::new_v4();
    let panel = PanelProjection {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
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
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: plan_id,
        name: "Recovered manufacturing plan".to_string(),
        board_or_panel: panel_id,
        variant: None,
        prefix: "recovered".to_string(),
        object_revision: ObjectRevision(0),
    };
    let job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
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
    let mut model = ProjectResolver::new(root)
        .resolve()
        .expect("resolve should succeed");
    model
        .commit_journaled(
            root,
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
    (plan_id, panel_id, job_id, plan, panel, job)
}

fn assert_recovered_records(
    model: &DesignModel,
    plan_id: Uuid,
    panel_id: Uuid,
    job_id: Uuid,
    plan: &ManufacturingPlan,
    panel: &PanelProjection,
    job: &OutputJob,
    dirty_state: SourceShardDirtyState,
) {
    assert_eq!(model.manufacturing_plans[&plan_id], *plan);
    assert_eq!(model.panel_projections[&panel_id], *panel);
    assert_eq!(model.output_jobs[&job_id], *job);
    assert!(model.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::ManufacturingPlan
            && shard.taxon == Some(SourceShardTaxon::ManufacturingPlan)
            && shard.relative_path == format!(".datum/manufacturing_plans/{plan_id}.json")
            && shard.dirty_state == dirty_state
    }));
    assert!(model.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::PanelProjection
            && shard.taxon == Some(SourceShardTaxon::PanelProjection)
            && shard.relative_path == format!(".datum/panel_projections/{panel_id}.json")
            && shard.dirty_state == dirty_state
    }));
    assert!(model.source_shards.iter().any(|shard| {
        shard.kind == SourceShardKind::OutputJob
            && shard.taxon == Some(SourceShardTaxon::OutputJob)
            && shard.relative_path == format!(".datum/output_jobs/{job_id}.json")
            && shard.dirty_state == dirty_state
    }));
}

#[test]
fn journal_replay_recovers_missing_production_shards() {
    let root = temp_project_root("journal_replay_recovers_production");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let (plan_id, panel_id, job_id, plan, panel, job) =
        seed_production_records(&root, project_id, board_id);

    for relative_path in [
        format!(".datum/manufacturing_plans/{plan_id}.json"),
        format!(".datum/panel_projections/{panel_id}.json"),
        format!(".datum/output_jobs/{job_id}.json"),
    ] {
        std::fs::remove_file(root.join(relative_path)).expect("production shard should remove");
    }

    let recovered = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should recover production records from journal");
    assert_recovered_records(
        &recovered,
        plan_id,
        panel_id,
        job_id,
        &plan,
        &panel,
        &job,
        SourceShardDirtyState::Missing,
    );
    assert!(recovered.diagnostics.is_empty());
}

#[test]
fn journal_replay_marks_stale_promoted_production_shards_dirty() {
    let root = temp_project_root("journal_replay_dirty_production");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let (plan_id, panel_id, job_id, plan, panel, job) =
        seed_production_records(&root, project_id, board_id);

    let mut stale_plan = plan.clone();
    stale_plan.name = "Stale manufacturing plan".to_string();
    let mut stale_panel = panel.clone();
    stale_panel.name = "Stale panel".to_string();
    let mut stale_job = job.clone();
    stale_job.name = "Stale Gerbers".to_string();
    write_json(
        &root.join(format!(".datum/manufacturing_plans/{plan_id}.json")),
        serde_json::to_value(stale_plan).expect("stale plan should serialize"),
    );
    write_json(
        &root.join(format!(".datum/panel_projections/{panel_id}.json")),
        serde_json::to_value(stale_panel).expect("stale panel should serialize"),
    );
    write_json(
        &root.join(format!(".datum/output_jobs/{job_id}.json")),
        serde_json::to_value(stale_job).expect("stale job should serialize"),
    );

    let recovered = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should recover stale production records from journal");
    assert_recovered_records(
        &recovered,
        plan_id,
        panel_id,
        job_id,
        &plan,
        &panel,
        &job,
        SourceShardDirtyState::Dirty,
    );
}

#[test]
fn journal_replay_recovers_unreadable_production_shards_as_unknown() {
    let root = temp_project_root("journal_replay_unknown_production");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);
    let (plan_id, panel_id, job_id, plan, panel, job) =
        seed_production_records(&root, project_id, board_id);

    for relative_path in [
        format!(".datum/manufacturing_plans/{plan_id}.json"),
        format!(".datum/panel_projections/{panel_id}.json"),
        format!(".datum/output_jobs/{job_id}.json"),
    ] {
        let path = root.join(relative_path);
        std::fs::remove_file(&path).expect("production shard should remove");
        std::fs::create_dir(&path).expect("unreadable production shard directory should create");
    }

    let recovered = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should recover unreadable production records from journal");
    assert_recovered_records(
        &recovered,
        plan_id,
        panel_id,
        job_id,
        &plan,
        &panel,
        &job,
        SourceShardDirtyState::Unknown,
    );
}
