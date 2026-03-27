use super::main_tests_project_forward_annotation_support::{
    find_action_id, native_symbol, query_board_components, query_forward_annotation_proposal,
    unique_project_root, write_board_packages, write_native_sheet_symbols,
};
use super::*;
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;

#[test]
fn project_apply_forward_annotation_action_applies_value_mismatch_update() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-update");
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
            "R1",
            "10k",
            "Device:R",
            Some(part_uuid),
            None,
        )],
    );

    let component_uuid = Uuid::new_v4();
    write_board_packages(
        &root,
        "Forward Annotation Apply Demo Board",
        vec![PlacedPackage {
            uuid: component_uuid,
            part: part_uuid,
            package: Uuid::new_v4(),
            reference: "R1".into(),
            value: "22k".into(),
            position: Point::new(0, 0),
            rotation: 0,
            layer: 1,
            locked: false,
        }],
    );

    let proposal = query_forward_annotation_proposal(&root);
    let action_id = find_action_id(&proposal, "update_component", Some("value_mismatch"));

    let apply_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "apply-forward-annotation-action",
        root.to_str().unwrap(),
        "--action-id",
        &action_id,
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&apply_output).expect("apply JSON");
    assert_eq!(applied["action"], "apply_forward_annotation_action");
    assert_eq!(applied["action_id"], action_id);
    assert_eq!(applied["proposal_action"], "update_component");
    assert_eq!(applied["reason"], "value_mismatch");
    assert_eq!(applied["component_report"]["value"], "10k");

    let components = query_board_components(&root);
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].value, "10k");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_apply_forward_annotation_action_applies_remove_component() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-remove");
    create_native_project(&root, Some("Forward Annotation Apply Demo".to_string()))
        .expect("initial scaffold should succeed");

    let component_uuid = Uuid::new_v4();
    write_board_packages(
        &root,
        "Forward Annotation Apply Demo Board",
        vec![PlacedPackage {
            uuid: component_uuid,
            part: Uuid::new_v4(),
            package: Uuid::new_v4(),
            reference: "U1".into(),
            value: "MCU".into(),
            position: Point::new(0, 0),
            rotation: 0,
            layer: 1,
            locked: false,
        }],
    );

    let proposal = query_forward_annotation_proposal(&root);
    let action_id = find_action_id(&proposal, "remove_component", None);

    let apply_cli = Cli::try_parse_from([
        "eda",
        "project",
        "apply-forward-annotation-action",
        root.to_str().unwrap(),
        "--action-id",
        &action_id,
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    assert!(apply_output.contains("proposal_action: remove_component"));
    assert!(apply_output.contains("reference: U1"));

    let components = query_board_components(&root);
    assert!(components.is_empty());

    let _ = std::fs::remove_dir_all(&root);
}
