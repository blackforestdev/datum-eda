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

fn clearance_rule_with_scope(
    rule_uuid: Uuid,
    name: &str,
    min_nm: i64,
    scope: serde_json::Value,
) -> serde_json::Value {
    let mut rule = clearance_rule(rule_uuid, name, min_nm);
    rule["scope"] = scope;
    rule
}

fn write_rule_payload(root: &Path, filename: &str, rule: serde_json::Value) -> PathBuf {
    let path = root.join(filename);
    std::fs::write(
        &path,
        format!(
            "{}\n",
            to_json_deterministic(&rule).expect("canonical serialization should succeed")
        ),
    )
    .expect("rule payload should write");
    path
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
    assert_eq!(query["rules"][0]["object_revision"], 0);

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
    assert_eq!(query["rules"][0]["object_revision"], 1);

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
    assert_eq!(query["rules"][0]["object_revision"], 1);

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
fn project_rule_mutations_reject_invalid_rule_schema_before_commit() {
    let root = unique_project_root("datum-eda-cli-project-rule-schema-validation");
    create_native_project(&root, Some("Rule Schema Demo".to_string()))
        .expect("initial scaffold should succeed");
    let journal_path = root.join(".datum/journal/transactions.jsonl");
    let rule_id = Uuid::new_v4();
    let invalid_rule_path = write_rule_payload(
        &root,
        "invalid-clearance-rule.json",
        clearance_rule(rule_id, "Invalid Clearance", 0),
    );

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-project-rule",
            root.to_str().unwrap(),
            "--rule-file",
            invalid_rule_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("invalid rule schema should fail");
    assert!(format!("{error:#}").contains("project rule param min_nm must be > 0"));
    assert!(
        !journal_path.exists(),
        "invalid rule create must not append a transaction journal"
    );

    let valid_rule_path = write_rule_payload(
        &root,
        "valid-clearance-rule.json",
        clearance_rule(rule_id, "Valid Clearance", 150000),
    );
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-project-rule",
            root.to_str().unwrap(),
            "--rule-file",
            valid_rule_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("valid rule create should succeed");
    let journal_len = std::fs::read_to_string(&journal_path)
        .expect("journal should read")
        .lines()
        .count();

    let invalid_set_path = write_rule_payload(
        &root,
        "invalid-set-clearance-rule.json",
        clearance_rule(rule_id, "Invalid Set Clearance", -1),
    );
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-project-rule",
            root.to_str().unwrap(),
            "--rule-file",
            invalid_set_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("invalid set rule schema should fail");
    assert!(format!("{error:#}").contains("project rule param min_nm must be > 0"));
    assert_eq!(
        std::fs::read_to_string(&journal_path)
            .expect("journal should read")
            .lines()
            .count(),
        journal_len,
        "invalid rule set must not append a transaction"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_rule_mutations_reject_invalid_rule_scope_before_commit() {
    let root = unique_project_root("datum-eda-cli-project-rule-scope-validation");
    create_native_project(&root, Some("Rule Scope Demo".to_string()))
        .expect("initial scaffold should succeed");
    let journal_path = root.join(".datum/journal/transactions.jsonl");
    let rule_id = Uuid::new_v4();
    let invalid_rule_path = write_rule_payload(
        &root,
        "invalid-scope-rule.json",
        clearance_rule_with_scope(
            rule_id,
            "Invalid Scope",
            150000,
            serde_json::json!({"Net": Uuid::nil()}),
        ),
    );

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-project-rule",
            root.to_str().unwrap(),
            "--rule-file",
            invalid_rule_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("invalid rule scope should fail");
    assert!(format!("{error:#}").contains("invalid project rule scope"));
    assert!(
        !journal_path.exists(),
        "invalid rule scope must not append a transaction journal"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_project_rules_rejects_invalid_rule_schema_before_commit() {
    let root = unique_project_root("datum-eda-cli-project-rules-schema-validation");
    create_native_project(&root, Some("Rules Schema Demo".to_string()))
        .expect("initial scaffold should succeed");
    let journal_path = root.join(".datum/journal/transactions.jsonl");
    let replacement_path = root.join("invalid-rules-root.json");
    std::fs::write(
        &replacement_path,
        format!(
            "{}\n",
            to_json_deterministic(&rules_root(serde_json::json!([clearance_rule(
                Uuid::new_v4(),
                "",
                150000
            )])))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("replacement rules should write");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-project-rules",
            root.to_str().unwrap(),
            "--rules-file",
            replacement_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("invalid rules root schema should fail");
    assert!(format!("{error:#}").contains("project rule name must not be empty"));
    assert!(
        !journal_path.exists(),
        "invalid rules root replacement must not append a transaction journal"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_rule_mutation_reports_use_resolver_materialized_rules() {
    let root = unique_project_root("datum-eda-cli-project-rule-report-resolver");
    create_native_project(&root, Some("Rule Report Resolver Demo".to_string()))
        .expect("initial scaffold should succeed");
    let rules_path = root.join("rules/rules.json");
    let original_text = std::fs::read_to_string(&rules_path).expect("rules root should read");
    let rule_id = Uuid::new_v4();
    let rule_path = root.join("clearance-rule.json");
    std::fs::write(
        &rule_path,
        format!(
            "{}\n",
            to_json_deterministic(&clearance_rule(rule_id, "Report Clearance", 175000))
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("rule file should write");

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "create-project-rule",
            root.to_str().unwrap(),
            "--rule-file",
            rule_path.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-project-rule should succeed");
    std::fs::write(&rules_path, &original_text).expect("stale promoted rules should write");

    let updated_rule_path = root.join("updated-clearance-rule.json");
    std::fs::write(
        &updated_rule_path,
        format!(
            "{}\n",
            to_json_deterministic(&clearance_rule(rule_id, "Report Clearance Updated", 225000))
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("updated rule file should write");
    let set_output = execute(
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
    .expect("set-project-rule should resolve stale promoted rules root");
    let set_report: serde_json::Value =
        serde_json::from_str(&set_output).expect("set-project-rule JSON should parse");
    assert_eq!(set_report["rule_count"], 1);
    assert_eq!(set_report["rules_object_revision"], 2);

    std::fs::write(&rules_path, original_text).expect("stale promoted rules should write");
    let delete_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "delete-project-rule",
            root.to_str().unwrap(),
            "--rule",
            &rule_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("delete-project-rule should resolve stale promoted rules root");
    let delete_report: serde_json::Value =
        serde_json::from_str(&delete_output).expect("delete-project-rule JSON should parse");
    assert_eq!(delete_report["rule_count"], 0);
    assert_eq!(delete_report["rules_object_revision"], 3);

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
    assert_eq!(report["rules_object_revision"], 1);
    let journal_log = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should read");
    let transaction: serde_json::Value = serde_json::from_str(
        journal_log
            .lines()
            .last()
            .expect("journal should contain set-project-rules transaction"),
    )
    .expect("transaction JSON should parse");
    assert_eq!(transaction["operations"].as_array().unwrap().len(), 1);
    assert_eq!(transaction["operations"][0]["kind"], "set_project_rules");

    std::fs::write(&rules_path, original_text).expect("stale promoted rules should write");
    let query = query_design_rules(&root);
    assert_eq!(query["count"], 1);
    assert_eq!(query["rules"][0]["name"], "Tight Copper Clearance");
    assert_eq!(query["rules"][0]["object_revision"], 0);

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
