use super::main_tests_project_check::seed_board_process_aperture_fixture;
use super::*;
use eda_engine::board::Track;
use eda_engine::ir::geometry::Point;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, Operation, OperationBatch, ProjectResolver, Proposal,
    ProposalSource, ProposalStatus,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn execute(cli: Cli) -> Result<String> {
    let (output, _) = execute_with_exit_code(cli)?;
    Ok(output)
}

pub(super) fn write_legacy_proposal_sidecar(root: &Path, proposal: &Proposal) -> Result<()> {
    let proposal_dir = root.join(".datum/proposals");
    std::fs::create_dir_all(&proposal_dir)?;
    std::fs::write(
        proposal_dir.join(format!("{}.json", proposal.proposal_id)),
        format!("{}\n", serde_json::to_string(proposal)?),
    )?;
    Ok(())
}

#[test]
fn proposal_create_command_writes_draft_without_mutating_source() {
    let root = unique_project_root("datum-eda-cli-proposal-create");
    create_native_project(&root, Some("Proposal Create Demo".to_string()))
        .expect("initial scaffold should succeed");
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let manifest_before = std::fs::read(root.join("project.json")).expect("manifest should read");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"proposal-create");
    let batch = OperationBatch {
        batch_id: Uuid::new_v5(&model.project.project_id, b"proposal-create-batch"),
        expected_model_revision: None,
        provenance: CommitProvenance {
            actor: "test".to_string(),
            source: CommitSource::Cli,
            reason: "create proposal from operation batch".to_string(),
        },
        operations: vec![Operation::SetProjectName {
            project_id: model.project.project_id,
            name: "Applied Through Proposal".to_string(),
        }],
    };
    let batch_path = root.join("proposal-batch.json");
    std::fs::write(
        &batch_path,
        serde_json::to_string_pretty(&batch).expect("batch should serialize"),
    )
    .expect("batch should write");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create",
            root.to_str().unwrap(),
            "--batch",
            batch_path.to_str().unwrap(),
            "--rationale",
            "exercise generic proposal creation",
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal create should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "create_proposal");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    assert_eq!(report["proposal"]["status"], "draft");
    assert_eq!(report["proposal"]["source"], "cli");
    assert!(report["proposal"]["batch"]["expected_model_revision"].is_string());
    let proposal_operations = report["proposal"]["batch"]["operations"]
        .as_array()
        .expect("proposal operations should be an array");
    assert_eq!(proposal_operations.len(), 2);
    assert_eq!(proposal_operations[0]["kind"], "guard_object_revision");
    assert_eq!(proposal_operations[1]["kind"], "set_project_name");
    assert_eq!(report["validation"]["can_apply"], false);
    assert_eq!(
        std::fs::read(root.join("project.json")).unwrap(),
        manifest_before
    );

    let preview_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "preview",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal preview should succeed");
    let preview: serde_json::Value = serde_json::from_str(&preview_output).unwrap();
    assert_eq!(preview["contract"], "proposal_preview_v1");
    assert_eq!(preview["proposal_id"], proposal_id.to_string());
    assert_eq!(preview["validation"]["can_apply"], false);
    assert_eq!(
        preview["diff"]["modified"],
        serde_json::json!([model.project.project_id.to_string()])
    );
    assert_eq!(
        std::fs::read(root.join("project.json")).unwrap(),
        manifest_before,
        "preview must not write project shards"
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal accept-apply should succeed");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(reopened.project.name, "Applied Through Proposal");
    assert_eq!(
        reopened.proposals.get(&proposal_id).unwrap().status,
        ProposalStatus::Applied
    );
}

