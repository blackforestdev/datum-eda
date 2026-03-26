use super::*;
use crate::import::{net_classes_sidecar, package_assignments_sidecar, part_assignments_sidecar};
use std::fs;

#[test]
fn run_drc_returns_connectivity_violation_for_partial_route_fixture() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("partial-route fixture import should succeed");

    let report = engine
        .run_drc(&[RuleType::Connectivity])
        .expect("drc should run on imported board");

    assert!(!report.passed);
    assert!(report.summary.errors >= 1);
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.code == "connectivity_unrouted_net")
    );
}

#[test]
fn get_design_rules_returns_empty_rules_for_imported_fixture() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_pcb"))
        .expect("fixture import should succeed");
    let rules = engine.get_design_rules().expect("design rules should query");
    assert!(rules.is_empty());
}

#[test]
fn save_writes_byte_identical_kicad_board_for_current_m3_slice() {
    let source = fixture_path("simple-demo.kicad_pcb");
    let expected = fs::read_to_string(&source).expect("fixture should read");
    let target = unique_temp_path("datum-eda-save-simple-demo.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    engine.save(&target).expect("save should succeed");

    let actual = fs::read_to_string(&target).expect("saved file should read");
    assert_eq!(actual, expected);

    let _ = fs::remove_file(&target);
}

#[test]
fn save_rejects_non_board_projects_in_current_m3_slice() {
    let source = fixture_path("simple-demo.kicad_sch");
    let target = unique_temp_path("datum-eda-save-simple-demo.kicad_sch");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    let err = engine
        .save(&target)
        .expect_err("schematic save should be rejected");
    assert!(
        err.to_string()
            .contains("save is currently implemented only for imported KiCad boards"),
        "{err}"
    );
}

#[test]
fn delete_track_updates_board_and_undo_redo_restore_it() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let before = engine.get_net_info().expect("net info should query");
    let deleted_uuid = engine
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .and_then(|board| board.tracks.keys().next().copied())
        .expect("fixture should have at least one track");

    let delete = engine
        .delete_track(&deleted_uuid)
        .expect("delete_track should succeed");
    assert_eq!(delete.diff.deleted.len(), 1);
    assert!(engine.can_undo());

    let after_delete = engine.get_net_info().expect("net info should query");
    assert_ne!(before, after_delete);

    let undo = engine.undo().expect("undo should succeed");
    assert_eq!(undo.diff.created.len(), 1);
    let after_undo = engine.get_net_info().expect("net info should query");
    assert_eq!(before, after_undo);
    assert!(engine.can_redo());

    let redo = engine.redo().expect("redo should succeed");
    assert_eq!(redo.diff.deleted.len(), 1);
    let after_redo = engine.get_net_info().expect("net info should query");
    assert_eq!(after_delete, after_redo);
}

#[test]
fn delete_via_updates_board_and_undo_redo_restore_it() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let before = engine.get_net_info().expect("net info should query");
    let deleted_uuid = engine
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .and_then(|board| board.vias.keys().next().copied())
        .expect("fixture should have at least one via");

    let delete = engine
        .delete_via(&deleted_uuid)
        .expect("delete_via should succeed");
    assert_eq!(delete.diff.deleted.len(), 1);
    assert_eq!(delete.diff.deleted[0].object_type, "via");
    assert!(engine.can_undo());

    let after_delete = engine.get_net_info().expect("net info should query");
    assert_ne!(before, after_delete);

    let undo = engine.undo().expect("undo should succeed");
    assert_eq!(undo.diff.created.len(), 1);
    assert_eq!(undo.diff.created[0].object_type, "via");
    let after_undo = engine.get_net_info().expect("net info should query");
    assert_eq!(before, after_undo);
    assert!(engine.can_redo());

    let redo = engine.redo().expect("redo should succeed");
    assert_eq!(redo.diff.deleted.len(), 1);
    assert_eq!(redo.diff.deleted[0].object_type, "via");
    let after_redo = engine.get_net_info().expect("net info should query");
    assert_eq!(after_delete, after_redo);
}

