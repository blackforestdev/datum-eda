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
fn project_create_pool_symbol_authors_typed_symbol_for_existing_unit() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol");
    create_native_project(&root, Some("Pool Symbol".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    let output = execute(
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
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("create-symbol report JSON should parse");
    assert_eq!(report["action"], "create_symbol");
    assert_eq!(report["object_kind"], "symbols");
    assert_eq!(
        report["relative_path"],
        format!("pool/symbols/{symbol_id}.json")
    );
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(payload["name"], "OpAmpSymbol");
    assert_eq!(payload["unit"], unit_id.to_string());
    assert!(payload.get("drawings").is_none());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_symbol_rejects_missing_unit() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-missing-unit");
    create_native_project(&root, Some("Pool Symbol Missing Unit".to_string()))
        .expect("initial scaffold should succeed");
    let symbol_id = Uuid::new_v4();
    let missing_unit_id = Uuid::new_v4();
    let error = execute(
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
            &missing_unit_id.to_string(),
            "--name",
            "DanglingSymbol",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("symbol with missing unit should fail");
    assert!(format!("{error:#}").contains("missing pool unit"));
    assert!(!root.join(format!("pool/symbols/{symbol_id}.json")).exists());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_line_authors_symbol_drawing_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-line");
    create_native_project(&root, Some("Pool Symbol Line".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-line",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--from-x-nm",
            "0",
            "--from-y-nm",
            "0",
            "--to-x-nm",
            "1000",
            "--to-y-nm",
            "0",
            "--width-nm",
            "100",
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol line add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("symbol line report JSON should parse");
    assert_eq!(report["action"], "add_symbol_line");
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(
        payload["drawings"]
            .as_array()
            .expect("drawings should be array")
            .len(),
        1
    );
    assert_eq!(payload["drawings"][0]["Line"]["to"]["x"], 1000);
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
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_line_rejects_invalid_inputs() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-line-invalid");
    create_native_project(&root, Some("Pool Symbol Line Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-line",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--from-x-nm",
            "0",
            "--from-y-nm",
            "0",
            "--to-x-nm",
            "0",
            "--to-y-nm",
            "0",
            "--width-nm",
            "100",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-length line should fail");
    assert!(format!("{error:#}").contains("distinct endpoints"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-line",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--from-x-nm",
            "0",
            "--from-y-nm",
            "0",
            "--to-x-nm",
            "100",
            "--to-y-nm",
            "0",
            "--width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("nonpositive width should fail");
    assert!(format!("{error:#}").contains("width-nm must be positive"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-line",
            root.to_str().unwrap(),
            "--symbol",
            &Uuid::new_v4().to_string(),
            "--from-x-nm",
            "0",
            "--from-y-nm",
            "0",
            "--to-x-nm",
            "100",
            "--to-y-nm",
            "0",
            "--width-nm",
            "100",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing symbol should fail");
    assert!(format!("{error:#}").contains("missing pool symbol"));
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert!(
        payload
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_polygon_authors_closed_polygon_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-polygon");
    create_native_project(&root, Some("Pool Symbol Polygon".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-polygon",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--vertices",
            "0,0;1000,0;1000,2000",
            "--closed",
            "true",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol polygon add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("symbol polygon report JSON should parse");
    assert_eq!(report["action"], "add_symbol_polygon");
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(
        payload["drawings"]
            .as_array()
            .expect("drawings should be array")
            .len(),
        1
    );
    assert_eq!(
        payload["drawings"][0]["Polygon"]["polygon"]["vertices"]
            .as_array()
            .expect("vertices should be array")
            .len(),
        3
    );
    assert_eq!(
        payload["drawings"][0]["Polygon"]["polygon"]["vertices"][1]["x"],
        1000
    );
    assert_eq!(payload["drawings"][0]["Polygon"]["polygon"]["closed"], true);
    assert_eq!(payload["drawings"][0]["Polygon"]["width"], 150);
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
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_polygon_authors_open_polyline() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-polyline");
    create_native_project(&root, Some("Pool Symbol Polyline".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-polygon",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--vertices",
            "0,0;1000,0",
            "--closed",
            "false",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol polyline add should succeed");
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(
        payload["drawings"][0]["Polygon"]["polygon"]["vertices"]
            .as_array()
            .expect("vertices should be array")
            .len(),
        2
    );
    assert_eq!(
        payload["drawings"][0]["Polygon"]["polygon"]["closed"],
        false
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_polygon_rejects_invalid_inputs() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-polygon-invalid");
    create_native_project(&root, Some("Pool Symbol Polygon Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-polygon",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--vertices",
            "0,0;1000,0",
            "--closed",
            "true",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("two-point closed polygon should fail");
    assert!(format!("{error:#}").contains("closed polygon must have at least 3 vertices"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-polygon",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--vertices",
            "0,0",
            "--closed",
            "false",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("one-point open polyline should fail");
    assert!(format!("{error:#}").contains("polyline must have at least 2 vertices"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-polygon",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--vertices",
            "0,0;1000",
            "--closed",
            "false",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("malformed vertex should fail");
    assert!(format!("{error:#}").contains("missing y coordinate"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-polygon",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--vertices",
            "0,0;1000,0",
            "--closed",
            "false",
            "--width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-width polygon should fail");
    assert!(format!("{error:#}").contains("width-nm must be positive"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-polygon",
            root.to_str().unwrap(),
            "--symbol",
            &Uuid::new_v4().to_string(),
            "--vertices",
            "0,0;1000,0",
            "--closed",
            "false",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing symbol should fail");
    assert!(format!("{error:#}").contains("missing pool symbol"));
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert!(
        payload
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_rect_authors_symbol_drawing_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-rect");
    create_native_project(&root, Some("Pool Symbol Rect".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-rect",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--min-x-nm",
            "1000",
            "--min-y-nm",
            "2000",
            "--max-x-nm",
            "3000",
            "--max-y-nm",
            "4000",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol rect add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("symbol rect report JSON should parse");
    assert_eq!(report["action"], "add_symbol_rect");
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(
        payload["drawings"]
            .as_array()
            .expect("drawings should be array")
            .len(),
        1
    );
    assert_eq!(payload["drawings"][0]["Rect"]["min"]["x"], 1000);
    assert_eq!(payload["drawings"][0]["Rect"]["max"]["y"], 4000);
    assert_eq!(payload["drawings"][0]["Rect"]["width"], 150);
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
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_rect_rejects_invalid_inputs() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-rect-invalid");
    create_native_project(&root, Some("Pool Symbol Rect Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-rect",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--min-x-nm",
            "1000",
            "--min-y-nm",
            "2000",
            "--max-x-nm",
            "1000",
            "--max-y-nm",
            "4000",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("degenerate x rect should fail");
    assert!(format!("{error:#}").contains("min-x-nm must be less than max-x-nm"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-rect",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--min-x-nm",
            "1000",
            "--min-y-nm",
            "2000",
            "--max-x-nm",
            "3000",
            "--max-y-nm",
            "2000",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("degenerate y rect should fail");
    assert!(format!("{error:#}").contains("min-y-nm must be less than max-y-nm"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-rect",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--min-x-nm",
            "1000",
            "--min-y-nm",
            "2000",
            "--max-x-nm",
            "3000",
            "--max-y-nm",
            "4000",
            "--width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("nonpositive width should fail");
    assert!(format!("{error:#}").contains("width-nm must be positive"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-rect",
            root.to_str().unwrap(),
            "--symbol",
            &Uuid::new_v4().to_string(),
            "--min-x-nm",
            "1000",
            "--min-y-nm",
            "2000",
            "--max-x-nm",
            "3000",
            "--max-y-nm",
            "4000",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing symbol should fail");
    assert!(format!("{error:#}").contains("missing pool symbol"));
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert!(
        payload
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_circle_authors_symbol_drawing_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-circle");
    create_native_project(&root, Some("Pool Symbol Circle".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-circle",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--center-x-nm",
            "1000",
            "--center-y-nm",
            "2000",
            "--radius-nm",
            "3000",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol circle add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("symbol circle report JSON should parse");
    assert_eq!(report["action"], "add_symbol_circle");
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(
        payload["drawings"]
            .as_array()
            .expect("drawings should be array")
            .len(),
        1
    );
    assert_eq!(payload["drawings"][0]["Circle"]["center"]["x"], 1000);
    assert_eq!(payload["drawings"][0]["Circle"]["center"]["y"], 2000);
    assert_eq!(payload["drawings"][0]["Circle"]["radius"], 3000);
    assert_eq!(payload["drawings"][0]["Circle"]["width"], 150);
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
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_circle_rejects_invalid_inputs() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-circle-invalid");
    create_native_project(&root, Some("Pool Symbol Circle Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-circle",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--center-x-nm",
            "1000",
            "--center-y-nm",
            "2000",
            "--radius-nm",
            "0",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("nonpositive radius should fail");
    assert!(format!("{error:#}").contains("radius-nm must be positive"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-circle",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--center-x-nm",
            "1000",
            "--center-y-nm",
            "2000",
            "--radius-nm",
            "3000",
            "--width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("nonpositive width should fail");
    assert!(format!("{error:#}").contains("width-nm must be positive"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-circle",
            root.to_str().unwrap(),
            "--symbol",
            &Uuid::new_v4().to_string(),
            "--center-x-nm",
            "1000",
            "--center-y-nm",
            "2000",
            "--radius-nm",
            "3000",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing symbol should fail");
    assert!(format!("{error:#}").contains("missing pool symbol"));
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert!(
        payload
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_arc_authors_symbol_drawing_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-arc");
    create_native_project(&root, Some("Pool Symbol Arc".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-arc",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--radius-nm",
            "3000",
            "--start-angle",
            "0",
            "--end-angle",
            "900",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol arc add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("symbol arc report JSON should parse");
    assert_eq!(report["action"], "add_symbol_arc");
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(
        payload["drawings"]
            .as_array()
            .expect("drawings should be array")
            .len(),
        1
    );
    assert_eq!(payload["drawings"][0]["Arc"]["arc"]["center"]["x"], 1000);
    assert_eq!(payload["drawings"][0]["Arc"]["arc"]["center"]["y"], 2000);
    assert_eq!(payload["drawings"][0]["Arc"]["arc"]["radius"], 3000);
    assert_eq!(payload["drawings"][0]["Arc"]["arc"]["start_angle"], 0);
    assert_eq!(payload["drawings"][0]["Arc"]["arc"]["end_angle"], 900);
    assert_eq!(payload["drawings"][0]["Arc"]["width"], 150);
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
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_arc_rejects_invalid_inputs() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-arc-invalid");
    create_native_project(&root, Some("Pool Symbol Arc Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-arc",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--radius-nm",
            "0",
            "--start-angle",
            "0",
            "--end-angle",
            "900",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("nonpositive radius should fail");
    assert!(format!("{error:#}").contains("radius-nm must be positive"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-arc",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--radius-nm",
            "3000",
            "--start-angle",
            "0",
            "--end-angle",
            "900",
            "--width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("nonpositive width should fail");
    assert!(format!("{error:#}").contains("width-nm must be positive"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-arc",
            root.to_str().unwrap(),
            "--symbol",
            &Uuid::new_v4().to_string(),
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--radius-nm",
            "3000",
            "--start-angle",
            "0",
            "--end-angle",
            "900",
            "--width-nm",
            "150",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing symbol should fail");
    assert!(format!("{error:#}").contains("missing pool symbol"));
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert!(
        payload
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_text_authors_symbol_drawing_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-text");
    create_native_project(&root, Some("Pool Symbol Text".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-text",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--text",
            "REF**",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--rotation",
            "900",
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol text add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("symbol text report JSON should parse");
    assert_eq!(report["action"], "add_symbol_text");
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert_eq!(
        payload["drawings"]
            .as_array()
            .expect("drawings should be array")
            .len(),
        1
    );
    assert_eq!(payload["drawings"][0]["Text"]["text"], "REF**");
    assert_eq!(payload["drawings"][0]["Text"]["position"]["x"], 1000);
    assert_eq!(payload["drawings"][0]["Text"]["position"]["y"], 2000);
    assert_eq!(payload["drawings"][0]["Text"]["rotation"], 900);
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
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_symbol_text_rejects_invalid_inputs() {
    let root = unique_project_root("datum-eda-cli-project-pool-symbol-text-invalid");
    create_native_project(&root, Some("Pool Symbol Text Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-text",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--text",
            "   ",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("blank text should fail");
    assert!(format!("{error:#}").contains("text must not be empty"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-symbol-text",
            root.to_str().unwrap(),
            "--symbol",
            &Uuid::new_v4().to_string(),
            "--text",
            "REF**",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing symbol should fail");
    assert!(format!("{error:#}").contains("missing pool symbol"));
    let payload = query_pool_object_payload(&root, "symbols", symbol_id);
    assert!(
        payload
            .get("drawings")
            .and_then(serde_json::Value::as_array)
            .map(Vec::is_empty)
            .unwrap_or(true)
    );
    let _ = std::fs::remove_dir_all(&root);
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

#[test]
fn project_create_pool_entity_authors_typed_entity_for_symbol_unit_pair() {
    let root = unique_project_root("datum-eda-cli-project-pool-entity");
    create_native_project(&root, Some("Pool Entity".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    create_typed_pool_unit(&root, unit_id);
    create_typed_pool_symbol(&root, symbol_id, unit_id);
    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-entity",
            root.to_str().unwrap(),
            "--entity",
            &entity_id.to_string(),
            "--gate",
            &gate_id.to_string(),
            "--unit",
            &unit_id.to_string(),
            "--symbol",
            &symbol_id.to_string(),
            "--name",
            "DualOpAmp",
            "--prefix",
            "U",
            "--manufacturer",
            "Datum",
            "--gate-name",
            "A",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool entity create should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("create-entity report JSON should parse");
    assert_eq!(report["action"], "create_entity");
    assert_eq!(report["object_kind"], "entities");
    assert_eq!(
        report["relative_path"],
        format!("pool/entities/{entity_id}.json")
    );
    let payload = query_pool_object_payload(&root, "entities", entity_id);
    assert_eq!(payload["prefix"], "U");
    assert_eq!(
        payload["gates"][gate_id.to_string()]["symbol"],
        symbol_id.to_string()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_entity_rejects_symbol_unit_mismatch() {
    let root = unique_project_root("datum-eda-cli-project-pool-entity-symbol-unit");
    create_native_project(&root, Some("Pool Entity Symbol Unit".to_string()))
        .expect("initial scaffold should succeed");
    let unit_id = Uuid::new_v4();
    let other_unit_id = Uuid::new_v4();
    for unit in [unit_id, other_unit_id] {
        create_typed_pool_unit(&root, unit);
    }
    let symbol_id = Uuid::new_v4();
    create_typed_pool_symbol(&root, symbol_id, other_unit_id);
    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-entity",
            root.to_str().unwrap(),
            "--entity",
            &entity_id.to_string(),
            "--gate",
            &gate_id.to_string(),
            "--unit",
            &unit_id.to_string(),
            "--symbol",
            &symbol_id.to_string(),
            "--name",
            "BadEntity",
            "--prefix",
            "U",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("entity with mismatched symbol unit should fail");
    assert!(format!("{error:#}").contains("does not reference unit"));
    assert!(
        !root
            .join(format!("pool/entities/{entity_id}.json"))
            .exists()
    );
    let _ = std::fs::remove_dir_all(&root);
}
