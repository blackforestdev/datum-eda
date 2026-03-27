use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_validate_gerber_mechanical_layer_reports_match_and_mismatch() {
    let root = unique_project_root("datum-eda-cli-project-gerber-mech-validate");
    create_native_project(&root, Some("Gerber Mech Validate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let keepout_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Mech Validate Demo Board",
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
                "keepouts": [{
                    "uuid": keepout_uuid,
                    "polygon": {
                        "vertices": [
                            { "x": 0, "y": 0 },
                            { "x": 1000000, "y": 0 },
                            { "x": 1000000, "y": 500000 }
                        ],
                        "closed": true
                    },
                    "layers": [41],
                    "kind": "mechanical"
                }],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let gerber_path = root.join("mech1.gbr");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-gerber-mechanical-layer",
        root.to_str().unwrap(),
        "--layer",
        "41",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("export CLI should parse");
    let _ = execute(export_cli).expect("gerber mechanical export should succeed");

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
    assert_eq!(report["matches_expected"], true);
    assert_eq!(report["keepout_count"], 1);

    std::fs::write(&gerber_path, "corrupted\n").expect("gerber overwrite should succeed");
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
    assert_eq!(exit_code, 1);
    assert_eq!(report["matches_expected"], false);
    assert_eq!(report["keepout_count"], 1);

    let _ = std::fs::remove_dir_all(&root);
}
