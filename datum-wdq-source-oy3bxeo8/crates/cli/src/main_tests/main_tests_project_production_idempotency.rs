use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn journal_transaction_count(root: &Path) -> usize {
    std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should read")
        .lines()
        .count()
}

#[test]
fn project_create_manufacturing_plan_is_idempotent_from_resolver_replay() {
    let root = unique_project_root("datum-eda-cli-production-plan-idempotency");
    create_native_project(&root, Some("Production Plan Idempotency".to_string()))
        .expect("initial scaffold should succeed");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-manufacturing-plan",
            root.to_str().unwrap(),
            "--prefix",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan create should succeed");
    let create_report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    let plan = create_report["manufacturing_plan"]["id"].as_str().unwrap();
    let shard = root.join(format!(".datum/manufacturing_plans/{plan}.json"));
    std::fs::remove_file(&shard).expect("promoted plan shard should remove");
    let transaction_count = journal_transaction_count(&root);

    let replayed_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-manufacturing-plan",
            root.to_str().unwrap(),
            "--prefix",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("manufacturing plan recreate should resolve journaled shard");
    let replayed: serde_json::Value = serde_json::from_str(&replayed_output).unwrap();
    assert_eq!(replayed["created"], false);
    assert_eq!(replayed["manufacturing_plan"]["id"], plan);
    assert_eq!(journal_transaction_count(&root), transaction_count);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_panel_projection_is_idempotent_from_resolver_replay() {
    let root = unique_project_root("datum-eda-cli-production-panel-idempotency");
    create_native_project(&root, Some("Production Panel Idempotency".to_string()))
        .expect("initial scaffold should succeed");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-panel-projection",
            root.to_str().unwrap(),
            "--key",
            "release-panel",
            "--name",
            "Release Panel",
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection create should succeed");
    let create_report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    let panel = create_report["panel_projection"]["id"].as_str().unwrap();
    let shard = root.join(format!(".datum/panel_projections/{panel}.json"));
    std::fs::remove_file(&shard).expect("promoted panel shard should remove");
    let transaction_count = journal_transaction_count(&root);

    let replayed_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-panel-projection",
            root.to_str().unwrap(),
            "--key",
            "release-panel",
            "--name",
            "Release Panel",
        ])
        .expect("CLI should parse"),
    )
    .expect("panel projection recreate should resolve journaled shard");
    let replayed: serde_json::Value = serde_json::from_str(&replayed_output).unwrap();
    assert_eq!(replayed["created"], false);
    assert_eq!(replayed["panel_projection"]["id"], panel);
    assert_eq!(journal_transaction_count(&root), transaction_count);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_output_job_is_idempotent_from_resolver_replay() {
    let root = unique_project_root("datum-eda-cli-production-output-job-idempotency");
    create_native_project(&root, Some("Production OutputJob Idempotency".to_string()))
        .expect("initial scaffold should succeed");

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-gerber-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--name",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("output job create should succeed");
    let create_report: serde_json::Value = serde_json::from_str(&create_output).unwrap();
    let output_job = create_report["output_job"]["id"].as_str().unwrap();
    let shard = root.join(format!(".datum/output_jobs/{output_job}.json"));
    std::fs::remove_file(&shard).expect("promoted output job shard should remove");
    let transaction_count = journal_transaction_count(&root);

    let replayed_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-gerber-output-job",
            root.to_str().unwrap(),
            "--prefix",
            "release-a",
            "--name",
            "Release A",
        ])
        .expect("CLI should parse"),
    )
    .expect("output job recreate should resolve journaled shard");
    let replayed: serde_json::Value = serde_json::from_str(&replayed_output).unwrap();
    assert_eq!(replayed["created"], false);
    assert_eq!(replayed["output_job"]["id"], output_job);
    assert_eq!(journal_transaction_count(&root), transaction_count);

    let _ = std::fs::remove_dir_all(&root);
}
