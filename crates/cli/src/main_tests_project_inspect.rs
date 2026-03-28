use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

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

#[test]
fn project_inspect_reports_resolved_pool_refs() {
    let root = unique_project_root("datum-eda-cli-project-inspect-pools");
    create_native_project(&root, Some("Inspect Pools".to_string()))
        .expect("initial scaffold should succeed");

    let relative_pool = root.join("pool");
    std::fs::create_dir_all(&relative_pool).expect("relative pool dir should exist");
    let absolute_pool = unique_project_root("datum-eda-cli-project-inspect-abs-pool");
    std::fs::create_dir_all(&absolute_pool).expect("absolute pool dir should exist");

    let project_json = root.join("project.json");
    let mut manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&project_json).expect("project manifest should read"),
    )
    .expect("project manifest should parse");
    manifest["pools"] = serde_json::json!([
        { "path": "pool", "priority": 1 },
        { "path": absolute_pool.to_str().unwrap(), "priority": 2 },
        { "path": "missing-pool", "priority": 3 }
    ]);
    std::fs::write(
        &project_json,
        format!(
            "{}\n",
            to_json_deterministic(&manifest).expect("manifest serialization should succeed")
        ),
    )
    .expect("project manifest should write");

    let text_output = execute(
        Cli::try_parse_from(["eda", "project", "inspect", root.to_str().unwrap()])
            .expect("CLI should parse"),
    )
    .expect("project inspect should succeed");
    assert!(text_output.contains("pools: 3"));
    assert!(text_output.contains("pool_refs:"));
    assert!(text_output.contains("path=pool priority=1"));
    assert!(text_output.contains(&format!(
        "resolved_path={} exists=true",
        relative_pool.display()
    )));
    assert!(text_output.contains(&format!(
        "path={} priority=2 resolved_path={} exists=true",
        absolute_pool.display(),
        absolute_pool.display()
    )));
    assert!(text_output.contains(&format!(
        "path=missing-pool priority=3 resolved_path={} exists=false",
        root.join("missing-pool").display()
    )));

    let json_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "inspect",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("project inspect should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&json_output).expect("project inspect JSON should parse");
    assert_eq!(report["pools"], 3);
    assert_eq!(report["pool_refs"].as_array().unwrap().len(), 3);
    assert_eq!(report["pool_refs"][0]["manifest_path"], "pool");
    assert_eq!(report["pool_refs"][0]["priority"], 1);
    assert_eq!(
        report["pool_refs"][0]["resolved_path"],
        relative_pool.display().to_string()
    );
    assert_eq!(report["pool_refs"][0]["exists"], true);
    assert_eq!(
        report["pool_refs"][1]["manifest_path"],
        absolute_pool.display().to_string()
    );
    assert_eq!(report["pool_refs"][1]["exists"], true);
    assert_eq!(report["pool_refs"][2]["manifest_path"], "missing-pool");
    assert_eq!(report["pool_refs"][2]["exists"], false);

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&absolute_pool);
}
