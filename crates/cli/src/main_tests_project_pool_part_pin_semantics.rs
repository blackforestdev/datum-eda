use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn run_project_command(args: &[&str]) -> Result<String> {
    execute(Cli::try_parse_from(args).expect("CLI should parse"))
}

fn query_pool_object_payload(root: &Path, kind: &str, object_id: Uuid) -> serde_json::Value {
    let output = run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "pool-library-objects",
        "--kind",
        kind,
        "--object",
        &object_id.to_string(),
        "--include-payload",
    ])
    .expect("pool query should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    assert_eq!(report["object_count"], 1);
    report["objects"][0]["payload"].clone()
}

#[test]
fn project_set_pool_unit_pin_accepts_electrical_type_alias() {
    let root = unique_project_root("datum-eda-cli-project-pool-unit-pin-electrical-type");
    create_native_project(&root, Some("Pool Unit Pin Electrical Type".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "create-pool-unit",
        root.to_str().unwrap(),
        "--unit",
        &unit_id.to_string(),
        "--name",
        "OpAmpUnit",
    ])
    .expect("unit create should succeed");
    run_project_command(&[
        "eda",
        "--format",
        "json",
        "project",
        "set-pool-unit-pin",
        root.to_str().unwrap(),
        "--unit",
        &unit_id.to_string(),
        "--pin",
        &pin_id.to_string(),
        "--name",
        "IN",
        "--electrical-type",
        "Input",
    ])
    .expect("unit pin set should succeed");
    let payload = query_pool_object_payload(&root, "units", unit_id);
    assert_eq!(payload["pins"][pin_id.to_string()]["name"], "IN");
    assert_eq!(payload["pins"][pin_id.to_string()]["direction"], "Input");
    assert_eq!(
        payload["pins"][pin_id.to_string()]["electrical_type"],
        "Input"
    );
    assert_eq!(payload["pins"][pin_id.to_string()]["swap_group"], 0);
    let _ = std::fs::remove_dir_all(&root);
}
