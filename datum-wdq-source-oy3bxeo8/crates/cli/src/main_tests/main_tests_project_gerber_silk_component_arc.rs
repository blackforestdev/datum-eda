use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_gerber_silkscreen_layer_supports_component_arcs() {
    let root = unique_project_root("datum-eda-cli-project-gerber-silk-component-arc");
    create_native_project(&root, Some("Gerber Silk Arc Demo".to_string()))
        .expect("initial scaffold should succeed");

    let component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Silk Arc Demo Board",
                "stackup": {
                    "layers": [
                        {"id": 3, "name": "Top Silk", "layer_type": "Silkscreen", "thickness_nm": 10000}
                    ]
                },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    component_uuid.to_string(): {
                        "uuid": component_uuid,
                        "part": Uuid::new_v4(),
                        "package": Uuid::new_v4(),
                        "reference": "U1",
                        "value": "DRV",
                        "position": { "x": 3000000, "y": 4000000 },
                        "rotation": 90,
                        "layer": 3,
                        "locked": false
                    }
                },
                "component_silkscreen": {},
                "component_silkscreen_texts": {},
                "component_silkscreen_arcs": {
                    component_uuid.to_string(): [{
                        "center": { "x": 0, "y": 0 },
                        "radius_nm": 1000000,
                        "start_angle": 0,
                        "end_angle": 900,
                        "width_nm": 150000,
                        "layer": 3
                    }]
                },
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let gerber_path = root.join("top-silk.gbr");
    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-silkscreen-layer",
        root.to_str().unwrap(),
        "--layer",
        "3",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("export CLI should parse");
    let export_output = execute(export_cli).expect("gerber silkscreen export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("export report JSON");
    assert_eq!(export_report["text_count"], 0);
    assert_eq!(export_report["component_text_count"], 0);
    assert_eq!(export_report["component_stroke_count"], 0);
    assert_eq!(export_report["component_arc_count"], 1);

    let gerber = std::fs::read_to_string(&gerber_path).expect("gerber should read");
    assert!(gerber.contains("%ADD10C,0.150000*%"));
    assert!(gerber.contains("X3000000Y5000000D02*"));
    assert!(gerber.contains("X2000000Y4000000D01*"));

    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-silkscreen-layer",
        root.to_str().unwrap(),
        "--layer",
        "3",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let validate_output = execute(validate_cli).expect("validation should succeed");
    let validate_report: serde_json::Value =
        serde_json::from_str(&validate_output).expect("validate report JSON");
    assert_eq!(validate_report["matches_expected"], true);
    assert_eq!(validate_report["component_arc_count"], 1);

    let compare_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-silkscreen-layer",
        root.to_str().unwrap(),
        "--layer",
        "3",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("compare CLI should parse");
    let compare_output = execute(compare_cli).expect("compare should succeed");
    let compare_report: serde_json::Value =
        serde_json::from_str(&compare_output).expect("compare report JSON");
    assert_eq!(compare_report["expected_component_arc_count"], 1);
    assert_eq!(compare_report["missing_count"], 0);
    assert_eq!(compare_report["extra_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
