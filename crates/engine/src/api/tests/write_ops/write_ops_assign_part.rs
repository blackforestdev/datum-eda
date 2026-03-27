use super::super::*;
use crate::import::part_assignments_sidecar;
use std::fs;

#[test]
fn assign_part_updates_board_and_undo_redo_restore_it() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let part_uuid = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("ALTAMP part should exist");
    let before = engine.get_components().expect("components should query");
    let package_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    let assign_result = engine
        .assign_part(AssignPartInput {
            uuid: package_uuid,
            part_uuid,
        })
        .expect("assign_part should succeed");
    assert_eq!(assign_result.diff.modified.len(), 1);

    let after_assign = engine.get_components().expect("components should query");
    let updated = after_assign
        .iter()
        .find(|component| component.uuid == package_uuid)
        .unwrap();
    assert_eq!(updated.value, "ALTAMP");
    assert_eq!(
        updated.package_uuid,
        uuid::Uuid::parse_str("3bbffc1f-f562-563a-b9da-4e0d73ab019e").unwrap()
    );
    let sig = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");
    assert_eq!(sig.pins.len(), 1);

    let undo = engine.undo().expect("undo should succeed");
    assert_eq!(undo.diff.modified.len(), 1);
    let after_undo = engine.get_components().expect("components should query");
    assert_eq!(before, after_undo);

    let redo = engine.redo().expect("redo should succeed");
    assert_eq!(redo.diff.modified.len(), 1);
    let after_redo = engine.get_components().expect("components should query");
    assert_eq!(after_assign, after_redo);
}

#[test]
fn save_persists_assign_part_for_current_m3_slice() {
    let source = fixture_path("partial-route-demo.kicad_pcb");
    let target = unique_temp_path("datum-eda-save-assign-part-board.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    let part_uuid = engine
        .search_pool("ALTAMP")
        .expect("search should succeed")
        .first()
        .map(|part| part.uuid)
        .expect("ALTAMP part should exist");
    engine
        .assign_part(AssignPartInput {
            uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            part_uuid,
        })
        .expect("assign_part should succeed");

    engine.save(&target).expect("save should succeed");

    let saved = fs::read_to_string(&target).expect("saved file should read");
    assert!(saved.contains("(property \"Value\" \"ALTAMP\""));
    assert!(saved.contains("(footprint \"ALT-3\""));

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
    assert_eq!(updated.value, "ALTAMP");
    let restored_part = reloaded
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .and_then(|board| {
            board
                .packages
                .get(&uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
        })
        .map(|package| package.part)
        .expect("reloaded package should exist");
    assert_eq!(restored_part, part_uuid);
    let restored_sig = reloaded
        .get_net_info()
        .expect("reloaded net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("reloaded SIG net should exist");
    assert_eq!(restored_sig.pins.len(), 1);

    let _ = fs::remove_file(&target);
    let _ = fs::remove_file(part_assignments_sidecar::sidecar_path_for_source(&target));
}

#[test]
fn assign_part_preserves_logical_nets_across_known_part_remap() {
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
        .expect("first assign_part should succeed");
    let after_lmv321_sig = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");

    engine
        .assign_part(AssignPartInput {
            uuid: component_uuid,
            part_uuid: altamp_part_uuid,
        })
        .expect("second assign_part should succeed");
    let after_altamp_sig = engine
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

    assert_eq!(updated.value, "ALTAMP");
    assert_eq!(after_altamp_sig.pins.len(), after_lmv321_sig.pins.len());
}
