use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_query_summary_reports_native_scaffold_counts() {
    let root = unique_project_root("datum-eda-cli-project-query-summary");
    create_native_project(&root, Some("Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4().to_string();
    let sheet_path = root
        .join("schematic/sheets")
        .join(format!("{sheet_uuid}.json"));
    std::fs::write(
        &sheet_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "uuid": sheet_uuid,
            "name": "Main",
            "frame": null,
            "symbols": { "sym-1": { "uuid": "sym-1" } },
            "wires": {},
            "junctions": {},
            "labels": { "label-1": { "uuid": "label-1" } },
            "buses": { "bus-1": { "uuid": "bus-1" } },
            "bus_entries": { "entry-1": { "uuid": "entry-1" } },
            "ports": { "port-1": { "uuid": "port-1" } },
            "noconnects": { "nc-1": { "uuid": "nc-1" } },
            "texts": {},
            "drawings": {}
        }))
        .expect("sheet JSON should serialize"),
    )
    .expect("sheet file should write");

    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    schematic_value["sheets"] = serde_json::json!({
        sheet_uuid.clone(): format!("sheets/{sheet_uuid}.json")
    });
    schematic_value["instances"] = serde_json::json!([{
        "uuid": Uuid::new_v4(),
        "definition": Uuid::new_v4(),
        "parent_sheet": null,
        "position": { "x": 0, "y": 0 },
        "name": "Main Sheet"
    }]);
    schematic_value["variants"] = serde_json::json!({
        "variant-1": {
            "name": "Default",
            "fitted_components": {}
        }
    });
    std::fs::write(
        &schematic_json,
        format!(
            "{}\n",
            to_json_deterministic(&schematic_value)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("schematic.json should write");

    let relative_pool = root.join("pool");
    std::fs::create_dir_all(&relative_pool).expect("relative pool dir should exist");
    let absolute_pool = unique_project_root("datum-eda-cli-project-query-summary-abs-pool");
    std::fs::create_dir_all(&absolute_pool).expect("absolute pool dir should exist");

    let project_json = root.join("project.json");
    let mut manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&project_json).expect("project.json should read"),
    )
    .expect("project.json should parse");
    manifest["pools"] = serde_json::json!([
        { "path": "pool", "priority": 1 },
        { "path": absolute_pool.to_str().unwrap(), "priority": 2 }
    ]);
    std::fs::write(
        &project_json,
        format!(
            "{}\n",
            to_json_deterministic(&manifest).expect("canonical serialization should succeed")
        ),
    )
    .expect("project.json should write");

    let cli = Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
        .expect("CLI should parse");

    let output = execute(cli).expect("project query summary should succeed");
    assert!(output.contains("project_name: Query Demo"));
    assert!(output.contains("schematic_sheets: 1"));
    assert!(output.contains("schematic_sheet_instances: 1"));
    assert!(output.contains("schematic_variants: 1"));
    assert!(output.contains("schematic_symbols: 1"));
    assert!(output.contains("schematic_labels: 1"));
    assert!(output.contains("schematic_ports: 1"));
    assert!(output.contains("schematic_buses: 1"));
    assert!(output.contains("schematic_bus_entries: 1"));
    assert!(output.contains("schematic_noconnects: 1"));
    assert!(output.contains("pools: 2"));
    assert!(output.contains("pool_refs:"));
    assert!(output.contains("path=pool priority=1"));
    assert!(output.contains(&format!(
        "resolved_path={} exists=true",
        relative_pool.display()
    )));
    assert!(output.contains(&format!(
        "path={} priority=2 resolved_path={} exists=true",
        absolute_pool.display(),
        absolute_pool.display()
    )));
    assert!(output.contains("board_components: 0"));
    assert!(output.contains("rule_count: 0"));

    let json_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "summary",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query summary should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&json_output).expect("project query summary JSON should parse");
    assert_eq!(report["pools"], 2);
    assert_eq!(report["pool_refs"].as_array().unwrap().len(), 2);
    assert_eq!(report["pool_refs"][0]["manifest_path"], "pool");
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

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&absolute_pool);
}

#[test]
fn project_query_design_rules_reports_native_rules_payload() {
    let root = unique_project_root("datum-eda-cli-project-query-rules");
    create_native_project(&root, Some("Rules Demo".to_string()))
        .expect("initial scaffold should succeed");

    let rules_json = root.join("rules/rules.json");
    std::fs::write(
        &rules_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "rules": [
                    {
                        "uuid": Uuid::new_v4(),
                        "name": "Default Clearance",
                        "scope": "All",
                        "priority": 1,
                        "enabled": true,
                        "rule_type": "clearance_copper",
                        "params": { "min_nm": 150000 }
                    }
                ]
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("rules.json should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "design-rules",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query design-rules should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("project query JSON should parse");
    assert_eq!(report["domain"], "native_project");
    assert_eq!(report["count"], 1);
    assert_eq!(report["rules"].as_array().unwrap().len(), 1);
    assert_eq!(report["rules"][0]["name"], "Default Clearance");

    let _ = std::fs::remove_dir_all(&root);
}
