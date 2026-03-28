use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_report_manufacturing_summarizes_supported_outputs_from_persisted_state() {
    let root = unique_project_root("datum-eda-cli-project-manufacturing-report");
    create_native_project(&root, Some("Manufacturing Report Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let component_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Manufacturing Report Demo Board",
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
                        "package": Uuid::new_v4(),
                        "part": Uuid::new_v4(),
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
                        },
                        {
                            "uuid": Uuid::new_v4(),
                            "name": "P2",
                            "position": { "x": 2000000, "y": 2500000 },
                            "padstack": Uuid::new_v4(),
                            "layer": 1,
                            "drill_nm": null,
                            "shape": "rect",
                            "diameter_nm": 0,
                            "width_nm": 800000,
                            "height_nm": 500000
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
                    Uuid::new_v4().to_string(): {
                        "uuid": Uuid::new_v4(),
                        "net": Uuid::new_v4(),
                        "position": { "x": 500000, "y": 500000 },
                        "diameter": 700000,
                        "drill": 300000,
                        "from_layer": 1,
                        "to_layer": 31
                    }
                },
                "zones": {},
                "nets": {},
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

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "report-manufacturing",
        root.to_str().unwrap(),
        "--prefix",
        "Release A",
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("manufacturing report should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "report_manufacturing");
    assert_eq!(report["prefix"], "release-a");
    assert_eq!(report["bom_component_count"], 1);
    assert_eq!(report["pnp_component_count"], 1);
    assert_eq!(report["drill_csv_row_count"], 1);
    assert_eq!(report["excellon_via_count"], 1);
    assert_eq!(report["excellon_component_pad_count"], 1);
    assert_eq!(report["excellon_hit_count"], 2);
    assert_eq!(report["drill_hole_class_count"], 1);
    assert_eq!(report["gerber_artifact_count"], 7);
    let gerber_artifacts = report["gerber_artifacts"]
        .as_array()
        .expect("gerber artifact array");
    assert!(gerber_artifacts.iter().any(|artifact| artifact["filename"] == "release-a-outline.gbr"));
    assert!(gerber_artifacts.iter().any(|artifact| artifact["filename"] == "release-a-l1-top-copper-copper.gbr"));
    assert!(gerber_artifacts.iter().any(|artifact| artifact["filename"] == "release-a-l2-top-mask-mask.gbr"));
    assert!(gerber_artifacts.iter().any(|artifact| artifact["filename"] == "release-a-l3-top-silk-silk.gbr"));
    assert!(gerber_artifacts.iter().any(|artifact| artifact["filename"] == "release-a-l4-top-paste-paste.gbr"));
    assert!(gerber_artifacts.iter().any(|artifact| artifact["filename"] == "release-a-l31-bottom-copper-copper.gbr"));
    assert!(gerber_artifacts.iter().any(|artifact| artifact["filename"] == "release-a-l41-mechanical-41-mech.gbr"));

    let _ = std::fs::remove_dir_all(&root);
}
