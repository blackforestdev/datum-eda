use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn rules_root(rules: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "rules": rules,
    })
}

fn clearance_rule(rule_uuid: Uuid, name: &str, min_nm: i64) -> serde_json::Value {
    serde_json::json!({
        "uuid": rule_uuid,
        "name": name,
        "scope": "All",
        "priority": 10,
        "enabled": true,
        "rule_type": "clearance_copper",
        "params": { "min_nm": min_nm }
    })
}

fn query_design_rules(root: &Path) -> serde_json::Value {
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
    serde_json::from_str(&output).expect("project query JSON should parse")
}

#[test]
fn project_create_delete_project_rule_round_trips_through_journal_and_resolver_query() {
    let root = unique_project_root("datum-eda-cli-project-rule-create-delete");
    create_native_project(&root, Some("Rule Edit Demo".to_string()))
        .expect("initial scaffold should succeed");
    let rules_path = root.join("rules/rules.json");
    let original_text = std::fs::read_to_string(&rules_path).expect("rules root should read");
    let rule_id = Uuid::new_v4();
    let rule_path = root.join("clearance-rule.json");
    std::fs::write(
        &rule_path,
        format!(
            "{}\n",
            to_json_deterministic(&clearance_rule(rule_id, "Authored Clearance", 175000))
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("rule file should write");

    let create = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "create-project-rule",
        root.to_str().unwrap(),
        "--rule-file",
        rule_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(create).expect("create-project-rule should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("create-project-rule JSON should parse");
    assert_eq!(report["action"], "create_project_rule");
    assert_eq!(report["rule_uuid"], rule_id.to_string());
    assert_eq!(report["rule_count"], 1);
    assert_eq!(report["rules_object_revision"], 1);

    std::fs::write(&rules_path, original_text.clone()).expect("stale promoted rules should write");
    let query = query_design_rules(&root);
    assert_eq!(query["count"], 1);
    assert_eq!(query["rules"][0]["name"], "Authored Clearance");

    let updated_rule_path = root.join("updated-clearance-rule.json");
    std::fs::write(
        &updated_rule_path,
        format!(
            "{}\n",
            to_json_deterministic(&clearance_rule(rule_id, "Updated Clearance", 225000))
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("updated rule file should write");
    let set = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-project-rule",
        root.to_str().unwrap(),
        "--rule-file",
        updated_rule_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(set).expect("set-project-rule should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("set-project-rule JSON should parse");
    assert_eq!(report["action"], "set_project_rule");
    assert_eq!(report["rule_uuid"], rule_id.to_string());
    assert_eq!(report["rule_count"], 1);
    assert_eq!(report["rules_object_revision"], 2);

    std::fs::write(&rules_path, original_text.clone()).expect("stale promoted rules should write");
    let query = query_design_rules(&root);
    assert_eq!(query["count"], 1);
    assert_eq!(query["rules"][0]["name"], "Updated Clearance");
    assert_eq!(query["rules"][0]["params"]["min_nm"], 225000);

    let delete = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "delete-project-rule",
        root.to_str().unwrap(),
        "--rule",
        &rule_id.to_string(),
    ])
    .expect("CLI should parse");
    let output = execute(delete).expect("delete-project-rule should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("delete-project-rule JSON should parse");
    assert_eq!(report["action"], "delete_project_rule");
    assert_eq!(report["rule_uuid"], rule_id.to_string());
    assert_eq!(report["rule_count"], 0);
    assert_eq!(report["rules_object_revision"], 3);

    std::fs::write(&rules_path, original_text).expect("stale promoted rules should write");
    let query = query_design_rules(&root);
    assert_eq!(query["count"], 0);

    let undo = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(undo).expect("project undo should succeed");
    let query = query_design_rules(&root);
    assert_eq!(query["count"], 1);
    assert_eq!(query["rules"][0]["uuid"], rule_id.to_string());
    assert_eq!(query["rules"][0]["name"], "Updated Clearance");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_project_rule_undo_restores_previous_rule_payload() {
    let root = unique_project_root("datum-eda-cli-project-rule-set-undo");
    create_native_project(&root, Some("Rule Set Undo Demo".to_string()))
        .expect("initial scaffold should succeed");
    let rule_id = Uuid::new_v4();
    let rule_path = root.join("clearance-rule.json");
    std::fs::write(
        &rule_path,
        format!(
            "{}\n",
            to_json_deterministic(&clearance_rule(rule_id, "Original Clearance", 150000))
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("rule file should write");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-project-rule",
            root.to_str().unwrap(),
            "--rule-file",
            rule_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-project-rule should succeed");

    let updated_rule_path = root.join("updated-clearance-rule.json");
    std::fs::write(
        &updated_rule_path,
        format!(
            "{}\n",
            to_json_deterministic(&clearance_rule(rule_id, "Undoable Clearance", 275000))
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("updated rule file should write");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-project-rule",
            root.to_str().unwrap(),
            "--rule-file",
            updated_rule_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("set-project-rule should succeed");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "undo",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("project undo should succeed");
    let query = query_design_rules(&root);
    assert_eq!(query["rules"][0]["name"], "Original Clearance");
    assert_eq!(query["rules"][0]["params"]["min_nm"], 150000);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_project_rules_round_trips_through_journal_and_resolver_query() {
    let root = unique_project_root("datum-eda-cli-project-rules");
    create_native_project(&root, Some("Rules Demo".to_string()))
        .expect("initial scaffold should succeed");
    let rules_path = root.join("rules/rules.json");
    let original_text = std::fs::read_to_string(&rules_path).expect("rules root should read");
    let replacement_path = root.join("replacement-rules.json");
    std::fs::write(
        &replacement_path,
        format!(
            "{}\n",
            to_json_deterministic(&rules_root(serde_json::json!([clearance_rule(
                Uuid::new_v4(),
                "Tight Copper Clearance",
                125000
            )])))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("replacement rules should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "set-project-rules",
        root.to_str().unwrap(),
        "--rules-file",
        replacement_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let output = execute(cli).expect("set-project-rules should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("set-project-rules JSON should parse");
    assert_eq!(report["action"], "set_project_rules");
    assert_eq!(report["rule_count"], 1);

    std::fs::write(&rules_path, original_text).expect("stale promoted rules should write");
    let query = query_design_rules(&root);
    assert_eq!(query["count"], 1);
    assert_eq!(query["rules"][0]["name"], "Tight Copper Clearance");

    let undo = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "undo",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    execute(undo).expect("project undo should succeed");
    let query = query_design_rules(&root);
    assert_eq!(query["count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
