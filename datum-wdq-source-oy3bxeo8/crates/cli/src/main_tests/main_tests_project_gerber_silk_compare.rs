use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_compare_gerber_silkscreen_layer_is_semantic_and_reports_drift() {
    let root = unique_project_root("datum-eda-cli-project-gerber-silk-compare");
    create_native_project(&root, Some("Gerber Silk Compare Demo".to_string()))
        .expect("initial scaffold should succeed");

    let text_uuid = Uuid::new_v4();
    let component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Silk Compare Demo Board",
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
                "component_silkscreen": {
                    component_uuid.to_string(): [{
                        "from": { "x": 0, "y": 0 },
                        "to": { "x": 1000000, "y": 0 },
                        "width_nm": 150000,
                        "layer": 3
                    }]
                },
                "component_silkscreen_texts": {
                    component_uuid.to_string(): [{
                        "text": "U",
                        "position": { "x": 500000, "y": 0 },
                        "rotation": 0,
                        "height_nm": 1000000,
                        "stroke_width_nm": 150000,
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
                "texts": [{
                    "uuid": text_uuid,
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

    let gerber_path = root.join("top-silk.gbr");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-gerber-silkscreen-layer",
        root.to_str().unwrap(),
        "--layer",
        "3",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("export CLI should parse");
    let _ = execute(export_cli).expect("gerber silkscreen export should succeed");

    let cli = Cli::try_parse_from([
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
    .expect("CLI should parse");
    let output = execute(cli).expect("silkscreen compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["action"], "compare_gerber_silkscreen_layer");
    assert_eq!(report["expected_text_count"], 1);
    assert_eq!(report["expected_component_text_count"], 1);
    assert_eq!(report["expected_component_stroke_count"], 1);
    assert!(report["actual_geometry_count"].as_u64().unwrap() > 0);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 0);

    std::fs::write(
        &gerber_path,
        concat!(
            "G04 silk drift fixture*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD10C,0.120000*%\n",
            "D10*\n",
            "X1000000Y2000000D02*\n",
            "X1000000Y3000000D01*\n",
            "M02*\n"
        ),
    )
    .expect("drift gerber should write");

    let cli = Cli::try_parse_from([
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
    .expect("CLI should parse");
    let output = execute(cli).expect("silkscreen compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert!(report["missing_count"].as_u64().unwrap() > 0);
    assert!(report["extra_count"].as_u64().unwrap() > 0);

    let _ = std::fs::remove_dir_all(&root);
}
