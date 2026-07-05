use super::*;

pub(super) const PACKAGE_PADSTACK_ID: Uuid = uuid::uuid!("11111111-1111-1111-1111-111111111111");
pub(super) const FOOTPRINT_PADSTACK_ID: Uuid = uuid::uuid!("22222222-2222-2222-2222-222222222222");
pub(super) const PACKAGE_ID: Uuid = uuid::uuid!("33333333-3333-3333-3333-333333333333");
pub(super) const PACKAGE_PAD_ID: Uuid = uuid::uuid!("44444444-4444-4444-4444-444444444444");
pub(super) const FOOTPRINT_ID: Uuid = uuid::uuid!("55555555-5555-5555-5555-555555555555");
pub(super) const FOOTPRINT_PAD_ID: Uuid = uuid::uuid!("66666666-6666-6666-6666-666666666666");
pub(super) const IPC_FOOTPRINT_ID: Uuid = uuid::uuid!("77777777-7777-7777-7777-777777777777");
pub(super) const IPC_PADSTACK_ID: Uuid = uuid::uuid!("88888888-8888-8888-8888-888888888888");
pub(super) const IPC_PAD_A_ID: Uuid = uuid::uuid!("99999999-9999-9999-9999-999999999999");
pub(super) const IPC_PAD_B_ID: Uuid = uuid::uuid!("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa");

