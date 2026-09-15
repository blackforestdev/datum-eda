use super::*;

#[test]
fn modify_board_supports_save_slice() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target =
        std::env::temp_dir().join(format!("{}-cli-save-simple-demo.kicad_pcb", Uuid::new_v4()));
    let deleted_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
    let report = modify_board(
        &source,
        &[deleted_uuid],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        None,
        0,
        0,
        Some(&target),
        false,
    )
    .expect("modify save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
    assert!(target.exists());
    let saved = std::fs::read_to_string(&target).expect("saved file should read");
    assert!(!saved.contains(&deleted_uuid.to_string()));
    let _ = std::fs::remove_file(target);
}

#[test]
fn modify_board_supports_delete_via_save_slice() {
    let source = kicad_fixture_path("simple-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-simple-demo-via.kicad_pcb",
        Uuid::new_v4()
    ));
    let deleted_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
    let report = modify_board(
        &source,
        &[],
        &[deleted_uuid],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        None,
        0,
        0,
        Some(&target),
        false,
    )
    .expect("modify via save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
    assert!(target.exists());
    let saved = std::fs::read_to_string(&target).expect("saved file should read");
    assert!(!saved.contains(&deleted_uuid.to_string()));
    let _ = std::fs::remove_file(target);
}

#[test]
fn modify_board_supports_set_design_rule_slice() {
    let source = kicad_fixture_path("simple-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-simple-demo-rule.kicad_pcb",
        Uuid::new_v4()
    ));
    let report = modify_board(
        &source,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        Some(125_000),
        0,
        0,
        Some(&target),
        false,
    )
    .expect("modify rule save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
    assert!(
        report
            .actions
            .contains(&"set_design_rule clearance_copper 125000".to_string())
    );
    let sidecar = target.with_file_name(format!(
        "{}.rules.json",
        target.file_name().unwrap().to_string_lossy()
    ));
    assert!(sidecar.exists());
    let _ = std::fs::remove_file(target);
    let _ = std::fs::remove_file(sidecar);
}

#[test]
fn modify_board_supports_set_value_slice() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-partial-route-value.kicad_pcb",
        Uuid::new_v4()
    ));
    let report = modify_board(
        &source,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[SetValueInput {
            uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            value: "22k".to_string(),
        }],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        None,
        0,
        0,
        Some(&target),
        false,
    )
    .expect("modify set_value save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
    let saved = std::fs::read_to_string(&target).expect("saved file should read");
    assert!(saved.contains("(property \"Value\" \"22k\""));
    let _ = std::fs::remove_file(target);
}

#[test]
fn modify_board_supports_move_component_slice() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-partial-route-move.kicad_pcb",
        Uuid::new_v4()
    ));
    let report = modify_board(
        &source,
        &[],
        &[],
        &[],
        &[],
        &[MoveComponentInput {
            uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            position: eda_engine::ir::geometry::Point::new(15_000_000, 12_000_000),
            rotation: Some(90),
        }],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        None,
        0,
        0,
        Some(&target),
        false,
    )
    .expect("modify move save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
    let saved = std::fs::read_to_string(&target).expect("saved file should read");
    assert!(saved.contains("(at 15 12 90)"));
    let _ = std::fs::remove_file(target);
}

#[test]
fn modify_board_supports_set_reference_slice() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-partial-route-reference.kicad_pcb",
        Uuid::new_v4()
    ));
    let report = modify_board(
        &source,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[SetReferenceInput {
            uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            reference: "R10".to_string(),
        }],
        None,
        0,
        0,
        Some(&target),
        false,
    )
    .expect("modify set_reference save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
    let saved = std::fs::read_to_string(&target).expect("saved file should read");
    assert!(saved.contains("(property \"Reference\" \"R10\""));
    let _ = std::fs::remove_file(target);
}

#[test]
fn modify_board_supports_delete_component_slice() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-partial-route-delete-component.kicad_pcb",
        Uuid::new_v4()
    ));
    let report = modify_board(
        &source,
        &[],
        &[],
        &[Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        None,
        0,
        0,
        Some(&target),
        false,
    )
    .expect("modify delete_component save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
    let saved = std::fs::read_to_string(&target).expect("saved file should read");
    assert!(!saved.contains("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"));
    let _ = std::fs::remove_file(target);
}

#[test]
fn modify_board_supports_rotate_component_slice() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-partial-route-rotate.kicad_pcb",
        Uuid::new_v4()
    ));
    let report = modify_board(
        &source,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[RotateComponentInput {
            uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            rotation: 180,
        }],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        None,
        0,
        0,
        Some(&target),
        false,
    )
    .expect("modify rotate_component save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
    let saved = std::fs::read_to_string(&target).expect("saved file should read");
    assert!(saved.contains("(at 10 10 180)"));
    let _ = std::fs::remove_file(target);
}