#[test]
fn proposal_preview_exposes_render_delta_for_set_board_track() {
    let root = unique_project_root("datum-eda-cli-proposal-preview-track");
    create_native_project(&root, Some("Proposal Preview Track Demo".to_string()))
        .expect("initial scaffold should succeed");
    let mut model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"proposal-preview-track");
    let track_id = Uuid::new_v5(&model.project.project_id, b"proposal-preview-track-object");
    let net_id = Uuid::new_v5(&model.project.project_id, b"proposal-preview-track-net");
    let original_track = Track {
        uuid: track_id,
        net: net_id,
        from: Point { x: 1000, y: 2000 },
        to: Point { x: 3000, y: 4000 },
        width: 250_000,
        layer: 1,
    };
    let track = Track {
        uuid: track_id,
        net: net_id,
        from: Point { x: 1100, y: 2100 },
        to: Point { x: 3100, y: 4100 },
        width: 275_000,
        layer: 2,
    };
    model
        .commit_journaled(
            &root,
            OperationBatch {
                batch_id: Uuid::new_v5(
                    &model.project.project_id,
                    b"proposal-preview-track-seed-batch",
                ),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Test,
                    reason: "seed existing board track".to_string(),
                },
                operations: vec![Operation::CreateBoardTrack {
                    track_id,
                    track: serde_json::to_value(original_track).expect("track should serialize"),
                }],
            },
        )
        .expect("track seed commit should succeed");
    write_legacy_proposal_sidecar(
        &root,
        &Proposal {
            schema_version: 1,
            proposal_id,
            project_id: model.project.project_id,
            prepared_against: model.model_revision.clone(),
            batch: OperationBatch {
                batch_id: Uuid::new_v5(&model.project.project_id, b"proposal-preview-track-batch"),
                expected_model_revision: Some(model.model_revision),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Cli,
                    reason: "preview board track render delta".to_string(),
                },
                operations: vec![Operation::SetBoardTrack {
                    track_id,
                    track: serde_json::to_value(track).expect("track should serialize"),
                }],
            },
            rationale: "preview board track render delta".to_string(),
            affected_objects: vec![track_id],
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
            source: ProposalSource::Cli,
            status: ProposalStatus::Draft,
            applied_transaction_id: None,
        },
    )
    .expect("proposal should write");

    let preview_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "preview",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal preview should succeed");
    let preview: serde_json::Value = serde_json::from_str(&preview_output).unwrap();
    assert_eq!(preview["contract"], "proposal_preview_v1");
    let render_deltas = preview["render_deltas"].as_array().unwrap();
    assert_eq!(render_deltas.len(), 1);
    let delta = &render_deltas[0];
    assert_eq!(delta["delta_kind"], "set");
    assert_eq!(delta["object_id"], track_id.to_string());
    assert_eq!(delta["primitive_kind"], "track_path");
    assert_eq!(delta["layer_id"], "L2");
    assert_eq!(delta["width_nm"], 275_000);
    assert_eq!(
        delta["path"],
        serde_json::json!([
            { "x": 1100, "y": 2100 },
            { "x": 3100, "y": 4100 }
        ])
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_list_command_is_resolver_backed() {
    let root = unique_project_root("datum-eda-cli-proposal-list");
    create_native_project(&root, Some("Proposal List Demo".to_string()))
        .expect("initial scaffold should succeed");
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"proposal-list");
    write_legacy_proposal_sidecar(
        &root,
        &Proposal {
            schema_version: 1,
            proposal_id,
            project_id: model.project.project_id,
            prepared_against: model.model_revision.clone(),
            batch: OperationBatch {
                batch_id: Uuid::new_v5(&model.project.project_id, b"proposal-list-batch"),
                expected_model_revision: Some(model.model_revision),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Cli,
                    reason: "canonical proposal list".to_string(),
                },
                operations: Vec::new(),
            },
            rationale: "exercise canonical proposal list".to_string(),
            affected_objects: Vec::new(),
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
            source: ProposalSource::Tool,
            status: ProposalStatus::Draft,
            applied_transaction_id: None,
        },
    )
    .expect("proposal should write");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "list",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal list should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(report["contract"], "proposals_query_v1");
    assert_eq!(report["proposal_count"], 1);
    assert_eq!(
        report["proposals"][proposal_id.to_string()]["status"],
        "draft"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_review_command_updates_status() {
    let root = unique_project_root("datum-eda-cli-proposal-review");
    create_native_project(&root, Some("Proposal Review Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);

    let generated_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "repair-standards",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("standards repair proposals should generate");
    let generated: serde_json::Value = serde_json::from_str(&generated_output).unwrap();
    let proposal_id =
        Uuid::parse_str(generated["proposals"][0]["proposal_id"].as_str().unwrap()).unwrap();

    let review_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "review",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
            "--status",
            "accepted",
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal review should succeed");
    let review: serde_json::Value = serde_json::from_str(&review_output).unwrap();

    assert_eq!(review["action"], "review_proposal");
    assert_eq!(review["status"], "accepted");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_proposals_is_resolver_backed_and_non_mutating() {
    let root = unique_project_root("datum-eda-cli-project-query-proposals");
    create_native_project(&root, Some("Proposal Query Demo".to_string()))
        .expect("initial scaffold should succeed");
    let board_before = std::fs::read(root.join("board/board.json")).expect("board should read");
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"query-proposals");
    write_legacy_proposal_sidecar(
        &root,
        &Proposal {
            schema_version: 1,
            proposal_id,
            project_id: model.project.project_id,
            prepared_against: model.model_revision.clone(),
            batch: OperationBatch {
                batch_id: Uuid::new_v5(&model.project.project_id, b"query-proposals-batch"),
                expected_model_revision: Some(model.model_revision),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Cli,
                    reason: "query proposals".to_string(),
                },
                operations: Vec::new(),
            },
            rationale: "exercise generic proposal query".to_string(),
            affected_objects: Vec::new(),
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
            source: ProposalSource::Tool,
            status: ProposalStatus::Draft,
            applied_transaction_id: None,
        },
    )
    .expect("proposal should write");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "proposals",
        ])
        .expect("CLI should parse"),
    )
    .expect("proposals query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["contract"], "proposals_query_v1");
    assert_eq!(report["proposal_count"], 1);
    assert_eq!(
        report["proposals"][proposal_id.to_string()]["status"],
        "draft"
    );
    assert_eq!(
        std::fs::read(root.join("board/board.json")).unwrap(),
        board_before
    );
}