#[test]
fn delete_component_updates_board_and_undo_redo_restore_it() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let before = engine.get_components().expect("components should query");
    let deleted_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();

    let delete = engine
        .delete_component(&deleted_uuid)
        .expect("delete_component should succeed");
    assert_eq!(delete.diff.deleted.len(), 1);
    assert_eq!(delete.diff.deleted[0].object_type, "component");
    assert!(engine.can_undo());

    let after_delete = engine.get_components().expect("components should query");
    assert!(after_delete.iter().all(|component| component.uuid != deleted_uuid));

    let undo = engine.undo().expect("undo should succeed");
    assert_eq!(undo.diff.created.len(), 1);
    assert_eq!(undo.diff.created[0].object_type, "component");
    let after_undo = engine.get_components().expect("components should query");
    assert_eq!(before, after_undo);
    assert!(engine.can_redo());

    let redo = engine.redo().expect("redo should succeed");
    assert_eq!(redo.diff.deleted.len(), 1);
    assert_eq!(redo.diff.deleted[0].object_type, "component");
    let after_redo = engine.get_components().expect("components should query");
    assert_eq!(after_delete, after_redo);
}

#[test]
fn delete_track_updates_derived_board_views_immediately() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let baseline_sig = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");
    let baseline_airwires = engine.get_unrouted().expect("unrouted should query");
    let baseline_diagnostics = engine
        .get_connectivity_diagnostics()
        .expect("diagnostics should query");
    let baseline_drc = engine
        .run_drc(&[RuleType::Connectivity])
        .expect("drc should run");

    assert_eq!(baseline_sig.tracks, 1);
    assert_eq!(baseline_airwires.len(), 1);
    assert!(
        baseline_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == "partially_routed_net")
    );
    assert!(
        baseline_drc
            .violations
            .iter()
            .any(|violation| violation.code == "connectivity_unrouted_net")
    );
    assert!(
        !baseline_drc
            .violations
            .iter()
            .any(|violation| violation.code == "connectivity_no_copper")
    );

    engine
        .delete_track(&uuid::Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap())
        .expect("delete_track should succeed");

    let after_sig = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "SIG")
        .expect("SIG net should exist");
    let after_airwires = engine.get_unrouted().expect("unrouted should query");
    let after_diagnostics = engine
        .get_connectivity_diagnostics()
        .expect("diagnostics should query");
    let after_drc = engine
        .run_drc(&[RuleType::Connectivity])
        .expect("drc should run");

    assert_eq!(after_sig.tracks, 0);
    assert_eq!(after_sig.routed_length_nm, 0);
    assert_eq!(after_airwires.len(), baseline_airwires.len());
    assert!(
        after_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == "net_without_copper")
    );
    assert!(
        !after_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == "partially_routed_net")
    );
    assert!(
        after_drc
            .violations
            .iter()
            .any(|violation| violation.code == "connectivity_no_copper")
    );
    assert!(
        after_drc
            .violations
            .iter()
            .any(|violation| violation.code == "connectivity_unrouted_net")
    );
}

#[test]
fn delete_via_updates_derived_net_state_immediately() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let baseline_gnd = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist");
    assert_eq!(baseline_gnd.vias, 1);

    engine
        .delete_via(&uuid::Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap())
        .expect("delete_via should succeed");

    let after_gnd = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist");
    assert_eq!(after_gnd.vias, 0);
    assert_eq!(after_gnd.tracks, baseline_gnd.tracks);
    assert_eq!(after_gnd.routed_length_nm, baseline_gnd.routed_length_nm);
}

