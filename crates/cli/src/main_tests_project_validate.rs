use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_validate_reports_clean_native_project() {
    let root = unique_project_root("datum-eda-cli-project-validate-clean");
    create_native_project(&root, Some("Validate Demo".to_string()))
        .expect("native project scaffold should succeed");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let (output, exit_code) = execute_with_exit_code(cli).expect("project validate should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("validation JSON should parse");
    assert_eq!(exit_code, 0);
    assert_eq!(report["action"], "validate_project");
    assert_eq!(report["project_root"], root.display().to_string());
    assert_eq!(report["valid"], true);
    assert_eq!(report["schema_compatible"], true);
    assert_eq!(report["required_files_expected"], 4);
    assert_eq!(report["required_files_validated"], 4);
    assert_eq!(report["checked_sheet_files"], 0);
    assert_eq!(report["checked_definition_files"], 0);
    assert_eq!(report["issue_count"], 0);
    assert_eq!(report["issues"], serde_json::json!([]));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_validate_reports_dangling_duplicate_and_missing_native_references() {
    let root = unique_project_root("datum-eda-cli-project-validate-invalid");
    create_native_project(&root, Some("Validate Invalid".to_string()))
        .expect("native project scaffold should succeed");

    let board_json = root.join("board/board.json");
    let mut board_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&board_json).expect("board.json should read"),
    )
    .expect("board.json should parse");

    let package_uuid = Uuid::new_v4();
    let package_part_uuid = Uuid::new_v4();
    let package_pkg_uuid = Uuid::new_v4();
    let missing_package_uuid = Uuid::new_v4();
    let net_uuid = Uuid::new_v4();
    let missing_net_uuid = Uuid::new_v4();
    let missing_net_class_uuid = Uuid::new_v4();
    let pad_key_uuid = Uuid::new_v4();
    let pad_value_uuid = Uuid::new_v4();
    let track_uuid = Uuid::new_v4();
    let second_track_key_uuid = Uuid::new_v4();

    board_value["packages"] = serde_json::json!({
        package_uuid: {
            "uuid": package_uuid,
            "part": package_part_uuid,
            "package": package_pkg_uuid,
            "reference": "U1",
            "value": "MCU",
            "position": { "x": 0, "y": 0 },
            "rotation": 0,
            "layer": 1,
            "locked": false
        }
    });
    board_value["nets"] = serde_json::json!({
        net_uuid: {
            "uuid": net_uuid,
            "name": "SIG",
            "class": missing_net_class_uuid
        }
    });
    board_value["pads"] = serde_json::json!({
        pad_key_uuid: {
            "uuid": pad_value_uuid,
            "package": missing_package_uuid,
            "name": "1",
            "net": missing_net_uuid,
            "position": { "x": 1000, "y": 2000 },
            "layer": 1,
            "shape": "circle",
            "diameter": 500000,
            "width": 0,
            "height": 0
        }
    });
    board_value["tracks"] = serde_json::json!({
        track_uuid: {
            "uuid": track_uuid,
            "net": net_uuid,
            "from": { "x": 0, "y": 0 },
            "to": { "x": 1000, "y": 0 },
            "width": 150000,
            "layer": 1
        },
        second_track_key_uuid: {
            "uuid": track_uuid,
            "net": missing_net_uuid,
            "from": { "x": 0, "y": 1000 },
            "to": { "x": 1000, "y": 1000 },
            "width": 150000,
            "layer": 1
        }
    });
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&board_value).expect("board serialization should succeed")
        ),
    )
    .expect("board.json should write");

    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    let missing_sheet_uuid = Uuid::new_v4();
    schematic_value["sheets"] = serde_json::json!({
        missing_sheet_uuid: "sheets/missing-sheet.json"
    });
    std::fs::write(
        &schematic_json,
        format!(
            "{}\n",
            to_json_deterministic(&schematic_value)
                .expect("schematic serialization should succeed")
        ),
    )
    .expect("schematic.json should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let (output, exit_code) = execute_with_exit_code(cli).expect("project validate should execute");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("validation JSON should parse");
    let issues = report["issues"]
        .as_array()
        .expect("issues should be an array");
    let issue_codes = issues
        .iter()
        .map(|issue| issue["code"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();

    assert_eq!(exit_code, 1);
    assert_eq!(report["valid"], false);
    assert!(report["issue_count"].as_u64().unwrap() >= 5);
    assert!(issue_codes.contains(&"missing_file".to_string()));
    assert!(issue_codes.contains(&"dangling_reference".to_string()));
    assert!(issue_codes.contains(&"duplicate_uuid_within_type".to_string()));
    assert!(issue_codes.contains(&"uuid_key_mismatch".to_string()));

    let text_output = execute(
        Cli::try_parse_from(["eda", "project", "validate", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("project validate text should execute");
    assert!(text_output.contains("valid: false"));
    assert!(text_output.contains("issues:"));
    assert!(text_output.contains("missing_file"));
    assert!(text_output.contains("dangling_reference"));
    assert!(text_output.contains("duplicate_uuid_within_type"));

    let _ = std::fs::remove_dir_all(&root);
}
