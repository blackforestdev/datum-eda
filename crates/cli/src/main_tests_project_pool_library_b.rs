use super::main_tests_project_pool_library::*;
use super::*;

#[test]
fn project_create_pool_padstack_rejects_invalid_aperture_arguments() {
    let root = unique_project_root("datum-eda-cli-project-pool-padstack-invalid");
    create_native_project(&root, Some("Pool Padstack Invalid".to_string()))
        .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    let padstack = padstack_id.to_string();
    let error = execute(
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
            "BadRect",
            "--aperture",
            "rect",
            "--diameter-nm",
            "1000000",
        ])
        .expect("CLI should parse"),
    )
    .expect_err("invalid padstack aperture should fail");
    assert!(format!("{error:#}").contains("rect padstack aperture does not accept diameter-nm"));
    assert!(
        !root
            .join(format!("pool/padstacks/{padstack_id}.json"))
            .exists()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_package_authors_typed_package_with_pad() {
    let root = unique_project_root("datum-eda-cli-project-pool-package");
    create_native_project(&root, Some("Pool Package".to_string()))
        .expect("initial scaffold should succeed");
    let padstack_id = Uuid::new_v4();
    create_typed_pool_padstack(&root, padstack_id);
    let package_id = Uuid::new_v4();
    let pad_id = Uuid::new_v4();
    let (package, pad, padstack) = (
        package_id.to_string(),
        pad_id.to_string(),
        padstack_id.to_string(),
    );
    let output = execute(
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
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("create-package report JSON should parse");
    assert_eq!(report["action"], "create_package");
    assert_eq!(
        report["relative_path"],
        format!("pool/packages/{package_id}.json")
    );
    let payload = query_pool_object_payload(&root, "packages", package_id);
    assert_eq!(
        payload["pads"][pad_id.to_string()]["padstack"],
        padstack_id.to_string()
    );
    assert_eq!(payload["pads"][pad_id.to_string()]["position"]["x"], 1000);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_create_pool_package_rejects_missing_padstack() {
    let root = unique_project_root("datum-eda-cli-project-pool-package-missing-padstack");
    create_native_project(&root, Some("Pool Package Missing Padstack".to_string()))
        .expect("initial scaffold should succeed");
    let (package_id, pad_id, padstack_id) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let (package, pad, padstack) = (
        package_id.to_string(),
        pad_id.to_string(),
        padstack_id.to_string(),
    );
    let error = execute(
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
            "BadPackage",
            "--pad",
            &pad,
            "--padstack",
            &padstack,
        ])
        .expect("CLI should parse"),
    )
    .expect_err("package with missing padstack should fail");
    assert!(format!("{error:#}").contains("missing pool padstack"));
    assert!(
        !root
            .join(format!("pool/packages/{package_id}.json"))
            .exists()
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_pool_library_object_set_is_journaled_query_visible_and_undoable() {
    let root = unique_project_root("datum-eda-cli-project-pool-library-set");
    create_native_project(&root, Some("Pool Library Set".to_string()))
        .expect("initial scaffold should succeed");
    let symbol_id = Uuid::new_v4();
    let initial_payload =
        write_pool_object_payload_named(&root, "symbols", symbol_id, "Initial Symbol");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-library-object",
            root.to_str().unwrap(),
            "--kind",
            "symbols",
            "--object",
            &symbol_id.to_string(),
            "--from-json",
            initial_payload.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool library object create should succeed");

    let replacement_payload =
        write_pool_object_payload_named(&root, "symbols", symbol_id, "Edited Symbol");
    let set_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-library-object",
            root.to_str().unwrap(),
            "--kind",
            "symbols",
            "--object",
            &symbol_id.to_string(),
            "--from-json",
            replacement_payload.to_str().unwrap(),
        ])
        .expect("CLI should parse"),
    )
    .expect("pool library object set should succeed");
    let set_report: serde_json::Value =
        serde_json::from_str(&set_output).expect("set report JSON should parse");
    assert_eq!(set_report["action"], "set");
    let journal = std::fs::read_to_string(root.join(".datum/journal/transactions.jsonl"))
        .expect("transaction journal should exist");
    assert!(journal.contains("\"kind\":\"set_pool_library_object\""));

    let query_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "pool-library-objects",
            "--kind",
            "symbols",
            "--object",
            &symbol_id.to_string(),
            "--include-payload",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool library objects query should succeed");
    let query_report: serde_json::Value =
        serde_json::from_str(&query_output).expect("query JSON should parse");
    assert_eq!(query_report["objects"][0]["object_revision"], 1);
    assert_eq!(
        query_report["objects"][0]["payload"]["name"],
        "Edited Symbol"
    );

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
    .expect("set undo should succeed");
    let restored: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join(format!("pool/symbols/{symbol_id}.json")))
            .expect("symbol payload should exist after undo"),
    )
    .expect("restored payload should parse");
    assert_eq!(restored["name"], "Initial Symbol");
    let _ = std::fs::remove_dir_all(&root);
}
