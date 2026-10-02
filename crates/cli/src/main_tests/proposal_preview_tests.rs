use super::*;

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
    let original_track = Track::straight(
        track_id,
        net_id,
        Point { x: 1000, y: 2000 },
        Point { x: 3000, y: 4000 },
        250_000,
        1,
    );
    let track = Track::straight(
        track_id,
        net_id,
        Point { x: 1100, y: 2100 },
        Point { x: 3100, y: 4100 },
        275_000,
        2,
    );
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
