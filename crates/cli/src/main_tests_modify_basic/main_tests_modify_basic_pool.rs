use super::*;

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
