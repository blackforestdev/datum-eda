use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::substrate::{
    CommitProvenance, CommitSource, ModelRevision, Operation, OperationBatch,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn read_project_core_files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    [
        "project.json",
        "schematic/schematic.json",
        "board/board.json",
        "rules/rules.json",
    ]
    .into_iter()
    .map(|relative| {
        let path = root.join(relative);
        let bytes = std::fs::read(&path).expect("project core file should read");
        (path, bytes)
    })
    .collect()
}

#[test]
fn query_summary_command_reports_native_project_summary() {
    let root = unique_project_root("datum-eda-cli-query-summary");
    create_native_project(&root, Some("Canonical Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "summary",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("query summary should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("query summary JSON should parse");

    assert_eq!(report["project_name"], "Canonical Query Demo");
    assert_eq!(report["schematic"]["sheets"], 0);
    assert_eq!(report["board"]["components"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn query_zone_fills_command_reports_resolver_zone_fill_state() {
    let root = unique_project_root("datum-eda-cli-query-zone-fills");
    create_native_project(&root, Some("Canonical Zone Fill Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "zone-fills",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("query zone-fills should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("zone-fills JSON should parse");

    assert_eq!(report["contract"], "zone_fills_query_v1");
    assert_eq!(report["zone_fill_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
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
    assert!(output.contains("board_components_with_persisted_silkscreen: 0"));
    assert!(output.contains("board_components_with_persisted_mechanical: 0"));
    assert!(output.contains("board_components_with_persisted_pads: 0"));
    assert!(output.contains("board_components_with_persisted_models_3d: 0"));
    assert!(output.contains("board_persisted_component_silkscreen_texts: 0"));
    assert!(output.contains("board_persisted_component_silkscreen_lines: 0"));
    assert!(output.contains("board_persisted_component_silkscreen_arcs: 0"));
    assert!(output.contains("board_persisted_component_silkscreen_circles: 0"));
    assert!(output.contains("board_persisted_component_silkscreen_polygons: 0"));
    assert!(output.contains("board_persisted_component_silkscreen_polylines: 0"));
    assert!(output.contains("board_persisted_component_mechanical_texts: 0"));
    assert!(output.contains("board_persisted_component_mechanical_lines: 0"));
    assert!(output.contains("board_persisted_component_mechanical_arcs: 0"));
    assert!(output.contains("board_persisted_component_mechanical_circles: 0"));
    assert!(output.contains("board_persisted_component_mechanical_polygons: 0"));
    assert!(output.contains("board_persisted_component_mechanical_polylines: 0"));
    assert!(output.contains("board_persisted_component_pads: 0"));
    assert!(output.contains("board_persisted_component_models_3d: 0"));
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
    assert_eq!(report["board"]["components_with_persisted_silkscreen"], 0);
    assert_eq!(report["board"]["components_with_persisted_mechanical"], 0);
    assert_eq!(report["board"]["components_with_persisted_pads"], 0);
    assert_eq!(report["board"]["components_with_persisted_models_3d"], 0);
    assert_eq!(report["board"]["persisted_component_silkscreen_texts"], 0);
    assert_eq!(report["board"]["persisted_component_silkscreen_lines"], 0);
    assert_eq!(report["board"]["persisted_component_silkscreen_arcs"], 0);
    assert_eq!(report["board"]["persisted_component_silkscreen_circles"], 0);
    assert_eq!(
        report["board"]["persisted_component_silkscreen_polygons"],
        0
    );
    assert_eq!(
        report["board"]["persisted_component_silkscreen_polylines"],
        0
    );
    assert_eq!(report["board"]["persisted_component_mechanical_texts"], 0);
    assert_eq!(report["board"]["persisted_component_mechanical_lines"], 0);
    assert_eq!(report["board"]["persisted_component_mechanical_arcs"], 0);
    assert_eq!(report["board"]["persisted_component_mechanical_circles"], 0);
    assert_eq!(
        report["board"]["persisted_component_mechanical_polygons"],
        0
    );
    assert_eq!(
        report["board"]["persisted_component_mechanical_polylines"],
        0
    );
    assert_eq!(report["board"]["persisted_component_pads"], 0);
    assert_eq!(report["board"]["persisted_component_models_3d"], 0);

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&absolute_pool);
}

#[test]
fn project_query_resolve_debug_is_deterministic_and_read_only() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug");
    create_native_project(&root, Some("Resolve Debug Demo".to_string()))
        .expect("initial scaffold should succeed");
    let before = read_project_core_files(&root);

    let output_a = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    let output_b = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");

    assert_eq!(output_a, output_b);

    let report: serde_json::Value =
        serde_json::from_str(&output_a).expect("resolve-debug JSON should parse");
    assert_eq!(report["contract"], "project_resolver_debug_v1");
    assert_eq!(report["project_name"], "Resolve Debug Demo");
    assert_eq!(report["diagnostics"].as_array().unwrap().len(), 0);
    assert!(report["model_revision"].as_str().unwrap().len() >= 64);
    assert!(report["source_shards"].as_array().unwrap().len() >= 4);
    assert!(
        report["source_shards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|shard| {
                shard["path"] == "board/board.json"
                    && shard["authority"] == "AuthoredDesign"
                    && shard["dirty_state"] == "Clean"
            })
    );
    assert!(report["object_count"].as_u64().unwrap() >= 3);

    for (path, bytes) in before {
        assert_eq!(
            std::fs::read(&path).expect("project core file should read after query"),
            bytes,
            "resolve-debug must not rewrite {}",
            path.display()
        );
    }
}

#[test]
fn project_query_resolve_debug_commit_batch_reports_in_memory_transaction() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-commit");
    create_native_project(&root, Some("Resolve Debug Commit Demo".to_string()))
        .expect("initial scaffold should succeed");
    let before = read_project_core_files(&root);
    let board: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("board/board.json")).expect("board should read"),
    )
    .expect("board should parse");

    let resolve_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    let resolve_report: serde_json::Value =
        serde_json::from_str(&resolve_output).expect("resolve-debug JSON should parse");
    let batch_path = root.join("commit-batch.json");
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(ModelRevision(
            resolve_report["model_revision"]
                .as_str()
                .expect("model revision should exist")
                .to_string(),
        )),
        provenance: CommitProvenance {
            actor: "cli-test".to_string(),
            source: CommitSource::Cli,
            reason: "prove CLI substrate commit debug path".to_string(),
        },
        operations: vec![Operation::BumpObjectRevision {
            object_id: Uuid::parse_str(board["uuid"].as_str().expect("board uuid should exist"))
                .expect("board uuid should parse"),
        }],
    };
    std::fs::write(
        &batch_path,
        to_json_deterministic(&batch).expect("batch should serialize"),
    )
    .expect("batch should write");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
            "--commit-batch",
            batch_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug --commit-batch should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("commit debug JSON should parse");

    assert_eq!(report["contract"], "operation_batch_commit_debug_v1");
    assert_eq!(report["mode"], "dry_run");
    assert_eq!(report["status"], "accepted");
    assert_eq!(
        report["write_boundary"],
        "in_memory_only_no_project_shards_written"
    );
    assert_ne!(
        report["before_model_revision"],
        report["after_model_revision"]
    );
    assert_eq!(report["journal_len"], 1);

    for (path, bytes) in before {
        assert_eq!(
            std::fs::read(&path).expect("project core file should read after query"),
            bytes,
            "commit debug must not rewrite {}",
            path.display()
        );
    }
}

#[test]
fn project_query_resolve_debug_commit_batch_apply_persists_journal_only() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-commit-apply");
    create_native_project(&root, Some("Resolve Debug Commit Apply Demo".to_string()))
        .expect("initial scaffold should succeed");
    let before = read_project_core_files(&root);
    let board: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("board/board.json")).expect("board should read"),
    )
    .expect("board should parse");

    let resolve_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    let resolve_report: serde_json::Value =
        serde_json::from_str(&resolve_output).expect("resolve-debug JSON should parse");
    let batch_path = root.join("commit-batch-apply.json");
    let batch = OperationBatch {
        batch_id: Uuid::new_v4(),
        expected_model_revision: Some(ModelRevision(
            resolve_report["model_revision"]
                .as_str()
                .expect("model revision should exist")
                .to_string(),
        )),
        provenance: CommitProvenance {
            actor: "cli-test".to_string(),
            source: CommitSource::Cli,
            reason: "prove CLI substrate journal apply path".to_string(),
        },
        operations: vec![Operation::BumpObjectRevision {
            object_id: Uuid::parse_str(board["uuid"].as_str().expect("board uuid should exist"))
                .expect("board uuid should parse"),
        }],
    };
    std::fs::write(
        &batch_path,
        to_json_deterministic(&batch).expect("batch should serialize"),
    )
    .expect("batch should write");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
            "--commit-batch",
            batch_path.to_str().unwrap(),
            "--apply",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug --commit-batch --apply should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("commit apply debug JSON should parse");

    assert_eq!(report["contract"], "operation_batch_commit_debug_v1");
    assert_eq!(report["mode"], "journal_apply");
    assert_eq!(
        report["write_boundary"],
        "journal_only_no_project_shards_written"
    );
    assert_eq!(report["journal_len"], 1);
    assert!(root.join(".datum/journal/transactions.jsonl").exists());

    let reopened_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed after apply");
    let reopened: serde_json::Value =
        serde_json::from_str(&reopened_output).expect("reopened resolve-debug JSON should parse");
    assert_eq!(
        reopened["model_revision"], report["after_model_revision"],
        "resolver should replay journaled transactions"
    );

    for (path, bytes) in before {
        assert_eq!(
            std::fs::read(&path).expect("project core file should read after query"),
            bytes,
            "journal apply must not rewrite {}",
            path.display()
        );
    }
}

