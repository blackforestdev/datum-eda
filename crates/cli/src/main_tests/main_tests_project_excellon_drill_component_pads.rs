use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_excellon_drill_and_hole_classes_include_component_pad_holes() {
    let root = unique_project_root("datum-eda-cli-project-excellon-drill-component-pads");
    create_native_project(&root, Some("Excellon Component Pads Demo".to_string()))
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
                "name": "Excellon Component Pads Demo Board",
                "stackup": {
                    "layers": [
                        { "id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000 },
                        { "id": 31, "name": "Bottom Copper", "layer_type": "Copper", "thickness_nm": 35000 }
                    ]
                },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
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
                            "position": { "x": 2000000, "y": 3000000 },
                            "padstack": Uuid::new_v4(),
                            "layer": 1,
                            "drill_nm": 350000,
                            "shape": null,
                            "diameter_nm": 0,
                            "width_nm": 0,
                            "height_nm": 0
                        },
                        {
                            "uuid": Uuid::new_v4(),
                            "name": "P3",
                            "position": { "x": 3000000, "y": 4500000 },
                            "padstack": Uuid::new_v4(),
                            "layer": 1,
                            "drill_nm": null,
                            "shape": "circle",
                            "diameter_nm": 500000,
                            "width_nm": 0,
                            "height_nm": 0
                        }
                    ]
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

    let drill_path = root.join("component-pads.drl");
    let export_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "export-excellon-drill",
        root.to_str().unwrap(),
        "--out",
        drill_path.to_str().unwrap(),
    ])
    .expect("export CLI should parse");
    let output = execute(export_cli).expect("excellon export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "export_excellon_drill");
    assert_eq!(report["via_count"], 0);
    assert_eq!(report["component_pad_count"], 2);
    assert_eq!(report["hit_count"], 2);
    assert_eq!(report["tool_count"], 2);
    assert_eq!(report["tools"][0]["diameter_mm"], "0.300000");
    assert_eq!(report["tools"][1]["diameter_mm"], "0.350000");

    let drill = std::fs::read_to_string(&drill_path).expect("drill should read");
    assert!(drill.contains("T01C0.300000"));
    assert!(drill.contains("T02C0.350000"));
    assert!(drill.contains("T01\nX1.000000Y1.500000"));
    assert!(drill.contains("T02\nX2.000000Y3.000000"));

    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-excellon-drill",
        root.to_str().unwrap(),
        "--drill",
        drill_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 0);
    assert_eq!(report["matches_expected"], true);
    assert_eq!(report["via_count"], 0);
    assert_eq!(report["component_pad_count"], 2);
    assert_eq!(report["hit_count"], 2);

    let compare_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "compare-excellon-drill",
        root.to_str().unwrap(),
        "--drill",
        drill_path.to_str().unwrap(),
    ])
    .expect("compare CLI should parse");
    let output = execute(compare_cli).expect("compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["expected_hit_count"], 2);
    assert_eq!(report["actual_hit_count"], 2);
    assert_eq!(report["matched_count"], 2);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 0);
    assert_eq!(report["hit_drift_count"], 0);

    let classes_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "report-drill-hole-classes",
        root.to_str().unwrap(),
    ])
    .expect("classes CLI should parse");
    let output = execute(classes_cli).expect("hole class report should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["via_count"], 0);
    assert_eq!(report["component_pad_count"], 2);
    assert_eq!(report["hit_count"], 2);
    assert_eq!(report["class_count"], 1);
    assert_eq!(report["classes"][0]["class"], "through");
    assert_eq!(report["classes"][0]["via_count"], 0);
    assert_eq!(report["classes"][0]["component_pad_count"], 2);
    assert_eq!(report["classes"][0]["hit_count"], 2);
    assert_eq!(report["classes"][0]["tool_count"], 2);

    let _ = std::fs::remove_dir_all(&root);
}
