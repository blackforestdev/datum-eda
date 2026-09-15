use eda_engine::board::Track;

use super::main_tests_project_route_proposal_artifact::{
    board_tracks_query_cli, rewrite_board_json, seed_route_path_candidate_project,
    seed_route_proposal_profile_project, unique_project_root,
};
use super::*;

#[test]
fn project_export_route_proposal_exports_selected_candidate_artifact() {
    let root = unique_project_root("datum-eda-cli-project-export-route-proposal");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);
    let artifact = root.join("selected-route-proposal.json");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-route-proposal",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
        "--out",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("selected export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["action"], "export_route_proposal");
    assert_eq!(report["selection_profile"], "default");
    assert_eq!(report["selected_candidate"], "route-path-candidate");
    assert_eq!(report["contract"], "m5_route_path_candidate_v2");
    assert_eq!(report["artifact_path"], artifact.display().to_string());
    assert!(artifact.exists());
}

#[test]
fn project_route_apply_selected_applies_selected_candidate_directly() {
    let root = unique_project_root("datum-eda-cli-project-route-apply-selected");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "route-apply-selected",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("selected apply should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["action"], "route_apply_selected");
    assert_eq!(report["selection_profile"], "default");
    assert_eq!(report["selected_candidate"], "route-path-candidate");
    assert_eq!(report["contract"], "m5_route_path_candidate_v2");
    assert_eq!(report["proposal_actions"], 1);
    assert_eq!(report["applied_actions"], 1);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 1);
    assert!(tracks.iter().all(|track| track.net == net_uuid));
}

#[test]
fn project_export_route_proposal_rejects_when_selector_finds_no_route() {
    let root = unique_project_root("datum-eda-cli-project-export-route-proposal-none");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_path_candidate_project(&root);

    rewrite_board_json(&root, |board| {
        board["nets"][net_uuid.to_string()]["class"] = serde_json::Value::Null;
    });

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-route-proposal",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
        "--out",
        root.join("no-route.json").to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let err = execute(cli).expect_err("selected export should fail when no route exists");
    assert_eq!(
        err.to_string(),
        "route-proposal found no selectable route under current authored constraints"
    );
}

#[test]
fn project_export_route_proposal_authored_copper_priority_profile_exports_selected_authored_copper()
{
    let root = unique_project_root("datum-eda-cli-project-export-route-proposal-profile");
    let (net_uuid, from_anchor_uuid, to_anchor_uuid) = seed_route_proposal_profile_project(&root);
    let artifact = root.join("selected-route-proposal-profile.json");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-route-proposal",
        root.to_str().unwrap(),
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor_uuid.to_string(),
        "--to-anchor",
        &to_anchor_uuid.to_string(),
        "--profile",
        "authored-copper-priority",
        "--out",
        artifact.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("selected export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("output should parse");

    assert_eq!(report["selection_profile"], "authored-copper-priority");
    assert_eq!(report["selected_candidate"], "authored-copper-graph");
    assert_eq!(report["selected_policy"], "plain");
    assert_eq!(
        report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );
    assert!(artifact.exists());
}
