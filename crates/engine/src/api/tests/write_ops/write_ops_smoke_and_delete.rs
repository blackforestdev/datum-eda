use super::super::*;
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
