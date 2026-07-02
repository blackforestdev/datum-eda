use super::main_tests_project_schematic_proposals::execute;
use super::main_tests_project_schematic_proposals::*;
use super::*;
use eda_engine::substrate::{ProjectResolver, ProposalStatus};

#[test]
fn proposal_create_place_symbol_is_non_mutating_until_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-symbol");
    create_native_project(&root, Some("Symbol Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"symbol-proposal");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-place-symbol",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--reference",
            "U1",
            "--value",
            "LM358",
            "--lib-id",
            "device:opamp",
            "--x-nm",
            "500",
            "--y-nm",
            "600",
            "--rotation-deg",
            "90",
            "--mirrored",
            "--proposal",
            &proposal_id.to_string(),
            "--rationale",
            "review schematic symbol placement",
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal create-place-symbol should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(report["contract"], "proposal_create_v1");
    assert_eq!(report["action"], "propose_place_symbol");
    assert_eq!(report["proposal_id"], proposal_id.to_string());
    assert_eq!(report["proposal"]["status"], "draft");
    assert_eq!(report["reference"], "U1");
    assert_eq!(report["value"], "LM358");
    assert_eq!(report["lib_id"], "device:opamp");
    assert_eq!(report["rotation_deg"], 90);
    assert_eq!(report["mirrored"], true);
    assert!(report["proposal"]["batch"]["expected_model_revision"].is_string());
    assert_eq!(query_symbol_count(&root), 0);

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
    .expect("proposal accept-apply should succeed");

    assert_eq!(query_symbol_count(&root), 1);
    let reopened = ProjectResolver::new(&root)
        .resolve()
        .expect("project should reopen");
    assert_eq!(
        reopened.proposals.get(&proposal_id).unwrap().status,
        ProposalStatus::Applied
    );
}

#[test]
fn proposal_create_place_symbol_binds_pool_symbol_and_component_instance_on_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-pool-symbol-binding");
    create_native_project(&root, Some("Pool Symbol Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    let entity_id = Uuid::new_v4();
    let gate_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();

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
            "Unit",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool unit create should succeed");
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
            "FAULT",
            "--electrical-type",
            "OpenCollector",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool unit pin set should succeed");
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
            "Symbol",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool symbol create should succeed");
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
            "0",
            "--y-nm",
            "0",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool symbol pin anchor set should succeed");
    execute(
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
            "Entity",
            "--prefix",
            "U",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool entity create should succeed");
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
            "PKG",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool package create should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-part",
            root.to_str().unwrap(),
            "--part",
            &part_id.to_string(),
            "--entity",
            &entity_id.to_string(),
            "--package",
            &package_id.to_string(),
            "--mpn",
            "TEST",
            "--manufacturer",
            "Datum",
            "--value",
            "TEST",
        ])
        .expect("CLI should parse"),
    )
    .expect("pool part create should succeed");

    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"pool-symbol-proposal");
    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-place-symbol",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--reference",
            "U1",
            "--value",
            "TEST",
            "--lib-id",
            &symbol_id.to_string(),
            "--x-nm",
            "500",
            "--y-nm",
            "600",
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal create-place-symbol should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).unwrap();
    let symbol_uuid = report["symbol_uuid"].as_str().unwrap();
    assert_eq!(report["binding_status"], "bound_with_part");
    assert_eq!(report["entity_uuid"], entity_id.to_string());
    assert_eq!(report["gate_uuid"], gate_id.to_string());
    assert_eq!(report["part_uuid"], part_id.to_string());
    assert!(report["component_instance_uuid"].is_string());
    assert_eq!(query_symbol_count(&root), 0);
    assert_eq!(
        query_component_instances(&root)["component_instance_count"],
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
    .expect("proposal accept-apply should succeed");

    assert_eq!(query_symbol_count(&root), 1);
    let component_instances = query_component_instances(&root);
    assert_eq!(component_instances["component_instance_count"], 1);
    let instance = component_instances["component_instances"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    assert_eq!(instance["part_ref"], part_id.to_string());
    assert_eq!(
        instance["placed_symbol_refs"],
        serde_json::json!([symbol_uuid])
    );

    let pins_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "symbol-pins",
            "--symbol",
            symbol_uuid,
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol-pins query should succeed");
    let pins: serde_json::Value = serde_json::from_str(&pins_output).unwrap();
    assert_eq!(pins[0]["electrical_type"], "OpenCollector");
}

#[test]
fn proposal_create_place_symbol_rejects_stale_accept_apply() {
    let root = unique_project_root("datum-eda-cli-proposal-symbol-stale");
    create_native_project(&root, Some("Stale Symbol Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let model = ProjectResolver::new(&root)
        .resolve()
        .expect("project resolves");
    let proposal_id = Uuid::new_v5(&model.project.project_id, b"symbol-stale-proposal");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "proposal",
            "create-place-symbol",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--reference",
            "R1",
            "--value",
            "10k",
            "--x-nm",
            "700",
            "--y-nm",
            "800",
            "--proposal",
            &proposal_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("proposal create-place-symbol should succeed");

    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-project-name",
            root.to_str().unwrap(),
            "--name",
            "Changed Before Symbol Proposal Apply",
        ])
        .expect("CLI should parse"),
    )
    .expect("project name should change");

    let err = execute(
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
    .expect_err("stale proposal accept-apply should fail");
    assert!(err.to_string().contains("cannot be accepted"));
    assert_eq!(query_symbol_count(&root), 0);
}
