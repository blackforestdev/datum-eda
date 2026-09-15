use super::super::*;
use crate::schematic::{CheckDomain, CheckWaiver, Schematic, WaiverTarget};
use std::path::Path;

#[test]
fn import_dispatch_recognizes_kicad_and_eagle_paths() {
    let mut engine = Engine::new().expect("engine should initialize");

    let err = engine
        .import(Path::new("demo.kicad_pcb"))
        .expect_err("bare KiCad board path should fail because fixture is absent");
    assert!(matches!(err, EngineError::Io(_)), "{err}");

    let report = engine
        .import(&fixture_path("simple-demo.kicad_pcb"))
        .expect("KiCad board skeleton import should succeed");
    assert!(matches!(report.kind, ImportKind::KiCadBoard));
    assert_eq!(
        report.metadata.get("footprint_count").map(String::as_str),
        Some("1")
    );
    let board_summary = engine
        .get_board_summary()
        .expect("imported board should populate in-memory design");
    assert_eq!(board_summary.name, "simple-demo");
    assert_eq!(board_summary.component_count, 1);
    assert_eq!(board_summary.net_count, 2);
    let components = engine
        .get_components()
        .expect("component query should succeed");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].reference, "R1");
    let nets = engine.get_net_info().expect("net query should succeed");
    assert_eq!(nets.len(), 2);
    let gnd = nets
        .iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist");
    assert_eq!(gnd.tracks, 1);
    assert_eq!(gnd.vias, 1);
    let stackup = engine.get_stackup().expect("stackup query should succeed");
    assert_eq!(stackup.layers.len(), 3);
    assert_eq!(stackup.layers[0].name, "F.Cu");
    let board_diagnostics = engine
        .get_connectivity_diagnostics()
        .expect("board diagnostics query should succeed");
    assert_eq!(board_diagnostics.len(), 1);
    assert_eq!(board_diagnostics[0].kind, "net_without_copper");
    assert_eq!(board_diagnostics[0].objects.len(), 1);
    let board_report = engine
        .get_check_report()
        .expect("board check report should succeed");
    match board_report {
        CheckReport::Board {
            summary,
            diagnostics,
        } => {
            assert_eq!(summary.status, CheckStatus::Info);
            assert_eq!(summary.errors, 0);
            assert_eq!(summary.warnings, 0);
            assert_eq!(summary.infos, 1);
            assert_eq!(summary.waived, 0);
            assert_eq!(summary.by_code.len(), 1);
            assert_eq!(summary.by_code[0].code, "net_without_copper");
            assert_eq!(summary.by_code[0].count, 1);
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].kind, "net_without_copper");
        }
        other => panic!("expected board check report, got {other:?}"),
    }

    let airwire_report = engine
        .import(&fixture_path("airwire-demo.kicad_pcb"))
        .expect("airwire fixture import should succeed");
    assert!(matches!(airwire_report.kind, ImportKind::KiCadBoard));
    let unrouted = engine
        .get_unrouted()
        .expect("unrouted query should succeed");
    assert_eq!(unrouted.len(), 1);
    assert_eq!(unrouted[0].net_name, "SIG");
    assert_eq!(unrouted[0].from.component, "R1");
    assert_eq!(unrouted[0].to.component, "R2");

    let partial_route_report = engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("partial-route fixture import should succeed");
    assert!(matches!(partial_route_report.kind, ImportKind::KiCadBoard));
    let partial_unrouted = engine
        .get_unrouted()
        .expect("partial-route unrouted query should succeed");
    assert_eq!(partial_unrouted.len(), 1);
    assert_eq!(partial_unrouted[0].net_name, "SIG");
    assert_eq!(partial_unrouted[0].from.component, "R1");
    assert_eq!(partial_unrouted[0].to.component, "R2");
    let partial_diagnostics = engine
        .get_connectivity_diagnostics()
        .expect("partial-route diagnostics query should succeed");
    assert_eq!(partial_diagnostics.len(), 2);
    assert!(
        partial_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == "partially_routed_net"
                && diagnostic.severity == "warning")
    );
    assert!(
        partial_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == "net_without_copper"
                && diagnostic.severity == "info")
    );
    let partial_report = engine
        .get_check_report()
        .expect("partial-route check report should succeed");
    match partial_report {
        CheckReport::Board {
            summary,
            diagnostics,
        } => {
            assert_eq!(summary.status, CheckStatus::Warning);
            assert_eq!(summary.errors, 0);
            assert_eq!(summary.warnings, 1);
            assert_eq!(summary.infos, 1);
            assert_eq!(summary.by_code.len(), 2);
            assert!(
                summary
                    .by_code
                    .iter()
                    .any(|entry| entry.code == "partially_routed_net" && entry.count == 1)
            );
            assert!(
                summary
                    .by_code
                    .iter()
                    .any(|entry| entry.code == "net_without_copper" && entry.count == 1)
            );
            assert_eq!(diagnostics.len(), 2);
            assert!(
                diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.kind == "partially_routed_net")
            );
        }
        other => panic!("expected board check report, got {other:?}"),
    }

    let err = engine
        .import(Path::new("demo.kicad_sch"))
        .expect_err("bare KiCad schematic path should fail because fixture is absent");
    assert!(matches!(err, EngineError::Io(_)), "{err}");

    let report = engine
        .import(&fixture_path("simple-demo.kicad_sch"))
        .expect("KiCad schematic skeleton import should succeed");
    assert!(matches!(report.kind, ImportKind::KiCadSchematic));
    assert_eq!(
        report.metadata.get("symbol_count").map(String::as_str),
        Some("1")
    );
    let schematic_summary = engine
        .get_schematic_summary()
        .expect("imported schematic should populate in-memory design");
    assert_eq!(schematic_summary.sheet_count, 2);
    assert_eq!(schematic_summary.net_label_count, 4);
    let sheets = engine.get_sheets().expect("sheet query should succeed");
    assert_eq!(sheets.len(), 2);
    assert!(sheets.iter().any(|sheet| {
        sheet.name == "Root" && sheet.labels == 3 && sheet.symbols == 1 && sheet.ports == 1
    }));
    assert!(sheets.iter().any(|sheet| {
        sheet.name == "Sub" && sheet.labels == 1 && sheet.symbols == 1 && sheet.ports == 0
    }));
    let symbols = engine
        .get_symbols(None)
        .expect("symbol query should succeed");
    assert_eq!(symbols.len(), 2);
    assert!(symbols.iter().any(|symbol| symbol.reference == "R1"));
    assert!(symbols.iter().any(|symbol| symbol.reference == "TP1"));
    let ports = engine.get_ports(None).expect("port query should succeed");
    assert_eq!(ports.len(), 1);
    assert_eq!(ports[0].name, "SUB_IN");
    let labels = engine.get_labels(None).expect("label query should succeed");
    assert_eq!(labels.len(), 4);
    let buses = engine.get_buses(None).expect("bus query should succeed");
    assert_eq!(buses.len(), 1);
    let noconnects = engine
        .get_noconnects(None)
        .expect("no-connect query should succeed");
    assert_eq!(noconnects.len(), 1);
    let hierarchy = engine
        .get_hierarchy()
        .expect("hierarchy query should succeed");
    assert_eq!(hierarchy.instances.len(), 1);
    assert_eq!(hierarchy.links.len(), 1);
    assert_eq!(hierarchy.instances[0].name, "Sub");
    let nets = engine
        .get_schematic_net_info()
        .expect("schematic net query should succeed");
    assert_eq!(nets.len(), 4);
    let scl = nets
        .iter()
        .find(|net| net.name == "SCL")
        .expect("SCL net should exist");
    assert_eq!(scl.labels, 1);
    assert_eq!(scl.ports, 0);
    assert_eq!(scl.pins.len(), 1);
    assert_eq!(scl.pins[0].component, "R1");
    assert_eq!(scl.pins[0].pin, "1");
    assert_eq!(scl.sheets, vec!["Root".to_string()]);
    let vcc = nets
        .iter()
        .find(|net| net.name == "VCC")
        .expect("VCC net should exist");
    assert_eq!(vcc.semantic_class.as_deref(), Some("power"));
    let sub_in = nets
        .iter()
        .find(|net| net.name == "SUB_IN")
        .expect("SUB_IN net should exist");
    assert_eq!(sub_in.ports, 1);
    assert_eq!(sub_in.labels, 2);
    assert_eq!(sub_in.pins.len(), 1);
    assert_eq!(sub_in.pins[0].component, "TP1");
    assert_eq!(sub_in.pins[0].pin, "1");
    assert_eq!(sub_in.sheets, vec!["Root".to_string(), "Sub".to_string()]);
    let diagnostics = engine
        .get_connectivity_diagnostics()
        .expect("diagnostics query should succeed");
    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == "dangling_component_pin")
    );
    let report = engine
        .get_check_report()
        .expect("schematic check report should succeed");
    match report {
        CheckReport::Schematic {
            summary,
            diagnostics,
            erc,
            drc,
        } => {
            assert_eq!(summary.status, CheckStatus::Warning);
            assert_eq!(summary.errors, 0);
            assert_eq!(summary.warnings, 3);
            assert_eq!(summary.infos, 0);
            assert_eq!(summary.waived, 0);
            assert_eq!(summary.by_code.len(), 3);
            assert!(drc.is_empty());
            assert!(
                summary
                    .by_code
                    .iter()
                    .any(|entry| entry.code == "dangling_component_pin" && entry.count == 1)
            );
            assert!(
                summary
                    .by_code
                    .iter()
                    .any(|entry| entry.code == "unconnected_component_pin" && entry.count == 1)
            );
            assert!(
                summary
                    .by_code
                    .iter()
                    .any(|entry| entry.code == "undriven_power_net" && entry.count == 1)
            );
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(erc.len(), 2);
        }
        other => panic!("expected schematic check report, got {other:?}"),
    }
    let dangling = nets
        .iter()
        .find(|net| net.name.starts_with("N$"))
        .expect("dangling symbol pin net should exist");
    assert_eq!(dangling.pins.len(), 1);
    assert_eq!(dangling.pins[0].component, "R1");
    assert_eq!(dangling.pins[0].pin, "2");
    let erc = engine
        .run_erc_prechecks()
        .expect("ERC precheck should succeed");
    assert_eq!(erc.len(), 2);
    let dangling_pin = erc
        .iter()
        .find(|finding| finding.code == "unconnected_component_pin")
        .expect("dangling pin finding should exist");
    assert_eq!(dangling_pin.component.as_deref(), Some("R1"));
    assert_eq!(dangling_pin.pin.as_deref(), Some("2"));
    let undriven_vcc = erc
        .iter()
        .find(|finding| finding.code == "undriven_power_net")
        .expect("undriven VCC finding should exist");
    assert_eq!(undriven_vcc.net_name.as_deref(), Some("VCC"));
    let mut config = ErcConfig::default();
    config
        .severity_overrides
        .insert("undriven_power_net".into(), erc::ErcSeverity::Error);
    let overridden = engine
        .run_erc_prechecks_with_config(&config)
        .expect("configured ERC precheck should succeed");
    let overridden_vcc = overridden
        .iter()
        .find(|finding| finding.code == "undriven_power_net")
        .expect("configured VCC finding should exist");
    assert_eq!(overridden_vcc.severity, erc::ErcSeverity::Error);
    assert_eq!(overridden_vcc.id, undriven_vcc.id);

    let waiver = CheckWaiver {
        uuid: uuid::Uuid::new_v4(),
        domain: CheckDomain::ERC,
        target: WaiverTarget::Object(dangling_pin.object_uuids[0]),
        rationale: "Intentional dangling input".into(),
        created_by: Some("api-test".into()),
    };
    let waived = engine
        .run_erc_prechecks_with_config_and_waivers(&ErcConfig::default(), &[waiver])
        .expect("configured ERC precheck with waiver should succeed");
    let waived_dangling = waived
        .iter()
        .find(|finding| finding.code == "unconnected_component_pin")
        .expect("waived dangling pin finding should exist");
    assert!(waived_dangling.waived);
    let still_unwaived_vcc = waived
        .iter()
        .find(|finding| finding.code == "undriven_power_net")
        .expect("unwaived VCC finding should exist");
    assert!(!still_unwaived_vcc.waived);

    let report = engine
        .import(&fixture_path("simple-demo.kicad_pro"))
        .expect("KiCad project metadata import should succeed");
    assert!(matches!(report.kind, ImportKind::KiCadProject));
    assert_eq!(
        report.metadata.get("project_name").map(String::as_str),
        Some("simple-demo")
    );

    let err = engine
        .import(Path::new("legacy.brd"))
        .expect_err("Eagle board import should be recognized but unimplemented");
    assert!(
        err.to_string()
            .contains("Eagle board import is not implemented yet"),
        "{}",
        err
    );
}

