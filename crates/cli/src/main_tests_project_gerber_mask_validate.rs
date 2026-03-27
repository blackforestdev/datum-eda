use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_validate_gerber_soldermask_layer_reports_match_and_mismatch() {
    let root = unique_project_root("datum-eda-cli-project-gerber-mask-validate");
    create_native_project(&root, Some("Gerber Mask Validate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let pad_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Mask Validate Demo Board",
                "stackup": {
                    "layers": [
                        {"id": 1, "name": "Top Copper", "layer_type": "Copper", "thickness_nm": 35000},
                        {"id": 2, "name": "Top Mask", "layer_type": "SolderMask", "thickness_nm": 10000}
                    ]
                },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {
                    pad_uuid.to_string(): {
                        "uuid": pad_uuid,
                        "package": Uuid::new_v4(),
                        "name": "1",
                        "net": null,
                        "position": { "x": 750000, "y": 250000 },
                        "layer": 1,
                        "diameter": 450000
                    }
                },
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

    let gerber_path = root.join("top-mask.gbr");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-gerber-soldermask-layer",
        root.to_str().unwrap(),
        "--layer",
        "2",
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("export CLI should parse");
    let _ = execute(export_cli).expect("gerber mask export should succeed");

    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-soldermask-layer",
        root.to_str().unwrap(),
        "--layer",
        "2",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 0);
    assert_eq!(report["matches_expected"], true);
    assert_eq!(report["source_copper_layer"], 1);
    assert_eq!(report["pad_count"], 1);

    std::fs::write(&gerber_path, "corrupted\n").expect("gerber overwrite should succeed");
    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-soldermask-layer",
        root.to_str().unwrap(),
        "--layer",
        "2",
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 1);
    assert_eq!(report["matches_expected"], false);
    assert_eq!(report["pad_count"], 1);

    let _ = std::fs::remove_dir_all(&root);
}
