use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn execute(cli: Cli) -> Result<String> {
    let (output, _) = execute_with_exit_code(cli)?;
    Ok(output)
}

fn create_typed_pool_padstack(root: &Path, padstack_id: Uuid) {
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-padstack",
            root.to_str().unwrap(),
            "--padstack",
            &padstack_id.to_string(),
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
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-package",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--name",
            "SOT23",
            "--pad",
            &pad_id.to_string(),
            "--padstack",
            &padstack_id.to_string(),
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
}

fn create_typed_pool_footprint(root: &Path, footprint_id: Uuid, package_id: Uuid) {
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-footprint",
            root.to_str().unwrap(),
            "--footprint",
            &footprint_id.to_string(),
            "--package",
            &package_id.to_string(),
            "--name",
            "SOT23_LandPattern",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool footprint create should succeed");
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
fn project_set_pool_package_courtyard_rect_authors_footprint_geometry_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-courtyard");
    create_native_project(&root, Some("Pool Package Courtyard".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id, footprint_id) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    create_typed_pool_footprint(&root, footprint_id, package_id);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-package-courtyard-rect",
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
        ])
        .expect("CLI should parse"),
    )
    .expect("courtyard rect set should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("set courtyard report JSON should parse");
    assert_eq!(report["action"], "set_package_courtyard_rect");
    assert_eq!(report["object_kind"], "footprints");
    assert_eq!(report["object_uuid"], footprint_id.to_string());

    let package_payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        package_payload["courtyard"]["vertices"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["courtyard"]["vertices"].as_array().unwrap().len(),
        4
    );
    assert_eq!(payload["courtyard"]["vertices"][0]["x"], 1000);
    assert_eq!(payload["courtyard"]["vertices"][2]["y"], 4000);

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
    .expect("courtyard undo should succeed");
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["courtyard"]["vertices"].as_array().unwrap().len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_package_courtyard_rect_rejects_zero_area() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-courtyard-invalid");
    create_native_project(&root, Some("Pool Package Courtyard Invalid".to_string()))
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
            "set-pool-package-courtyard-rect",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--min-x-nm",
            "1000",
            "--min-y-nm",
            "0",
            "--max-x-nm",
            "1000",
            "--max-y-nm",
            "4000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-width courtyard should fail");
    assert!(format!("{error:#}").contains("min-x-nm must be less than max-x-nm"));
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["courtyard"]["vertices"].as_array().unwrap().len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_package_courtyard_polygon_authors_footprint_geometry_and_undoes() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-courtyard-polygon");
    create_native_project(&root, Some("Pool Package Courtyard Polygon".to_string()))
        .expect("initial scaffold should succeed");
    let (padstack_id, package_id, footprint_id) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    create_typed_pool_padstack(&root, padstack_id);
    create_typed_pool_package(&root, package_id, padstack_id);
    create_typed_pool_footprint(&root, footprint_id, package_id);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-package-courtyard-polygon",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--vertices",
            "0,0;1000,0;1000,2000;0,2000",
        ])
        .expect("CLI should parse"),
    )
    .expect("courtyard polygon set should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("set courtyard polygon report JSON should parse");
    assert_eq!(report["action"], "set_package_courtyard_polygon");
    assert_eq!(report["object_kind"], "footprints");
    assert_eq!(report["object_uuid"], footprint_id.to_string());

    let package_payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        package_payload["courtyard"]["vertices"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["courtyard"]["vertices"].as_array().unwrap().len(),
        4
    );
    assert_eq!(payload["courtyard"]["vertices"][1]["x"], 1000);
    assert_eq!(payload["courtyard"]["vertices"][2]["y"], 2000);

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
    .expect("courtyard polygon undo should succeed");
    let payload = query_pool_object_payload(&root, "footprints", footprint_id);
    assert_eq!(
        payload["courtyard"]["vertices"].as_array().unwrap().len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_package_courtyard_polygon_rejects_invalid_inputs() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-courtyard-polygon-invalid");
    create_native_project(
        &root,
        Some("Pool Package Courtyard Polygon Invalid".to_string()),
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
            "set-pool-package-courtyard-polygon",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--vertices",
            "0,0;1000,0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("two-point courtyard polygon should fail");
    assert!(format!("{error:#}").contains("courtyard polygon must have at least 3 vertices"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-package-courtyard-polygon",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--vertices",
            "0,0;1000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("malformed courtyard vertex should fail");
    assert!(format!("{error:#}").contains("missing y coordinate"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-package-courtyard-polygon",
            root.to_str().unwrap(),
            "--pool",
            "../pool",
            "--package",
            &package_id.to_string(),
            "--vertices",
            "0,0;1000,0;1000,2000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("non-project-local pool path should fail");
    assert!(format!("{error:#}").contains("must not contain parent-directory components"));
    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-package-courtyard-polygon",
            root.to_str().unwrap(),
            "--package",
            &Uuid::new_v4().to_string(),
            "--vertices",
            "0,0;1000,0;1000,2000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing package should fail");
    assert!(format!("{error:#}").contains("missing pool package"));
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["courtyard"]["vertices"].as_array().unwrap().len(),
        0
    );
    let _ = std::fs::remove_dir_all(&root);
}
