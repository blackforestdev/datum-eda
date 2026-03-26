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
fn modify_board_supports_assign_part_slice() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-partial-route-assign-part.kicad_pcb",
        Uuid::new_v4()
    ));
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    let part_uuid = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("ALTAMP part should exist");

    let report = modify_board(
        &source,
        &[],
        &[],
        &[],
        &[eagle_fixture_path("simple-opamp.lbr")],
        &[],
        &[],
        &[],
        &[AssignPartInput {
            uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            part_uuid,
        }],
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
    .expect("modify assign_part save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
    let saved = std::fs::read_to_string(&target).expect("saved file should read");
    assert!(saved.contains("(property \"Value\" \"ALTAMP\""));
    assert!(saved.contains("(footprint \"ALT-3\""));
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(target.with_file_name(format!(
        "{}.parts.json",
        target.file_name().unwrap().to_string_lossy()
    )));
}

#[test]
fn modify_board_assign_part_preserves_logical_nets_across_known_part_remap() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-partial-route-assign-part-remap.kicad_pcb",
        Uuid::new_v4()
    ));
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    let lmv321_part_uuid = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("LMV321 part should exist");
    let altamp_part_uuid = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("ALTAMP part should exist");

    let report = modify_board(
        &source,
        &[],
        &[],
        &[],
        &[eagle_fixture_path("simple-opamp.lbr")],
        &[],
        &[],
        &[],
        &[
            AssignPartInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                part_uuid: lmv321_part_uuid,
            },
            AssignPartInput {
                uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                part_uuid: altamp_part_uuid,
            },
        ],
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
    .expect("modify assign_part remap save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));

    let mut reloaded = Engine::new().expect("engine should initialize");
    reloaded
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    reloaded.import(&target).expect("saved board should reimport");
    let sig = reloaded
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");
    assert_eq!(sig.pins.len(), 2);

    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(target.with_file_name(format!(
        "{}.parts.json",
        target.file_name().unwrap().to_string_lossy()
    )));
    let _ = std::fs::remove_file(target.with_file_name(format!(
        "{}.packages.json",
        target.file_name().unwrap().to_string_lossy()
    )));
}

#[test]
fn modify_board_supports_set_package_slice() {
    let source = kicad_fixture_path("partial-route-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-partial-route-set-package.kicad_pcb",
        Uuid::new_v4()
    ));
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    let package_uuid = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .map(|part| part.package_uuid)
        .expect("ALTAMP package should exist");

    let report = modify_board(
        &source,
        &[],
        &[],
        &[],
        &[eagle_fixture_path("simple-opamp.lbr")],
        &[],
        &[],
        &[],
        &[],
        &[SetPackageInput {
            uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            package_uuid,
        }],
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
    .expect("modify set_package save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
    let updated = match query_components(&target).expect("saved components should query") {
        ComponentListView::Board { components } => components
            .into_iter()
            .find(|component| {
                component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
            })
            .expect("target component should exist"),
    };
    assert_eq!(updated.package_uuid, package_uuid);
    let sig = match query_nets(&target).expect("saved nets should query") {
        NetListView::Board { nets } => nets
            .into_iter()
            .find(|net| net.name == "SIG")
            .expect("SIG net should exist"),
        NetListView::Schematic { .. } => panic!("expected board net list"),
    };
    assert_eq!(sig.pins.len(), 1);
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(target.with_file_name(format!(
        "{}.packages.json",
        target.file_name().unwrap().to_string_lossy()
    )));
}

#[test]
fn modify_board_supports_set_net_class_slice() {
    let source = kicad_fixture_path("simple-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-save-simple-demo-net-class.kicad_pcb",
        Uuid::new_v4()
    ));
    let net_uuid = match query_nets(&source).expect("nets should query") {
        NetListView::Board { nets } => nets
            .into_iter()
            .find(|net| net.name == "GND")
            .expect("GND net should exist")
            .uuid,
        NetListView::Schematic { .. } => panic!("expected board net list"),
    };

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
        &[SetNetClassInput {
            net_uuid,
            class_name: "power".to_string(),
            clearance: 125_000,
            track_width: 250_000,
            via_drill: 300_000,
            via_diameter: 600_000,
            diffpair_width: 0,
            diffpair_gap: 0,
        }],
        &[],
        None,
        0,
        0,
        Some(&target),
        false,
    )
    .expect("modify set_net_class save should succeed");
    assert_eq!(report.saved_path.as_deref(), Some(target.to_str().unwrap()));
    let nets = query_nets(&target).expect("saved nets should query");
    let gnd = match nets {
        NetListView::Board { nets } => nets
            .into_iter()
            .find(|net| net.uuid == net_uuid)
            .expect("updated GND net should exist"),
        NetListView::Schematic { .. } => panic!("expected board net list"),
    };
    assert_eq!(gnd.class, "power");
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(target.with_file_name(format!(
        "{}.net-classes.json",
        target.file_name().unwrap().to_string_lossy()
    )));
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
