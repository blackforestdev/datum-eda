use std::path::Path;

use super::main_tests_project_check::unique_project_root;
use super::*;
use eda_engine::substrate::{ProjectResolver, ProposalStatus, SourceShardKind};

#[test]
fn project_generate_standards_repair_proposals_links_copper_clearance_findings() {
    let root = unique_project_root("datum-eda-cli-project-clearance-standards-repair");
    create_native_project(&root, Some("Clearance Standards Repair Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_clearance_repair_fixture(&root);

    let report = generate_standards_repair_report(&root);
    let proposal = report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "copper_clearance")
        .expect("copper clearance repair proposal should be reported");
    assert_eq!(proposal["operations"], 1);
    assert_eq!(
        proposal["affected_track"],
        moved_clearance_track_uuid().to_string()
    );
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
            .any(|code| code == "clearance_copper")
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with clearance repair proposal");
    let proposal_id = Uuid::parse_str(proposal["proposal_id"].as_str().unwrap()).unwrap();
    let proposal = model.proposals.get(&proposal_id).unwrap();
    assert_eq!(
        proposal.source,
        eda_engine::substrate::ProposalSource::Check
    );
    assert_eq!(proposal.finding_fingerprints.len(), 1);
    let eda_engine::substrate::Operation::SetBoardTrack { track, .. } =
        &proposal.batch.operations[0]
    else {
        panic!("clearance standards repair should use SetBoardTrack");
    };
    assert_eq!(track["uuid"], moved_clearance_track_uuid().to_string());
    assert_eq!(track["from"]["y"], 250000);
    assert_eq!(track["to"]["y"], 250000);

    let linked = query_check_run(&root);
    let finding = linked["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            entry["code"] == "clearance_copper"
                && entry["proposal_refs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|value| value == &proposal_id.to_string())
        })
        .expect("clearance finding should link to repair proposal");
    assert_eq!(finding["domain"], "drc");
    assert_eq!(finding["primary_target"]["kind"], "object_uuid");
    let finding_targets = std::iter::once(&finding["primary_target"])
        .chain(finding["related_targets"].as_array().unwrap().iter())
        .map(|target| target["id"].as_str().unwrap().to_string())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        finding_targets.contains(&fixed_clearance_track_uuid().to_string())
            && finding_targets.contains(&moved_clearance_track_uuid().to_string())
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_accept_apply_copper_clearance_repair_updates_board_and_findings() {
    let root = unique_project_root("datum-eda-cli-project-clearance-standards-repair-apply");
    create_native_project(
        &root,
        Some("Clearance Standards Repair Apply Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    seed_board_clearance_repair_fixture(&root);

    let report = generate_standards_repair_report(&root);
    let proposal = report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "copper_clearance")
        .expect("copper clearance repair proposal should exist");
    let proposal_id = proposal["proposal_id"].as_str().unwrap().to_string();

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
    .expect("copper clearance proposal should accept/apply");
    let apply: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply JSON should parse");
    assert_eq!(apply["action"], "apply_proposal");
    assert_eq!(apply["status"], "applied");
    assert_eq!(apply["validation"]["can_apply"], true);

    let proposal_uuid = Uuid::parse_str(&proposal_id).unwrap();
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve after clearance repair apply");
    assert_eq!(
        model.proposals.get(&proposal_uuid).unwrap().status,
        ProposalStatus::Applied
    );
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .expect("materialized board should exist");
    let track = &board["tracks"][moved_clearance_track_uuid().to_string()];
    assert_eq!(track["from"]["y"], 250000);
    assert_eq!(track["to"]["y"], 250000);

    let linked = query_check_run(&root);
    assert!(
        linked["findings"].as_array().unwrap().iter().all(|entry| {
            entry["code"] != "clearance_copper"
                || !entry["payload"]["objects"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|value| value == &moved_clearance_track_uuid().to_string())
        }),
        "applied repair should clear the copper-clearance finding for the moved track"
    );

    let _ = std::fs::remove_dir_all(&root);
}

fn fixed_clearance_track_uuid() -> Uuid {
    Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap()
}

fn moved_clearance_track_uuid() -> Uuid {
    Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap()
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
    .expect("check-run should succeed");
    serde_json::from_str(&output).expect("check-run JSON should parse")
}

fn seed_board_clearance_repair_fixture(root: &Path) {
    let net_class_uuid = Uuid::parse_str("33333333-3333-4333-8333-333333333333").unwrap();
    let net_a_uuid = Uuid::parse_str("44444444-4444-4444-8444-444444444444").unwrap();
    let net_b_uuid = Uuid::parse_str("55555555-5555-4555-8555-555555555555").unwrap();
    let board_json = root.join("board/board.json");
    let mut board: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&board_json).expect("board file should read"),
    )
    .expect("board JSON should parse");
    board["pads"] = serde_json::json!({});
    board["vias"] = serde_json::json!({});
    board["zones"] = serde_json::json!({});
    board["texts"] = serde_json::json!([]);
    board["nets"] = serde_json::json!({
        net_a_uuid.to_string(): {
            "uuid": net_a_uuid,
            "name": "A",
            "class": net_class_uuid
        },
        net_b_uuid.to_string(): {
            "uuid": net_b_uuid,
            "name": "B",
            "class": net_class_uuid
        }
    });
    board["net_classes"] = serde_json::json!({
        net_class_uuid.to_string(): {
            "uuid": net_class_uuid,
            "name": "Default",
            "clearance": 150000,
            "track_width": 100000,
            "via_drill": 300000,
            "via_diameter": 600000,
            "diffpair_width": 0,
            "diffpair_gap": 0
        }
    });
    board["tracks"] = serde_json::json!({
        fixed_clearance_track_uuid().to_string(): {
            "uuid": fixed_clearance_track_uuid(),
            "net": net_a_uuid,
            "from": { "x": 0, "y": 0 },
            "to": { "x": 1000000, "y": 0 },
            "width": 100000,
            "layer": 1
        },
        moved_clearance_track_uuid().to_string(): {
            "uuid": moved_clearance_track_uuid(),
            "net": net_b_uuid,
            "from": { "x": 0, "y": 100000 },
            "to": { "x": 1000000, "y": 100000 },
            "width": 100000,
            "layer": 1
        }
    });
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
