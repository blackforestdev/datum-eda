use super::super::*;
use crate::import::package_assignments_sidecar;
use std::fs;

#[test]
fn set_package_updates_board_and_undo_redo_restore_it() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let package_uuid = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .map(|part| part.package_uuid)
        .expect("ALTAMP package should exist");
    let before = engine.get_components().expect("components should query");
    let component_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    let set_result = engine
        .set_package(SetPackageInput {
            uuid: component_uuid,
            package_uuid,
        })
        .expect("set_package should succeed");
    assert_eq!(set_result.diff.modified.len(), 1);

    let after_set = engine.get_components().expect("components should query");
    let updated = after_set
        .iter()
        .find(|component| component.uuid == component_uuid)
        .unwrap();
    assert_eq!(updated.package_uuid, package_uuid);
    let pad_count_after_set = engine
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .map(|board| {
            board
                .pads
                .values()
                .filter(|pad| pad.package == component_uuid)
                .count()
        })
        .expect("board should exist");
    assert_eq!(pad_count_after_set, 3);
    let net_info_after_set = engine.get_net_info().expect("net info should query");
    let sig_after_set = net_info_after_set
        .iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");
    assert_eq!(sig_after_set.pins.len(), 1);

    let undo = engine.undo().expect("undo should succeed");
    assert_eq!(undo.diff.modified.len(), 1);
    let after_undo = engine.get_components().expect("components should query");
    assert_eq!(before, after_undo);

    let redo = engine.redo().expect("redo should succeed");
    assert_eq!(redo.diff.modified.len(), 1);
    let after_redo = engine.get_components().expect("components should query");
    assert_eq!(after_set, after_redo);
}

#[test]
fn save_persists_set_package_for_current_m3_slice() {
    let source = fixture_path("partial-route-demo.kicad_pcb");
    let target = unique_temp_path("datum-eda-save-set-package-board.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    let package_uuid = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .map(|part| part.package_uuid)
        .expect("ALTAMP package should exist");
    engine
        .set_package(SetPackageInput {
            uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            package_uuid,
        })
        .expect("set_package should succeed");

    engine.save(&target).expect("save should succeed");

    let saved = fs::read_to_string(&target).expect("saved file should read");
    assert!(saved.contains("(footprint \"ALT-3\""));
    assert_eq!(saved.matches("(pad \"").count(), 4);

    let mut reloaded = Engine::new().expect("engine should initialize");
    reloaded
        .import(&target)
        .expect("saved board should reimport successfully");
    let components = reloaded.get_components().expect("components should query");
    let updated = components
        .iter()
        .find(|component| {
            component.uuid == uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
        })
        .unwrap();
    assert_eq!(updated.package_uuid, package_uuid);
    let restored_package = reloaded
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .and_then(|board| {
            board
                .packages
                .get(&uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
        })
        .map(|package| package.package)
        .expect("reloaded package should exist");
    assert_eq!(restored_package, package_uuid);

    let _ = fs::remove_file(&target);
    let _ = fs::remove_file(package_assignments_sidecar::sidecar_path_for_source(
        &target,
    ));
}

#[test]
fn set_package_preserves_logical_nets_across_known_part_remap() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let lmv321_part_uuid = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("LMV321 part should exist");
    let altamp_package_uuid = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .map(|part| part.package_uuid)
        .expect("ALTAMP package should exist");
    let altamp_part_uuid = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("ALTAMP part should exist");
    let component_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();

    engine
        .assign_part(AssignPartInput {
            uuid: component_uuid,
            part_uuid: lmv321_part_uuid,
        })
        .expect("assign_part should succeed");
    let intermediate_sig = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");

    engine
        .set_package(SetPackageInput {
            uuid: component_uuid,
            package_uuid: altamp_package_uuid,
        })
        .expect("set_package should succeed");
    let after_sig = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");
    let updated = engine
        .get_components()
        .expect("components should query")
        .into_iter()
        .find(|component| component.uuid == component_uuid)
        .expect("updated component should exist");
    let assigned_part = engine
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .and_then(|board| board.packages.get(&component_uuid))
        .map(|component| component.part)
        .expect("component should exist");

    assert_eq!(updated.package_uuid, altamp_package_uuid);
    assert_eq!(updated.value, "ALTAMP");
    assert_eq!(assigned_part, altamp_part_uuid);
    assert_eq!(after_sig.pins.len(), intermediate_sig.pins.len());
}

#[test]
fn set_package_with_part_preserves_logical_nets_for_explicit_candidate() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let lmv321_part_uuid = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("LMV321 part should exist");
    let altamp = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .cloned()
        .expect("ALTAMP part should exist");
    let component_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();

    engine
        .assign_part(AssignPartInput {
            uuid: component_uuid,
            part_uuid: lmv321_part_uuid,
        })
        .expect("assign_part should succeed");
    let intermediate_sig = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");

    engine
        .set_package_with_part(SetPackageWithPartInput {
            uuid: component_uuid,
            package_uuid: altamp.package_uuid,
            part_uuid: altamp.uuid,
        })
        .expect("set_package_with_part should succeed");
    let after_sig = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");
    let updated = engine
        .get_components()
        .expect("components should query")
        .into_iter()
        .find(|component| component.uuid == component_uuid)
        .expect("updated component should exist");
    let assigned_part = engine
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .and_then(|board| board.packages.get(&component_uuid))
        .map(|component| component.part)
        .expect("component should exist");

    assert_eq!(updated.package_uuid, altamp.package_uuid);
    assert_eq!(updated.value, "ALTAMP");
    assert_eq!(assigned_part, altamp.uuid);
    assert_eq!(after_sig.pins.len(), intermediate_sig.pins.len());
}

