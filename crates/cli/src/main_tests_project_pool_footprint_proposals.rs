use super::*;

#[test]
fn proposal_create_pool_footprint_defers_until_accept_apply() {
    let root =
        main_tests_project_pool_library::unique_project_root("datum-eda-cli-footprint-proposal");
    create_native_project(&root, Some("Footprint Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::PACKAGE_PADSTACK_ID,
    );
    seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID,
    );
    seed_promoted_pool_package(&root);
    let proposal_id = Uuid::new_v4();
    let footprint_path = root.join(format!(
        "pool/footprints/{}.json",
        main_tests_project_pool_footprint::FOOTPRINT_ID
    ));

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-pool-footprint",
            root.to_str().unwrap(),
            "--footprint",
            &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
            "--package",
            &main_tests_project_pool_footprint::PACKAGE_ID.to_string(),
            "--name",
            "SOIC-8_Narrow",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review AI-authored footprint",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool footprint proposal create should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");
    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "create_pool_footprint_proposal");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    assert_eq!(report["proposal"]["source"], "tool");
    assert!(
        !footprint_path.exists(),
        "proposal creation must not write the pool footprint shard"
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool footprint proposal accept-apply should succeed");

    assert!(footprint_path.exists());
    let payload = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        main_tests_project_pool_footprint::FOOTPRINT_ID,
    );
    assert_eq!(payload["name"], "SOIC-8_Narrow");
    assert_eq!(
        payload["package"],
        main_tests_project_pool_footprint::PACKAGE_ID.to_string()
    );
    assert_eq!(payload["pads"].as_object().unwrap().len(), 0);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_set_pool_footprint_pad_defers_until_accept_apply() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-pad-proposal",
    );
    create_native_project(&root, Some("Footprint Pad Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::PACKAGE_PADSTACK_ID,
    );
    seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID,
    );
    seed_promoted_pool_package(&root);
    seed_promoted_pool_footprint(&root);
    let proposal_id = Uuid::new_v4();

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "set-pool-footprint-pad",
            root.to_str().unwrap(),
            "--footprint",
            &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
            "--pad",
            &main_tests_project_pool_footprint::FOOTPRINT_PAD_ID.to_string(),
            "--padstack",
            &main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID.to_string(),
            "--pad-name",
            "A1",
            "--x-nm",
            "123",
            "--y-nm",
            "456",
            "--layer",
            "1",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review AI-authored footprint pad",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool footprint pad proposal create should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");
    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "set_pool_footprint_pad_proposal");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    let before = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        main_tests_project_pool_footprint::FOOTPRINT_ID,
    );
    assert!(
        before["pads"]
            .get(main_tests_project_pool_footprint::FOOTPRINT_PAD_ID.to_string())
            .is_none()
    );

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool footprint pad proposal accept-apply should succeed");

    let payload = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        main_tests_project_pool_footprint::FOOTPRINT_ID,
    );
    let pad = &payload["pads"][main_tests_project_pool_footprint::FOOTPRINT_PAD_ID.to_string()];
    assert_eq!(
        pad["padstack"],
        main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID.to_string()
    );
    assert_eq!(pad["name"], "A1");
    assert_eq!(pad["position"]["x"], 123);
    assert_eq!(pad["layer"], 1);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_create_pool_footprint_rejects_missing_package() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-proposal-missing-package",
    );
    create_native_project(
        &root,
        Some("Missing Package Footprint Proposal".to_string()),
    )
    .expect("initial scaffold should succeed");

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-pool-footprint",
            root.to_str().unwrap(),
            "--footprint",
            &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
            "--package",
            &main_tests_project_pool_footprint::PACKAGE_ID.to_string(),
            "--name",
            "MissingPackage",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing package should fail proposal creation");
    assert!(format!("{error:#}").contains("missing pool package"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_set_pool_footprint_pad_rejects_invalid_pad_inputs() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-pad-proposal-invalid",
    );
    create_native_project(&root, Some("Invalid Footprint Pad Proposal".to_string()))
        .expect("initial scaffold should succeed");
    seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::PACKAGE_PADSTACK_ID,
    );
    seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID,
    );
    seed_promoted_pool_package(&root);
    seed_promoted_pool_footprint(&root);

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "set-pool-footprint-pad",
            root.to_str().unwrap(),
            "--footprint",
            &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
            "--pad",
            &main_tests_project_pool_footprint::FOOTPRINT_PAD_ID.to_string(),
            "--padstack",
            &main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID.to_string(),
            "--pad-name",
            "",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("blank footprint pad should fail proposal creation");
    assert!(format!("{error:#}").contains("footprint pad name must not be empty"));

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "set-pool-footprint-pad",
            root.to_str().unwrap(),
            "--footprint",
            &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
            "--pad",
            &main_tests_project_pool_footprint::FOOTPRINT_PAD_ID.to_string(),
            "--padstack",
            &Uuid::new_v4().to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing padstack should fail proposal creation");
    assert!(format!("{error:#}").contains("missing pool padstack"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_set_pool_footprint_courtyard_rect_defers_until_accept_apply() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-courtyard-proposal",
    );
    create_native_project(&root, Some("Footprint Courtyard Proposal".to_string()))
        .expect("initial scaffold should succeed");
    seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::PACKAGE_PADSTACK_ID,
    );
    seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID,
    );
    seed_promoted_pool_package(&root);
    seed_promoted_pool_footprint(&root);
    let proposal_id = Uuid::new_v4();

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "set-pool-footprint-courtyard-rect",
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
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review footprint courtyard",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool footprint courtyard proposal create should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");
    assert_eq!(
        report["action"],
        "set_pool_footprint_courtyard_rect_proposal"
    );
    let before = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        main_tests_project_pool_footprint::FOOTPRINT_ID,
    );
    assert_eq!(before["courtyard"]["vertices"].as_array().unwrap().len(), 0);

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "accept-apply",
            root.to_str().unwrap(),
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool footprint courtyard proposal accept-apply should succeed");

    let payload = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        main_tests_project_pool_footprint::FOOTPRINT_ID,
    );
    assert_eq!(payload["courtyard"]["closed"], true);
    assert_eq!(
        payload["courtyard"]["vertices"].as_array().unwrap().len(),
        4
    );
    assert_eq!(payload["courtyard"]["vertices"][0]["x"], 1000);
    assert_eq!(payload["courtyard"]["vertices"][2]["y"], 4000);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_set_pool_footprint_courtyard_rejects_invalid_inputs() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-courtyard-proposal-invalid",
    );
    create_native_project(
        &root,
        Some("Invalid Footprint Courtyard Proposal".to_string()),
    )
    .expect("initial scaffold should succeed");
    seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::PACKAGE_PADSTACK_ID,
    );
    seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID,
    );
    seed_promoted_pool_package(&root);
    seed_promoted_pool_footprint(&root);

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "set-pool-footprint-courtyard-rect",
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
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-area footprint courtyard should fail proposal creation");
    assert!(format!("{error:#}").contains("footprint courtyard min-x-nm"));

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "set-pool-footprint-courtyard-polygon",
            root.to_str().unwrap(),
            "--footprint",
            &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
            "--vertices",
            "0,0;1000,0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("too few footprint courtyard vertices should fail proposal creation");
    assert!(
        format!("{error:#}").contains("footprint courtyard polygon must have at least 3 vertices")
    );

    let _ = std::fs::remove_dir_all(&root);
}