#[test]
fn project_create_pool_footprint_is_journaled_query_visible_and_undoable() {
    let root = main_tests_project_pool_library::unique_project_root("datum-eda-cli-footprint");
    create_native_project(&root, Some("Footprint Demo".to_string()))
        .expect("initial scaffold should succeed");
    create_package_fixture(&root);
    let footprint_path = root.join(format!("pool/footprints/{FOOTPRINT_ID}.json"));

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-footprint",
            root.to_str().unwrap(),
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--package",
            &PACKAGE_ID.to_string(),
            "--name",
            "SOIC-8_Narrow",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool footprint create should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("create report JSON should parse");
    assert_eq!(report["action"], "create_footprint");
    assert_eq!(report["object_kind"], "footprints");
    assert!(footprint_path.exists());

    let payload = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        FOOTPRINT_ID,
    );
    assert_eq!(payload["name"], "SOIC-8_Narrow");
    assert_eq!(payload["package"], PACKAGE_ID.to_string());
    assert_eq!(payload["pads"].as_object().unwrap().len(), 0);

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
    .expect("footprint create undo should succeed");
    assert!(!footprint_path.exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_footprint_pad_updates_footprint_payload() {
    let root = main_tests_project_pool_library::unique_project_root("datum-eda-cli-footprint-pad");
    create_native_project(&root, Some("Footprint Pad Demo".to_string()))
        .expect("initial scaffold should succeed");
    create_footprint_fixture(&root);

    execute_set_footprint_pad(&root, "A1", 1).expect("typed pool footprint pad set should succeed");
    let payload = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        FOOTPRINT_ID,
    );
    let pad = &payload["pads"][FOOTPRINT_PAD_ID.to_string()];
    assert_eq!(pad["uuid"], FOOTPRINT_PAD_ID.to_string());
    assert_eq!(pad["name"], "A1");
    assert_eq!(pad["padstack"], FOOTPRINT_PADSTACK_ID.to_string());
    assert_eq!(pad["position"], serde_json::json!({"x": 123, "y": 456}));
    assert_eq!(pad["layer"], 1);
    assert_eq!(payload["package"], PACKAGE_ID.to_string());

    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert!(journal.contains("\"kind\":\"set_pool_library_object\""));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_footprint_rejects_missing_package() {
    let root =
        main_tests_project_pool_library::unique_project_root("datum-eda-cli-footprint-no-package");
    create_native_project(&root, Some("Footprint Missing Package".to_string()))
        .expect("initial scaffold should succeed");

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-footprint",
            root.to_str().unwrap(),
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--package",
            &PACKAGE_ID.to_string(),
            "--name",
            "MissingPackage",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing package should fail");
    assert!(format!("{err:#}").contains("missing pool package"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_generate_ipc7351b_two_terminal_chip_creates_footprint_and_padstack() {
    let root = main_tests_project_pool_library::unique_project_root("datum-eda-cli-ipc-footprint");
    create_native_project(&root, Some("IPC Footprint Demo".to_string()))
        .expect("initial scaffold should succeed");
    create_package_fixture(&root);
    let footprint_path = root.join(format!("pool/footprints/{IPC_FOOTPRINT_ID}.json"));
    let padstack_path = root.join(format!("pool/padstacks/{IPC_PADSTACK_ID}.json"));

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "generate-ipc7351b-two-terminal-chip",
            root.to_str().unwrap(),
            "--footprint",
            &IPC_FOOTPRINT_ID.to_string(),
            "--package",
            &PACKAGE_ID.to_string(),
            "--padstack",
            &IPC_PADSTACK_ID.to_string(),
            "--pad-a",
            &IPC_PAD_A_ID.to_string(),
            "--pad-b",
            &IPC_PAD_B_ID.to_string(),
            "--metric-code",
            "0603",
            "--body-length-nm",
            "1600000",
            "--body-width-nm",
            "800000",
            "--terminal-length-nm",
            "300000",
            "--terminal-width-nm",
            "800000",
            "--density",
            "nominal",
        ])
        .expect("CLI should parse"),
    )
    .expect("IPC footprint generation should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("generate report JSON should parse");
    assert_eq!(report["action"], "generate_ipc7351b_two_terminal_chip");
    assert!(footprint_path.exists());
    assert!(padstack_path.exists());

    let footprint = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        IPC_FOOTPRINT_ID,
    );
    assert_eq!(footprint["package"], PACKAGE_ID.to_string());
    assert_eq!(footprint["ipc_basis"]["family"], "IPC-7351");
    assert_eq!(footprint["ipc_basis"]["revision"], "B");
    assert_eq!(
        footprint["ipc_basis"]["package_family"],
        "two_terminal_chip"
    );
    assert_eq!(footprint["pads"].as_object().unwrap().len(), 2);
    let padstack = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "padstacks",
        IPC_PADSTACK_ID,
    );
    assert_eq!(padstack["mask_policy"]["expansion_nm"], 50_000);
    assert_eq!(padstack["paste_policy"]["expansion_nm"], -50_000);

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
    .expect("IPC footprint generation undo should succeed");
    assert!(!footprint_path.exists());
    assert!(!padstack_path.exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_footprint_pad_rejects_invalid_pad_inputs() {
    let root =
        main_tests_project_pool_library::unique_project_root("datum-eda-cli-footprint-pad-invalid");
    create_native_project(&root, Some("Footprint Invalid Pad".to_string()))
        .expect("initial scaffold should succeed");
    create_footprint_fixture(&root);

    let err = execute_set_footprint_pad(&root, "", 1).expect_err("blank pad name should fail");
    assert!(format!("{err:#}").contains("footprint pad name must be non-empty"));

    let err = execute_set_footprint_pad(&root, "A1", 0).expect_err("bad layer should fail");
    assert!(format!("{err:#}").contains("footprint pad layer must be positive"));

    let missing_padstack = Uuid::new_v4();
    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-footprint-pad",
            root.to_str().unwrap(),
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--pad",
            &FOOTPRINT_PAD_ID.to_string(),
            "--padstack",
            &missing_padstack.to_string(),
            "--pad-name",
            "A1",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing padstack should fail");
    assert!(format!("{err:#}").contains("missing pool padstack"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_footprint_courtyard_rect_authors_geometry_and_undoes() {
    let root =
        main_tests_project_pool_library::unique_project_root("datum-eda-cli-footprint-courtyard");
    create_native_project(&root, Some("Footprint Courtyard Demo".to_string()))
        .expect("initial scaffold should succeed");
    create_footprint_fixture(&root);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-footprint-courtyard-rect",
            root.to_str().unwrap(),
            "--footprint",
            &FOOTPRINT_ID.to_string(),
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
    .expect("footprint courtyard rect set should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("set courtyard report JSON should parse");
    assert_eq!(report["action"], "set_footprint_courtyard_rect");

    let payload = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        FOOTPRINT_ID,
    );
    assert_eq!(payload["courtyard"]["closed"], true);
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
    let payload = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        FOOTPRINT_ID,
    );
    assert_eq!(
        payload["courtyard"]["vertices"].as_array().unwrap().len(),
        0
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_footprint_courtyard_rejects_invalid_inputs() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-courtyard-invalid",
    );
    create_native_project(&root, Some("Footprint Courtyard Invalid".to_string()))
        .expect("initial scaffold should succeed");
    create_footprint_fixture(&root);

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-footprint-courtyard-rect",
            root.to_str().unwrap(),
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--min-x-nm",
            "1000",
            "--min-y-nm",
            "2000",
            "--max-x-nm",
            "1000",
            "--max-y-nm",
            "4000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-width courtyard should fail");
    assert!(format!("{err:#}").contains("footprint courtyard min-x-nm"));

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-footprint-courtyard-polygon",
            root.to_str().unwrap(),
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--vertices",
            "0,0;1000,0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("two-point courtyard polygon should fail");
    assert!(
        format!("{err:#}").contains("footprint courtyard polygon must have at least 3 vertices")
    );

    let err = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-footprint-courtyard-polygon",
            root.to_str().unwrap(),
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--vertices",
            "0,0;bad;1000,0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("malformed courtyard vertex should fail");
    assert!(format!("{err:#}").contains("must be formatted as x,y"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_pool_footprint_courtyard_polygon_authors_geometry() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-courtyard-polygon",
    );
    create_native_project(&root, Some("Footprint Courtyard Polygon".to_string()))
        .expect("initial scaffold should succeed");
    create_footprint_fixture(&root);

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-footprint-courtyard-polygon",
            root.to_str().unwrap(),
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--vertices",
            "0,0;1000,0;1000,2000;0,2000",
        ])
        .expect("CLI should parse"),
    )
    .expect("footprint courtyard polygon set should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("set courtyard polygon report JSON should parse");
    assert_eq!(report["action"], "set_footprint_courtyard_polygon");

    let payload = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        FOOTPRINT_ID,
    );
    assert_eq!(payload["courtyard"]["closed"], true);
    assert_eq!(
        payload["courtyard"]["vertices"].as_array().unwrap().len(),
        4
    );
    assert_eq!(payload["courtyard"]["vertices"][1]["x"], 1000);
    assert_eq!(payload["courtyard"]["vertices"][2]["y"], 2000);

    let _ = std::fs::remove_dir_all(&root);
}

pub(super) fn create_footprint_fixture(root: &Path) {
    create_package_fixture(root);
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-footprint",
            root.to_str().unwrap(),
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--package",
            &PACKAGE_ID.to_string(),
            "--name",
            "SOIC-8_Narrow",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool footprint create should succeed");
}

pub(super) fn create_package_fixture(root: &Path) {
    main_tests_project_pool_library::create_typed_pool_padstack(root, PACKAGE_PADSTACK_ID);
    main_tests_project_pool_library::create_typed_pool_padstack(root, FOOTPRINT_PADSTACK_ID);
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-package",
            root.to_str().unwrap(),
            "--package",
            &PACKAGE_ID.to_string(),
            "--name",
            "SOIC-8 Body",
            "--pad",
            &PACKAGE_PAD_ID.to_string(),
            "--padstack",
            &PACKAGE_PADSTACK_ID.to_string(),
            "--pad-name",
            "LEGACY1",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool package create should succeed");
}

pub(super) fn execute_set_footprint_pad(root: &Path, pad_name: &str, layer: i32) -> Result<String> {
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-footprint-pad",
            root.to_str().unwrap(),
            "--footprint",
            &FOOTPRINT_ID.to_string(),
            "--pad",
            &FOOTPRINT_PAD_ID.to_string(),
            "--padstack",
            &FOOTPRINT_PADSTACK_ID.to_string(),
            "--pad-name",
            pad_name,
            "--x-nm",
            "123",
            "--y-nm",
            "456",
            "--layer",
            &layer.to_string(),
        ])
        .expect("CLI should parse"),
    )
}
