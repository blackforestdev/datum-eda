use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn route_corridor_query_cli(root: &Path, net_uuid: Uuid) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "route-corridor",
        "--net",
        &net_uuid.to_string(),
    ])
    .expect("CLI should parse")
}

#[test]
fn project_query_route_corridor_reports_persisted_single_net_geometry_deterministically() {
    let root = unique_project_root("datum-eda-cli-project-route-corridor");
    create_native_project(&root, Some("Route Corridor Demo".to_string()))
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
                "name": "Route Corridor Demo Board",
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

    let output =
        execute(route_corridor_query_cli(&root, target_net_uuid)).expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(report["contract"], "m5_route_corridor_v1");
    assert_eq!(report["persisted_native_board_state_only"], true);
    assert_eq!(report["status"], "corridor_available");
    assert_eq!(report["net_uuid"], target_net_uuid.to_string());
    assert_eq!(report["net_name"], "SIG");
    assert_eq!(report["anchors"].as_array().unwrap().len(), 2);
    assert_eq!(
        report["candidate_copper_layers"].as_array().unwrap().len(),
        2
    );
    assert_eq!(report["summary"]["anchor_pair_count"], 1);
    assert_eq!(report["summary"]["obstacle_count"], 0);
    assert_eq!(report["summary"]["span_count"], 2);
    assert_eq!(report["summary"]["available_span_count"], 2);
    assert_eq!(report["summary"]["blocked_span_count"], 0);

    let repeated =
        execute(route_corridor_query_cli(&root, target_net_uuid)).expect("repeat should succeed");
    assert_eq!(output, repeated);

    let text_output = execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "query",
            root.to_str().unwrap(),
            "route-corridor",
            "--net",
            &target_net_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("text query should succeed");
    assert!(text_output.contains("contract: m5_route_corridor_v1"));
    assert!(text_output.contains("status: corridor_available"));
    assert!(text_output.contains("corridor_spans: 2"));
    assert!(text_output.contains("blocked_spans: 0"));

    let _ = std::fs::remove_dir_all(&root);
}
