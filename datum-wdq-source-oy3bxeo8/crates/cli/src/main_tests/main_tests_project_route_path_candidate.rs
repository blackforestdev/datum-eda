use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn route_path_candidate_query_cli(
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
        "route-path-candidate",
        "--net",
        &net_uuid.to_string(),
        "--from-anchor",
        &from_anchor.to_string(),
        "--to-anchor",
        &to_anchor.to_string(),
    ])
    .expect("CLI should parse")
}

fn route_path_candidate_explain_cli(
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
fn project_query_route_path_candidate_reports_deterministic_single_layer_path() {
    let root = unique_project_root("datum-eda-cli-project-route-path-candidate");
    create_native_project(&root, Some("Route Path Candidate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::new_v4();
    let other_net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let package_a_uuid = Uuid::new_v4();
    let package_b_uuid = Uuid::new_v4();
    let anchor_a_uuid = Uuid::new_v4();
    let anchor_b_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Route Path Candidate Demo Board",
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
                    },
                    other_net_uuid.to_string(): {
                        "uuid": other_net_uuid,
                        "name": "GND",
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

    let output = execute(route_path_candidate_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(report["contract"], "m5_route_path_candidate_v2");
    assert_eq!(report["persisted_native_board_state_only"], true);
    assert_eq!(
        report["selection_rule"],
        "select the first unblocked matching corridor span in corridor report order (sorted by candidate copper layer order, then pair index)"
    );
    assert_eq!(report["status"], "deterministic_path_found");
    assert_eq!(report["net_uuid"], target_net_uuid.to_string());
    assert_eq!(report["from_anchor_pad_uuid"], anchor_a_uuid.to_string());
    assert_eq!(report["to_anchor_pad_uuid"], anchor_b_uuid.to_string());
    assert_eq!(report["summary"]["matching_span_count"], 2);
    assert_eq!(report["summary"]["available_span_count"], 2);
    assert_eq!(report["path"]["layer"], 1);
    assert_eq!(report["path"]["points"].as_array().unwrap().len(), 2);

    let repeated = execute(route_path_candidate_query_cli(
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
            "route-path-candidate",
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
    assert!(text_output.contains("contract: m5_route_path_candidate_v2"));
    assert!(text_output.contains("selection_rule: select the first unblocked matching corridor span in corridor report order (sorted by candidate copper layer order, then pair index)"));
    assert!(text_output.contains("status: deterministic_path_found"));
    assert!(text_output.contains("path_layer: 1"));
    assert!(text_output.contains("path_points: 2"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_route_path_candidate_reads_resolver_materialized_journal_state() {
    let root = unique_project_root("datum-eda-cli-project-route-path-candidate-resolved");
    create_native_project(
        &root,
        Some("Route Path Candidate Resolved Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let board_json = root.join("board/board.json");
    let stale_board = std::fs::read_to_string(&board_json).expect("board file should read");
    let package_a_uuid = Uuid::from_u128(0x8a00);
    let package_b_uuid = Uuid::from_u128(0x8a01);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-board-stackup",
            root.to_str().unwrap(),
            "--layer",
            "1:Top:Copper:35000",
            "--layer",
            "2:Core:Dielectric:1600000",
            "--layer",
            "3:Bottom:Copper:35000",
        ])
        .expect("CLI should parse"),
    )
    .expect("set board stackup should succeed");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-board-outline",
            root.to_str().unwrap(),
            "--vertex",
            "0:0",
            "--vertex",
            "5000000:0",
            "--vertex",
            "5000000:3000000",
            "--vertex",
            "0:3000000",
        ])
        .expect("CLI should parse"),
    )
    .expect("set board outline should succeed");

    let class_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-net-class",
            root.to_str().unwrap(),
            "--name",
            "Default",
            "--clearance-nm",
            "150000",
            "--track-width-nm",
            "200000",
            "--via-drill-nm",
            "300000",
            "--via-diameter-nm",
            "600000",
        ])
        .expect("CLI should parse"),
    )
    .expect("place board net class should succeed");
    let class_report: serde_json::Value =
        serde_json::from_str(&class_output).expect("class output should parse");

    let net_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-net",
            root.to_str().unwrap(),
            "--name",
            "SIG",
            "--class",
            class_report["net_class_uuid"].as_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("place board net should succeed");
    let net_report: serde_json::Value =
        serde_json::from_str(&net_output).expect("net output should parse");
    let net_uuid = net_report["net_uuid"].as_str().unwrap().to_string();

    let anchor_a_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-pad",
            root.to_str().unwrap(),
            "--package",
            &package_a_uuid.to_string(),
            "--name",
            "1",
            "--x-nm",
            "500000",
            "--y-nm",
            "600000",
            "--layer",
            "1",
            "--diameter-nm",
            "450000",
            "--net",
            &net_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("place first board pad should succeed");
    let anchor_a_report: serde_json::Value =
        serde_json::from_str(&anchor_a_output).expect("first pad output should parse");
    let anchor_a_uuid = anchor_a_report["pad_uuid"].as_str().unwrap().to_string();

    let anchor_b_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-board-pad",
            root.to_str().unwrap(),
            "--package",
            &package_b_uuid.to_string(),
            "--name",
            "1",
            "--x-nm",
            "4500000",
            "--y-nm",
            "2400000",
            "--layer",
            "1",
            "--diameter-nm",
            "450000",
            "--net",
            &net_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("place second board pad should succeed");
    let anchor_b_report: serde_json::Value =
        serde_json::from_str(&anchor_b_output).expect("second pad output should parse");
    let anchor_b_uuid = anchor_b_report["pad_uuid"].as_str().unwrap().to_string();

    std::fs::write(&board_json, stale_board).expect("stale board file should restore");

    let output = execute(route_path_candidate_query_cli(
        &root,
        Uuid::parse_str(&net_uuid).expect("net UUID should parse"),
        Uuid::parse_str(&anchor_a_uuid).expect("first pad UUID should parse"),
        Uuid::parse_str(&anchor_b_uuid).expect("second pad UUID should parse"),
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");
    assert_eq!(report["status"], "deterministic_path_found");
    assert_eq!(report["summary"]["matching_span_count"], 2);
    assert_eq!(report["path"]["layer"], 1);

    let preflight_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "route-preflight",
            "--net",
            &net_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("preflight query should succeed");
    let preflight: serde_json::Value =
        serde_json::from_str(&preflight_output).expect("preflight should parse");
    assert_eq!(preflight["status"], "preflight_ready");
    assert_eq!(preflight["summary"]["anchor_count"], 2);

    let corridor_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "route-corridor",
            "--net",
            &net_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("corridor query should succeed");
    let corridor: serde_json::Value =
        serde_json::from_str(&corridor_output).expect("corridor should parse");
    assert_eq!(corridor["status"], "corridor_available");
    assert_eq!(corridor["summary"]["available_span_count"], 2);

    let explain_output = execute(route_path_candidate_explain_cli(
        &root,
        Uuid::parse_str(&net_uuid).expect("net UUID should parse"),
        Uuid::parse_str(&anchor_a_uuid).expect("first pad UUID should parse"),
        Uuid::parse_str(&anchor_b_uuid).expect("second pad UUID should parse"),
    ))
    .expect("explain query should succeed");
    let explain: serde_json::Value =
        serde_json::from_str(&explain_output).expect("explain should parse");
    assert_eq!(explain["status"], "deterministic_path_found");
    assert_eq!(explain["selected_span"]["layer"], 1);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_route_path_candidate_reports_no_path_for_non_matching_anchor_pair() {
    let root = unique_project_root("datum-eda-cli-project-route-path-candidate-no-match");
    create_native_project(&root, Some("Route Path Candidate No Match".to_string()))
        .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0x810);
    let class_uuid = Uuid::from_u128(0x811);
    let package_a_uuid = Uuid::from_u128(0x812);
    let package_b_uuid = Uuid::from_u128(0x813);
    let package_c_uuid = Uuid::from_u128(0x814);
    let anchor_a_uuid = Uuid::from_u128(0x815);
    let anchor_b_uuid = Uuid::from_u128(0x816);
    let anchor_c_uuid = Uuid::from_u128(0x817);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x818),
                "name": "Route Path Candidate No Match Board",
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
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(route_path_candidate_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_c_uuid,
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(
        report["status"],
        "no_path_under_current_authored_constraints"
    );
    assert_eq!(report["summary"]["matching_span_count"], 0);
    assert!(report["path"].is_null());

    let _ = std::fs::remove_dir_all(&root);
}
