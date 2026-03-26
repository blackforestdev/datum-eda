use super::super::*;
use crate::import::net_classes_sidecar;
use std::fs;

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
