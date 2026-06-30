use super::*;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
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

fn seed_promoted_pool_padstack(root: &Path, padstack_id: Uuid) {
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

fn seed_promoted_pool_package(root: &Path, package_id: Uuid, pad_id: Uuid, padstack_id: Uuid) {
    let package_path = root.join(format!("pool/packages/{package_id}.json"));
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
            "uuid": package_id,
            "name": "SOT23",
            "pads": {
                pad_id.to_string(): {
                    "uuid": pad_id,
                    "name": "1",
                    "position": {"x": 1000, "y": 2000},
                    "padstack": padstack_id,
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

#[test]
fn proposal_create_pool_package_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-proposal");
    create_native_project(&root, Some("Pool Package Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    let proposal_id = Uuid::new_v4();
    seed_promoted_pool_padstack(&root, padstack_id);

    let package_path = root.join(format!("pool/packages/{package_id}.json"));
    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review AI-authored package",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool package proposal create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create report JSON should parse");
    assert_eq!(create_report["contract"], "proposal_create_v1");
    assert_eq!(create_report["action"], "create_pool_package_proposal");
    assert_eq!(create_report["proposal_id"], proposal_id.to_string());
    assert_eq!(create_report["proposal"]["source"], "tool");
    assert!(
        !package_path.exists(),
        "proposal creation must not write the pool package shard"
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
    .expect("pool package proposal accept-apply should succeed");

    assert!(package_path.exists());
    let queried = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(queried["uuid"], package_id.to_string());
    assert_eq!(queried["name"], "SOT23");
    assert_eq!(
        queried["pads"][pad_id.to_string()]["padstack"],
        padstack_id.to_string()
    );
    assert_eq!(queried["pads"][pad_id.to_string()]["position"]["x"], 1000);
    assert_eq!(queried["pads"][pad_id.to_string()]["position"]["y"], 2000);
    assert_eq!(queried["pads"][pad_id.to_string()]["layer"], 1);
}

#[test]
fn proposal_create_pool_package_authors_body_only_package() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-body-proposal");
    create_native_project(&root, Some("Pool Package Body Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let package_id = Uuid::new_v4();
    let proposal_id = Uuid::new_v4();

    let package_path = root.join(format!("pool/packages/{package_id}.json"));
    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-pool-package",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--name",
            "SOT23",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review package body",
        ])
        .expect("CLI should parse"),
    )
    .expect("body-only pool package proposal create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create report JSON should parse");
    assert_eq!(create_report["contract"], "proposal_create_v1");
    assert_eq!(create_report["action"], "create_pool_package_proposal");
    assert!(
        !package_path.exists(),
        "proposal creation must not write the pool package shard"
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
    .expect("body-only pool package proposal accept-apply should succeed");

    let queried = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(queried["uuid"], package_id.to_string());
    assert_eq!(queried["name"], "SOT23");
    assert_eq!(
        queried["pads"]
            .as_object()
            .expect("pads should be an object")
            .len(),
        0
    );
}

#[test]
fn proposal_create_pool_package_rejects_missing_padstack() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-proposal-missing-padstack");
    create_native_project(
        &root,
        Some("Pool Package Proposal Missing Padstack".to_string()),
    )
    .expect("initial scaffold should succeed");
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    let padstack_id = Uuid::new_v4();

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-pool-package",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--name",
            "BadPackage",
            "--pad",
            &pad_id.to_string(),
            "--padstack",
            &padstack_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("missing package padstack should fail proposal creation");
    assert!(format!("{error:#}").contains("missing pool padstack"));
    assert!(
        !root
            .join(format!("pool/packages/{package_id}.json"))
            .exists()
    );
}

#[test]
fn proposal_set_pool_package_pad_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-pad-proposal");
    create_native_project(&root, Some("Pool Package Pad Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let existing_pad_id = Uuid::new_v4();
    let new_pad_id = Uuid::new_v4();
    let proposal_id = Uuid::new_v4();
    seed_promoted_pool_padstack(&root, padstack_id);
    seed_promoted_pool_package(&root, package_id, existing_pad_id, padstack_id);

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "set-pool-package-pad",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--pad",
            &new_pad_id.to_string(),
            "--padstack",
            &padstack_id.to_string(),
            "--pad-name",
            "2",
            "--x-nm",
            "3000",
            "--y-nm",
            "4000",
            "--layer",
            "1",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review AI-authored package pad",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool package pad proposal create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create report JSON should parse");
    assert_eq!(create_report["contract"], "proposal_create_v1");
    assert_eq!(create_report["action"], "set_pool_package_pad_proposal");
    assert_eq!(create_report["proposal_id"], proposal_id.to_string());
    assert_eq!(create_report["proposal"]["source"], "tool");
    let before_apply = query_pool_object_payload(&root, "packages", package_id);
    assert!(before_apply["pads"].get(new_pad_id.to_string()).is_none());

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
    .expect("pool package pad proposal accept-apply should succeed");

    let queried = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        queried["pads"][new_pad_id.to_string()]["padstack"],
        padstack_id.to_string()
    );
    assert_eq!(queried["pads"][new_pad_id.to_string()]["name"], "2");
    assert_eq!(
        queried["pads"][new_pad_id.to_string()]["position"]["x"],
        3000
    );
    assert_eq!(queried["pads"][new_pad_id.to_string()]["layer"], 1);
}

#[test]
fn proposal_set_pool_package_pad_rejects_duplicate_pad() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-pad-proposal-duplicate");
    create_native_project(
        &root,
        Some("Pool Package Pad Proposal Duplicate".to_string()),
    )
    .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    seed_promoted_pool_padstack(&root, padstack_id);
    seed_promoted_pool_package(&root, package_id, pad_id, padstack_id);

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "set-pool-package-pad",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--pad",
            &pad_id.to_string(),
            "--padstack",
            &padstack_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("duplicate package pad should fail proposal creation");
    assert!(format!("{error:#}").contains("already has pad"));
}

#[test]
fn proposal_set_pool_package_courtyard_rect_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-courtyard-rect-proposal");
    create_native_project(
        &root,
        Some("Pool Package Courtyard Rect Proposal".to_string()),
    )
    .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    let proposal_id = Uuid::new_v4();
    seed_promoted_pool_padstack(&root, padstack_id);
    seed_promoted_pool_package(&root, package_id, pad_id, padstack_id);

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review package courtyard",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool package courtyard rect proposal create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create report JSON should parse");
    assert_eq!(create_report["contract"], "proposal_create_v1");
    assert_eq!(
        create_report["action"],
        "set_pool_package_courtyard_rect_proposal"
    );
    assert_eq!(create_report["proposal_id"], proposal_id.to_string());
    let before_apply = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        before_apply["courtyard"]["vertices"]
            .as_array()
            .expect("vertices should be array")
            .len(),
        0
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
    .expect("pool package courtyard rect proposal accept-apply should succeed");

    let queried = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(queried["courtyard"]["closed"], true);
    assert_eq!(
        queried["courtyard"]["vertices"]
            .as_array()
            .expect("vertices should be array")
            .len(),
        4
    );
    assert_eq!(queried["courtyard"]["vertices"][0]["x"], 1000);
    assert_eq!(queried["courtyard"]["vertices"][2]["y"], 4000);
}

