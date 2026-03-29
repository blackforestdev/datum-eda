use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn route_path_candidate_authored_copper_graph_query_cli(
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
        "route-path-candidate-authored-copper-graph",
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
fn project_query_route_path_candidate_authored_copper_graph_reports_deterministic_existing_copper_path() {
    let root =
        unique_project_root("datum-eda-cli-project-route-path-candidate-authored-copper-graph");
    create_native_project(
        &root,
        Some("Route Path Candidate Authored Copper Graph Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let target_net_uuid = Uuid::from_u128(0x3250);
    let class_uuid = Uuid::from_u128(0x3251);
    let package_a_uuid = Uuid::from_u128(0x3252);
    let package_b_uuid = Uuid::from_u128(0x3253);
    let anchor_a_uuid = Uuid::from_u128(0x3254);
    let anchor_b_uuid = Uuid::from_u128(0x3255);
    let track_uuid = Uuid::from_u128(0x3256);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::from_u128(0x3257),
                "name": "Route Path Candidate Authored Copper Graph Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 }
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
                "tracks": {
                    track_uuid.to_string(): {
                        "uuid": track_uuid,
                        "net": target_net_uuid,
                        "from": { "x": 500000, "y": 600000 },
                        "to": { "x": 4500000, "y": 2400000 },
                        "width": 200000,
                        "layer": 1
                    }
                },
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

    let output = execute(route_path_candidate_authored_copper_graph_query_cli(
        &root,
        target_net_uuid,
        anchor_a_uuid,
        anchor_b_uuid,
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(
        report["contract"],
        "m5_route_path_candidate_authored_copper_graph_v1"
    );
    assert_eq!(report["persisted_native_board_state_only"], true);
    assert_eq!(report["status"], "deterministic_path_found");
    assert_eq!(report["summary"]["candidate_track_count"], 1);
    assert_eq!(report["summary"]["candidate_via_count"], 0);
    assert_eq!(report["path"]["steps"].as_array().unwrap().len(), 1);
    assert_eq!(report["path"]["steps"][0]["object_uuid"], track_uuid.to_string());

    let repeated = execute(route_path_candidate_authored_copper_graph_query_cli(
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
            "route-path-candidate-authored-copper-graph",
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
    assert!(text_output.contains("contract: m5_route_path_candidate_authored_copper_graph_v1"));
    assert!(text_output.contains("status: deterministic_path_found"));
    assert!(text_output.contains("path_steps: 1"));

    let _ = std::fs::remove_dir_all(&root);
}