#[test]
fn project_show_validate_and_defer_proposal_lifecycle() {
    let root = unique_project_root("datum-eda-cli-project-proposal-lifecycle");
    create_native_project(&root, Some("Proposal Lifecycle Demo".to_string()))
        .expect("initial scaffold should succeed");
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"proposal-lifecycle");
    write_legacy_proposal_sidecar(
        &root,
        &Proposal {
            schema_version: 1,
            proposal_id,
            project_id: model.project.project_id,
            prepared_against: model.model_revision.clone(),
            batch: OperationBatch {
                batch_id: Uuid::new_v5(&model.project.project_id, b"proposal-lifecycle-batch"),
                expected_model_revision: Some(model.model_revision),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Cli,
                    reason: "validate proposal".to_string(),
                },
                operations: Vec::new(),
            },
            rationale: "exercise proposal lifecycle".to_string(),
            affected_objects: Vec::new(),
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
            source: ProposalSource::Tool,
            status: ProposalStatus::Draft,
            applied_transaction_id: None,
        },
    )
    .expect("proposal should write");

    let show_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "show-proposal",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal show should succeed");
    let show: serde_json::Value = serde_json::from_str(&show_output).unwrap();
    assert_eq!(show["contract"], "proposal_show_v1");
    assert_eq!(show["proposal"]["status"], "draft");
    assert_eq!(show["validation"]["can_apply"], false);

    let validate_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-proposal",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal validate should succeed");
    let validate: serde_json::Value = serde_json::from_str(&validate_output).unwrap();
    assert_eq!(validate["contract"], "proposal_validation_v1");
    assert_eq!(validate["prepared_against_current_model"], true);
    assert_eq!(validate["batch_revision_guard_matches"], true);
    assert_eq!(validate["can_apply"], false);

    let defer_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "defer-proposal",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal defer should succeed");
    let defer: serde_json::Value = serde_json::from_str(&defer_output).unwrap();
    assert_eq!(defer["action"], "review_proposal");
    assert_eq!(defer["status"], "deferred");

    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(
        reopened.proposals.get(&proposal_id).unwrap().status,
        ProposalStatus::Deferred
    );
}

#[test]
fn project_validate_proposal_reports_stale_revision() {
    let root = unique_project_root("datum-eda-cli-project-proposal-stale");
    create_native_project(&root, Some("Proposal Stale Demo".to_string()))
        .expect("initial scaffold should succeed");
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"proposal-stale");
    write_legacy_proposal_sidecar(
        &root,
        &Proposal {
            schema_version: 1,
            proposal_id,
            project_id: model.project.project_id,
            prepared_against: model.model_revision.clone(),
            batch: OperationBatch {
                batch_id: Uuid::new_v5(&model.project.project_id, b"proposal-stale-batch"),
                expected_model_revision: Some(model.model_revision.clone()),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Cli,
                    reason: "stale proposal".to_string(),
                },
                operations: Vec::new(),
            },
            rationale: "exercise stale validation".to_string(),
            affected_objects: Vec::new(),
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
            source: ProposalSource::Tool,
            status: ProposalStatus::Accepted,
            applied_transaction_id: None,
        },
    )
    .expect("proposal should write");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-project-name",
            root.to_str().unwrap(),
            "--name",
            "Changed",
        ])
        .expect("CLI should parse"),
    )
    .expect("project name should change");

    let validate_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate-proposal",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal validate should succeed");
    let validate: serde_json::Value = serde_json::from_str(&validate_output).unwrap();
    assert_eq!(validate["prepared_against_current_model"], false);
    assert_eq!(validate["batch_revision_guard_matches"], true);
    assert_eq!(validate["can_apply"], false);
    assert_eq!(validate["blocker_codes"][0], "stale_model_revision");
    assert!(
        validate["blockers"][0]["message"]
            .as_str()
            .unwrap()
            .contains("prepared against")
    );
}

