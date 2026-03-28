use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_mechanical_gerber_supports_board_text_strokes() {
    let root = unique_project_root("datum-eda-cli-project-gerber-mech-text");
    create_native_project(&root, Some("Gerber Mech Text Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Mech Text Demo Board",
                "stackup": {
                    "layers": [
                        {"id": 41, "name": "Mechanical 1", "layer_type": "Mechanical", "thickness_nm": 0}
                    ]
                },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
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
                    "uuid": Uuid::new_v4(),
                    "text": "L",
                    "position": { "x": 200000, "y": 400000 },
                    "rotation": 0,
                    "layer": 41,
                    "height_nm": 1000000,
                    "stroke_width_nm": 100000
                }]
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let gerber_path = root.join("mech-text.gbr");
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
    .expect("export CLI should parse");
    let output = execute(export_cli).expect("mechanical export should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["board_text_count"], 1);

    let gerber = std::fs::read_to_string(&gerber_path).expect("gerber should read");
    assert!(gerber.contains("X200000Y1200000D02*"));
    assert!(gerber.contains("X200000Y400000D01*"));
    assert!(gerber.contains("X200000Y400000D02*"));
    assert!(gerber.contains("X600000Y400000D01*"));

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
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 0);
    assert_eq!(report["board_text_count"], 1);
    assert_eq!(report["matches_expected"], true);

    std::fs::write(
        &gerber_path,
        concat!(
            "G04 semantic mech text fixture*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD10C,0.100000*%\n",
            "D10*\n",
            "X200000Y1200000D02*\n",
            "X200000Y400000D01*\n",
            "X200000Y400000D02*\n",
            "X600000Y400000D01*\n",
            "M02*\n"
        ),
    )
    .expect("compare gerber should write");

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
    .expect("compare CLI should parse");
    let output = execute(compare_cli).expect("compare should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(report["expected_board_text_count"], 1);
    assert_eq!(report["matched_count"], 2);
    assert_eq!(report["missing_count"], 0);
    assert_eq!(report["extra_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
