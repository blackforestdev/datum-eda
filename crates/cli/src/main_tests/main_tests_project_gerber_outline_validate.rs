use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_validate_gerber_outline_reports_match_and_mismatch() {
    let root = unique_project_root("datum-eda-cli-project-gerber-outline-validate");
    create_native_project(&root, Some("Gerber Outline Validate Demo".to_string()))
        .expect("initial scaffold should succeed");

    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Gerber Outline Validate Demo Board",
                "stackup": { "layers": [] },
                "outline": {
                    "vertices": [
                        { "x": 0, "y": 0 },
                        { "x": 1000000, "y": 0 },
                        { "x": 1000000, "y": 500000 },
                        { "x": 0, "y": 500000 }
                    ],
                    "closed": true
                },
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
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let gerber_path = root.join("outline.gbr");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-gerber-outline",
        root.to_str().unwrap(),
        "--out",
        gerber_path.to_str().unwrap(),
    ])
    .expect("export CLI should parse");
    let _ = execute(export_cli).expect("gerber outline export should succeed");

    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-outline",
        root.to_str().unwrap(),
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 0);
    assert_eq!(report["action"], "validate_gerber_outline");
    assert_eq!(report["matches_expected"], true);

    std::fs::write(&gerber_path, "corrupted\n").expect("gerber overwrite should succeed");
    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-outline",
        root.to_str().unwrap(),
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 1);
    assert_eq!(report["matches_expected"], false);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_validate_gerber_outline_uses_resolver_materialized_board_state() {
    let root = unique_project_root("datum-eda-cli-project-gerber-outline-resolved-validate");
    create_native_project(
        &root,
        Some("Gerber Outline Resolved Validate Demo".to_string()),
    )
    .expect("initial scaffold should succeed");
    let board_json = root.join("board/board.json");
    let stale_board = std::fs::read_to_string(&board_json).expect("board file should read");

    let set_outline_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-board-outline",
        root.to_str().unwrap(),
        "--vertex",
        "0:0",
        "--vertex",
        "2000000:0",
        "--vertex",
        "2000000:1000000",
        "--vertex",
        "0:1000000",
    ])
    .expect("CLI should parse");
    let _ = execute(set_outline_cli).expect("set board outline should succeed");
    std::fs::write(&board_json, stale_board).expect("stale board file should restore");

    let gerber_path = root.join("outline-resolved.gbr");
    std::fs::write(
        &gerber_path,
        concat!(
            "G04 datum-eda native board outline*\n",
            "%FSLAX46Y46*%\n",
            "%MOMM*%\n",
            "%LPD*%\n",
            "%ADD10C,0.100000*%\n",
            "D10*\n",
            "X0Y0D02*\n",
            "X2000000Y0D01*\n",
            "X2000000Y1000000D01*\n",
            "X0Y1000000D01*\n",
            "X0Y0D01*\n",
            "M02*\n"
        ),
    )
    .expect("gerber file should write");

    let validate_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "validate-gerber-outline",
        root.to_str().unwrap(),
        "--gerber",
        gerber_path.to_str().unwrap(),
    ])
    .expect("validate CLI should parse");
    let (output, exit_code) = execute_with_exit_code(validate_cli).expect("validation should run");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report JSON");
    assert_eq!(exit_code, 0);
    assert_eq!(report["matches_expected"], true);
    assert_eq!(report["outline_vertex_count"], 4);

    let _ = std::fs::remove_dir_all(&root);
}
