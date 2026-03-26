use super::super::*;
use std::fs;

#[test]
fn set_value_updates_board_and_undo_redo_restore_it() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let before = engine.get_components().expect("components should query");
    let package_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    let set_result = engine
        .set_value(SetValueInput {
            uuid: package_uuid,
            value: "22k".to_string(),
        })
        .expect("set_value should succeed");
    assert_eq!(set_result.diff.modified.len(), 1);

    let after_set = engine.get_components().expect("components should query");
    let r1_after_set = after_set
        .iter()
        .find(|component| component.reference == "R1")
        .unwrap();
    assert_eq!(r1_after_set.value, "22k");

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
fn save_persists_set_value_for_current_m3_slice() {
    let source = fixture_path("partial-route-demo.kicad_pcb");
    let target = unique_temp_path("datum-eda-save-set-value-board.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    engine
        .set_value(SetValueInput {
            uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            value: "22k".to_string(),
        })
        .expect("set_value should succeed");

    engine.save(&target).expect("save should succeed");

    let saved = fs::read_to_string(&target).expect("saved file should read");
    assert!(saved.contains("(property \"Value\" \"22k\""));

    let mut reloaded = Engine::new().expect("engine should initialize");
    reloaded
        .import(&target)
        .expect("saved board should reimport successfully");
    let components = reloaded.get_components().expect("components should query");
    let r1 = components
        .iter()
        .find(|component| component.reference == "R1")
        .unwrap();
    assert_eq!(r1.value, "22k");

    let _ = fs::remove_file(&target);
}

#[test]
fn set_reference_updates_board_and_undo_redo_restore_it() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let before = engine.get_components().expect("components should query");
    let package_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    let set_result = engine
        .set_reference(SetReferenceInput {
            uuid: package_uuid,
            reference: "R10".to_string(),
        })
        .expect("set_reference should succeed");
    assert_eq!(set_result.diff.modified.len(), 1);

    let after_set = engine.get_components().expect("components should query");
    let r1_after_set = after_set
        .iter()
        .find(|component| component.uuid == package_uuid)
        .unwrap();
    assert_eq!(r1_after_set.reference, "R10");

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
fn save_persists_set_reference_for_current_m3_slice() {
    let source = fixture_path("partial-route-demo.kicad_pcb");
    let target = unique_temp_path("datum-eda-save-set-reference-board.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    engine
        .set_reference(SetReferenceInput {
            uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            reference: "R10".to_string(),
        })
        .expect("set_reference should succeed");

    engine.save(&target).expect("save should succeed");

    let saved = fs::read_to_string(&target).expect("saved file should read");
    assert!(saved.contains("(property \"Reference\" \"R10\""));

    let mut reloaded = Engine::new().expect("engine should initialize");
    reloaded
        .import(&target)
        .expect("saved board should reimport successfully");
    let components = reloaded.get_components().expect("components should query");
    let r1 = components
        .iter()
        .find(|component| {
            component.uuid
                == uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
        })
        .unwrap();
    assert_eq!(r1.reference, "R10");

    let _ = fs::remove_file(&target);
}

#[test]
fn rotate_component_updates_board_and_undo_redo_restore_it() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let before = engine.get_components().expect("components should query");
    let package_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    let rotate_result = engine
        .rotate_component(RotateComponentInput {
            uuid: package_uuid,
            rotation: 180,
        })
        .expect("rotate_component should succeed");
    assert_eq!(rotate_result.diff.modified.len(), 1);

    let after_rotate = engine.get_components().expect("components should query");
    let rotated = after_rotate
        .iter()
        .find(|component| component.uuid == package_uuid)
        .unwrap();
    assert_eq!(rotated.rotation, 180);

    let undo = engine.undo().expect("undo should succeed");
    assert_eq!(undo.diff.modified.len(), 1);
    let after_undo = engine.get_components().expect("components should query");
    assert_eq!(before, after_undo);

    let redo = engine.redo().expect("redo should succeed");
    assert_eq!(redo.diff.modified.len(), 1);
    let after_redo = engine.get_components().expect("components should query");
    assert_eq!(after_rotate, after_redo);
}

#[test]
fn save_persists_rotate_component_for_current_m3_slice() {
    let source = fixture_path("partial-route-demo.kicad_pcb");
    let target = unique_temp_path("datum-eda-save-rotate-component-board.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    engine
        .rotate_component(RotateComponentInput {
            uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            rotation: 180,
        })
        .expect("rotate_component should succeed");

    engine.save(&target).expect("save should succeed");

    let saved = fs::read_to_string(&target).expect("saved file should read");
    assert!(saved.contains("(at 10 10 180)"));

    let mut reloaded = Engine::new().expect("engine should initialize");
    reloaded
        .import(&target)
        .expect("saved board should reimport successfully");
    let components = reloaded.get_components().expect("components should query");
    let rotated = components
        .iter()
        .find(|component| {
            component.uuid
                == uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
        })
        .unwrap();
    assert_eq!(rotated.rotation, 180);

    let _ = fs::remove_file(&target);
}
