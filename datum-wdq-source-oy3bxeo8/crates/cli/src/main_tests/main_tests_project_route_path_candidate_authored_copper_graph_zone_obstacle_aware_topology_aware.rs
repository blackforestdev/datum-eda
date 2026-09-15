use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn topology_aware_query_cli(
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
        "route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware",
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
fn project_query_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_is_deterministic()
 {
    let root = std::env::temp_dir().join(format!(
        "datum-eda-cli-route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware-{}",
        Uuid::new_v4()
    ));
    create_native_project(&root, Some("Topology Aware".to_string()))
        .expect("initial scaffold should succeed");

    let net_uuid = Uuid::new_v4();
    let class_uuid = Uuid::new_v4();
    let from_anchor_uuid = Uuid::new_v4();
    let to_anchor_uuid = Uuid::new_v4();
    let anchor_via_uuid = Uuid::from_u128(2);
    let lower_track_first_uuid = Uuid::from_u128(3);
    let lower_track_second_uuid = Uuid::from_u128(4);
    let board_json = root.join("board/board.json");

    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Topology Aware Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Inner", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 5000000, "y": 0 },
                        { "x": 5000000, "y": 5000000 },
                        { "x": 0, "y": 5000000 }
                    ],
                    "closed": true
                },
                "packages": {},
                "pads": {
                    from_anchor_uuid.to_string(): {
                        "uuid": from_anchor_uuid,
                        "package": Uuid::new_v4(),
                        "name": "1",
                        "net": net_uuid,
                        "position": { "x": 500000, "y": 500000 },
                        "layer": 1,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    },
                    to_anchor_uuid.to_string(): {
                        "uuid": to_anchor_uuid,
                        "package": Uuid::new_v4(),
                        "name": "2",
                        "net": net_uuid,
                        "position": { "x": 3500000, "y": 500000 },
                        "layer": 2,
                        "shape": "circle",
                        "diameter": 300000,
                        "width": 0,
                        "height": 0
                    }
                },
                "tracks": {
                    Uuid::from_u128(1).to_string(): {
                        "uuid": Uuid::from_u128(1),
                        "net": net_uuid,
                        "from": { "x": 500000, "y": 500000 },
                        "to": { "x": 1500000, "y": 500000 },
                        "width": 120000,
                        "layer": 1
                    },
                    lower_track_first_uuid.to_string(): {
                        "uuid": lower_track_first_uuid,
                        "net": net_uuid,
                        "from": { "x": 500000, "y": 500000 },
                        "to": { "x": 2000000, "y": 500000 },
                        "width": 120000,
                        "layer": 2
                    },
                    lower_track_second_uuid.to_string(): {
                        "uuid": lower_track_second_uuid,
                        "net": net_uuid,
                        "from": { "x": 2000000, "y": 500000 },
                        "to": { "x": 3500000, "y": 500000 },
                        "width": 120000,
                        "layer": 2
                    },
                    Uuid::from_u128(6).to_string(): {
                        "uuid": Uuid::from_u128(6),
                        "net": net_uuid,
                        "from": { "x": 1500000, "y": 500000 },
                        "to": { "x": 3500000, "y": 500000 },
                        "width": 120000,
                        "layer": 2
                    }
                },
                "vias": {
                    anchor_via_uuid.to_string(): {
                        "uuid": anchor_via_uuid,
                        "net": net_uuid,
                        "position": { "x": 500000, "y": 500000 },
                        "from_layer": 1,
                        "to_layer": 2,
                        "diameter": 300000,
                        "drill": 150000
                    },
                    Uuid::from_u128(5).to_string(): {
                        "uuid": Uuid::from_u128(5),
                        "net": net_uuid,
                        "position": { "x": 1500000, "y": 500000 },
                        "from_layer": 1,
                        "to_layer": 2,
                        "diameter": 300000,
                        "drill": 150000
                    }
                },
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "SIG",
                        "class": class_uuid
                    }
                },
                "net_classes": {
                    class_uuid.to_string(): {
                        "uuid": class_uuid,
                        "name": "Default",
                        "clearance": 100000,
                        "track_width": 120000,
                        "via_drill": 150000,
                        "via_diameter": 300000,
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

    let output = execute(topology_aware_query_cli(
        &root,
        net_uuid,
        from_anchor_uuid,
        to_anchor_uuid,
    ))
    .expect("query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(
        report["contract"],
        "m5_route_path_candidate_authored_copper_graph_zone_obstacle_aware_topology_aware_v1"
    );
    assert_eq!(report["status"], "deterministic_path_found");
    assert_eq!(report["summary"]["topology_transition_count"], 1);
    assert_eq!(report["summary"]["path_via_step_count"], 1);
    assert_eq!(
        report["path"]["steps"][0]["object_uuid"],
        anchor_via_uuid.to_string()
    );
    assert_eq!(
        report["path"]["steps"][1]["object_uuid"],
        lower_track_first_uuid.to_string()
    );
    assert_eq!(
        report["path"]["steps"][2]["object_uuid"],
        lower_track_second_uuid.to_string()
    );

    let repeated = execute(topology_aware_query_cli(
        &root,
        net_uuid,
        from_anchor_uuid,
        to_anchor_uuid,
    ))
    .expect("repeat query should succeed");
    assert_eq!(output, repeated);

    let text_output = execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "query",
            root.to_str().unwrap(),
            "route-path-candidate-authored-copper-graph-zone-obstacle-aware-topology-aware",
            "--net",
            &net_uuid.to_string(),
            "--from-anchor",
            &from_anchor_uuid.to_string(),
            "--to-anchor",
            &to_anchor_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("text query should succeed");
    assert!(text_output.contains("status: deterministic_path_found"));
    assert!(text_output.contains("topology_transitions: 1"));

    let _ = std::fs::remove_dir_all(&root);
}