#[test]
fn proposal_set_pool_package_courtyard_rect_rejects_zero_area() {
    let root =
        unique_project_root("datum-eda-cli-project-pool-package-courtyard-rect-proposal-invalid");
    create_native_project(
        &root,
        Some("Pool Package Courtyard Rect Invalid".to_string()),
    )
    .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    seed_promoted_pool_padstack(&root, padstack_id);
    seed_promoted_pool_package(&root, package_id, pad_id, padstack_id);

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
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
    .expect_err("zero-width courtyard should fail proposal creation");
    assert!(format!("{error:#}").contains("min-x-nm must be less than max-x-nm"));
}

#[test]
fn proposal_set_pool_package_courtyard_polygon_defers_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-courtyard-polygon-proposal");
    create_native_project(
        &root,
        Some("Pool Package Courtyard Polygon Proposal".to_string()),
    )
    .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    let proposal_id = Uuid::new_v4();
    seed_promoted_pool_padstack(&root, padstack_id);
    seed_promoted_pool_package(&root, package_id, pad_id, padstack_id);

    let create_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "set-pool-package-courtyard-polygon",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--vertices",
            "0,0;1000,0;1000,2000;0,2000",
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool package courtyard polygon proposal create should succeed");
    let create_report: serde_json::Value =
        serde_json::from_str(&create_output).expect("create report JSON should parse");
    assert_eq!(
        create_report["action"],
        "set_pool_package_courtyard_polygon_proposal"
    );
    let before_apply = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        before_apply["courtyard"]["vertices"]
            .as_array()
            .expect("vertices should be array")
            .len(),
        0
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
    .expect("pool package courtyard polygon proposal accept-apply should succeed");

    let queried = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(queried["courtyard"]["closed"], true);
    assert_eq!(
        queried["courtyard"]["vertices"]
            .as_array()
            .expect("vertices should be array")
            .len(),
        4
    );
    assert_eq!(queried["courtyard"]["vertices"][1]["x"], 1000);
    assert_eq!(queried["courtyard"]["vertices"][2]["y"], 2000);
}

#[test]
fn proposal_set_pool_package_courtyard_polygon_rejects_too_few_vertices() {
    let root = unique_project_root(
        "datum-eda-cli-project-pool-package-courtyard-polygon-proposal-invalid",
    );
    create_native_project(
        &root,
        Some("Pool Package Courtyard Polygon Invalid".to_string()),
    )
    .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    seed_promoted_pool_padstack(&root, padstack_id);
    seed_promoted_pool_package(&root, package_id, pad_id, padstack_id);

    let error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "set-pool-package-courtyard-polygon",
            root.to_str().unwrap(),
            "--package",
            &package_id.to_string(),
            "--vertices",
            "0,0;1000,0",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("too few courtyard vertices should fail proposal creation");
    assert!(format!("{error:#}").contains("must have at least 3 vertices"));
}