pub(crate) fn seed_promoted_pool_padstack(root: &Path, padstack_id: Uuid) {
    let project_json = root.join("project.json");
    let mut project: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&project_json).expect("project manifest should read"),
    )
    .expect("project manifest JSON should parse");
    project["pools"] = serde_json::json!([{ "path": "pool", "priority": 1 }]);
    std::fs::write(
        &project_json,
        serde_json::to_string_pretty(&project).expect("project manifest should serialize"),
    )
    .expect("project manifest should write");

    let padstack_path = root.join(format!("pool/padstacks/{padstack_id}.json"));
    std::fs::create_dir_all(
        padstack_path
            .parent()
            .expect("padstack path should have parent"),
    )
    .expect("pool padstacks directory should be created");
    std::fs::write(
        &padstack_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "uuid": padstack_id,
            "name": "RoundViaPad",
            "aperture": {"circle": {"diameter_nm": 1200000}},
            "drill_nm": 600000
        }))
        .expect("padstack payload should serialize"),
    )
    .expect("padstack payload should write");
}

pub(crate) fn seed_promoted_pool_package(root: &Path) {
    let package_path = root.join(format!(
        "pool/packages/{}.json",
        main_tests_project_pool_footprint::PACKAGE_ID
    ));
    std::fs::create_dir_all(
        package_path
            .parent()
            .expect("package path should have parent"),
    )
    .expect("pool packages directory should be created");
    std::fs::write(
        &package_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "uuid": main_tests_project_pool_footprint::PACKAGE_ID,
            "name": "SOIC-8 Body",
            "pads": {
                main_tests_project_pool_footprint::PACKAGE_PAD_ID.to_string(): {
                    "uuid": main_tests_project_pool_footprint::PACKAGE_PAD_ID,
                    "name": "LEGACY1",
                    "position": {"x": 0, "y": 0},
                    "padstack": main_tests_project_pool_footprint::PACKAGE_PADSTACK_ID,
                    "layer": 1
                }
            },
            "courtyard": {"vertices": [], "closed": true},
            "silkscreen": [],
            "models_3d": [],
            "body_height_nm": null,
            "body_height_mounted_nm": null,
            "tags": []
        }))
        .expect("package payload should serialize"),
    )
    .expect("package payload should write");
}

pub(crate) fn seed_promoted_pool_footprint(root: &Path) {
    let footprint_path = root.join(format!(
        "pool/footprints/{}.json",
        main_tests_project_pool_footprint::FOOTPRINT_ID
    ));
    std::fs::create_dir_all(
        footprint_path
            .parent()
            .expect("footprint path should have parent"),
    )
    .expect("pool footprints directory should be created");
    std::fs::write(
        &footprint_path,
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": 1,
            "uuid": main_tests_project_pool_footprint::FOOTPRINT_ID,
            "name": "SOIC-8_Narrow",
            "package": main_tests_project_pool_footprint::PACKAGE_ID,
            "pads": {},
            "courtyard": {"vertices": [], "closed": true},
            "silkscreen": [],
            "fab": [],
            "assembly": [],
            "mechanical": [],
            "models_3d": [],
            "standards_basis": null,
            "process_aperture_policy": null,
            "tags": []
        }))
        .expect("footprint payload should serialize"),
    )
    .expect("footprint payload should write");
}
