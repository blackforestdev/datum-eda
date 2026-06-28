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
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: plan_id,
        name: "Assembly A".to_string(),
        board_or_panel: panel_id,
        variant: None,
        prefix: "assy-a".to_string(),
        object_revision: ObjectRevision(0),
    };
    let panel = PanelProjection {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
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
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
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
fn production_output_job_rejects_missing_manufacturing_plan() {
    let root = temp_project_root("production_job_missing_plan");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let job_id = Uuid::new_v4();
    let missing_plan_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: job_id,
        name: "Dangling job".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "dangling".to_string(),
        output_dir: None,
        board_or_panel: board_id,
        variant: None,
        manufacturing_plan: Some(missing_plan_id),
        object_revision: ObjectRevision(0),
    };
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"missing-plan-output-job"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "reject output job missing manufacturing plan".to_string(),
                },
                operations: vec![Operation::CreateOutputJob {
                    output_job_id: job_id,
                    output_job: serde_json::to_value(&job).expect("job should serialize"),
                }],
            },
        )
        .expect_err("missing manufacturing plan should be rejected");
    assert!(
        format!("{error:#}").contains("references missing manufacturing plan"),
        "unexpected error: {error:#}"
    );
    assert!(
        !root
            .join(format!(".datum/output_jobs/{job_id}.json"))
            .exists()
    );
}

#[test]
fn production_records_reject_missing_variant() {
    let root = temp_project_root("production_records_missing_variant");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let plan_id = Uuid::new_v4();
    let missing_variant_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let plan = ManufacturingPlan {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: plan_id,
        name: "Variant plan".to_string(),
        board_or_panel: board_id,
        variant: Some(missing_variant_id),
        prefix: "variant".to_string(),
        object_revision: ObjectRevision(0),
    };
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"missing-variant-plan"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "reject production missing variant".to_string(),
                },
                operations: vec![Operation::CreateManufacturingPlan {
                    manufacturing_plan_id: plan_id,
                    manufacturing_plan: serde_json::to_value(&plan).expect("plan should serialize"),
                }],
            },
        )
        .expect_err("missing variant should be rejected");
    assert!(
        format!("{error:#}").contains("references missing variant"),
        "unexpected error: {error:#}"
    );
    assert!(
        !root
            .join(format!(".datum/manufacturing_plans/{plan_id}.json"))
            .exists()
    );
}

#[test]
fn panel_projection_rejects_unknown_board_instance() {
    let root = temp_project_root("panel_projection_unknown_board");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let panel_id = Uuid::new_v4();
    let unknown_board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let panel = PanelProjection {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: panel_id,
        name: "Invalid panel".to_string(),
        board_instances: vec![PanelBoardInstance {
            board: unknown_board_id,
            x_nm: 0,
            y_nm: 0,
            rotation_deg: 0,
        }],
        object_revision: ObjectRevision(0),
    };
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"unknown-board-panel"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "unit-test".to_string(),
                    source: CommitSource::Test,
                    reason: "reject panel unknown board".to_string(),
                },
                operations: vec![Operation::CreatePanelProjection {
                    panel_projection_id: panel_id,
                    panel_projection: serde_json::to_value(&panel).expect("panel should serialize"),
                }],
            },
        )
        .expect_err("unknown board should be rejected");
    assert!(
        format!("{error:#}").contains("references missing board"),
        "unexpected error: {error:#}"
    );
    assert!(
        !root
            .join(format!(".datum/panel_projections/{panel_id}.json"))
            .exists()
    );
}

#[test]
fn resolver_rejects_production_shard_filename_payload_mismatch() {
    let root = temp_project_root("production_filename_payload_mismatch");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let filename_id = Uuid::new_v4();
    let payload_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    write_json(
        &root.join(format!(".datum/output_jobs/{filename_id}.json")),
        serde_json::json!({
            "id": payload_id,
            "name": "Mismatched job",
            "include": ["gerber_set"],
            "prefix": "mismatch",
            "output_dir": null,
            "board_or_panel": board_id,
            "variant": null,
            "manufacturing_plan": null,
            "object_revision": 0
        }),
    );
    let resolved = ProjectResolver::new(&root)
        .resolve()
        .expect("resolver should continue after bad production shard");
    assert!(resolved.output_jobs.is_empty());
    assert!(resolved.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "invalid_output_job"
            && diagnostic
                .message
                .contains("manifest filename does not match embedded id")
    }));
}

#[test]
fn assistant_production_direct_commit_requires_proposal() {
    let root = temp_project_root("assistant_production_direct_commit_requires_proposal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let job_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: job_id,
        name: "Assistant direct job".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "assistant-direct".to_string(),
        output_dir: None,
        board_or_panel: board_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: ObjectRevision(0),
    };
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let error = model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(&project_id, b"assistant-direct-output-job"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "assistant".to_string(),
                    source: CommitSource::Assistant,
                    reason: "attempt direct production write".to_string(),
                },
                operations: vec![Operation::CreateOutputJob {
                    output_job_id: job_id,
                    output_job: serde_json::to_value(&job).expect("job should serialize"),
                }],
            },
        )
        .expect_err("assistant-authored production writes must be proposals");
    assert!(
        format!("{error:#}").contains("proposal_required_for_automated_production_operation"),
        "unexpected error: {error:#}"
    );
    assert!(
        !root
            .join(format!(".datum/output_jobs/{job_id}.json"))
            .exists()
    );
}