#[test]
fn save_persists_deleted_track_for_current_m3_slice() {
    let source = fixture_path("partial-route-demo.kicad_pcb");
    let target = unique_temp_path("datum-eda-save-modified-board.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    let deleted_uuid = engine
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .and_then(|board| board.tracks.keys().next().copied())
        .expect("fixture should have at least one track");
    engine
        .delete_track(&deleted_uuid)
        .expect("delete_track should succeed");

    let expected_after_delete = engine.get_net_info().expect("net info should query");

    engine
        .save(&target)
        .expect("save should persist current delete_track slice");

    let saved = fs::read_to_string(&target).expect("saved file should read");
    assert!(!saved.contains(&deleted_uuid.to_string()));

    let mut reloaded = Engine::new().expect("engine should initialize");
    reloaded
        .import(&target)
        .expect("saved board should reimport successfully");
    let actual_after_reload = reloaded.get_net_info().expect("net info should query");
    assert_eq!(actual_after_reload, expected_after_delete);

    let _ = fs::remove_file(&target);
}

#[test]
fn save_persists_deleted_via_for_current_m3_slice() {
    let source = fixture_path("simple-demo.kicad_pcb");
    let target = unique_temp_path("datum-eda-save-modified-via-board.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    let deleted_uuid = engine
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .and_then(|board| board.vias.keys().next().copied())
        .expect("fixture should have at least one via");
    engine
        .delete_via(&deleted_uuid)
        .expect("delete_via should succeed");

    let expected_after_delete = engine.get_net_info().expect("net info should query");

    engine
        .save(&target)
        .expect("save should persist current delete_via slice");

    let saved = fs::read_to_string(&target).expect("saved file should read");
    assert!(!saved.contains(&deleted_uuid.to_string()));

    let mut reloaded = Engine::new().expect("engine should initialize");
    reloaded
        .import(&target)
        .expect("saved board should reimport successfully");
    let actual_after_reload = reloaded.get_net_info().expect("net info should query");
    assert_eq!(actual_after_reload, expected_after_delete);

    let _ = fs::remove_file(&target);
}

#[test]
fn save_persists_deleted_component_for_current_m3_slice() {
    let source = fixture_path("partial-route-demo.kicad_pcb");
    let target = unique_temp_path("datum-eda-save-deleted-component-board.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    let deleted_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    engine
        .delete_component(&deleted_uuid)
        .expect("delete_component should succeed");

    let expected_after_delete = engine.get_components().expect("components should query");

    engine
        .save(&target)
        .expect("save should persist current delete_component slice");

    let saved = fs::read_to_string(&target).expect("saved file should read");
    assert!(!saved.contains(&deleted_uuid.to_string()));

    let mut reloaded = Engine::new().expect("engine should initialize");
    reloaded
        .import(&target)
        .expect("saved board should reimport successfully");
    let actual_after_reload = reloaded.get_components().expect("components should query");
    assert_eq!(actual_after_reload, expected_after_delete);

    let _ = fs::remove_file(&target);
}

#[test]
fn set_design_rule_updates_board_and_undo_redo_restore_it() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let before = engine.get_design_rules().expect("rules should query");
    assert!(before.is_empty());

    let set = engine
        .set_design_rule(SetDesignRuleInput {
            rule_type: RuleType::ClearanceCopper,
            scope: crate::rules::ast::RuleScope::All,
            parameters: crate::rules::ast::RuleParams::Clearance { min: 125_000 },
            priority: 10,
            name: Some("default clearance".to_string()),
        })
        .expect("set_design_rule should succeed");
    assert_eq!(set.diff.created.len(), 1);

    let after_set = engine.get_design_rules().expect("rules should query");
    assert_eq!(after_set.len(), 1);
    assert_eq!(after_set[0].name, "default clearance");

    let undo = engine.undo().expect("undo should succeed");
    assert_eq!(undo.diff.deleted.len(), 1);
    let after_undo = engine.get_design_rules().expect("rules should query");
    assert!(after_undo.is_empty());

    let redo = engine.redo().expect("redo should succeed");
    assert_eq!(redo.diff.created.len(), 1);
    let after_redo = engine.get_design_rules().expect("rules should query");
    assert_eq!(after_redo.len(), 1);
    assert_eq!(after_redo[0].name, "default clearance");
}

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
            component.uuid
                == uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
        })
        .unwrap();
    assert_eq!(updated.value, "ALTAMP");
    let restored_part = reloaded
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .and_then(|board| {
            board.packages.get(&uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
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
        .map(|board| board.pads.values().filter(|pad| pad.package == component_uuid).count())
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
            component.uuid
                == uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
        })
        .unwrap();
    assert_eq!(updated.package_uuid, package_uuid);
    let restored_package = reloaded
        .design
        .as_ref()
        .and_then(|design| design.board.as_ref())
        .and_then(|board| {
            board.packages.get(&uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
        })
        .map(|package| package.package)
        .expect("reloaded package should exist");
    assert_eq!(restored_package, package_uuid);

    let _ = fs::remove_file(&target);
    let _ = fs::remove_file(package_assignments_sidecar::sidecar_path_for_source(&target));
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
    assert_eq!(result.description, format!("replace_component {}", component_uuid));
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
    let reverted = engine.get_components().expect("components should query after undo");
    assert_eq!(
        reverted
            .iter()
            .filter(|component| component.value == "10k")
            .count(),
        2
    );

    let redo = engine.redo().expect("redo should succeed");
    assert_eq!(redo.description, "redo replace_components 2");
    let redone = engine.get_components().expect("components should query after redo");
    assert_eq!(
        redone
            .iter()
            .filter(|component| component.value == "ALTAMP")
            .count(),
        2
    );
}

#[test]
fn set_net_class_updates_board_and_undo_redo_restore_it() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let before = engine.get_net_info().expect("net info should query");
    let gnd = before
        .iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist");
    assert_eq!(gnd.class, "Default");

    let set_result = engine
        .set_net_class(SetNetClassInput {
            net_uuid: gnd.uuid,
            class_name: "power".to_string(),
            clearance: 125_000,
            track_width: 250_000,
            via_drill: 300_000,
            via_diameter: 600_000,
            diffpair_width: 0,
            diffpair_gap: 0,
        })
        .expect("set_net_class should succeed");
    assert_eq!(set_result.diff.modified.len(), 1);

    let after_set = engine.get_net_info().expect("net info should query");
    let gnd_after_set = after_set
        .iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist after set");
    assert_eq!(gnd_after_set.class, "power");

    let undo = engine.undo().expect("undo should succeed");
    assert_eq!(undo.diff.modified.len(), 1);
    let after_undo = engine.get_net_info().expect("net info should query");
    let gnd_after_undo = after_undo
        .iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist after undo");
    assert_eq!(gnd_after_undo.class, "Default");

    let redo = engine.redo().expect("redo should succeed");
    assert_eq!(redo.diff.modified.len(), 1);
    let after_redo = engine.get_net_info().expect("net info should query");
    let gnd_after_redo = after_redo
        .iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist after redo");
    assert_eq!(gnd_after_redo.class, "power");
}

