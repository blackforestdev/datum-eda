use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_mechanical_layer_component_arcs_export_validate_and_compare() {
    let root = unique_project_root("datum-eda-cli-project-gerber-mech-component-arc");
    create_native_project(&root, Some("Gerber Mech Component Arc Demo".to_string()))
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
                "name": "Gerber Mech Component Arc Demo Board",
                "stackup": {
                    "layers": [
                        {"id": 41, "name": "Mechanical 1", "layer_type": "Mechanical", "thickness_nm": 0}
                    ]
                },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    component_uuid.to_string(): {
                        "uuid": component_uuid,
                        "part": Uuid::new_v4(),
                        "package": Uuid::new_v4(),
                        "reference": "M5",
                        "value": "BracketArc",
                        "position": {"x": 1000000, "y": 2000000},
                        "rotation": 0,
                        "layer": 41,
                        "locked": false
                    }
                },
                "component_mechanical_lines": {
                    component_uuid.to_string(): []
                },
                "component_mechanical_polygons": {
                    component_uuid.to_string(): []
                },
                "component_mechanical_polylines": {
                    component_uuid.to_string(): []
                },
                "component_mechanical_circles": {
                    component_uuid.to_string(): []
                },
                "component_mechanical_arcs": {
                    component_uuid.to_string(): [{
                        "center": {"x": 0, "y": 0},
                        "radius_nm": 500000,
                        "start_angle": 0,
                        "end_angle": 900,
                        "width_nm": 120000,
                        "layer": 41
                    }]
                },
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

    let gerber_path = root.join("mech-component-arc.gbr");
    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-gerber-mechanical-layer",
        root.to_str().unwrap(),
        "--layer",
        "41",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let export_output = execute(export_cli).expect("mechanical export should succeed");
    let export_report: serde_json::Value =
        serde_json::from_str(&export_output).expect("report JSON");
    assert_eq!(export_report["keepout_count"], 0);
    assert_eq!(export_report["component_arc_count"], 1);

    let gerber = std::fs::read_to_string(&gerber_path).expect("gerber should read");
    assert!(gerber.contains("%ADD11C,0.120000*%"));
    assert!(gerber.contains("D11*"));
    assert!(gerber.contains("X1500000Y2000000D02*"));
    assert!(gerber.contains("X1000000Y2500000D01*"));

    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-mechanical-layer",
        root.to_str().unwrap(),
        "--layer",
        "41",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let validate_output = execute(validate_cli).expect("mechanical validate should succeed");
    let validate_report: serde_json::Value =
        serde_json::from_str(&validate_output).expect("report JSON");
    assert_eq!(validate_report["matches_expected"], true);
    assert_eq!(validate_report["component_arc_count"], 1);

    let compare_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-gerber-mechanical-layer",
        root.to_str().unwrap(),
        "--layer",
        "41",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let compare_output = execute(compare_cli).expect("mechanical compare should succeed");
    let compare_report: serde_json::Value =
        serde_json::from_str(&compare_output).expect("report JSON");
    assert_eq!(compare_report["expected_component_arc_count"], 1);
    assert_eq!(compare_report["matched_count"], 6);
    assert_eq!(compare_report["missing_count"], 0);
    assert_eq!(compare_report["extra_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
