use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_inspect_manufacturing_set_reports_present_missing_and_extra_files() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-inspect");
    create_native_project(&root, Some("Manufacturing Inspect Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let component_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Manufacturing Inspect Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Top Mask", "layer_type": "SolderMask", "thickness_nm": 10000 },
                        { "id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000 },
                        { "id": 4, "name": "Top Paste", "layer_type": "Paste", "thickness_nm": 10000 },
                        { "id": 41, "name": "Mechanical 41", "layer_type": "Mechanical", "thickness_nm": 0 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 1000000, "y": 0 },
                        { "x": 1000000, "y": 500000 }
                    ],
                    "closed": true
                },
                "packages": {
                    component_uuid.to_string(): {
                        "uuid": component_uuid,
                        "part": part_uuid,
                        "package": package_uuid,
                        "reference": "U1",
                        "value": "MCU",
                        "position": { "x": 1000, "y": 2000 },
                        "rotation": 90,
                        "layer": 1,
                        "locked": false
                    }
                },
                "component_silkscreen": {},
                "component_silkscreen_texts": {},
                "component_silkscreen_arcs": {},
                "component_silkscreen_circles": {},
                "component_silkscreen_polygons": {},
                "component_silkscreen_polylines": {},
                "component_mechanical_lines": {},
                "component_mechanical_texts": {},
                "component_mechanical_polygons": {},
                "component_mechanical_polylines": {},
                "component_mechanical_circles": {},
                "component_mechanical_arcs": {},
                "component_pads": {},
                "component_models_3d": {},
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output_dir = root.join("out");
    std::fs::create_dir_all(&output_dir).expect("output dir should exist");
    std::fs::write(
        output_dir.join("manufacturing-inspect-demo-board-bom.csv"),
        "stub\n",
    )
    .expect("bom should write");
    std::fs::write(
        output_dir.join("manufacturing-inspect-demo-board-drill.csv"),
        "stub\n",
    )
    .expect("drill csv should write");
    std::fs::write(output_dir.join("unexpected.txt"), "extra\n").expect("extra should write");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("inspect should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "inspect_manufacturing_set");
    assert_eq!(
        report["expected_count"],
        report["entries"].as_array().unwrap().len()
    );
    assert_eq!(report["present_count"], 2);
    assert_eq!(report["missing_count"].as_u64().unwrap() > 0, true);
    assert_eq!(report["extra_count"], 1);
    assert_eq!(
        report["entries"][0]["filename"],
        "manufacturing-inspect-demo-board-bom.csv"
    );
    assert_eq!(report["entries"][0]["present"], true);
    assert_eq!(report["extra"][0], "unexpected.txt");

    let _ = std::fs::remove_dir_all(&root);
}
