use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_validate_excellon_drill_reports_match_and_mismatch() {
    let root = unique_project_root("datum-eda-cli-project-excellon-drill-validate");
    create_native_project(&root, Some("Excellon Drill Validate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    let net_uuid = Uuid::new_v4();
    let via_uuid = Uuid::new_v4();
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Excellon Drill Validate Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {},
                "pads": {},
                "tracks": {},
                "vias": {
                    via_uuid.to_string(): {
                        "uuid": via_uuid,
                        "net": net_uuid,
                        "position": { "x": 1000000, "y": 1500000 },
                        "drill": 300000,
                        "diameter": 600000,
                        "from_layer": 1,
                        "to_layer": 31
                    }
                },
                "zones": {},
                "nets": {
                    net_uuid.to_string(): {
                        "uuid": net_uuid,
                        "name": "N$1",
                        "class": Uuid::new_v4()
                    }
                },
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

    let drill_path = root.join("drill.drl");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-excellon-drill",
        root.to_str().unwrap(),
        "--out",
        drill_path.to_str().unwrap(),
    ])
    .expect("export CLI should parse");
    let _ = execute(export_cli).expect("excellon drill export should succeed");

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
    assert_eq!(report["action"], "validate_excellon_drill");
    assert_eq!(report["matches_expected"], true);
    assert_eq!(report["via_count"], 1);
    assert_eq!(report["tool_count"], 1);
    assert_eq!(report["tools"][0]["tool"], "T01");
    assert_eq!(report["tools"][0]["diameter_mm"], "0.300000");
    assert_eq!(report["tools"][0]["hits"], 1);

    std::fs::write(&drill_path, "corrupted\n").expect("drill overwrite should succeed");
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
    assert_eq!(exit_code, 1);
    assert_eq!(report["matches_expected"], false);
    assert_eq!(report["via_count"], 1);
    assert_eq!(report["tool_count"], 1);
    assert_eq!(report["tools"][0]["tool"], "T01");

    let _ = std::fs::remove_dir_all(&root);
}