#[test]
fn replace_component_preserves_logical_nets_for_explicit_candidate() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let lmv321_part_uuid = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("LMV321 part should exist");
    let altamp = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .cloned()
        .expect("ALTAMP part should exist");
    let component_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();

    engine
        .assign_part(AssignPartInput {
            uuid: component_uuid,
            part_uuid: lmv321_part_uuid,
        })
        .expect("assign_part should succeed");
    let intermediate_sig = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");

    let result = engine
        .replace_component(ReplaceComponentInput {
            uuid: component_uuid,
            package_uuid: altamp.package_uuid,
            part_uuid: altamp.uuid,
        })
        .expect("replace_component should succeed");
    assert_eq!(
        result.description,
        format!("replace_component {}", component_uuid)
    );
    let after_sig = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");
    let updated = engine
        .get_components()
        .expect("components should query")
        .into_iter()
        .find(|component| component.uuid == component_uuid)
        .expect("updated component should exist");
    let assigned_part = engine
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .and_then(|board| board.packages.get(&component_uuid))
        .map(|component| component.part)
        .expect("component should exist");

    assert_eq!(updated.package_uuid, altamp.package_uuid);
    assert_eq!(updated.value, "ALTAMP");
    assert_eq!(assigned_part, altamp.uuid);
    assert_eq!(after_sig.pins.len(), intermediate_sig.pins.len());
}

#[test]
fn replace_components_batches_multiple_replacements_into_one_undo_step() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let altamp = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .cloned()
        .expect("ALTAMP part should exist");
    let first_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    let second_uuid = uuid::Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();

    let result = engine
        .replace_components(vec![
            ReplaceComponentInput {
                uuid: first_uuid,
                package_uuid: altamp.package_uuid,
                part_uuid: altamp.uuid,
            },
            ReplaceComponentInput {
                uuid: second_uuid,
                package_uuid: altamp.package_uuid,
                part_uuid: altamp.uuid,
            },
        ])
        .expect("replace_components should succeed");
    assert_eq!(result.description, "replace_components 2");
    assert_eq!(result.diff.modified.len(), 2);

    let replaced = engine.get_components().expect("components should query");
    assert_eq!(
        replaced
            .iter()
            .filter(|component| component.value == "ALTAMP")
            .count(),
        2
    );

    let undo = engine.undo().expect("undo should succeed");
    assert_eq!(undo.description, "undo replace_components 2");
    let reverted = engine
        .get_components()
        .expect("components should query after undo");
    assert_eq!(
        reverted
            .iter()
            .filter(|component| component.value == "10k")
            .count(),
        2
    );

    let redo = engine.redo().expect("redo should succeed");
    assert_eq!(redo.description, "redo replace_components 2");
    let redone = engine
        .get_components()
        .expect("components should query after redo");
    assert_eq!(
        redone
            .iter()
            .filter(|component| component.value == "ALTAMP")
            .count(),
        2
    );
}