#[test]
fn import_dispatch_rejects_unknown_extensions() {
    let mut engine = Engine::new().expect("engine should initialize");
    let err = engine
        .import(Path::new("unknown.txt"))
        .expect_err("unknown extension must fail");
    assert!(err.to_string().contains("unsupported import path"), "{err}");
}

#[test]
fn run_drc_honors_authored_drc_waivers_from_loaded_schematic() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("partial-route board import should succeed");

    let initial = engine
        .run_drc(&[RuleType::Connectivity])
        .expect("initial DRC should succeed");
    assert!(!initial.passed);
    let violation = initial
        .violations
        .iter()
        .find(|entry| entry.code == "connectivity_unrouted_net")
        .expect("fixture should produce an unrouted-net violation");

    engine
        .design
        .as_mut()
        .expect("design should be present")
        .schematic = Some(Schematic {
        uuid: uuid::Uuid::new_v4(),
        sheets: Default::default(),
        sheet_definitions: Default::default(),
        sheet_instances: Default::default(),
        variants: Default::default(),
        waivers: vec![CheckWaiver {
            uuid: uuid::Uuid::new_v4(),
            domain: CheckDomain::DRC,
            target: WaiverTarget::Object(violation.objects[0]),
            rationale: "Intentional partial-route fixture waiver".into(),
            created_by: Some("api-test".into()),
        }],
    });

    let waived = engine
        .run_drc(&[RuleType::Connectivity])
        .expect("waived DRC should succeed");
    let waived_violation = waived
        .violations
        .iter()
        .find(|entry| entry.code == "connectivity_unrouted_net")
        .expect("fixture should still report the waived violation");
    assert!(waived.passed);
    assert_eq!(waived.summary.errors, 0);
    assert_eq!(waived.summary.waived, 1);
    assert!(waived_violation.waived);
}

