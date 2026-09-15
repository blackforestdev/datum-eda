use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn execute(cli: Cli) -> Result<String> {
    let (output, _) = execute_with_exit_code(cli)?;
    Ok(output)
}

fn create_typed_pool_padstack(root: &Path, padstack_id: Uuid) {
    let padstack = padstack_id.to_string();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-padstack",
            root.to_str().unwrap(),
            "--padstack",
            &padstack,
            "--name",
            "RoundViaPad",
            "--aperture",
            "circle",
            "--diameter-nm",
            "1200000",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool padstack create should succeed");
}

fn create_typed_pool_package(root: &Path, package_id: Uuid, padstack_id: Uuid) {
    let pad_id = Uuid::new_v4();
    let (package, pad, padstack) = (
        package_id.to_string(),
        pad_id.to_string(),
        padstack_id.to_string(),
    );
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-package",
            root.to_str().unwrap(),
            "--package",
            &package,
            "--name",
            "SOT23",
            "--pad",
            &pad,
            "--padstack",
            &padstack,
            "--pad-name",
            "1",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
            "--layer",
            "1",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool package create should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-footprint",
            root.to_str().unwrap(),
            "--footprint",
            &package_footprint_id(package_id).to_string(),
            "--package",
            &package_id.to_string(),
            "--name",
            "SOT23_LandPattern",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool footprint create should succeed");
}

fn package_footprint_id(package_id: Uuid) -> Uuid {
    Uuid::new_v5(&package_id, b"package-geometry-test-footprint")
}

