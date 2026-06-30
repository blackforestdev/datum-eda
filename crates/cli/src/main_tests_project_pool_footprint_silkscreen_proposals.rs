use super::*;

#[test]
fn proposal_add_pool_footprint_silkscreen_line_defers_until_accept_apply() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-silkscreen-line-proposal",
    );
    create_native_project(&root, Some("Footprint Silkscreen Proposal".to_string()))
        .expect("initial scaffold should succeed");
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::PACKAGE_PADSTACK_ID,
    );
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID,
    );
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_package(&root);
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_footprint(&root);
    let proposal_id = Uuid::new_v4();

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review footprint silkscreen line",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool footprint silkscreen line proposal create should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("report should parse");
    assert_eq!(
        report["action"],
        "add_pool_footprint_silkscreen_line_proposal"
    );
    let before = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        main_tests_project_pool_footprint::FOOTPRINT_ID,
    );
    assert_eq!(before["silkscreen"].as_array().unwrap().len(), 0);

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
    .expect("pool footprint silkscreen line proposal accept-apply should succeed");

    let payload = main_tests_project_pool_library::query_pool_object_payload(
        &root,
        "footprints",
        main_tests_project_pool_footprint::FOOTPRINT_ID,
    );
    assert_eq!(payload["silkscreen"].as_array().unwrap().len(), 1);
    assert_eq!(payload["silkscreen"][0]["Line"]["from"]["x"], 1000);
    assert_eq!(payload["silkscreen"][0]["Line"]["to"]["y"], 4000);
    assert_eq!(payload["silkscreen"][0]["Line"]["width"], 150000);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_add_pool_footprint_silkscreen_line_rejects_invalid_geometry() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-silkscreen-line-proposal-invalid",
    );
    create_native_project(
        &root,
        Some("Invalid Footprint Silkscreen Proposal".to_string()),
    )
    .expect("initial scaffold should succeed");
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::PACKAGE_PADSTACK_ID,
    );
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_padstack(
        &root,
        main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID,
    );
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_package(&root);
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_footprint(&root);

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
    .expect_err("zero-length footprint silkscreen line should fail proposal creation");
    assert!(
        format!("{error:#}").contains("footprint silkscreen line must have distinct endpoints")
    );

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("zero-width footprint silkscreen line should fail proposal creation");
    assert!(format!("{error:#}").contains("footprint silkscreen line width-nm must be positive"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_add_pool_footprint_silkscreen_shapes_defer_until_accept_apply() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-silkscreen-shapes-proposal",
    );
    create_native_project(
        &root,
        Some("Footprint Silkscreen Shape Proposals".to_string()),
    )
    .expect("initial scaffold should succeed");
    seed_footprint(&root);

    let rect_proposal = Uuid::new_v4();
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
            "--proposal",
            &rect_proposal.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("footprint silkscreen rect proposal create should succeed");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&output).unwrap()["action"],
        "add_pool_footprint_silkscreen_rect_proposal"
    );
    assert_eq!(
        footprint_payload(&root)["silkscreen"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    accept_apply(&root, rect_proposal);

    let circle_proposal = Uuid::new_v4();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
            "--proposal",
            &circle_proposal.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("footprint silkscreen circle proposal create should succeed");
    accept_apply(&root, circle_proposal);

    let polygon_proposal = Uuid::new_v4();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "add-pool-footprint-silkscreen-polygon",
            root.to_str().unwrap(),
            "--footprint",
            &main_tests_project_pool_footprint::FOOTPRINT_ID.to_string(),
            "--vertices",
            "0,0;1000,0;1000,1000",
            "--closed",
            "true",
            "--width-nm",
            "150000",
            "--proposal",
            &polygon_proposal.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("footprint silkscreen polygon proposal create should succeed");
    accept_apply(&root, polygon_proposal);

    let payload = footprint_payload(&root);
    assert_eq!(payload["silkscreen"].as_array().unwrap().len(), 3);
    assert_eq!(payload["silkscreen"][0]["Rect"]["max"]["x"], 3000);
    assert_eq!(payload["silkscreen"][1]["Circle"]["radius"], 7000);
    assert_eq!(
        payload["silkscreen"][2]["Polygon"]["polygon"]["closed"],
        true
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn proposal_add_pool_footprint_silkscreen_shapes_reject_invalid_geometry() {
    let root = main_tests_project_pool_library::unique_project_root(
        "datum-eda-cli-footprint-silkscreen-shapes-proposal-invalid",
    );
    create_native_project(
        &root,
        Some("Invalid Footprint Silkscreen Shape Proposals".to_string()),
    )
    .expect("initial scaffold should succeed");
    seed_footprint(&root);

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
    .expect_err("zero-area footprint silkscreen rect should fail proposal creation");
    assert!(format!("{error:#}").contains("footprint silkscreen rect min-x-nm"));

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
    .expect_err("zero-radius footprint silkscreen circle should fail proposal creation");
    assert!(format!("{error:#}").contains("footprint silkscreen circle radius-nm"));

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
    .expect_err("closed footprint silkscreen polygon should fail proposal creation");
    assert!(format!("{error:#}").contains("footprint silkscreen closed polygon"));

    let _ = std::fs::remove_dir_all(&root);
}

fn seed_footprint(root: &Path) {
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_padstack(
        root,
        main_tests_project_pool_footprint::PACKAGE_PADSTACK_ID,
    );
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_padstack(
        root,
        main_tests_project_pool_footprint::FOOTPRINT_PADSTACK_ID,
    );
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_package(root);
    main_tests_project_pool_footprint_proposals::seed_promoted_pool_footprint(root);
}

fn accept_apply(root: &Path, proposal_id: Uuid) {
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
    .expect("pool footprint silkscreen proposal accept-apply should succeed");
}

fn footprint_payload(root: &Path) -> serde_json::Value {
    main_tests_project_pool_library::query_pool_object_payload(
        root,
        "footprints",
        main_tests_project_pool_footprint::FOOTPRINT_ID,
    )
}