#[test]
fn close_project_clears_open_design() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_pcb"))
        .expect("fixture should import");
    assert!(engine.has_open_project());
    engine.close_project();
    assert!(!engine.has_open_project());
    assert!(matches!(
        engine.get_board_summary(),
        Err(EngineError::NoProjectOpen)
    ));
}

#[test]
fn import_dispatch_expands_supported_kicad_bus_members_into_existing_surfaces() {
    let mut engine = Engine::new().expect("engine should initialize");

    let report = engine
        .import(&fixture_path("bus-demo.kicad_sch"))
        .expect("KiCad bus subset fixture should import");
    assert!(matches!(report.kind, ImportKind::KiCadSchematic));

    let buses = engine.get_buses(None).expect("bus query should succeed");
    assert_eq!(buses.len(), 1);
    assert_eq!(buses[0].name, "DATA");
    assert_eq!(
        buses[0].members,
        vec!["DATA0".to_string(), "DATA1".to_string()]
    );

    let bus_entries = engine
        .get_bus_entries(None)
        .expect("bus-entry query should succeed");
    assert_eq!(bus_entries.len(), 2);
    assert!(bus_entries.iter().all(|entry| entry.bus == buses[0].uuid));
    assert!(bus_entries.iter().all(|entry| entry.wire.is_some()));

    let labels = engine.get_labels(None).expect("label query should succeed");
    assert!(labels.iter().any(|label| label.name == "DATA[0..1]"));

    let nets = engine
        .get_schematic_net_info()
        .expect("schematic net query should succeed");
    assert_eq!(nets.len(), 2);
    assert!(nets.iter().any(|net| {
        net.name == "DATA0" && net.pins.len() == 1 && net.pins[0].component == "TP1"
    }));
    assert!(nets.iter().any(|net| {
        net.name == "DATA1" && net.pins.len() == 1 && net.pins[0].component == "TP2"
    }));

    let diagnostics = engine
        .get_connectivity_diagnostics()
        .expect("diagnostics query should succeed");
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");

    let report = engine
        .get_check_report()
        .expect("schematic check report should succeed");
    match report {
        CheckReport::Schematic {
            summary,
            diagnostics,
            erc,
            drc,
        } => {
            assert_eq!(summary.status, CheckStatus::Ok);
            assert_eq!(summary.errors, 0);
            assert_eq!(summary.warnings, 0);
            assert_eq!(summary.infos, 0);
            assert!(diagnostics.is_empty());
            assert!(erc.is_empty());
            assert!(drc.is_empty());
        }
        other => panic!("expected schematic report, got {other:?}"),
    }
}
