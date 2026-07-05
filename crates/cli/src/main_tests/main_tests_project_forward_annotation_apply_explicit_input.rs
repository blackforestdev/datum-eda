use super::main_tests_project_forward_annotation_support::{
    native_symbol, query_board_components, query_forward_annotation_proposal, unique_project_root,
    write_native_sheet_symbols,
};
use super::*;
use eda_engine::ir::geometry::Point;

#[test]
fn project_apply_forward_annotation_action_applies_add_component_with_explicit_resolution() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-add");
    create_native_project(&root, Some("Forward Annotation Apply Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    write_native_sheet_symbols(
        &root,
        sheet_uuid,
        "Main",
        vec![native_symbol(
            symbol_uuid,
            "C1",
            "1u",
            "Device:C",
            Some(part_uuid),
            None,
        )],
    );

    let proposal = query_forward_annotation_proposal(&root);
    let action_id = proposal["actions"][0]["action_id"]
        .as_str()
        .unwrap()
        .to_string();
    let package_uuid = Uuid::new_v4();

    let apply_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "apply-forward-annotation-action",
        root.to_str().unwrap(),
        "--action-id",
        &action_id,
        "--package",
        &package_uuid.to_string(),
        "--x-nm",
        "2500000",
        "--y-nm",
        "3500000",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("add_component should succeed");
    let applied: serde_json::Value = serde_json::from_str(&apply_output).expect("apply JSON");
    assert_eq!(applied["proposal_action"], "add_component");
    assert_eq!(applied["reason"], "symbol_missing_on_board");
    assert_eq!(applied["component_report"]["reference"], "C1");
    assert_eq!(
        applied["component_report"]["part_uuid"],
        part_uuid.to_string()
    );
    assert_eq!(
        applied["component_report"]["package_uuid"],
        package_uuid.to_string()
    );
    assert_eq!(applied["component_report"]["x_nm"], 2500000);
    assert_eq!(applied["component_report"]["y_nm"], 3500000);

    let components = query_board_components(&root);
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].reference, "C1");
    assert_eq!(components[0].value, "1u");
    assert_eq!(components[0].part, part_uuid);
    assert_eq!(components[0].package, package_uuid);
    assert_eq!(components[0].position, Point::new(2500000, 3500000));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_apply_forward_annotation_action_rejects_unresolved_add_without_part_override() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-add-unresolved");
    create_native_project(&root, Some("Forward Annotation Apply Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    write_native_sheet_symbols(
        &root,
        sheet_uuid,
        "Main",
        vec![native_symbol(
            symbol_uuid,
            "C1",
            "1u",
            "Device:C",
            None,
            Some(Uuid::new_v4()),
        )],
    );

    let proposal = query_forward_annotation_proposal(&root);
    let action_id = proposal["actions"][0]["action_id"]
        .as_str()
        .unwrap()
        .to_string();

    let apply_cli = Cli::try_parse_from([
        "eda",
        "project",
        "apply-forward-annotation-action",
        root.to_str().unwrap(),
        "--action-id",
        &action_id,
        "--package",
        &Uuid::new_v4().to_string(),
        "--x-nm",
        "2500000",
        "--y-nm",
        "3500000",
        "--layer",
        "1",
    ])
    .expect("CLI should parse");
    let err = execute(apply_cli).expect_err("unresolved add_component should fail without part");
    let msg = format!("{err:#}");
    assert!(msg.contains("requires --part <uuid>"), "{msg}");

    let _ = std::fs::remove_dir_all(&root);
}