#[test]
fn assistant_generated_evidence_direct_commit_requires_proposal() {
    let root = temp_project_root("assistant_generated_evidence_direct_commit_requires_proposal");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let generated_id = Uuid::new_v4();
    let generated_payload = serde_json::Value::Null;
    let cases = [
        Operation::SetOutputJobRun {
            run_id: generated_id,
            previous_output_job_run: None,
            output_job_run: generated_payload.clone(),
        },
        Operation::DeleteOutputJobRun {
            run_id: generated_id,
            output_job_run: generated_payload.clone(),
        },
        Operation::SetArtifactRun {
            run_id: generated_id,
            previous_artifact_run: None,
            artifact_run: generated_payload.clone(),
        },
        Operation::DeleteArtifactRun {
            run_id: generated_id,
            artifact_run: generated_payload.clone(),
        },
        Operation::SetCheckRun {
            check_run_id: generated_id,
            previous_check_run: None,
            check_run: generated_payload.clone(),
        },
        Operation::DeleteCheckRun {
            check_run_id: generated_id,
            check_run: generated_payload.clone(),
        },
        Operation::SetArtifactMetadata {
            artifact_id: generated_id,
            previous_artifact_metadata: None,
            artifact_metadata: generated_payload.clone(),
        },
        Operation::DeleteArtifactMetadata {
            artifact_id: generated_id,
            artifact_metadata: generated_payload.clone(),
        },
        Operation::SetZoneFill {
            zone_id: generated_id,
            previous_zone_fill: None,
            zone_fill: generated_payload.clone(),
        },
        Operation::DeleteZoneFill {
            zone_id: generated_id,
            zone_fill: generated_payload,
        },
    ];

    for operation in cases {
        let mut model = ProjectResolver::new(&root)
            .resolve()
            .expect("resolve should succeed");
        let error = model
            .commit_journaled(
                &root,
                OperationBatch {
                    batch_id: Uuid::new_v5(
                        &project_id,
                        format!("assistant-generated-evidence-{operation:?}").as_bytes(),
                    ),
                    expected_model_revision: Some(model.model_revision.clone()),
                    provenance: CommitProvenance {
                        actor: "assistant".to_string(),
                        source: CommitSource::Assistant,
                        reason: "attempt direct generated evidence write".to_string(),
                    },
                    operations: vec![operation],
                },
            )
            .expect_err("assistant-authored generated evidence writes must be proposals");
        assert!(
            format!("{error:#}")
                .contains("proposal_required_for_automated_generated_evidence_operation"),
            "unexpected error: {error:#}"
        );
    }

    assert!(
        !root
            .join(format!(".datum/check_runs/{generated_id}.json"))
            .exists()
    );
    assert!(
        !root
            .join(format!(".datum/zone_fills/{generated_id}.json"))
            .exists()
    );
}

#[test]
fn accepted_assistant_production_proposal_applies_through_commit_gateway() {
    let root = temp_project_root("accepted_assistant_production_proposal_applies");
    let project_id = Uuid::new_v4();
    let board_id = Uuid::new_v4();
    let job_id = Uuid::new_v4();
    write_minimal_project(&root, project_id, board_id);

    let before = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed");
    let job = OutputJob {
        schema_version: PRODUCTION_RECORD_SCHEMA_VERSION,
        id: job_id,
        name: "Assistant proposed job".to_string(),
        include: vec![ArtifactKind::GerberSet],
        prefix: "assistant-proposed".to_string(),
        output_dir: None,
        board_or_panel: board_id,
        variant: None,
        manufacturing_plan: None,
        object_revision: ObjectRevision(0),
    };
    let proposal_id = Uuid::new_v5(&project_id, b"assistant-output-job-proposal");
    let proposal = Proposal {
        schema_version: 1,
        proposal_id,
        project_id,
        prepared_against: before.model_revision.clone(),
        batch: OperationBatch {
            batch_id: Uuid::new_v5(&project_id, b"assistant-output-job-proposal-batch"),
            expected_model_revision: Some(before.model_revision.clone()),
            provenance: CommitProvenance {
                actor: "assistant".to_string(),
                source: CommitSource::Assistant,
                reason: "propose production write".to_string(),
            },
            operations: vec![Operation::CreateOutputJob {
                output_job_id: job_id,
                output_job: serde_json::to_value(&job).expect("job should serialize"),
            }],
        },
        rationale: "create manufacturing output job for review".to_string(),
        affected_objects: vec![job_id],
        checks_run: Vec::new(),
        finding_fingerprints: Vec::new(),
        source: ProposalSource::Assistant,
        status: ProposalStatus::Accepted,
        applied_transaction_id: None,
    };
    let proposal_dir = root.join(".datum/proposals");
    std::fs::create_dir_all(&proposal_dir).expect("proposal dir should create");
    std::fs::write(
        proposal_dir.join(format!("{proposal_id}.json")),
        format!(
            "{}\n",
            to_json_deterministic(&proposal).expect("proposal should serialize")
        ),
    )
    .expect("proposal should write");

    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed with proposal");
    apply_accepted_proposal(&mut model, &root, proposal_id)
        .expect("accepted assistant proposal should apply");
    assert!(
        root.join(format!(".datum/output_jobs/{job_id}.json"))
            .exists()
    );
    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("resolve should succeed after proposal apply");
    assert_eq!(reopened.output_jobs.get(&job_id), Some(&job));
    assert_eq!(
        model
            .proposals
            .get(&proposal_id)
            .map(|proposal| proposal.status),
        Some(ProposalStatus::Applied)
    );
}