#[test]
fn save_persists_set_net_class_sidecar_for_current_m3_slice() {
    let source = fixture_path("simple-demo.kicad_pcb");
    let target = unique_temp_path("datum-eda-save-net-class-board.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    let gnd_uuid = engine
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist")
        .uuid;
    engine
        .set_net_class(SetNetClassInput {
            net_uuid: gnd_uuid,
            class_name: "power".to_string(),
            clearance: 125_000,
            track_width: 250_000,
            via_drill: 300_000,
            via_diameter: 600_000,
            diffpair_width: 0,
            diffpair_gap: 0,
        })
        .expect("set_net_class should succeed");

    engine.save(&target).expect("save should succeed");

    let sidecar = net_classes_sidecar::sidecar_path_for_source(&target);
    assert!(sidecar.exists());

    let mut reloaded = Engine::new().expect("engine should initialize");
    reloaded
        .import(&target)
        .expect("saved board should reimport successfully");
    let gnd = reloaded
        .get_net_info()
        .expect("net info should query")
        .into_iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist after reload");
    assert_eq!(gnd.class, "power");

    let _ = fs::remove_file(&target);
    let _ = fs::remove_file(sidecar);
}

#[test]
fn save_persists_set_design_rule_sidecar_for_current_m3_slice() {
    let source = fixture_path("simple-demo.kicad_pcb");
    let target = unique_temp_path("datum-eda-save-rule-board.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    engine
        .set_design_rule(SetDesignRuleInput {
            rule_type: RuleType::ClearanceCopper,
            scope: crate::rules::ast::RuleScope::All,
            parameters: crate::rules::ast::RuleParams::Clearance { min: 125_000 },
            priority: 10,
            name: Some("default clearance".to_string()),
        })
        .expect("set_design_rule should succeed");

    engine.save(&target).expect("save should succeed");

    let sidecar = crate::import::rules_sidecar::sidecar_path_for_source(&target);
    assert!(sidecar.exists());

    let mut reloaded = Engine::new().expect("engine should initialize");
    reloaded
        .import(&target)
        .expect("saved board should reimport successfully");
    let rules = reloaded.get_design_rules().expect("rules should query");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "default clearance");

    let _ = fs::remove_file(&target);
    let _ = fs::remove_file(&sidecar);
}

#[test]
fn move_component_updates_board_and_undo_redo_restore_it() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let before = engine.get_components().expect("components should query");
    let package_uuid = uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
    let move_result = engine
        .move_component(MoveComponentInput {
            uuid: package_uuid,
            position: crate::ir::geometry::Point::new(15_000_000, 12_000_000),
            rotation: Some(90),
        })
        .expect("move_component should succeed");
    assert_eq!(move_result.diff.modified.len(), 1);

    let after_move = engine.get_components().expect("components should query");
    assert_ne!(before, after_move);

    let undo = engine.undo().expect("undo should succeed");
    assert_eq!(undo.diff.modified.len(), 1);
    let after_undo = engine.get_components().expect("components should query");
    assert_eq!(before, after_undo);

    let redo = engine.redo().expect("redo should succeed");
    assert_eq!(redo.diff.modified.len(), 1);
    let after_redo = engine.get_components().expect("components should query");
    assert_eq!(after_move, after_redo);
}

