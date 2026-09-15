use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn query_component_instances_command_reports_resolver_component_instances() {
    let root = unique_project_root("datum-eda-cli-query-component-instances");
    create_native_project(
        &root,
        Some("Canonical Component Instance Query Demo".to_string()),
    )
    .expect("initial scaffold should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "component-instances",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("query component-instances should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("component-instances JSON should parse");

    assert_eq!(report["contract"], "component_instances_query_v1");
    assert_eq!(report["component_instance_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn query_output_jobs_command_reports_resolver_output_jobs() {
    let root = unique_project_root("datum-eda-cli-query-output-jobs");
    create_native_project(&root, Some("Canonical Output Job Query Demo".to_string()))
        .expect("initial scaffold should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "output-jobs",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("query output-jobs should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("output-jobs JSON should parse");

    assert_eq!(report["contract"], "output_job_list_v1");
    assert_eq!(report["output_job_count"], 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn query_sheets_command_reports_native_sheet_index() {
    let root = unique_project_root("datum-eda-cli-query-sheets");
    create_native_project(&root, Some("Canonical Sheets Query Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = Uuid::new_v4();

    execute(
        Cli::try_parse_from([
            "eda",
            "project",
            "create-sheet",
            root.to_str().unwrap(),
            "--name",
            "Main",
            "--sheet",
            &sheet_uuid.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("create-sheet should succeed");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "query",
            "sheets",
            root.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("query sheets should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("sheets JSON");

    assert_eq!(report.as_array().unwrap().len(), 1);
    assert_eq!(report[0]["uuid"], sheet_uuid.to_string());
    assert_eq!(report[0]["name"], "Main");

    let _ = std::fs::remove_dir_all(&root);
}
