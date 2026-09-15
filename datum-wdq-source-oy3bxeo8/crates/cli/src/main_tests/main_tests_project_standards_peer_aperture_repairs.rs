use std::path::Path;

use super::main_tests_project_check::{seed_board_process_aperture_fixture, unique_project_root};
use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{ProjectResolver, ProposalStatus, SourceShardKind};

#[test]
fn project_generate_standards_repair_proposals_links_peer_aperture_findings() {
    let root = unique_project_root("datum-eda-cli-project-peer-aperture-standards-repair");
    create_native_project(&root, Some("Peer Aperture Repair Demo".to_string()))
        .expect("initial scaffold should succeed");
    let target_pad = seed_peer_process_aperture_fixture(&root);

    let report = generate_standards_repair_report(&root);
    let proposal = peer_aperture_proposal(&report, &target_pad);
    assert_eq!(proposal["affected_pad"], target_pad);
    assert_eq!(proposal["repair_kind"], "process_aperture");
    assert!(
        proposal["codes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|code| code == "pad_process_aperture_inconsistent_with_peer_footprint")
    );

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve with peer aperture proposal");
    let proposal_id = Uuid::parse_str(proposal["proposal_id"].as_str().unwrap()).unwrap();
    let proposal = model.proposals.get(&proposal_id).unwrap();
    assert_eq!(
        proposal.source,
        eda_engine::substrate::ProposalSource::Check
    );
    let eda_engine::substrate::Operation::SetBoardPad { pad_id, pad } =
        &proposal.batch.operations[0]
    else {
        panic!("peer aperture repair should use SetBoardPad");
    };
    assert_eq!(*pad_id, Uuid::parse_str(&target_pad).unwrap());
    assert_eq!(pad["solder_mask_margin_nm"], 127000);
    assert_eq!(pad["solder_paste_margin_nm"], -127000);
    assert_eq!(pad["solder_paste_margin_ratio_ppm"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_accept_apply_peer_aperture_repair_updates_board_and_findings() {
    let root = unique_project_root("datum-eda-cli-project-peer-aperture-standards-apply");
    create_native_project(&root, Some("Peer Aperture Repair Apply Demo".to_string()))
        .expect("initial scaffold should succeed");
    let target_pad = seed_peer_process_aperture_fixture(&root);
    let generated = generate_standards_repair_report(&root);
    let proposal = peer_aperture_proposal(&generated, &target_pad);
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
    .expect("peer aperture proposal should accept/apply");
    let apply: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply JSON should parse");
    assert_eq!(apply["status"], "applied");
    assert_eq!(apply["validation"]["can_apply"], true);

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve after peer aperture repair apply");
    let proposal_uuid = Uuid::parse_str(&proposal_id).unwrap();
    assert_eq!(
        model.proposals.get(&proposal_uuid).unwrap().status,
        ProposalStatus::Applied
    );
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .expect("materialized board should exist");
    let pad = &board["pads"][&target_pad];
    assert_eq!(pad["solder_mask_margin_nm"], 127000);
    assert_eq!(pad["solder_paste_margin_nm"], -127000);

    let check = query_check_run(&root);
    assert!(
        check["findings"].as_array().unwrap().iter().all(|entry| {
            entry["code"] != "pad_process_aperture_inconsistent_with_peer_footprint"
                || entry["payload"]["objects"]
                    .as_array()
                    .unwrap()
                    .first()
                    .and_then(serde_json::Value::as_str)
                    != Some(target_pad.as_str())
        }),
        "applied repair should clear peer aperture finding for target pad"
    );

    let _ = std::fs::remove_dir_all(&root);
}

fn seed_peer_process_aperture_fixture(root: &Path) -> String {
    seed_board_process_aperture_fixture(root);
    let board_json = root.join("board/board.json");
    let mut board: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&board_json).unwrap()).unwrap();
    let target_pad = board["pads"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .to_string();
    let package = board["pads"][&target_pad]["package"].clone();
    let net = board["pads"][&target_pad]["net"].clone();
    board["pads"][&target_pad]["solder_mask_margin_nm"] = serde_json::json!(200000);
    board["pads"][&target_pad]["solder_paste_margin_nm"] = serde_json::json!(-127000);
    board["pads"][&target_pad]["solder_paste_margin_ratio_ppm"] = serde_json::json!(0);

    for (name, x_nm) in [("2", 1_000_000), ("3", 2_000_000)] {
        let pad_uuid = Uuid::new_v4();
        board["pads"][pad_uuid.to_string()] = serde_json::json!({
            "uuid": pad_uuid,
            "package": package,
            "name": name,
            "net": net,
            "position": { "x": x_nm, "y": 0 },
            "layer": 1,
            "copper_layers": [1],
            "shape": "rect",
            "width": 1000000,
            "height": 500000,
            "mask_layers": [2],
            "paste_layers": [4],
            "solder_mask_margin_nm": 127000,
            "solder_paste_margin_nm": -127000,
            "solder_paste_margin_ratio_ppm": 0
        });
    }
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&board).expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");
    target_pad
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

fn peer_aperture_proposal<'a>(
    report: &'a serde_json::Value,
    target_pad: &str,
) -> &'a serde_json::Value {
    report["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            entry["repair_kind"] == "process_aperture"
                && entry["affected_pad"].as_str() == Some(target_pad)
                && entry["codes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|code| code == "pad_process_aperture_inconsistent_with_peer_footprint")
        })
        .expect("peer aperture repair proposal should exist")
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