#[test]
fn move_component_updates_derived_board_views_immediately() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture import should succeed");

    let baseline_airwires = engine.get_unrouted().expect("unrouted should query");
    let baseline_diagnostics = engine
        .get_connectivity_diagnostics()
        .expect("diagnostics should query");
    let baseline_drc = engine
        .run_drc(&[RuleType::Connectivity])
        .expect("drc should run");

    assert_eq!(baseline_airwires.len(), 1);
    assert!(
        baseline_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == "partially_routed_net")
    );
    assert!(
        baseline_drc
            .violations
            .iter()
            .any(|violation| violation.code == "connectivity_unrouted_net")
    );

    engine
        .move_component(MoveComponentInput {
            uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            position: crate::ir::geometry::Point::new(15_000_000, 12_000_000),
            rotation: Some(90),
        })
        .expect("move_component should succeed");

    let after_airwires = engine.get_unrouted().expect("unrouted should query");
    let after_diagnostics = engine
        .get_connectivity_diagnostics()
        .expect("diagnostics should query");
    let after_drc = engine
        .run_drc(&[RuleType::Connectivity])
        .expect("drc should run");

    assert_eq!(after_airwires.len(), 1);
    assert_ne!(
        after_airwires[0].distance_nm,
        baseline_airwires[0].distance_nm
    );
    assert!(
        after_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == "partially_routed_net")
    );
    assert!(
        after_drc
            .violations
            .iter()
            .any(|violation| violation.code == "connectivity_unrouted_net")
    );
}

#[test]
fn save_persists_moved_component_for_current_m3_slice() {
    let source = fixture_path("partial-route-demo.kicad_pcb");
    let target = unique_temp_path("datum-eda-save-moved-component-board.kicad_pcb");

    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&source)
        .expect("fixture import should succeed");
    engine
        .move_component(MoveComponentInput {
            uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            position: crate::ir::geometry::Point::new(15_000_000, 12_000_000),
            rotation: Some(90),
        })
        .expect("move_component should succeed");

    engine.save(&target).expect("save should succeed");

    let saved = fs::read_to_string(&target).expect("saved file should read");
    assert!(saved.contains("(at 15 12 90)"));

    let mut reloaded = Engine::new().expect("engine should initialize");
    reloaded
        .import(&target)
        .expect("saved board should reimport successfully");
    let moved = reloaded.get_components().expect("components should query");
    let r1 = moved
        .iter()
        .find(|component| component.reference == "R1")
        .unwrap();
    assert_eq!(r1.position.x, 15_000_000);
    assert_eq!(r1.position.y, 12_000_000);
    assert_eq!(r1.rotation, 90);

    let _ = fs::remove_file(&target);
}
