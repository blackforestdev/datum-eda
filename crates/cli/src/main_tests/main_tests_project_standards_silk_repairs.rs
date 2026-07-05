use std::path::Path;

use super::main_tests_project_check::unique_project_root;
use super::*;
use eda_engine::substrate::{ProjectResolver, ProposalStatus, SourceShardKind};

#[test]
fn project_generate_standards_repair_proposals_links_silk_clearance_findings() {
    let root = unique_project_root("datum-eda-cli-project-check-run-silk-repair");
    create_native_project(&root, Some("Silk Repair Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_silk_clearance_fixture(&root);

    let output = execute(
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
    .expect("standards silk repair proposals should generate");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("repair report JSON should parse");
    let proposal = report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "silk_clearance")
        .expect("silkscreen clearance repair proposal should be reported");
    assert_eq!(proposal["operations"], 1);
    assert_eq!(proposal["affected_text"], silk_text_uuid().to_string());
    assert_eq!(proposal["prepared_against"], report["model_revision"]);
    assert_eq!(proposal["prepared_against_current_model"], true);
    assert_eq!(proposal["can_apply"], false);
    assert_eq!(
        proposal["blocker_codes"],
        serde_json::json!(["missing_acceptance"])
    );
    assert!(
        proposal["codes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|code| code == "silk_clearance_copper")
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with silk repair proposal");
    let proposal_id = Uuid::parse_str(proposal["proposal_id"].as_str().unwrap()).unwrap();
    let proposal = model.proposals.get(&proposal_id).unwrap();
    assert_eq!(
        proposal.source,
        eda_engine::substrate::ProposalSource::Check
    );
    assert_eq!(proposal.finding_fingerprints.len(), 1);
    let eda_engine::substrate::Operation::SetBoardText { text, .. } = &proposal.batch.operations[0]
    else {
        panic!("silk standards repair should use SetBoardText");
    };
    assert_eq!(text["uuid"], silk_text_uuid().to_string());
    assert!(
        text["position"]["y"].as_i64().unwrap() > 10_000_000,
        "silk repair should move board text away from copper"
    );

    let linked_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "check-run",
        ])
        .expect("CLI should parse"),
    )
    .expect("linked check-run should succeed");
    let linked: serde_json::Value =
        serde_json::from_str(&linked_output).expect("linked check-run JSON should parse");
    let finding = linked["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            entry["code"] == "silk_clearance_copper"
                && entry["proposal_refs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|value| value == &proposal_id.to_string())
        })
        .expect("silk finding should link to repair proposal");
    assert_eq!(finding["domain"], "drc");
    assert_eq!(finding["primary_target"]["kind"], "object_uuid");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_accept_apply_silk_clearance_repair_updates_board_and_findings() {
    let root = unique_project_root("datum-eda-cli-project-silk-standards-repair-apply");
    create_native_project(&root, Some("Silk Standards Repair Apply Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_silk_clearance_fixture(&root);

    let generated = generate_standards_repair_report(&root);
    let proposal = generated["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "silk_clearance")
        .expect("silkscreen clearance repair proposal should exist");
    let proposal_id = proposal["proposal_id"].as_str().unwrap().to_string();
    assert_eq!(proposal["affected_text"], silk_text_uuid().to_string());

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id,
        ])
        .expect("CLI should parse"),
    )
    .expect("silkscreen standards repair proposal should accept/apply");
    let apply: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply JSON should parse");
    assert_eq!(apply["action"], "apply_proposal");
    assert_eq!(apply["status"], "applied");
    assert_eq!(apply["validation"]["can_apply"], true);

    let proposal_uuid = Uuid::parse_str(&proposal_id).unwrap();
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve after silk repair apply");
    assert_eq!(
        model.proposals.get(&proposal_uuid).unwrap().status,
        ProposalStatus::Applied
    );
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .expect("materialized board should exist");
    let repaired_text = board["texts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["uuid"] == silk_text_uuid().to_string())
        .expect("repaired text should remain on board");
    assert!(
        repaired_text["position"]["y"].as_i64().unwrap() > 10_000_000,
        "applied repair should move the board text away from copper"
    );

    let linked = query_check_run(&root);
    assert!(
        linked["findings"].as_array().unwrap().iter().all(|entry| {
            entry["code"] != "silk_clearance_copper"
                || !entry["payload"]["objects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|value| value == &silk_text_uuid().to_string())
        }),
        "applied repair should clear the silkscreen-clearance finding for the repaired text"
    );

    let _ = std::fs::remove_dir_all(&root);
}

fn silk_text_uuid() -> Uuid {
    Uuid::parse_str("bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb").unwrap()
}

fn generate_standards_repair_report(root: &Path) -> serde_json::Value {
    let output = execute(
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
    serde_json::from_str(&output).expect("repair report JSON should parse")
}

fn query_check_run(root: &Path) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "check-run",
        ])
        .expect("CLI should parse"),
    )
    .expect("check-run after repair should succeed");
    serde_json::from_str(&output).expect("check-run JSON should parse")
}

fn seed_board_silk_clearance_fixture(root: &Path) {
    let net_class_uuid = Uuid::parse_str("cccccccc-cccc-4ccc-8ccc-cccccccccccc").unwrap();
    let net_uuid = Uuid::parse_str("dddddddd-dddd-4ddd-8ddd-dddddddddddd").unwrap();
    let track_uuid = Uuid::parse_str("eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee").unwrap();
    let text_uuid = silk_text_uuid();
    let board_json = root.join("board/board.json");
    let mut board: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&board_json).expect("board file should read"),
    )
    .expect("board JSON should parse");
    board["nets"] = serde_json::json!({
        net_uuid.to_string(): {
            "uuid": net_uuid,
            "name": "SIG",
            "class": net_class_uuid
        }
    });
    board["net_classes"] = serde_json::json!({
        net_class_uuid.to_string(): {
            "uuid": net_class_uuid,
            "name": "Default",
            "clearance": 150000,
            "track_width": 200000,
            "via_drill": 300000,
            "via_diameter": 600000,
            "diffpair_width": 0,
            "diffpair_gap": 0
        }
    });
    board["tracks"] = serde_json::json!({
        track_uuid.to_string(): {
            "uuid": track_uuid,
            "net": net_uuid,
            "from": { "x": 9800000, "y": 10000000 },
            "to": { "x": 10200000, "y": 10000000 },
            "width": 100000,
            "layer": 0
        }
    });
    board["texts"] = serde_json::json!([
        {
            "uuid": text_uuid,
            "text": "REF",
            "position": { "x": 10000000, "y": 10000000 },
            "rotation": 0,
            "layer": 37,
            "render_intent": "manufacturing",
            "height_nm": 1000000,
            "stroke_width_nm": 100000
        }
    ]);
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            eda_engine::ir::serialization::to_json_deterministic(&board)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");
}