fn query_pool_object_payload(root: &Path, kind: &str, object_id: Uuid) -> serde_json::Value {
    let object = object_id.to_string();
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
            &object,
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
fn project_add_pool_package_silkscreen_line_authors_primitive_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-line");
    create_native_project(&root, Some("Pool Package Silkscreen".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-line",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--from-x-nm",
            "1000",
            "--from-y-nm",
            "2000",
            "--to-x-nm",
            "3000",
            "--to-y-nm",
            "4000",
            "--width-nm",
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect("silkscreen line add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("add silkscreen report JSON should parse");
    assert_eq!(report["action"], "add_package_silkscreen_line");
    let footprint_id = package_footprint_id(package_id);
    assert_eq!(report["object_kind"], "footprints");
    assert_eq!(report["object_uuid"], footprint_id.to_string());
    let package_payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(package_payload["silkscreen"].as_array().unwrap().len(), 0);
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        1
    );
    assert_eq!(payload["silkscreen"][0]["Line"]["from"]["x"], 1000);
    assert_eq!(payload["silkscreen"][0]["Line"]["to"]["y"], 4000);
    assert_eq!(payload["silkscreen"][0]["Line"]["width"], 150000);

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
    .expect("silkscreen undo should succeed");
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_line_rejects_degenerate_geometry() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-line-invalid");
    create_native_project(&root, Some("Pool Package Silkscreen Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-line",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--from-x-nm",
            "1000",
            "--from-y-nm",
            "2000",
            "--to-x-nm",
            "1000",
            "--to-y-nm",
            "2000",
            "--width-nm",
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-length silkscreen line should fail");
    assert!(format!("{error:#}").contains("distinct endpoints"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-line",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--from-x-nm",
            "1000",
            "--from-y-nm",
            "2000",
            "--to-x-nm",
            "3000",
            "--to-y-nm",
            "4000",
            "--width-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-width silkscreen line should fail");
    assert!(format!("{error:#}").contains("width-nm must be positive"));
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_rect_authors_primitive_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-rect");
    create_native_project(&root, Some("Pool Package Silkscreen Rect".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-rect",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--min-x-nm",
            "1000",
            "--min-y-nm",
            "2000",
            "--max-x-nm",
            "3000",
            "--max-y-nm",
            "4000",
            "--width-nm",
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect("silkscreen rect add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("add silkscreen rect report JSON should parse");
    assert_eq!(report["action"], "add_package_silkscreen_rect");
    let footprint_id = package_footprint_id(package_id);
    assert_eq!(report["object_kind"], "footprints");
    assert_eq!(report["object_uuid"], footprint_id.to_string());
    let package_payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(package_payload["silkscreen"].as_array().unwrap().len(), 0);
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        1
    );
    assert_eq!(payload["silkscreen"][0]["Rect"]["min"]["x"], 1000);
    assert_eq!(payload["silkscreen"][0]["Rect"]["max"]["y"], 4000);
    assert_eq!(payload["silkscreen"][0]["Rect"]["width"], 150000);

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
    .expect("silkscreen rect undo should succeed");
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_rect_rejects_degenerate_geometry() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-rect-invalid");
    create_native_project(
        &root,
        Some("Pool Package Silkscreen Rect Invalid".to_string()),
    )
    .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-rect",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--min-x-nm",
            "1000",
            "--min-y-nm",
            "2000",
            "--max-x-nm",
            "1000",
            "--max-y-nm",
            "4000",
            "--width-nm",
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-width silkscreen rect should fail");
    assert!(format!("{error:#}").contains("min-x-nm must be less than max-x-nm"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-rect",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
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
    .expect_err("zero-width stroke should fail");
    assert!(format!("{error:#}").contains("width-nm must be positive"));
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_circle_authors_primitive_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-circle");
    create_native_project(&root, Some("Pool Package Silkscreen Circle".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-circle",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--center-x-nm",
            "1000",
            "--center-y-nm",
            "2000",
            "--radius-nm",
            "3000",
            "--width-nm",
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect("silkscreen circle add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("add silkscreen circle report JSON should parse");
    assert_eq!(report["action"], "add_package_silkscreen_circle");
    let footprint_id = package_footprint_id(package_id);
    assert_eq!(report["object_kind"], "footprints");
    assert_eq!(report["object_uuid"], footprint_id.to_string());
    let package_payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(package_payload["silkscreen"].as_array().unwrap().len(), 0);
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        1
    );
    assert_eq!(payload["silkscreen"][0]["Circle"]["center"]["x"], 1000);
    assert_eq!(payload["silkscreen"][0]["Circle"]["radius"], 3000);
    assert_eq!(payload["silkscreen"][0]["Circle"]["width"], 150000);

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
    .expect("silkscreen circle undo should succeed");
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_circle_rejects_degenerate_geometry() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-circle-invalid");
    create_native_project(
        &root,
        Some("Pool Package Silkscreen Circle Invalid".to_string()),
    )
    .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-circle",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--center-x-nm",
            "1000",
            "--center-y-nm",
            "2000",
            "--radius-nm",
            "0",
            "--width-nm",
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-radius silkscreen circle should fail");
    assert!(format!("{error:#}").contains("radius-nm must be positive"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-circle",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
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
    .expect_err("zero-width circle stroke should fail");
    assert!(format!("{error:#}").contains("width-nm must be positive"));
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_arc_authors_primitive_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-arc");
    create_native_project(&root, Some("Pool Package Silkscreen Arc".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-arc",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
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
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect("silkscreen arc add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("add silkscreen arc report JSON should parse");
    assert_eq!(report["action"], "add_package_silkscreen_arc");
    let footprint_id = package_footprint_id(package_id);
    assert_eq!(report["object_kind"], "footprints");
    assert_eq!(report["object_uuid"], footprint_id.to_string());
    let package_payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(package_payload["silkscreen"].as_array().unwrap().len(), 0);
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        1
    );
    assert_eq!(payload["silkscreen"][0]["Arc"]["arc"]["center"]["x"], 1000);
    assert_eq!(payload["silkscreen"][0]["Arc"]["arc"]["radius"], 3000);
    assert_eq!(payload["silkscreen"][0]["Arc"]["arc"]["end_angle"], 900);
    assert_eq!(payload["silkscreen"][0]["Arc"]["width"], 150000);

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
    .expect("silkscreen arc undo should succeed");
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_arc_rejects_degenerate_geometry() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-arc-invalid");
    create_native_project(
        &root,
        Some("Pool Package Silkscreen Arc Invalid".to_string()),
    )
    .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-arc",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
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
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-radius silkscreen arc should fail");
    assert!(format!("{error:#}").contains("radius-nm must be positive"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-arc",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
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
    .expect_err("zero-width arc stroke should fail");
    assert!(format!("{error:#}").contains("width-nm must be positive"));
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_polygon_authors_closed_polygon_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-polygon");
    create_native_project(&root, Some("Pool Package Silkscreen Polygon".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-polygon",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--vertices",
            "0,0;1000,0;1000,2000",
            "--closed",
            "true",
            "--width-nm",
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect("silkscreen polygon add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("add silkscreen polygon report JSON should parse");
    assert_eq!(report["action"], "add_package_silkscreen_polygon");
    let footprint_id = package_footprint_id(package_id);
    assert_eq!(report["object_kind"], "footprints");
    assert_eq!(report["object_uuid"], footprint_id.to_string());
    let package_payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(package_payload["silkscreen"].as_array().unwrap().len(), 0);
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        1
    );
    assert_eq!(
        payload["silkscreen"][0]["Polygon"]["polygon"]["closed"],
        true
    );
    assert_eq!(
        payload["silkscreen"][0]["Polygon"]["polygon"]["vertices"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        payload["silkscreen"][0]["Polygon"]["polygon"]["vertices"][2]["y"],
        2000
    );
    assert_eq!(payload["silkscreen"][0]["Polygon"]["width"], 150000);

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
    .expect("silkscreen polygon undo should succeed");
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_polygon_authors_open_polyline() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-polyline");
    create_native_project(&root, Some("Pool Package Silkscreen Polyline".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-polygon",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--vertices",
            "0,0;1000,0",
            "--closed",
            "false",
            "--width-nm",
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect("silkscreen polyline add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("add silkscreen polyline report JSON should parse");
    assert_eq!(report["action"], "add_package_silkscreen_polygon");
    let footprint_id = package_footprint_id(package_id);
    assert_eq!(report["object_kind"], "footprints");
    assert_eq!(report["object_uuid"], footprint_id.to_string());
    let package_payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(package_payload["silkscreen"].as_array().unwrap().len(), 0);
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        1
    );
    assert_eq!(
        payload["silkscreen"][0]["Polygon"]["polygon"]["closed"],
        false
    );
    assert_eq!(
        payload["silkscreen"][0]["Polygon"]["polygon"]["vertices"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_polygon_rejects_degenerate_geometry() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-polygon-invalid");
    create_native_project(
        &root,
        Some("Pool Package Silkscreen Polygon Invalid".to_string()),
    )
    .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-polygon",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--vertices",
            "0,0;1000,0",
            "--closed",
            "true",
            "--width-nm",
            "150000",
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
            "add-pool-package-silkscreen-polygon",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--vertices",
            "0,0;1000",
            "--closed",
            "false",
            "--width-nm",
            "150000",
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
            "add-pool-package-silkscreen-polygon",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
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
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_text_authors_primitive_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-text");
    create_native_project(&root, Some("Pool Package Silkscreen Text".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-text",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
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
    .expect("silkscreen text add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("add silkscreen text report JSON should parse");
    assert_eq!(report["action"], "add_package_silkscreen_text");
    let footprint_id = package_footprint_id(package_id);
    assert_eq!(report["object_kind"], "footprints");
    assert_eq!(report["object_uuid"], footprint_id.to_string());
    let package_payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(package_payload["silkscreen"].as_array().unwrap().len(), 0);
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        1
    );
    assert_eq!(payload["silkscreen"][0]["Text"]["text"], "REF**");
    assert_eq!(payload["silkscreen"][0]["Text"]["position"]["x"], 1000);
    assert_eq!(payload["silkscreen"][0]["Text"]["rotation"], 900);

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
    .expect("silkscreen text undo should succeed");
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_silkscreen_text_rejects_blank_text() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-silkscreen-text-invalid");
    create_native_project(
        &root,
        Some("Pool Package Silkscreen Text Invalid".to_string()),
    )
    .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-silkscreen-text",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--text",
            "   ",
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("blank silkscreen text should fail");
    assert!(format!("{error:#}").contains("text must not be empty"));
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["silkscreen"]
            .as_array()
            .expect("silkscreen should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_model_3d_authors_model_ref_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-model-3d");
    create_native_project(&root, Some("Pool Package Model 3D".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-model-3d",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--model-path",
            "models/sot23.step",
            "--tx-nm",
            "1",
            "--ty-nm",
            "2",
            "--tz-nm",
            "3",
            "--yaw-tenths-deg",
            "900",
            "--scale",
            "1.25",
        ])
        .expect("CLI should parse"),
    )
    .expect("model add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("add model report JSON should parse");
    assert_eq!(report["action"], "add_package_model_3d");
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["models_3d"]
            .as_array()
            .expect("models should be array")
            .len(),
        1
    );
    assert_eq!(payload["models_3d"][0]["path"], "models/sot23.step");
    assert_eq!(payload["models_3d"][0]["format"], "Step");
    assert_eq!(
        payload["models_3d"][0]["transform"]["translation_nm"]["z"],
        3
    );
    assert_eq!(
        payload["models_3d"][0]["transform"]["rotation_tenths_deg"]["yaw_tenths_deg"],
        900
    );
    assert_eq!(payload["models_3d"][0]["transform"]["scale"], 1.25);

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
    .expect("model undo should succeed");
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["models_3d"]
            .as_array()
            .expect("models should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_package_model_3d_rejects_invalid_inputs() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-model-3d-invalid");
    create_native_project(&root, Some("Pool Package Model 3D Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-model-3d",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--model-path",
            "../sot23.step",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("traversal model path should fail");
    assert!(format!("{error:#}").contains("must not contain traversal"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-model-3d",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--model-path",
            "/tmp/sot23.step",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("absolute model path should fail");
    assert!(format!("{error:#}").contains("must be relative"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-model-3d",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--model-path",
            "models/sot23.step",
            "--format",
            "Unknown",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("bad model format should fail");
    assert!(format!("{error:#}").contains("ModelFormat"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-model-3d",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--model-path",
            "models/sot23.unknown",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("uninferable model format should fail");
    assert!(format!("{error:#}").contains("could not be inferred"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-package-model-3d",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--model-path",
            "models/sot23.step",
            "--scale",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("nonpositive scale should fail");
    assert!(format!("{error:#}").contains("scale must be a positive JSON number"));
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["models_3d"]
            .as_array()
            .expect("models should be array")
            .len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_package_body_heights_updates_clears_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-body-heights");
    create_native_project(&root, Some("Pool Package Body Heights".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-package-body-heights",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--body-height-nm",
            "1750000",
            "--body-height-mounted-nm",
            "1850000",
        ])
        .expect("CLI should parse"),
    )
    .expect("body heights set should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("body-height report JSON should parse");
    assert_eq!(report["action"], "set_package_body_heights");
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(payload["body_height_nm"], 1_750_000);
    assert_eq!(payload["body_height_mounted_nm"], 1_850_000);
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-package-body-heights",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--clear",
        ])
        .expect("CLI should parse"),
    )
    .expect("body heights clear should succeed");
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert!(payload["body_height_nm"].is_null());
    assert!(payload["body_height_mounted_nm"].is_null());
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
    .expect("body heights clear undo should succeed");
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(payload["body_height_nm"], 1_750_000);
    assert_eq!(payload["body_height_mounted_nm"], 1_850_000);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_package_body_heights_rejects_invalid_requests() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-body-heights-invalid");
    create_native_project(&root, Some("Pool Package Body Heights Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id) = (Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-package-body-heights",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("empty body-height request should fail");
    assert!(format!("{error:#}").contains("requires --clear"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-package-body-heights",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--body-height-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("nonpositive body height should fail");
    assert!(format!("{error:#}").contains("body-height-nm must be positive"));
    let _ = std::fs::remove_dir_all(&root);
}
