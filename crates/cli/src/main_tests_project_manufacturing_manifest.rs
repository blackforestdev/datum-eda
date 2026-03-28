use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_manifest_manufacturing_set_reports_deterministic_expected_artifacts() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-manifest");
    create_native_project(&root, Some("Manufacturing Manifest Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let component_uuid = Uuid::new_v4();
    let package_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    let via_uuid = Uuid::new_v4();
    let net_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Manufacturing Manifest Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 2, "name": "Top Mask", "layer_type": "SolderMask", "thickness_nm": 10000 },
                        { "id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000 },
                        { "id": 4, "name": "Top Paste", "layer_type": "Paste", "thickness_nm": 10000 },
                        { "id": 31, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 41, "name": "Mechanical 41", "layer_type": "Mechanical", "thickness_nm": 0 }
                    ]
                },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 1000, "y": 0 },
                        { "x": 1000, "y": 500 },
                        { "x": 0, "y": 500 }
                    ],
                    "closed": true
                },
                "packages": {
                    component_uuid.to_string(): {
                        "uuid": component_uuid,
                        "reference": "U1",
                        "value": "MCU",
                        "package": package_uuid,
                        "part": part_uuid,
                        "position": { "x": 1000000, "y": 2000000 },
                        "rotation": 0,
                        "layer": 1,
                        "locked": false
                    }
                },
                "component_pads": {
                    component_uuid.to_string(): [
                        {
                            "uuid": Uuid::new_v4(),
                            "name": "P1",
                            "position": { "x": 1000000, "y": 1500000 },
                            "padstack": Uuid::new_v4(),
                            "layer": 1,
                            "drill_nm": 300000,
                            "shape": "circle",
                            "diameter_nm": 600000,
                            "width_nm": 0,
                            "height_nm": 0
                        }
                    ]
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
                "component_models_3d": {},
                "pads": {},
                "tracks": {},
                "vias": {
                    via_uuid.to_string(): {
                        "uuid": via_uuid,
                        "net": net_uuid,
                        "position": { "x": 500000, "y": 500000 },
                        "diameter": 700000,
                        "drill": 300000,
                        "from_layer": 1,
                        "to_layer": 31
                    }
                },
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "N$1",
                        "class": null
                    }
                },
                "net_classes": {},
                "keepouts": [],
                "dimensions": [],
                "texts": [{
                    "uuid": Uuid::new_v4(),
                    "text": "TOP",
                    "position": { "x": 1000000, "y": 2000000 },
                    "rotation": 0,
                    "height_nm": 1000000,
                    "stroke_width_nm": 120000,
                    "layer": 3
                }]
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let output_dir = root.join("manufacturing");
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "manifest-manufacturing-set",
            root.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--prefix",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("manifest should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "manifest_manufacturing_set");
    assert_eq!(report["prefix"], "release-a");
    assert_eq!(report["expected_count"], 11);
    let entries = report["entries"].as_array().expect("entries should be an array");
    assert_eq!(entries.len(), 11);
    assert_eq!(entries[0]["kind"], "bom");
    assert_eq!(entries[0]["contract"], "semantic");
    assert_eq!(entries[0]["filename"], "release-a-bom.csv");
    assert_eq!(entries[2]["kind"], "drill_csv");
    assert_eq!(entries[2]["contract"], "strict");
    assert_eq!(entries[3]["kind"], "excellon_drill");
    assert_eq!(entries[3]["contract"], "semantic");
    assert_eq!(entries[4]["kind"], "gerber_outline");
    assert_eq!(entries[10]["filename"], "release-a-l41-mechanical-41-mech.gbr");

    let _ = std::fs::remove_dir_all(&root);
}
