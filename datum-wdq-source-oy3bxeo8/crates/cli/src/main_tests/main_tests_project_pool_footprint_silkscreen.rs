use super::*;

#[test]
fn project_add_pool_footprint_silkscreen_line_authors_primitive_and_undoes() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-silkscreen-line",
    );
    create_native_project(&root, Some("Footprint Silkscreen Line".to_string()))
        .expect("initial scaffold should succeed");
    main_tests_project_pool_footprint::create_footprint_fixture(&root);

    let output = execute_footprint_silkscreen_line(&root)
        .expect("footprint silkscreen line add should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("add silkscreen report JSON should parse");
    assert_eq!(report["action"], "add_footprint_silkscreen_line");

    let payload = footprint_payload(&root);
    assert_eq!(payload["silkscreen"].as_array().unwrap().len(), 1);
    assert_eq!(payload["silkscreen"][0]["Line"]["from"]["x"], 1000);
    assert_eq!(payload["silkscreen"][0]["Line"]["to"]["y"], 4000);
    assert_eq!(payload["silkscreen"][0]["Line"]["width"], 150000);

    undo_last(&root);
    assert_eq!(
        footprint_payload(&root)["silkscreen"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_footprint_silkscreen_rect_circle_polygon_author_primitives() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-silkscreen-shapes",
    );
    create_native_project(&root, Some("Footprint Silkscreen Shapes".to_string()))
        .expect("initial scaffold should succeed");
    main_tests_project_pool_footprint::create_footprint_fixture(&root);

    let rect = execute_footprint_silkscreen_rect(&root)
        .expect("footprint silkscreen rect add should succeed");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&rect).unwrap()["action"],
        "add_footprint_silkscreen_rect"
    );
    execute_footprint_silkscreen_circle(&root)
        .expect("footprint silkscreen circle add should succeed");
    execute_footprint_silkscreen_polygon(&root, true)
        .expect("footprint silkscreen polygon add should succeed");

    let payload = footprint_payload(&root);
    let silkscreen = payload["silkscreen"].as_array().unwrap();
    assert_eq!(silkscreen.len(), 3);
    assert_eq!(silkscreen[0]["Rect"]["min"]["x"], 1000);
    assert_eq!(silkscreen[0]["Rect"]["max"]["y"], 4000);
    assert_eq!(silkscreen[1]["Circle"]["center"]["y"], 6000);
    assert_eq!(silkscreen[1]["Circle"]["radius"], 7000);
    assert_eq!(
        silkscreen[2]["Polygon"]["polygon"]["vertices"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(silkscreen[2]["Polygon"]["polygon"]["closed"], true);

    undo_last(&root);
    assert_eq!(
        footprint_payload(&root)["silkscreen"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_footprint_silkscreen_polyline_authors_open_polygon_primitive() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-silkscreen-polyline",
    );
    create_native_project(&root, Some("Footprint Silkscreen Polyline".to_string()))
        .expect("initial scaffold should succeed");
    main_tests_project_pool_footprint::create_footprint_fixture(&root);

    execute_footprint_silkscreen_polygon(&root, false)
        .expect("footprint silkscreen polyline add should succeed");
    let payload = footprint_payload(&root);
    assert_eq!(
        payload["silkscreen"][0]["Polygon"]["polygon"]["closed"],
        false
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_pool_footprint_silkscreen_rejects_invalid_geometry() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-silkscreen-invalid",
    );
    create_native_project(&root, Some("Invalid Footprint Silkscreen".to_string()))
        .expect("initial scaffold should succeed");
    main_tests_project_pool_footprint::create_footprint_fixture(&root);

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-footprint-silkscreen-line",
            root.to_str().unwrap(),
            "--footprint",
            &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
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
    assert!(format!("{err:#}").contains("footprint silkscreen line must have distinct endpoints"));

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-footprint-silkscreen-rect",
            root.to_str().unwrap(),
            "--footprint",
            &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
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
    .expect_err("zero-area silkscreen rect should fail");
    assert!(format!("{err:#}").contains("footprint silkscreen rect min-x-nm"));

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-footprint-silkscreen-circle",
            root.to_str().unwrap(),
            "--footprint",
            &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
            "--center-x-nm",
            "5000",
            "--center-y-nm",
            "6000",
            "--radius-nm",
            "0",
            "--width-nm",
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-radius silkscreen circle should fail");
    assert!(format!("{err:#}").contains("footprint silkscreen circle radius-nm"));

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "add-pool-footprint-silkscreen-polygon",
            root.to_str().unwrap(),
            "--footprint",
            &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
            "--vertices",
            "0,0;1000,0",
            "--closed",
            "true",
            "--width-nm",
            "150000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("closed silkscreen polygon with too few vertices should fail");
    assert!(format!("{err:#}").contains("footprint silkscreen closed polygon"));

    let _ = std::fs::remove_dir_all(&root);
}

fn execute_footprint_silkscreen_line(root: &Path) -> Result<String> {
    execute(Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "add-pool-footprint-silkscreen-line",
        root.to_str().unwrap(),
        "--footprint",
        &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
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
    ])?)
}

fn execute_footprint_silkscreen_rect(root: &Path) -> Result<String> {
    execute(Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "add-pool-footprint-silkscreen-rect",
        root.to_str().unwrap(),
        "--footprint",
        &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
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
    ])?)
}

fn execute_footprint_silkscreen_circle(root: &Path) -> Result<String> {
    execute(Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "add-pool-footprint-silkscreen-circle",
        root.to_str().unwrap(),
        "--footprint",
        &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
        "--center-x-nm",
        "5000",
        "--center-y-nm",
        "6000",
        "--radius-nm",
        "7000",
        "--width-nm",
        "150000",
    ])?)
}

fn execute_footprint_silkscreen_polygon(root: &Path, closed: bool) -> Result<String> {
    execute(Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "add-pool-footprint-silkscreen-polygon",
        root.to_str().unwrap(),
        "--footprint",
        &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
        "--vertices",
        "0,0;1000,0;1000,1000",
        "--closed",
        if closed { "true" } else { "false" },
        "--width-nm",
        "150000",
    ])?)
}

fn footprint_payload(root: &Path) -> serde_json::Value {
    main_tests_project_pool_library::query_pool_object_payload(
        root,
        "footprints",
        main_tests_project_pool_footprint::FOOTPRINT_ID,
    )
}

fn undo_last(root: &Path) {
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
    .expect("footprint silkscreen undo should succeed");
}
