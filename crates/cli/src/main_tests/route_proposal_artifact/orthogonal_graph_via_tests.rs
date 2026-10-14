//! Native route artifact export/apply coverage for one through six reused vias.

use super::*;

#[test]
fn project_route_proposal_artifact_supports_orthogonal_graph_via_candidate() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-orthogonal-graph-via-artifact",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_uuid) =
        seed_route_path_candidate_orthogonal_graph_via_project(&root);
    let artifact = root.join("route-path-candidate-orthogonal-graph-via-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-orthogonal-graph-via",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["actions"], 5);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_via_v1"
    );
    assert!(export_report["selected_path_bend_count"].as_u64().unwrap() >= 1);
    let export_segment_evidence = export_report["segment_evidence"]
        .as_array()
        .expect("export segment evidence should be an array");
    assert_eq!(export_segment_evidence.len(), 2);
    assert_eq!(export_segment_evidence[0]["layer_segment_count"], 2);
    assert_eq!(export_segment_evidence[1]["layer_segment_count"], 2);
    assert!(
        export_segment_evidence[0]["track_action_count"]
            .as_u64()
            .unwrap()
            >= 1
    );
    assert!(
        export_segment_evidence[1]["track_action_count"]
            .as_u64()
            .unwrap()
            >= 1
    );

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    assert_eq!(artifact_value["actions"].as_array().unwrap().len(), 5);
    assert_eq!(
        artifact_value["actions"][0]["reason"],
        "route_path_candidate_orthogonal_graph_via"
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuid"],
        via_uuid.to_string()
    );
    assert!(
        artifact_value["actions"][0]["selected_path_bend_count"]
            .as_u64()
            .unwrap()
            >= 1
    );

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-route-proposal-artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    let inspect_segment_evidence = inspect_report["segment_evidence"]
        .as_array()
        .expect("inspect segment evidence should be an array");
    assert_eq!(inspect_segment_evidence.len(), 2);
    assert_eq!(inspect_segment_evidence[0]["layer_segment_count"], 2);
    assert_eq!(inspect_segment_evidence[1]["layer_segment_count"], 2);
    assert!(
        inspect_segment_evidence[0]["track_action_count"]
            .as_u64()
            .unwrap()
            >= 1
    );
    assert!(
        inspect_segment_evidence[1]["track_action_count"]
            .as_u64()
            .unwrap()
            >= 1
    );

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"], 5);
    assert_eq!(apply_report["applied_actions"], 5);
    assert!(apply_report["selected_path_bend_count"].as_u64().unwrap() >= 1);

    let board_tracks_output = execute(board_tracks_query_cli(&root)).expect("query should succeed");
    let board_tracks: serde_json::Value =
        serde_json::from_str(&board_tracks_output).expect("board tracks should parse");
    assert_eq!(board_tracks.as_array().unwrap().len(), 9);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_proposal_artifact_supports_orthogonal_graph_two_via_candidate() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-orthogonal-graph-two-via-artifact",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_a_uuid, via_b_uuid) =
        seed_route_path_candidate_orthogonal_graph_two_via_project(&root);
    let artifact = root.join("route-path-candidate-orthogonal-graph-two-via-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-orthogonal-graph-two-via",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    assert_eq!(export_report["actions"], 7);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_two_via_v1"
    );
    assert!(export_report["selected_path_bend_count"].as_u64().unwrap() >= 1);
    let export_segment_evidence = export_report["segment_evidence"]
        .as_array()
        .expect("export segment evidence should be an array");
    assert_eq!(export_segment_evidence.len(), 3);
    assert_eq!(export_segment_evidence[0]["layer_segment_count"], 3);
    assert_eq!(export_segment_evidence[1]["layer_segment_count"], 3);
    assert_eq!(export_segment_evidence[2]["layer_segment_count"], 3);

    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    assert_eq!(artifact_value["actions"].as_array().unwrap().len(), 7);
    assert_eq!(
        artifact_value["actions"][0]["reason"],
        "route_path_candidate_orthogonal_graph_two_via"
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuid"],
        via_a_uuid.to_string()
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"]
            .as_array()
            .unwrap(),
        &vec![
            serde_json::Value::String(via_a_uuid.to_string()),
            serde_json::Value::String(via_b_uuid.to_string()),
        ]
    );
    assert!(
        artifact_value["actions"][0]["selected_path_bend_count"]
            .as_u64()
            .unwrap()
            >= 1
    );

    let inspect_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-route-proposal-artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let inspect_report: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspect report should parse");
    let inspect_segment_evidence = inspect_report["segment_evidence"]
        .as_array()
        .expect("inspect segment evidence should be an array");
    assert_eq!(inspect_segment_evidence.len(), 3);
    assert_eq!(inspect_segment_evidence[0]["layer_segment_count"], 3);
    assert_eq!(inspect_segment_evidence[1]["layer_segment_count"], 3);
    assert_eq!(inspect_segment_evidence[2]["layer_segment_count"], 3);

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"], 7);
    assert_eq!(apply_report["applied_actions"], 7);
    assert!(apply_report["selected_path_bend_count"].as_u64().unwrap() >= 1);

    let board_tracks_output = execute(board_tracks_query_cli(&root)).expect("query should succeed");
    let board_tracks: serde_json::Value =
        serde_json::from_str(&board_tracks_output).expect("board tracks should parse");
    assert_eq!(board_tracks.as_array().unwrap().len(), 10);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_proposal_artifact_supports_orthogonal_graph_three_via_candidate() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-orthogonal-graph-three-via-artifact",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_a_uuid, via_b_uuid, via_c_uuid) =
        seed_route_path_candidate_three_via_project(&root);
    let artifact = root.join("route-path-candidate-orthogonal-graph-three-via-proposal.json");

    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-orthogonal-graph-three-via",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"].as_array().unwrap().len() as u64;
    assert_eq!(export_report["actions"].as_u64().unwrap(), actions);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_three_via_v1"
    );
    assert_eq!(
        artifact_value["actions"][0]["reason"],
        "route_path_candidate_orthogonal_graph_three_via"
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuid"],
        via_a_uuid.to_string()
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"][0],
        via_a_uuid.to_string()
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"][1],
        via_b_uuid.to_string()
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"][2],
        via_c_uuid.to_string()
    );

    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"].as_u64().unwrap(), actions);
    assert_eq!(apply_report["applied_actions"].as_u64().unwrap(), actions);

    let board_tracks: serde_json::Value = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("query should succeed"),
    )
    .expect("board tracks should parse");
    assert_eq!(board_tracks.as_array().unwrap().len() as u64, actions);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_proposal_artifact_supports_orthogonal_graph_four_via_candidate() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-orthogonal-graph-four-via-artifact",
    );
    let (
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
        via_a_uuid,
        via_b_uuid,
        via_c_uuid,
        via_d_uuid,
    ) = seed_route_path_candidate_four_via_project(&root);
    let artifact = root.join("route-path-candidate-orthogonal-graph-four-via-proposal.json");
    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-orthogonal-graph-four-via",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"].as_array().unwrap().len() as u64;
    assert_eq!(export_report["actions"].as_u64().unwrap(), actions);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_four_via_v1"
    );
    assert_eq!(
        artifact_value["actions"][0]["reason"],
        "route_path_candidate_orthogonal_graph_four_via"
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuid"],
        via_a_uuid.to_string()
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"][1],
        via_b_uuid.to_string()
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"][2],
        via_c_uuid.to_string()
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"][3],
        via_d_uuid.to_string()
    );
    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"].as_u64().unwrap(), actions);
    assert_eq!(apply_report["applied_actions"].as_u64().unwrap(), actions);
    let board_tracks: serde_json::Value = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("query should succeed"),
    )
    .expect("board tracks should parse");
    assert_eq!(board_tracks.as_array().unwrap().len() as u64, actions);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_proposal_artifact_supports_orthogonal_graph_five_via_candidate() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-orthogonal-graph-five-via-artifact",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_a_uuid, _, _, _, via_e_uuid) =
        seed_route_path_candidate_five_via_project(&root);
    let artifact = root.join("route-path-candidate-orthogonal-graph-five-via-proposal.json");
    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-orthogonal-graph-five-via",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"].as_array().unwrap().len() as u64;
    assert_eq!(export_report["actions"].as_u64().unwrap(), actions);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_five_via_v1"
    );
    assert_eq!(
        artifact_value["actions"][0]["reason"],
        "route_path_candidate_orthogonal_graph_five_via"
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuid"],
        via_a_uuid.to_string()
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"][4],
        via_e_uuid.to_string()
    );
    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"].as_u64().unwrap(), actions);
    assert_eq!(apply_report["applied_actions"].as_u64().unwrap(), actions);
    let board_tracks: serde_json::Value = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("query should succeed"),
    )
    .expect("board tracks should parse");
    assert_eq!(board_tracks.as_array().unwrap().len() as u64, actions);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_route_proposal_artifact_supports_orthogonal_graph_six_via_candidate() {
    let root = unique_project_root(
        "datum-eda-cli-project-route-path-candidate-orthogonal-graph-six-via-artifact",
    );
    let (target_net_uuid, anchor_a_uuid, anchor_b_uuid, via_a_uuid, _, _, _, _, via_f_uuid) =
        seed_route_path_candidate_six_via_project(&root);
    let artifact = root.join("route-path-candidate-orthogonal-graph-six-via-proposal.json");
    let export_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "export-route-path-proposal",
            root.to_str().unwrap(),
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
            "--candidate",
            "route-path-candidate-orthogonal-graph-six-via",
            "--out",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report should parse");
    let artifact_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&artifact).expect("artifact should read"))
            .expect("artifact should parse");
    let actions = artifact_value["actions"].as_array().unwrap().len() as u64;
    assert_eq!(export_report["actions"].as_u64().unwrap(), actions);
    assert_eq!(
        export_report["contract"],
        "m5_route_path_candidate_orthogonal_graph_six_via_v1"
    );
    assert_eq!(
        artifact_value["actions"][0]["reason"],
        "route_path_candidate_orthogonal_graph_six_via"
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuid"],
        via_a_uuid.to_string()
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(
        artifact_value["actions"][0]["reused_via_uuids"][5],
        via_f_uuid.to_string()
    );
    let apply_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "apply-route-proposal-artifact",
            root.to_str().unwrap(),
            "--artifact",
            artifact.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("apply should succeed");
    let apply_report: serde_json::Value =
        serde_json::from_str(&apply_output).expect("apply report should parse");
    assert_eq!(apply_report["artifact_actions"].as_u64().unwrap(), actions);
    assert_eq!(apply_report["applied_actions"].as_u64().unwrap(), actions);
    let board_tracks: serde_json::Value = serde_json::from_str(
        &execute(board_tracks_query_cli(&root)).expect("query should succeed"),
    )
    .expect("board tracks should parse");
    assert_eq!(board_tracks.as_array().unwrap().len() as u64, actions);
    let _ = std::fs::remove_dir_all(&root);
}
