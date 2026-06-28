use super::main_tests_project_board_zone::{
    check_run_query, place_zone_fixture_with_thermal, zone_fills_query,
};
use super::main_tests_project_check::{seed_board_process_aperture_fixture, unique_project_root};
use super::main_tests_project_standards_repairs::seed_board_dimension_rule_fixture;
use super::*;
use eda_engine::substrate::{ProjectResolver, ProposalStatus, SourceShardKind};

#[test]
fn project_accept_apply_standards_repair_proposal_updates_board_and_findings() {
    let root = unique_project_root("datum-eda-cli-project-standards-repair-apply");
    create_native_project(&root, Some("Standards Repair Apply Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_process_aperture_fixture(&root);
    let generated = generate_standards_repair_report(&root);
    let proposal = generated["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "process_aperture")
        .expect("process aperture repair proposal should exist");
    let proposal_id = proposal["proposal_id"].as_str().unwrap().to_string();
    let affected_pad = proposal["affected_pad"].as_str().unwrap().to_string();

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
    .expect("standards repair proposal should accept/apply");
    let apply: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply JSON should parse");
    assert_eq!(apply["action"], "apply_proposal");
    assert_eq!(apply["status"], "applied");
    assert_eq!(apply["validation"]["can_apply"], true);

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve after repair apply");
    let proposal_uuid = Uuid::parse_str(&proposal_id).unwrap();
    assert_eq!(
        model.proposals.get(&proposal_uuid).unwrap().status,
        ProposalStatus::Applied
    );
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .expect("materialized board should exist");
    let pad = &board["pads"][&affected_pad];
    assert_eq!(pad["solder_mask_margin_nm"], 127000);
    assert_eq!(pad["solder_paste_margin_nm"], -127000);

    let linked = query_check_run(&root);
    let process_findings = linked["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| {
            matches!(
                entry["code"].as_str(),
                Some("pad_process_aperture_inherited_from_copper")
                    | Some("pad_mask_expansion_missing")
                    | Some("pad_paste_reduction_missing")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        process_findings.iter().all(|entry| {
            entry["payload"]["objects"]
                .as_array()
                .unwrap()
                .first()
                .and_then(serde_json::Value::as_str)
                != Some(affected_pad.as_str())
        }),
        "applied repair should clear process-aperture findings for the repaired pad"
    );
    assert_eq!(
        process_findings.len(),
        3,
        "one unfixed fixture pad should still report three process-aperture findings"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_accept_apply_track_geometry_repair_updates_board_and_findings() {
    let root = unique_project_root("datum-eda-cli-project-track-standards-repair-apply");
    create_native_project(&root, Some("Track Standards Repair Apply Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_dimension_rule_fixture(&root);
    let generated = generate_standards_repair_report(&root);
    let proposal = generated["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "track_geometry")
        .expect("track geometry repair proposal should exist");
    let proposal_id = proposal["proposal_id"].as_str().unwrap().to_string();
    let affected_track = proposal["affected_track"].as_str().unwrap().to_string();

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
    .expect("track standards repair proposal should accept/apply");
    let apply: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply JSON should parse");
    assert_eq!(apply["action"], "apply_proposal");
    assert_eq!(apply["status"], "applied");
    assert_eq!(apply["validation"]["can_apply"], true);

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve after track repair apply");
    let proposal_uuid = Uuid::parse_str(&proposal_id).unwrap();
    assert_eq!(
        model.proposals.get(&proposal_uuid).unwrap().status,
        ProposalStatus::Applied
    );
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .expect("materialized board should exist");
    let track = &board["tracks"][&affected_track];
    assert_eq!(track["width"], 200000);

    let linked = query_check_run(&root);
    let track_findings = linked["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["code"] == "track_width_below_min")
        .collect::<Vec<_>>();
    assert!(
        track_findings.iter().all(|entry| {
            entry["payload"]["objects"]
                .as_array()
                .unwrap()
                .first()
                .and_then(serde_json::Value::as_str)
                != Some(affected_track.as_str())
        }),
        "applied repair should clear the track-width finding for the repaired track"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_accept_apply_via_geometry_repair_updates_board_and_findings() {
    let root = unique_project_root("datum-eda-cli-project-via-standards-repair-apply");
    create_native_project(&root, Some("Via Standards Repair Apply Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_board_dimension_rule_fixture(&root);
    let generated = generate_standards_repair_report(&root);
    let proposal = generated["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "via_geometry")
        .expect("via geometry repair proposal should exist");
    let proposal_id = proposal["proposal_id"].as_str().unwrap().to_string();
    let affected_via = proposal["affected_via"].as_str().unwrap().to_string();

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
    .expect("via standards repair proposal should accept/apply");
    let apply: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply JSON should parse");
    assert_eq!(apply["action"], "apply_proposal");
    assert_eq!(apply["status"], "applied");
    assert_eq!(apply["validation"]["can_apply"], true);

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve after via repair apply");
    let proposal_uuid = Uuid::parse_str(&proposal_id).unwrap();
    assert_eq!(
        model.proposals.get(&proposal_uuid).unwrap().status,
        ProposalStatus::Applied
    );
    let board = model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .expect("materialized board should exist");
    let via = &board["vias"][&affected_via];
    assert_eq!(via["drill"], 300000);
    assert_eq!(via["diameter"], 600000);

    let linked = query_check_run(&root);
    let via_findings = linked["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| {
            matches!(
                entry["code"].as_str(),
                Some("via_hole_out_of_range") | Some("via_annular_below_min")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        via_findings.iter().all(|entry| {
            entry["payload"]["objects"]
                .as_array()
                .unwrap()
                .first()
                .and_then(serde_json::Value::as_str)
                != Some(affected_via.as_str())
        }),
        "applied repair should clear via geometry findings for the repaired via"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_accept_apply_zone_fill_repair_updates_evidence_and_findings() {
    let root = unique_project_root("datum-eda-cli-project-zone-fill-standards-repair-apply");
    create_native_project(
        &root,
        Some("Zone Fill Standards Repair Apply Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let zone_uuid = place_zone_fixture_with_thermal(&root, false);
    let generated = generate_standards_repair_report(&root);
    let proposal = generated["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["repair_kind"] == "zone_fill")
        .expect("zone-fill repair proposal should exist");
    let proposal_id = proposal["proposal_id"].as_str().unwrap().to_string();
    assert_eq!(proposal["affected_zone"], zone_uuid);

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
    .expect("zone-fill standards repair proposal should accept/apply");
    let apply: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply JSON should parse");
    assert_eq!(apply["action"], "apply_proposal");
    assert_eq!(apply["status"], "applied");
    assert_eq!(apply["validation"]["can_apply"], true);

    let proposal_uuid = Uuid::parse_str(&proposal_id).unwrap();
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project should resolve after zone-fill repair apply");
    assert_eq!(
        model.proposals.get(&proposal_uuid).unwrap().status,
        ProposalStatus::Applied
    );
    let fills = zone_fills_query(&root);
    let fill = fills["zone_fills"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["zone_id"] == zone_uuid)
        .expect("zone fill should remain visible");
    assert_eq!(fill["state"], "filled");

    let linked = check_run_query(&root);
    assert!(
        linked["findings"].as_array().unwrap().iter().all(|entry| {
            !matches!(
                entry["code"].as_str(),
                Some("zone_fill_unfilled") | Some("zone_fill_stale")
            ) || entry["payload"]["zone_id"] != zone_uuid
        }),
        "applied repair should clear zone-fill findings for the repaired zone"
    );

    let _ = std::fs::remove_dir_all(&root);
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
