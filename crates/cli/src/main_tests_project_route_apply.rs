use super::*;
use eda_engine::board::Track;

use super::main_tests_project_route_proposal_artifact::{
    board_tracks_query_cli, seed_plus_one_gap_project,
    seed_route_path_candidate_authored_copper_graph_obstacle_aware_project,
    seed_route_path_candidate_authored_via_chain_project,
    seed_route_path_candidate_five_via_project, seed_route_path_candidate_four_via_project,
    seed_route_path_candidate_orthogonal_graph_two_via_project,
    seed_route_path_candidate_orthogonal_graph_via_project, seed_route_path_candidate_project,
    seed_route_path_candidate_six_via_project, seed_route_path_candidate_three_via_project,
    seed_route_path_candidate_two_via_project, seed_route_path_candidate_via_project,
    unique_project_root,
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

#[test]
fn project_route_apply_rejects_unknown_candidate_at_parse_time() {
    let err = match Cli::try_parse_from([
        "eda",
        "project",
        "route-apply",
        "/tmp/demo",
        "--net",
        &Uuid::nil().to_string(),
        "--from-anchor",
        &Uuid::nil().to_string(),
        "--to-anchor",
        &Uuid::nil().to_string(),
        "--candidate",
        "not-a-valid-candidate",
    ]) {
        Ok(_) => panic!("CLI should reject unknown candidate"),
        Err(err) => err,
    };

    let rendered = err.to_string();
    assert!(rendered.contains("invalid value"));
    assert!(rendered.contains("route-path-candidate"));
    assert!(rendered.contains("authored-copper-graph"));
}

#[test]
fn project_route_apply_accepts_authored_copper_graph_policy_candidate_directly() {
    let root = unique_project_root("datum-eda-cli-project-route-apply-authored-copper-graph");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, track_uuid) =
        seed_route_path_candidate_authored_copper_graph_obstacle_aware_project(&root);

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
        "authored-copper-graph",
        "--policy",
        "plain",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "route_apply");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_authored_copper_graph_policy_v1"
    );
    assert_eq!(apply_report["proposal_actions"], 1);
    assert_eq!(apply_report["applied_actions"], 0);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 1);
    assert!(
        tracks
            .iter()
            .any(|track| track.uuid == track_uuid && track.net == target_net_uuid)
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_rejects_policy_for_non_policy_candidate() {
    let root = unique_project_root("datum-eda-cli-project-route-apply-policy-misuse");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid) = seed_route_path_candidate_project(&root);

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
        "route-path-candidate",
        "--policy",
        "plain",
    ])
    .expect("CLI should parse");
    let err = execute(apply_cli).expect_err("non-policy candidate should reject --policy");
    assert_eq!(
        err.to_string(),
        "route-apply --policy is supported only for candidate authored-copper-graph"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_single_layer_route_path_candidate_directly() {
    let root = unique_project_root("datum-eda-cli-project-route-apply-route-path-candidate");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid) = seed_route_path_candidate_project(&root);

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
        "route-path-candidate",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "route_apply");
    assert_eq!(apply_report["contract"], "m5_route_path_candidate_v2");
    assert_eq!(apply_report["proposal_actions"], 1);
    assert_eq!(apply_report["applied_actions"], 1);
    assert_eq!(apply_report["applied"][0]["from_x_nm"], 500000);
    assert_eq!(apply_report["applied"][0]["to_x_nm"], 4500000);
    assert_eq!(apply_report["applied"][0]["from_y_nm"], 600000);
    assert_eq!(apply_report["applied"][0]["to_y_nm"], 2400000);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 1);
    assert!(tracks.iter().all(|track| track.net == target_net_uuid));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_single_via_route_path_candidate_directly() {
    let root = unique_project_root("datum-eda-cli-project-route-apply-route-path-candidate-via");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_uuid) =
        seed_route_path_candidate_via_project(&root);

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
        "route-path-candidate-via",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "route_apply");
    assert_eq!(apply_report["contract"], "m5_route_path_candidate_via_v1");
    assert_eq!(apply_report["proposal_actions"], 2);
    assert_eq!(apply_report["applied_actions"], 2);
    assert_eq!(apply_report["applied"][0]["from_x_nm"], 500000);
    assert_eq!(apply_report["applied"][0]["to_x_nm"], 2500000);
    assert_eq!(apply_report["applied"][1]["from_x_nm"], 2500000);
    assert_eq!(apply_report["applied"][1]["to_x_nm"], 4500000);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 2);
    assert!(tracks.iter().all(|track| track.net == target_net_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_uuid));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_orthogonal_graph_via_candidate_directly() {
    let root =
        unique_project_root("datum-eda-cli-project-route-apply-route-path-candidate-graph-via");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_uuid) =
        seed_route_path_candidate_orthogonal_graph_via_project(&root);

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
        "route-path-candidate-orthogonal-graph-via",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_via_v1"
    );
    assert_eq!(apply_report["proposal_actions"], 5);
    assert_eq!(apply_report["applied_actions"], 5);
    assert_eq!(
        apply_report["applied"][0]["reused_via_uuid"],
        serde_json::Value::Null
    );

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 9);
    assert!(tracks.iter().any(|track| track.net == target_net_uuid));
    let _ = via_uuid;

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_orthogonal_graph_two_via_candidate_directly() {
    let root =
        unique_project_root("datum-eda-cli-project-route-apply-route-path-candidate-graph-two-via");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_a_uuid, via_b_uuid) =
        seed_route_path_candidate_orthogonal_graph_two_via_project(&root);

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
        "route-path-candidate-orthogonal-graph-two-via",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_two_via_v1"
    );
    assert_eq!(apply_report["proposal_actions"], 7);
    assert_eq!(apply_report["applied_actions"], 7);
    assert_eq!(
        apply_report["applied"][0]["reused_via_uuid"],
        serde_json::Value::Null
    );

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 10);
    assert!(tracks.iter().any(|track| track.net == target_net_uuid));
    let _ = (via_a_uuid, via_b_uuid);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_orthogonal_graph_three_via_candidate_directly() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-apply-route-path-candidate-graph-three-via",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, _, _, _) =
        seed_route_path_candidate_three_via_project(&root);

    let apply_output = execute(
        Cli::try_parse_from([
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
            "route-path-candidate-orthogonal-graph-three-via",
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_three_via_v1"
    );
    assert_eq!(
        apply_report["applied_actions"],
        apply_report["proposal_actions"]
    );

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(
        tracks.len() as u64,
        apply_report["applied_actions"].as_u64().unwrap()
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_orthogonal_graph_four_via_candidate_directly() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-apply-route-path-candidate-graph-four-via",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, _, _, _, _) =
        seed_route_path_candidate_four_via_project(&root);

    let apply_output = execute(
        Cli::try_parse_from([
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
            "route-path-candidate-orthogonal-graph-four-via",
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_four_via_v1"
    );
    assert_eq!(
        apply_report["applied_actions"],
        apply_report["proposal_actions"]
    );

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(
        tracks.len() as u64,
        apply_report["applied_actions"].as_u64().unwrap()
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_orthogonal_graph_five_via_candidate_directly() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-apply-route-path-candidate-graph-five-via",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, _, _, _, _, _) =
        seed_route_path_candidate_five_via_project(&root);

    let apply_output = execute(
        Cli::try_parse_from([
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
            "route-path-candidate-orthogonal-graph-five-via",
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_five_via_v1"
    );
    assert_eq!(
        apply_report["applied_actions"],
        apply_report["proposal_actions"]
    );

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(
        tracks.len() as u64,
        apply_report["applied_actions"].as_u64().unwrap()
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_orthogonal_graph_six_via_candidate_directly() {
    let root =
        unique_project_root("datum-eda-cli-project-route-apply-route-path-candidate-graph-six-via");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, _, _, _, _, _, _) =
        seed_route_path_candidate_six_via_project(&root);

    let apply_output = execute(
        Cli::try_parse_from([
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
            "route-path-candidate-orthogonal-graph-six-via",
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_six_via_v1"
    );
    assert_eq!(
        apply_report["applied_actions"],
        apply_report["proposal_actions"]
    );

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(
        tracks.len() as u64,
        apply_report["applied_actions"].as_u64().unwrap()
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_two_via_route_path_candidate_directly() {
    let root =
        unique_project_root("datum-eda-cli-project-route-apply-route-path-candidate-two-via");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_a_uuid, via_b_uuid) =
        seed_route_path_candidate_two_via_project(&root);

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
        "route-path-candidate-two-via",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "route_apply");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_two_via_v1"
    );
    assert_eq!(apply_report["proposal_actions"], 3);
    assert_eq!(apply_report["applied_actions"], 3);
    assert_eq!(apply_report["applied"][0]["from_x_nm"], 500000);
    assert_eq!(apply_report["applied"][0]["to_x_nm"], 1500000);
    assert_eq!(apply_report["applied"][1]["from_x_nm"], 1500000);
    assert_eq!(apply_report["applied"][1]["to_x_nm"], 3500000);
    assert_eq!(apply_report["applied"][2]["from_x_nm"], 3500000);
    assert_eq!(apply_report["applied"][2]["to_x_nm"], 4500000);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 3);
    assert!(tracks.iter().all(|track| track.net == target_net_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_a_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_b_uuid));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_three_via_route_path_candidate_directly() {
    let root =
        unique_project_root("datum-eda-cli-project-route-apply-route-path-candidate-three-via");
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_a_uuid, via_b_uuid, via_c_uuid) =
        seed_route_path_candidate_three_via_project(&root);

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
        "route-path-candidate-three-via",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "route_apply");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_three_via_v1"
    );
    assert_eq!(apply_report["proposal_actions"], 4);
    assert_eq!(apply_report["applied_actions"], 4);
    assert_eq!(apply_report["applied"][0]["from_x_nm"], 500000);
    assert_eq!(apply_report["applied"][0]["to_x_nm"], 1200000);
    assert_eq!(apply_report["applied"][1]["from_x_nm"], 1200000);
    assert_eq!(apply_report["applied"][1]["to_x_nm"], 2500000);
    assert_eq!(apply_report["applied"][2]["from_x_nm"], 2500000);
    assert_eq!(apply_report["applied"][2]["to_x_nm"], 3800000);
    assert_eq!(apply_report["applied"][3]["from_x_nm"], 3800000);
    assert_eq!(apply_report["applied"][3]["to_x_nm"], 4500000);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 4);
    assert!(tracks.iter().all(|track| track.net == target_net_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_a_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_b_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_c_uuid));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_four_via_route_path_candidate_directly() {
    let root =
        unique_project_root("datum-eda-cli-project-route-apply-route-path-candidate-four-via");
    let (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
    ) = seed_route_path_candidate_four_via_project(&root);

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
        "route-path-candidate-four-via",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "route_apply");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_four_via_v1"
    );
    assert_eq!(apply_report["proposal_actions"], 5);
    assert_eq!(apply_report["applied_actions"], 5);
    assert_eq!(apply_report["applied"][0]["from_x_nm"], 500000);
    assert_eq!(apply_report["applied"][0]["to_x_nm"], 1100000);
    assert_eq!(apply_report["applied"][1]["from_x_nm"], 1100000);
    assert_eq!(apply_report["applied"][1]["to_x_nm"], 2000000);
    assert_eq!(apply_report["applied"][2]["from_x_nm"], 2000000);
    assert_eq!(apply_report["applied"][2]["to_x_nm"], 3000000);
    assert_eq!(apply_report["applied"][3]["from_x_nm"], 3000000);
    assert_eq!(apply_report["applied"][3]["to_x_nm"], 3900000);
    assert_eq!(apply_report["applied"][4]["from_x_nm"], 3900000);
    assert_eq!(apply_report["applied"][4]["to_x_nm"], 4500000);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 5);
    assert!(tracks.iter().all(|track| track.net == target_net_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_a_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_b_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_c_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_d_uuid));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_five_via_route_path_candidate_directly() {
    let root =
        unique_project_root("datum-eda-cli-project-route-apply-route-path-candidate-five-via");
    let (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
        via_e_uuid,
    ) = seed_route_path_candidate_five_via_project(&root);

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
        "route-path-candidate-five-via",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "route_apply");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_five_via_v1"
    );
    assert_eq!(apply_report["proposal_actions"], 6);
    assert_eq!(apply_report["applied_actions"], 6);
    assert_eq!(apply_report["applied"][0]["from_x_nm"], 500000);
    assert_eq!(apply_report["applied"][0]["to_x_nm"], 1000000);
    assert_eq!(apply_report["applied"][1]["from_x_nm"], 1000000);
    assert_eq!(apply_report["applied"][1]["to_x_nm"], 1700000);
    assert_eq!(apply_report["applied"][2]["from_x_nm"], 1700000);
    assert_eq!(apply_report["applied"][2]["to_x_nm"], 2400000);
    assert_eq!(apply_report["applied"][3]["from_x_nm"], 2400000);
    assert_eq!(apply_report["applied"][3]["to_x_nm"], 3100000);
    assert_eq!(apply_report["applied"][4]["from_x_nm"], 3100000);
    assert_eq!(apply_report["applied"][4]["to_x_nm"], 3800000);
    assert_eq!(apply_report["applied"][5]["from_x_nm"], 3800000);
    assert_eq!(apply_report["applied"][5]["to_x_nm"], 4500000);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 6);
    assert!(tracks.iter().all(|track| track.net == target_net_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_a_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_b_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_c_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_d_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_e_uuid));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_six_via_route_path_candidate_directly() {
    let root =
        unique_project_root("datum-eda-cli-project-route-apply-route-path-candidate-six-via");
    let (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
        via_e_uuid,
        via_f_uuid,
    ) = seed_route_path_candidate_six_via_project(&root);

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
        "route-path-candidate-six-via",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "route_apply");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_six_via_v1"
    );
    assert_eq!(apply_report["proposal_actions"], 7);
    assert_eq!(apply_report["applied_actions"], 7);
    assert_eq!(apply_report["applied"][0]["from_x_nm"], 500000);
    assert_eq!(apply_report["applied"][0]["to_x_nm"], 900000);
    assert_eq!(apply_report["applied"][1]["from_x_nm"], 900000);
    assert_eq!(apply_report["applied"][1]["to_x_nm"], 1450000);
    assert_eq!(apply_report["applied"][2]["from_x_nm"], 1450000);
    assert_eq!(apply_report["applied"][2]["to_x_nm"], 2050000);
    assert_eq!(apply_report["applied"][3]["from_x_nm"], 2050000);
    assert_eq!(apply_report["applied"][3]["to_x_nm"], 2700000);
    assert_eq!(apply_report["applied"][4]["from_x_nm"], 2700000);
    assert_eq!(apply_report["applied"][4]["to_x_nm"], 3350000);
    assert_eq!(apply_report["applied"][5]["from_x_nm"], 3350000);
    assert_eq!(apply_report["applied"][5]["to_x_nm"], 3950000);
    assert_eq!(apply_report["applied"][6]["from_x_nm"], 3950000);
    assert_eq!(apply_report["applied"][6]["to_x_nm"], 4500000);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 7);
    assert!(tracks.iter().all(|track| track.net == target_net_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_a_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_b_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_c_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_d_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_e_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_f_uuid));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_apply_applies_authored_via_chain_route_path_candidate_directly() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-apply-route-path-candidate-authored-via-chain",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_a_uuid, via_b_uuid) =
        seed_route_path_candidate_authored_via_chain_project(&root);

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
        "route-path-candidate-authored-via-chain",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["action"], "route_apply");
    assert_eq!(
        apply_report["contract"],
        "m5_route_path_candidate_authored_via_chain_v1"
    );
    assert_eq!(apply_report["proposal_actions"], 3);
    assert_eq!(apply_report["applied_actions"], 3);
    assert_eq!(apply_report["applied"][0]["from_x_nm"], 500000);
    assert_eq!(apply_report["applied"][0]["to_x_nm"], 1300000);
    assert_eq!(apply_report["applied"][1]["from_x_nm"], 1300000);
    assert_eq!(apply_report["applied"][1]["to_x_nm"], 2600000);
    assert_eq!(apply_report["applied"][2]["from_x_nm"], 2600000);
    assert_eq!(apply_report["applied"][2]["to_x_nm"], 4500000);

    let tracks_output =
        execute(board_tracks_query_cli(&root)).expect("board tracks query should succeed");
    let tracks: Vec<Track> =
        serde_json::from_str(&tracks_output).expect("track query output should parse");
    assert_eq!(tracks.len(), 3);
    assert!(tracks.iter().all(|track| track.net == target_net_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_a_uuid));
    assert!(tracks.iter().all(|track| track.uuid != via_b_uuid));

    let _ = std::fs::remove_dir_all(&root);
}
