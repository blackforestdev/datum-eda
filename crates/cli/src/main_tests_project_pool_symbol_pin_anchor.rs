use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn create_typed_pool_unit(root: &Path, unit_id: Uuid) {
    execute(
        Cli::try_parse_from([
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
        .expect("CLI should parse"),
    )
    .expect("typed pool unit create should succeed");
}

fn create_typed_pool_symbol(root: &Path, symbol_id: Uuid, unit_id: Uuid) {
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-symbol",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--unit",
            &unit_id.to_string(),
            "--name",
            "OpAmpSymbol",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool symbol create should succeed");
}

fn set_typed_pool_unit_pin(root: &Path, unit_id: Uuid, pin_id: Uuid) {
    execute(
        Cli::try_parse_from([
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
            "OUT",
            "--direction",
            "Output",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool unit pin set should succeed");
}

fn query_pool_object_payload(root: &Path, kind: &str, object_id: Uuid) -> serde_json::Value {
    let output = execute(
        Cli::try_parse_from([
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
        .expect("CLI should parse"),
    )
    .expect("pool object query should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("query report JSON should parse");
    assert_eq!(report["object_count"], 1);
    report["objects"][0]["payload"].clone()
}

#[test]
fn project_set_pool_symbol_pin_anchor_authors_anchor_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-pin-anchor");
    create_native_project(&root, Some("Pool Symbol Pin Anchor".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    set_typed_pool_unit_pin(&root, unit_id, pin_id);
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-symbol-pin-anchor",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--pin",
            &pin_id.to_string(),
            "--x-nm",
            "100",
            "--y-nm",
            "200",
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol pin anchor set should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("pin anchor report JSON should parse");
    assert_eq!(report["action"], "set_symbol_pin_anchor");
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(
        payload["pin_anchors"]
            .as_array()
            .expect("anchors should be array")
            .len(),
        1
    );
    assert_eq!(payload["pin_anchors"][0]["pin"], pin_id.to_string());
    assert_eq!(payload["pin_anchors"][0]["position"]["x"], 100);
    assert_eq!(payload["pin_anchors"][0]["orientation"], "Right");
    assert!(payload["pin_anchors"][0]["length_nm"].is_null());
    assert_eq!(payload["pin_anchors"][0]["decoration"], "none");
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
    .expect("undo should succeed");
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert!(
        payload
            .get("pin_anchors")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_symbol_pin_anchor_authors_style_fields() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-pin-anchor-style");
    create_native_project(&root, Some("Pool Symbol Pin Anchor Style".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    set_typed_pool_unit_pin(&root, unit_id, pin_id);
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-symbol-pin-anchor",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--pin",
            &pin_id.to_string(),
            "--x-nm",
            "100",
            "--y-nm",
            "200",
            "--orientation",
            "Left",
            "--length-nm",
            "2540000",
            "--decoration",
            "inverted",
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol pin anchor set should succeed");
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(payload["pin_anchors"][0]["pin"], pin_id.to_string());
    assert_eq!(payload["pin_anchors"][0]["position"]["x"], 100);
    assert_eq!(payload["pin_anchors"][0]["position"]["y"], 200);
    assert_eq!(payload["pin_anchors"][0]["orientation"], "Left");
    assert_eq!(payload["pin_anchors"][0]["length_nm"], 2540000);
    assert_eq!(payload["pin_anchors"][0]["decoration"], "inverted");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_symbol_pin_anchor_rejects_missing_unit_pin() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-pin-anchor-invalid");
    create_native_project(&root, Some("Pool Symbol Pin Anchor Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let missing_pin = Uuid::new_v4();
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-symbol-pin-anchor",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--pin",
            &missing_pin.to_string(),
            "--x-nm",
            "100",
            "--y-nm",
            "200",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing unit pin should fail");
    assert!(format!("{error:#}").contains("has no pin"));
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert!(
        payload
            .get("pin_anchors")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}