#[test]
fn project_query_pools_reports_resolved_pool_refs_directly() {
    let root = unique_project_root("datum-eda-cli-project-query-pools");
    create_native_project(&root, Some("Query Pools Demo".to_string()))
        .expect("initial scaffold should succeed");

    let relative_pool = root.join("pool");
    std::fs::create_dir_all(&relative_pool).expect("relative pool dir should exist");
    let absolute_pool = unique_project_root("datum-eda-cli-project-query-pools-abs-pool");
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

    let text_output = execute(
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "pools"])
            .expect("CLI should parse"),
    )
    .expect("project query pools should succeed");
    assert!(text_output.contains("pool"));
    assert!(text_output.contains(&relative_pool.display().to_string()));
    assert!(text_output.contains(&absolute_pool.display().to_string()));

    let json_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "pools",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query pools should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&json_output).expect("project query pools JSON should parse");
    assert_eq!(report.as_array().unwrap().len(), 2);
    assert_eq!(report[0]["manifest_path"], "pool");
    assert_eq!(report[0]["priority"], 1);
    assert_eq!(
        report[0]["resolved_path"],
        relative_pool.display().to_string()
    );
    assert_eq!(report[0]["exists"], true);
    assert_eq!(
        report[1]["manifest_path"],
        absolute_pool.display().to_string()
    );
    assert_eq!(report[1]["priority"], 2);
    assert_eq!(
        report[1]["resolved_path"],
        absolute_pool.display().to_string()
    );
    assert_eq!(report[1]["exists"], true);

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
