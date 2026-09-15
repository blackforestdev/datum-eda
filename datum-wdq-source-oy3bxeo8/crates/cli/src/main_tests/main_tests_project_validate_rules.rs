use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_validate_reports_invalid_native_rule_schema() {
    let root = unique_project_root("datum-eda-cli-project-validate-rule-schema");
    create_native_project(&root, Some("Validate Rule Schema".to_string()))
        .expect("native project scaffold should succeed");
    let rules_json = root.join("rules/rules.json");
    let rule_id = Uuid::new_v4();
    std::fs::write(
        &rules_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "object_revision": 0,
                "rules": [{
                    "uuid": rule_id,
                    "name": "Broken Clearance",
                    "scope": "All",
                    "priority": 1,
                    "enabled": true,
                    "rule_type": "clearance_copper",
                    "params": { "min_nm": 0 }
                }]
            }))
            .expect("rules serialization should succeed")
        ),
    )
    .expect("rules.json should write");

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
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("validation JSON should parse");

    assert_eq!(exit_code, 1);
    assert_eq!(report["valid"], false);
    assert!(report["issues"].as_array().unwrap().iter().any(|issue| {
        issue["code"] == "invalid_json"
            && issue["subject"] == "rules/rules.json#rules/0"
            && issue["message"]
                .as_str()
                .is_some_and(|message| message.contains("project rule param min_nm must be > 0"))
    }));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_validate_reports_invalid_native_rule_scope() {
    let root = unique_project_root("datum-eda-cli-project-validate-rule-scope");
    create_native_project(&root, Some("Validate Rule Scope".to_string()))
        .expect("native project scaffold should succeed");
    let rules_json = root.join("rules/rules.json");
    let rule_id = Uuid::new_v4();
    std::fs::write(
        &rules_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "object_revision": 0,
                "rules": [{
                    "uuid": rule_id,
                    "name": "Broken Scope Clearance",
                    "scope": {"Net": Uuid::nil()},
                    "priority": 1,
                    "enabled": true,
                    "rule_type": "clearance_copper",
                    "params": { "min_nm": 150000 }
                }]
            }))
            .expect("rules serialization should succeed")
        ),
    )
    .expect("rules.json should write");

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
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("validation JSON should parse");

    assert_eq!(exit_code, 1);
    assert_eq!(report["valid"], false);
    assert!(report["issues"].as_array().unwrap().iter().any(|issue| {
        issue["code"] == "invalid_json"
            && issue["subject"] == "rules/rules.json#rules/0"
            && issue["message"]
                .as_str()
                .is_some_and(|message| message.contains("invalid project rule scope"))
    }));

    let _ = std::fs::remove_dir_all(&root);
}
