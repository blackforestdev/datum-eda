use super::main_tests_project_forward_annotation_support::{
    find_action_id, native_symbol, query_board_components, query_forward_annotation_proposal,
    unique_project_root, write_board_packages, write_native_sheet_symbols,
};
use super::*;
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;

#[test]
fn project_apply_forward_annotation_action_applies_part_mismatch_with_explicit_package() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-part-mismatch");
    create_native_project(&root, Some("Forward Annotation Apply Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let schematic_part_uuid = Uuid::new_v4();
    write_native_sheet_symbols(
        &root,
        sheet_uuid,
        "Main",
        vec![native_symbol(
            symbol_uuid,
            "U1",
            "MCU",
            "Device:U",
            Some(schematic_part_uuid),
            None,
        )],
    );

    let component_uuid = Uuid::new_v4();
    let board_package_uuid = Uuid::new_v4();
    write_board_packages(
        &root,
        "Forward Annotation Apply Demo Board",
        vec![PlacedPackage {
            uuid: component_uuid,
            part: Uuid::new_v4(),
            package: board_package_uuid,
            reference: "U1".into(),
            value: "MCU".into(),
            position: Point::new(0, 0),
            rotation: 0,
            layer: 1,
            locked: false,
        }],
    );

    let proposal = query_forward_annotation_proposal(&root);
    let action_id = find_action_id(&proposal, "update_component", Some("part_mismatch"));
    let replacement_package_uuid = Uuid::new_v4();

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
        &replacement_package_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("part_mismatch apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&apply_output).expect("apply JSON");
    assert_eq!(applied["proposal_action"], "update_component");
    assert_eq!(applied["reason"], "part_mismatch");
    assert_eq!(
        applied["component_report"]["component_uuid"],
        component_uuid.to_string()
    );
    assert_eq!(
        applied["component_report"]["part_uuid"],
        schematic_part_uuid.to_string()
    );
    assert_eq!(
        applied["component_report"]["package_uuid"],
        replacement_package_uuid.to_string()
    );

    let components = query_board_components(&root);
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].part, schematic_part_uuid);
    assert_eq!(components[0].package, replacement_package_uuid);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_apply_forward_annotation_action_rejects_part_mismatch_without_package() {
    let root =
        unique_project_root("datum-eda-cli-project-forward-annotation-apply-part-mismatch-missing");
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
            "U1",
            "MCU",
            "Device:U",
            Some(Uuid::new_v4()),
            None,
        )],
    );

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
    let action_id = find_action_id(&proposal, "update_component", Some("part_mismatch"));

    let apply_cli = Cli::try_parse_from([
        "eda",
        "project",
        "apply-forward-annotation-action",
        root.to_str().unwrap(),
        "--action-id",
        &action_id,
    ])
    .expect("CLI should parse");
    let err = execute(apply_cli).expect_err("part_mismatch apply should require package");
    let msg = format!("{err:#}");
    assert!(msg.contains("requires --package <uuid>"), "{msg}");

    let _ = std::fs::remove_dir_all(&root);
}
