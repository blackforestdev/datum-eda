use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn run_project_command(args: &[&str]) -> Result<String> {
    execute(Cli::try_parse_from(args).expect("CLI should parse"))
}

fn validate_project_json(root: &Path) -> (serde_json::Value, i32) {
    let (output, exit_code) = execute_with_exit_code(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "validate",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("project validate should execute");
    (
        serde_json::from_str(&output).expect("validation JSON should parse"),
        exit_code,
    )
}

#[test]
fn project_validate_reads_resolver_materialized_authored_shards() {
    let root = unique_project_root("datum-eda-cli-project-validate-resolver-materialized");
    create_native_project(&root, Some("Validate Resolver Demo".to_string()))
        .expect("native project scaffold should succeed");
    let board_json = root.join("board/board.json");
    let schematic_json = root.join("schematic/schematic.json");
    let stale_board = std::fs::read_to_string(&board_json).expect("board.json should read");
    let stale_schematic =
        std::fs::read_to_string(&schematic_json).expect("schematic.json should read");
    let sheet_uuid = Uuid::new_v4();

    let created_sheet_output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-sheet",
        root.to_str().unwrap(),
        "--name",
        "Resolved Sheet",
        "--sheet",
        &sheet_uuid.to_string(),
    ])
    .expect("sheet create should succeed");
    let created_sheet: serde_json::Value =
        serde_json::from_str(&created_sheet_output).expect("sheet create JSON should parse");
    let sheet_path = PathBuf::from(created_sheet["sheet_path"].as_str().unwrap());
    let stale_sheet = std::fs::read_to_string(&sheet_path).expect("sheet should read");

    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "draw-wire",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--from-x-nm",
        "10",
        "--from-y-nm",
        "20",
        "--to-x-nm",
        "30",
        "--to-y-nm",
        "40",
    ])
    .expect("wire draw should succeed");

    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "place-board-text",
        root.to_str().unwrap(),
        "--text",
        "VALID",
        "--x-nm",
        "100",
        "--y-nm",
        "200",
        "--layer",
        "1",
    ])
    .expect("board text create should succeed");

    std::fs::write(&board_json, stale_board).expect("stale board should restore");
    std::fs::write(&schematic_json, stale_schematic).expect("stale schematic should restore");
    std::fs::write(&sheet_path, stale_sheet).expect("stale sheet should restore");

    let (report, exit_code) = validate_project_json(&root);
    assert_eq!(exit_code, 0);
    assert_eq!(report["valid"], true);
    assert_eq!(report["required_files_validated"], 4);
    assert_eq!(report["checked_sheet_files"], 1);
    assert_eq!(report["issue_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