#[test]
fn project_review_proposal_rejects_stale_acceptance() {
    let root = unique_project_root("datum-eda-cli-project-proposal-stale-review");
    create_native_project(&root, Some("Proposal Stale Review Demo".to_string()))
        .expect("initial scaffold should succeed");
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"proposal-stale-review");
    write_legacy_proposal_sidecar(
        &root,
        &Proposal {
            schema_version: 1,
            proposal_id,
            project_id: model.project.project_id,
            prepared_against: model.model_revision.clone(),
            batch: OperationBatch {
                batch_id: Uuid::new_v5(&model.project.project_id, b"proposal-stale-review-batch"),
                expected_model_revision: Some(model.model_revision),
                provenance: CommitProvenance {
                    actor: "test".to_string(),
                    source: CommitSource::Cli,
                    reason: "stale proposal review".to_string(),
                },
                operations: Vec::new(),
            },
            rationale: "exercise stale review rejection".to_string(),
            affected_objects: Vec::new(),
            checks_run: Vec::new(),
            finding_fingerprints: Vec::new(),
            source: ProposalSource::Tool,
            status: ProposalStatus::Draft,
            applied_transaction_id: None,
        },
    )
    .expect("proposal should write");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-project-name",
            root.to_str().unwrap(),
            "--name",
            "Changed Before Review",
        ])
        .expect("CLI should parse"),
    )
    .expect("project name should change");

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "review-proposal",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
            "--status",
            "accepted",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("stale proposal review should fail");
    assert!(err.to_string().contains("cannot be accepted"));
}

#[test]
fn project_review_and_apply_proposal_uses_generic_gateway() {
    let root = unique_project_root("datum-eda-cli-project-review-apply-proposal");
    create_native_project(&root, Some("Generic Proposal Apply Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);

    let generated_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "generate-standards-repair-proposals",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("standards repair proposals should generate");
    let generated: serde_json::Value = serde_json::from_str(&generated_output).unwrap();
    let proposal_id =
        Uuid::parse_str(generated["proposals"][0]["proposal_id"].as_str().unwrap()).unwrap();

    let review_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "review-proposal",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
            "--status",
            "accepted",
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal review should succeed");
    let review: serde_json::Value = serde_json::from_str(&review_output).unwrap();
    assert_eq!(review["action"], "review_proposal");
    assert_eq!(review["status"], "accepted");

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-proposal",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal apply should succeed");
    let apply: serde_json::Value = serde_json::from_str(&apply_output).unwrap();
    assert_eq!(apply["action"], "apply_proposal");
    assert_eq!(apply["status"], "applied");
    assert!(apply["transaction_id"].as_str().is_some());

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    let proposal = model.proposals.get(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Applied);
    assert!(proposal.applied_transaction_id.is_some());
}

#[test]
fn proposal_accept_apply_command_uses_generic_gateway() {
    let root = unique_project_root("datum-eda-cli-proposal-accept-apply");
    create_native_project(&root, Some("Proposal Accept Apply Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);

    let generated_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "check",
            "repair-standards",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("standards repair proposals should generate");
    let generated: serde_json::Value = serde_json::from_str(&generated_output).unwrap();
    let proposal_id =
        Uuid::parse_str(generated["proposals"][0]["proposal_id"].as_str().unwrap()).unwrap();

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal accept-apply should succeed");
    let apply: serde_json::Value = serde_json::from_str(&apply_output).unwrap();
    assert_eq!(apply["action"], "apply_proposal");
    assert_eq!(apply["status"], "applied");
    assert!(apply["transaction_id"].as_str().is_some());

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    let proposal = model.proposals.get(&proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Applied);
    assert!(proposal.applied_transaction_id.is_some());
}
