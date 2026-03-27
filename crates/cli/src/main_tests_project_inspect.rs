use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_inspect_reports_scaffold_summary() {
    let root = unique_project_root("datum-eda-cli-project-inspect");
    create_native_project(&root, Some("Inspect Demo".to_string()))
        .expect("initial scaffold should succeed");

    let cli = Cli::try_parse_from(["eda", "project", "inspect", root.to_str().unwrap()])
        .expect("CLI should parse");

    let output = execute(cli).expect("project inspect should succeed");
    assert!(output.contains("project_name: Inspect Demo"));
    assert!(output.contains("schema_version: 1"));
    assert!(output.contains("sheet_count: 0"));
    assert!(output.contains("board_net_count: 0"));
    assert!(output.contains("rule_count: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_inspect_json_output_reports_resolved_paths_and_counts() {
    let root = unique_project_root("datum-eda-cli-project-inspect-json");
    create_native_project(&root, Some("Inspect JSON".to_string()))
        .expect("initial scaffold should succeed");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "inspect",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project inspect should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("project inspect JSON should parse");
    assert_eq!(report["project_root"], root.display().to_string());
    assert_eq!(report["project_name"], "Inspect JSON");
    assert_eq!(report["schema_version"], 1);
    assert!(report["project_uuid"].as_str().is_some());
    assert!(report["schematic_uuid"].as_str().is_some());
    assert!(report["board_uuid"].as_str().is_some());
    assert_eq!(
        report["schematic_path"],
        root.join("schematic/schematic.json").display().to_string()
    );
    assert_eq!(
        report["board_path"],
        root.join("board/board.json").display().to_string()
    );
    assert_eq!(
        report["rules_path"],
        root.join("rules/rules.json").display().to_string()
    );
    assert_eq!(report["sheet_count"], 0);
    assert_eq!(report["sheet_definition_count"], 0);
    assert_eq!(report["sheet_instance_count"], 0);
    assert_eq!(report["variant_count"], 0);
    assert_eq!(report["board_package_count"], 0);
    assert_eq!(report["board_net_count"], 0);
    assert_eq!(report["board_track_count"], 0);
    assert_eq!(report["board_via_count"], 0);
    assert_eq!(report["board_zone_count"], 0);
    assert_eq!(report["rule_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
