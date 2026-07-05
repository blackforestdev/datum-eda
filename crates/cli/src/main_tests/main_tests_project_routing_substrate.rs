use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn routing_substrate_query_cli(root: &Path) -> Cli {
    Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "routing-substrate",
    ])
    .expect("CLI should parse")
}

#[test]
fn project_query_routing_substrate_reads_persisted_native_board_state_only() {
    let root = unique_project_root("datum-eda-cli-project-routing-substrate");
    create_native_project(&root, Some("Routing Substrate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_uuid = Uuid::new_v4();
    let component_uuid = Uuid::new_v4();
    let board_pad_uuid = Uuid::new_v4();
    let component_pad_uuid = Uuid::new_v4();
    let padstack_uuid = Uuid::new_v4();
    let net_class_uuid = Uuid::new_v4();
    let net_uuid = Uuid::new_v4();
    let track_uuid = Uuid::new_v4();
    let via_uuid = Uuid::new_v4();
    let zone_uuid = Uuid::new_v4();
    let keepout_uuid = Uuid::new_v4();

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": board_uuid,
                "name": "Routing Substrate Demo Board",
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
                "component_pads": {
                    component_uuid.to_string(): [{
                        "uuid": component_pad_uuid,
                        "name": "1",
                        "position": { "x": 400000, "y": 500000 },
                        "padstack": padstack_uuid,
                        "layer": 1,
                        "drill_nm": 300000,
                        "shape": "circle",
                        "diameter_nm": 600000,
                        "width_nm": 0,
                        "height_nm": 0
                    }]
                },
                "component_models_3d": {},
                "pads": {
                    board_pad_uuid.to_string(): {
                        "uuid": board_pad_uuid,
                        "package": component_uuid,
                        "name": "TP1",
                        "net": net_uuid,
                        "position": { "x": 1000000, "y": 1100000 },
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
                        "net": net_uuid,
                        "from": { "x": 1000000, "y": 1100000 },
                        "to": { "x": 1600000, "y": 1100000 },
                        "width": 200000,
                        "layer": 1
                    }
                },
                "vias": {
                    via_uuid.to_string(): {
                        "uuid": via_uuid,
                        "net": net_uuid,
                        "position": { "x": 1600000, "y": 1100000 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 3
                    }
                },
                "zones": {
                    zone_uuid.to_string(): {
                        "uuid": zone_uuid,
                        "net": net_uuid,
                        "polygon": {
                            "vertices": [
                                { "x": 2000000, "y": 2000000 },
                                { "x": 2600000, "y": 2000000 },
                                { "x": 2600000, "y": 2600000 }
                            ],
                            "closed": true
                        },
                        "layer": 1,
                        "priority": 1,
                        "thermal_relief": true,
                        "thermal_gap": 150000,
                        "thermal_spoke_width": 120000
                    }
                },
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "SIG",
                        "class": net_class_uuid
                    }
                },
                "net_classes": {
                    net_class_uuid.to_string(): {
                        "uuid": net_class_uuid,
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
                            { "x": 3000000, "y": 300000 },
                            { "x": 3400000, "y": 300000 },
                            { "x": 3400000, "y": 700000 }
                        ],
                        "closed": true
                    },
                    "layers": [1],
                    "kind": "route"
                }],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output = execute(routing_substrate_query_cli(&root))
        .expect("routing substrate query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");

    assert_eq!(report["contract"], "m5_routing_substrate_v1");
    assert_eq!(report["persisted_native_board_state_only"], true);
    assert_eq!(report["summary"]["outline_vertex_count"], 4);
    assert_eq!(report["summary"]["layer_count"], 3);
    assert_eq!(report["summary"]["copper_layer_count"], 2);
    assert_eq!(report["summary"]["board_pad_count"], 1);
    assert_eq!(report["summary"]["component_pad_count"], 1);
    assert_eq!(report["summary"]["track_count"], 1);
    assert_eq!(report["summary"]["via_count"], 1);
    assert_eq!(report["summary"]["zone_count"], 1);
    assert_eq!(report["summary"]["net_count"], 1);
    assert_eq!(report["summary"]["net_class_count"], 1);
    assert_eq!(report["copper_layer_ids"], serde_json::json!([1, 3]));
    assert_eq!(report["pads"].as_array().unwrap().len(), 2);
    assert_eq!(report["pads"][0]["source"], "board_pad");
    assert_eq!(report["pads"][0]["net"], net_uuid.to_string());
    assert_eq!(report["pads"][1]["source"], "component_pad");
    assert_eq!(
        report["pads"][1]["padstack_uuid"],
        padstack_uuid.to_string()
    );
    assert_eq!(report["pads"][1]["drill_nm"], 300000);
    assert_eq!(report["nets"][0]["name"], "SIG");
    assert_eq!(report["net_classes"][0]["track_width"], 200000);

    let repeated = execute(routing_substrate_query_cli(&root))
        .expect("repeated routing substrate query should succeed");
    assert_eq!(output, repeated);

    let text_output = execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "query",
            root.to_str().unwrap(),
            "routing-substrate",
        ])
        .expect("CLI should parse"),
    )
    .expect("text routing substrate query should succeed");
    assert!(text_output.contains("contract: m5_routing_substrate_v1"));
    assert!(text_output.contains("persisted_native_board_state_only: true"));
    assert!(text_output.contains("board_pads: 1"));
    assert!(text_output.contains("component_pads: 1"));
    assert!(text_output.contains("copper_layer_ids: 1,3"));

    let _ = std::fs::remove_dir_all(&root);
}
