use super::*;
use eda_engine::board::Track;

use super::main_tests_project_route_proposal_artifact::{
    board_tracks_query_cli, seed_plus_one_gap_project, unique_project_root,
};

#[test]
fn project_route_apply_applies_plus_one_gap_candidate_directly() {
    let root = unique_project_root("datum-eda-cli-project-route-apply");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, _) = seed_plus_one_gap_project(&root);

    let apply_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-apply",
        root.to_str().unwrap(),
        "--net",
        &target_net_uuid.to_string(),
        "--from-anchor",
        &anchor_a_uuid.to_string(),
        "--to-anchor",
        &anchor_b_uuid.to_string(),
        "--candidate",
        "authored-copper-plus-one-gap",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "route_apply");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_authored_copper_plus_one_gap_v1"
    );
    assert_eq!(apply_report["proposal_actions"], 1);
    assert_eq!(apply_report["applied_actions"], 1);
    assert_eq!(apply_report["applied"][0]["from_x_nm"], 700000);
    assert_eq!(apply_report["applied"][0]["to_x_nm"], 1300000);
    assert_eq!(apply_report["applied"][0]["width_nm"], 150000);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 3);
    assert!(tracks.iter().any(|track| {
        track.net == target_net_uuid
            && track.from.x == 700000
            && track.to.x == 1300000
            && track.width == 150000
            && track.layer == 1
    }));

    let _ = std::fs::remove_dir_all(&root);
}
