use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn route_path_candidate_explain_query_cli(
    root: &Path,
    net_uuid: Uuid,
    from_anchor: Uuid,
    to_anchor: Uuid,
) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "route-path-candidate-explain",
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor.to_string(),
        "--to-anchor",
        &to_anchor.to_string(),
    ])
    .expect("CLI should parse")
}

#[test]
fn project_query_route_path_candidate_explain_reports_deterministic_reasoning() {
    let root = unique_project_root("datum-eda-cli-project-route-path-candidate-explain");
    create_native_project(&root, Some("Route Path Candidate Explain Demo".to_string()))
        .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0x710);
    let class_uuid = Uuid::from_u128(0x711);
    let package_a_uuid = Uuid::from_u128(0x712);
    let package_b_uuid = Uuid::from_u128(0x713);
    let anchor_a_uuid = Uuid::from_u128(0x714);
    let anchor_b_uuid = Uuid::from_u128(0x715);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x716),
                "name": "Route Path Candidate Explain Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 500000, "y": 600000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 4500000, "y": 2400000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(route_path_candidate_explain_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(report["contract"], "m5_route_path_candidate_explain_v1");
    assert_eq!(report["persisted_native_board_state_only"], true);
    assert_eq!(report["status"], "deterministic_path_found");
    assert_eq!(report["explanation_kind"], "deterministic_path_found");
    assert_eq!(report["summary"]["matching_span_count"], 2);
    assert_eq!(report["selected_span"]["layer"], 1);
    assert_eq!(
        report["selection_rule"],
        "select the first unblocked matching corridor span in corridor report order (sorted by candidate copper layer order, then pair index)"
    );

    let repeated = execute(route_path_candidate_explain_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
    ))
    .expect("repeat should succeed");
    assert_eq!(output, repeated);

    let text_output = execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "query",
            root.to_str().unwrap(),
            "route-path-candidate-explain",
            "--net",
            &target_net_uuid.to_string(),
            "--from-anchor",
            &anchor_a_uuid.to_string(),
            "--to-anchor",
            &anchor_b_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("text query should succeed");
    assert!(text_output.contains("contract: m5_route_path_candidate_explain_v1"));
    assert!(text_output.contains("explanation_kind: deterministic_path_found"));
    assert!(text_output.contains("selected_span_layer: 1"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_route_path_candidate_explain_preserves_reversed_anchor_orientation() {
    let root = unique_project_root("datum-eda-cli-project-route-path-candidate-explain-reversed");
    create_native_project(
        &root,
        Some("Route Path Candidate Explain Reversed".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0x820);
    let class_uuid = Uuid::from_u128(0x821);
    let package_a_uuid = Uuid::from_u128(0x822);
    let package_b_uuid = Uuid::from_u128(0x823);
    let anchor_a_uuid = Uuid::from_u128(0x824);
    let anchor_b_uuid = Uuid::from_u128(0x825);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x826),
                "name": "Route Path Candidate Explain Reversed Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 500000, "y": 600000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 4500000, "y": 2400000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(route_path_candidate_explain_query_cli(
        &root,
        target_net_uuid,
        anchor_b_uuid,
        anchor_a_uuid,
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(report["explanation_kind"], "deterministic_path_found");
    assert_eq!(report["selected_span"]["layer"], 1);
    assert_eq!(
        report["selected_span"]["from"],
        serde_json::json!({"x": 4500000, "y": 2400000})
    );
    assert_eq!(
        report["selected_span"]["to"],
        serde_json::json!({"x": 500000, "y": 600000})
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_route_path_candidate_explain_distinguishes_no_match_from_all_blocked() {
    let root = unique_project_root("datum-eda-cli-project-route-path-candidate-explain-negative");
    create_native_project(
        &root,
        Some("Route Path Candidate Explain Negative".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0x830);
    let class_uuid = Uuid::from_u128(0x831);
    let package_a_uuid = Uuid::from_u128(0x832);
    let package_b_uuid = Uuid::from_u128(0x833);
    let package_c_uuid = Uuid::from_u128(0x834);
    let anchor_a_uuid = Uuid::from_u128(0x835);
    let anchor_b_uuid = Uuid::from_u128(0x836);
    let anchor_c_uuid = Uuid::from_u128(0x837);
    let keepout_uuid = Uuid::from_u128(0x838);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x839),
                "name": "Route Path Candidate Explain Negative Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Core", "layer_type": "Dielectric", "thickness_nm": 1600000 },
                        { "id": 3, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 3000000 },
                        { "x": 0, "y": 3000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    anchor_a_uuid.to_string(): {
                        "uuid": anchor_a_uuid,
                        "package": package_a_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 500000, "y": 600000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_b_uuid.to_string(): {
                        "uuid": anchor_b_uuid,
                        "package": package_b_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 2500000, "y": 1500000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    },
                    anchor_c_uuid.to_string(): {
                        "uuid": anchor_c_uuid,
                        "package": package_c_uuid,
                        "name": "1",
                        "net": target_net_uuid,
                        "position": { "x": 4500000, "y": 2400000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 450000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {
                    target_net_uuid.to_string(): {
                        "uuid": target_net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 150000,
                        "track_width": 200000,
                        "via_drill": 300000,
                        "via_diameter": 600000,
                        "diffpair_width": 0,
                        "diffpair_gap": 0
                    }
                },
                "keepouts": [{
                    "uuid": keepout_uuid,
                    "polygon": {
                        "vertices": [
                            { "x": 1200000, "y": 900000 },
                            { "x": 3800000, "y": 900000 },
                            { "x": 3800000, "y": 2100000 },
                            { "x": 1200000, "y": 2100000 }
                        ],
                        "closed": true
                    },
                    "layers": [1, 3],
                    "kind": "route"
                }],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let blocked_output = execute(route_path_candidate_explain_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
    ))
    .expect("blocked query should succeed");
    let blocked_report: serde_json::Value =
        serde_json::from_str(&blocked_output).expect("blocked report should parse");
    assert_eq!(
        blocked_report["explanation_kind"],
        "all_matching_spans_blocked"
    );
    assert_eq!(blocked_report["summary"]["matching_span_count"], 2);
    assert_eq!(blocked_report["summary"]["blocked_span_count"], 2);

    let no_match_output = execute(route_path_candidate_explain_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_c_uuid,
    ))
    .expect("no-match query should succeed");
    let no_match_report: serde_json::Value =
        serde_json::from_str(&no_match_output).expect("no-match report should parse");
    assert_eq!(
        no_match_report["explanation_kind"],
        "no_matching_corridor_span"
    );
    assert_eq!(no_match_report["summary"]["matching_span_count"], 0);
    assert_eq!(
        no_match_report["blocked_matching_spans"],
        serde_json::json!([])
    );

    let _ = std::fs::remove_dir_all(&root);
}
