use super::*;
use crate::board::{Net, PlacedPackage, Stackup, StackupLayer, StackupLayerType};
use crate::ir::geometry::{Point, Polygon};
use crate::schematic::{CheckDomain, LabelKind, NetLabel, Sheet, WaiverTarget};
use std::collections::HashMap;
use std::path::Path;

#[test]
fn summaries_require_open_project() {
    let engine = Engine::new().expect("engine should initialize");
    assert!(matches!(
        engine.get_board_summary(),
        Err(EngineError::NoProjectOpen)
    ));
    assert!(matches!(
        engine.get_schematic_summary(),
        Err(EngineError::NoProjectOpen)
    ));
}

#[test]
fn schematic_check_summary_includes_info_level_erc_codes() {
    let summary = summarize_schematic_checks(
        &[],
        &[crate::erc::ErcFinding {
            id: uuid::Uuid::new_v4(),
            code: "input_without_explicit_driver",
            severity: crate::erc::ErcSeverity::Info,
            message: "analog input has passive biasing".into(),
            net_name: Some("IN_P".into()),
            component: None,
            pin: None,
            objects: vec![crate::erc::ErcObjectRef {
                kind: "pin",
                key: "Q1.1".into(),
            }],
            object_uuids: vec![uuid::Uuid::new_v4()],
            waived: false,
        }],
    );

    assert_eq!(summary.status, CheckStatus::Info);
    assert_eq!(summary.errors, 0);
    assert_eq!(summary.warnings, 0);
    assert_eq!(summary.infos, 1);
    assert_eq!(summary.waived, 0);
    assert!(
        summary
            .by_code
            .iter()
            .any(|entry| entry.code == "input_without_explicit_driver" && entry.count == 1)
    );
}

#[test]
fn summaries_read_from_in_memory_design() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine.design = Some(Design {
        board: Some(Board {
            uuid: uuid::Uuid::new_v4(),
            name: "demo-board".into(),
            stackup: Stackup {
                layers: vec![StackupLayer {
                    id: 1,
                    name: "Top".into(),
                    layer_type: StackupLayerType::Copper,
                    thickness_nm: 35_000,
                }],
            },
            outline: Polygon::new(vec![
                Point::new(0, 0),
                Point::new(10, 0),
                Point::new(10, 10),
                Point::new(0, 10),
            ]),
            packages: HashMap::from([(
                uuid::Uuid::new_v4(),
                PlacedPackage {
                    uuid: uuid::Uuid::new_v4(),
                    part: uuid::Uuid::new_v4(),
                    package: uuid::Uuid::nil(),
                    reference: "R1".into(),
                    value: "10k".into(),
                    position: Point::new(0, 0),
                    rotation: 0,
                    layer: 1,
                    locked: false,
                },
            )]),
            pads: HashMap::new(),
            tracks: HashMap::new(),
            vias: HashMap::new(),
            zones: HashMap::new(),
            nets: HashMap::from([(
                uuid::Uuid::new_v4(),
                Net {
                    uuid: uuid::Uuid::new_v4(),
                    name: "VCC".into(),
                    class: uuid::Uuid::new_v4(),
                },
            )]),
            net_classes: HashMap::new(),
            rules: Vec::new(),
            keepouts: Vec::new(),
            dimensions: Vec::new(),
            texts: Vec::new(),
        }),
        schematic: Some(Schematic {
            uuid: uuid::Uuid::new_v4(),
            sheets: HashMap::from([(
                uuid::Uuid::new_v4(),
                Sheet {
                    uuid: uuid::Uuid::new_v4(),
                    name: "Sheet1".into(),
                    frame: None,
                    symbols: HashMap::new(),
                    wires: HashMap::new(),
                    junctions: HashMap::new(),
                    labels: HashMap::from([(
                        uuid::Uuid::new_v4(),
                        NetLabel {
                            uuid: uuid::Uuid::new_v4(),
                            kind: LabelKind::Local,
                            name: "VCC".into(),
                            position: Point::new(0, 0),
                        },
                    )]),
                    buses: HashMap::new(),
                    bus_entries: HashMap::new(),
                    ports: HashMap::new(),
                    noconnects: HashMap::new(),
                    texts: HashMap::new(),
                    drawings: HashMap::new(),
                },
            )]),
            sheet_definitions: HashMap::new(),
            sheet_instances: HashMap::new(),
            variants: HashMap::new(),
            waivers: Vec::new(),
        }),
    });

    let board_summary = engine
        .get_board_summary()
        .expect("board summary should exist");
    let schematic_summary = engine
        .get_schematic_summary()
        .expect("schematic summary should exist");

    assert_eq!(board_summary.name, "demo-board");
    assert_eq!(board_summary.component_count, 1);
    assert_eq!(schematic_summary.sheet_count, 1);
    assert_eq!(schematic_summary.net_label_count, 1);
    assert_eq!(engine.get_sheets().unwrap().len(), 1);
}

#[test]
fn get_symbol_fields_returns_fields_for_matching_symbol() {
    let symbol_uuid = uuid::Uuid::new_v4();
    let field_uuid = uuid::Uuid::new_v4();
    let mut engine = Engine::new().expect("engine should initialize");
    engine.design = Some(Design {
        board: None,
        schematic: Some(Schematic {
            uuid: uuid::Uuid::new_v4(),
            sheets: HashMap::from([(
                uuid::Uuid::new_v4(),
                Sheet {
                    uuid: uuid::Uuid::new_v4(),
                    name: "Root".into(),
                    frame: None,
                    symbols: HashMap::from([(
                        symbol_uuid,
                        crate::schematic::PlacedSymbol {
                            uuid: symbol_uuid,
                            part: None,
                            entity: None,
                            gate: None,
                            lib_id: None,
                            reference: "R1".into(),
                            value: "10k".into(),
                            fields: vec![crate::schematic::SymbolField {
                                uuid: field_uuid,
                                key: "Value".into(),
                                value: "10k".into(),
                                position: Some(Point::new(1, 2)),
                                visible: true,
                            }],
                            pins: Vec::new(),
                            position: Point::new(0, 0),
                            rotation: 0,
                            mirrored: false,
                            unit_selection: None,
                            display_mode: crate::schematic::SymbolDisplayMode::LibraryDefault,
                            pin_overrides: Vec::new(),
                            hidden_power_behavior:
                                crate::schematic::HiddenPowerBehavior::PreservedAsImportedMetadata,
                        },
                    )]),
                    wires: HashMap::new(),
                    junctions: HashMap::new(),
                    labels: HashMap::new(),
                    buses: HashMap::new(),
                    bus_entries: HashMap::new(),
                    ports: HashMap::new(),
                    noconnects: HashMap::new(),
                    texts: HashMap::new(),
                    drawings: HashMap::new(),
                },
            )]),
            sheet_definitions: HashMap::new(),
            sheet_instances: HashMap::new(),
            variants: HashMap::new(),
            waivers: Vec::new(),
        }),
    });

    let fields = engine
        .get_symbol_fields(&symbol_uuid)
        .expect("symbol fields query should succeed");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].uuid, field_uuid);
    assert_eq!(fields[0].symbol, symbol_uuid);
    assert_eq!(fields[0].key, "Value");
    assert_eq!(fields[0].value, "10k");
    assert!(fields[0].visible);
}

#[test]
fn get_symbol_fields_returns_not_found_for_unknown_symbol() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_sch"))
        .expect("fixture should import");
    let missing = uuid::Uuid::new_v4();
    assert!(matches!(
        engine.get_symbol_fields(&missing),
        Err(EngineError::NotFound {
            object_type: "symbol",
            uuid
        }) if uuid == missing
    ));
}

#[test]
fn get_bus_entries_returns_entries_for_matching_sheet_selection() {
    let sheet_uuid = uuid::Uuid::new_v4();
    let bus_uuid = uuid::Uuid::new_v4();
    let entry_uuid = uuid::Uuid::new_v4();
    let mut engine = Engine::new().expect("engine should initialize");
    engine.design = Some(Design {
        board: None,
        schematic: Some(Schematic {
            uuid: uuid::Uuid::new_v4(),
            sheets: HashMap::from([(
                sheet_uuid,
                Sheet {
                    uuid: sheet_uuid,
                    name: "Root".into(),
                    frame: None,
                    symbols: HashMap::new(),
                    wires: HashMap::new(),
                    junctions: HashMap::new(),
                    labels: HashMap::new(),
                    buses: HashMap::from([(
                        bus_uuid,
                        crate::schematic::Bus {
                            uuid: bus_uuid,
                            name: "DATA".into(),
                            members: vec!["DATA0".into()],
                        },
                    )]),
                    bus_entries: HashMap::from([(
                        entry_uuid,
                        crate::schematic::BusEntry {
                            uuid: entry_uuid,
                            bus: bus_uuid,
                            wire: None,
                            position: Point::new(10, 20),
                        },
                    )]),
                    ports: HashMap::new(),
                    noconnects: HashMap::new(),
                    texts: HashMap::new(),
                    drawings: HashMap::new(),
                },
            )]),
            sheet_definitions: HashMap::new(),
            sheet_instances: HashMap::new(),
            variants: HashMap::new(),
            waivers: Vec::new(),
        }),
    });

    let all = engine
        .get_bus_entries(None)
        .expect("all-sheet bus entries should succeed");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].uuid, entry_uuid);
    assert_eq!(all[0].sheet, sheet_uuid);

    let selected = engine
        .get_bus_entries(Some(&sheet_uuid))
        .expect("sheet-select bus entries should succeed");
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].bus, bus_uuid);
}

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
    let components = engine.get_components().expect("component query should succeed");
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
    let unrouted = engine.get_unrouted().expect("unrouted query should succeed");
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
    assert!(partial_diagnostics.iter().any(
        |diagnostic| diagnostic.kind == "partially_routed_net" && diagnostic.severity == "warning"
    ));
    assert!(partial_diagnostics.iter().any(
        |diagnostic| diagnostic.kind == "net_without_copper" && diagnostic.severity == "info"
    ));
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
    assert_eq!(schematic_summary.sheet_count, 1);
    assert_eq!(schematic_summary.net_label_count, 3);
    let sheets = engine.get_sheets().expect("sheet query should succeed");
    assert_eq!(sheets.len(), 1);
    assert_eq!(sheets[0].labels, 3);
    assert_eq!(sheets[0].symbols, 1);
    assert_eq!(sheets[0].ports, 1);
    let symbols = engine.get_symbols(None).expect("symbol query should succeed");
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].reference, "R1");
    let ports = engine.get_ports(None).expect("port query should succeed");
    assert_eq!(ports.len(), 1);
    assert_eq!(ports[0].name, "SUB_IN");
    let labels = engine.get_labels(None).expect("label query should succeed");
    assert_eq!(labels.len(), 3);
    let buses = engine.get_buses(None).expect("bus query should succeed");
    assert_eq!(buses.len(), 1);
    let noconnects = engine
        .get_noconnects(None)
        .expect("no-connect query should succeed");
    assert_eq!(noconnects.len(), 1);
    let hierarchy = engine.get_hierarchy().expect("hierarchy query should succeed");
    assert_eq!(hierarchy.instances.len(), 1);
    assert!(hierarchy.links.is_empty());
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
    assert_eq!(sub_in.labels, 1);
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
        } => {
            assert_eq!(summary.status, CheckStatus::Warning);
            assert_eq!(summary.errors, 0);
            assert_eq!(summary.warnings, 3);
            assert_eq!(summary.infos, 0);
            assert_eq!(summary.waived, 0);
            assert_eq!(summary.by_code.len(), 3);
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
    let erc = engine.run_erc_prechecks().expect("ERC precheck should succeed");
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
fn get_part_returns_pool_part_details() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&eagle_fixture_path("bjt-sot23.lbr"))
        .expect("fixture should import");
    let part_uuid = engine
        .search_pool("sot23")
        .expect("search should succeed")
        .first()
        .expect("part should exist")
        .uuid;
    let part = engine.get_part(&part_uuid).expect("part query should succeed");
    assert_eq!(part.uuid, part_uuid);
    assert!(!part.package.name.is_empty());
    assert!(part.package.pads > 0);
    assert!(!part.entity.gates.is_empty());
}

#[test]
fn get_package_returns_pool_package_details() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&eagle_fixture_path("bjt-sot23.lbr"))
        .expect("fixture should import");
    let part_uuid = engine
        .search_pool("sot23")
        .expect("search should succeed")
        .first()
        .expect("part should exist")
        .uuid;
    let part = engine.get_part(&part_uuid).expect("part query should succeed");
    let package_uuid = engine
        .pool
        .parts
        .get(&part_uuid)
        .expect("part should exist in pool")
        .package;
    let package = engine
        .get_package(&package_uuid)
        .expect("package query should succeed");
    assert_eq!(package.uuid, package_uuid);
    assert_eq!(package.name, part.package.name);
    assert!(!package.pads.is_empty());
}

#[test]
fn get_package_change_candidates_reports_unique_compatible_packages() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import_eagle_library(&eagle_fixture_path("simple-opamp.lbr"))
        .expect("library import should succeed");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture should import");
    let lmv321_part_uuid = engine
        .search_pool("LMV321")
        .expect("search should succeed")
        .first()
        .expect("LMV321 part should exist")
        .uuid;
    engine
        .assign_part(AssignPartInput {
            uuid: uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            part_uuid: lmv321_part_uuid,
        })
        .expect("assign_part should succeed");

    let report = engine
        .get_package_change_candidates(
            &uuid::Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        )
        .expect("candidate query should succeed");
    assert_eq!(
        report.status,
        PackageChangeCompatibilityStatus::CandidatesAvailable
    );
    assert_eq!(report.current_part_uuid, Some(lmv321_part_uuid));
    assert_eq!(report.ambiguous_package_count, 0);
    assert_eq!(report.candidates.len(), 1);
    assert_eq!(report.candidates[0].package_name, "ALT-3");
    assert_eq!(report.candidates[0].compatible_part_value, "ALTAMP");
}

#[test]
fn get_netlist_returns_board_nets_for_board_project() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_pcb"))
        .expect("fixture should import");
    let nets = engine.get_netlist().expect("netlist query should succeed");
    assert_eq!(nets.len(), 2);
    let gnd = nets
        .iter()
        .find(|net| net.name == "GND")
        .expect("GND net should exist");
    assert_eq!(gnd.class.as_deref(), Some("Default"));
    assert!(gnd.routed_pct.is_some());
    assert!(gnd.labels.is_none());
    assert!(gnd.ports.is_none());
}

#[test]
fn explain_violation_returns_erc_explanation_for_valid_index() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_sch"))
        .expect("fixture should import");
    let explanation = engine
        .explain_violation(ViolationDomain::Erc, 0)
        .expect("explanation should succeed");
    assert!(!explanation.explanation.is_empty());
    assert!(explanation.rule_detail.starts_with("erc "));
    assert!(!explanation.suggestion.is_empty());
}

#[test]
fn explain_violation_returns_drc_explanation_for_valid_index() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("partial-route-demo.kicad_pcb"))
        .expect("fixture should import");
    let explanation = engine
        .explain_violation(ViolationDomain::Drc, 0)
        .expect("explanation should succeed");
    assert!(!explanation.explanation.is_empty());
    assert!(explanation.rule_detail.starts_with("drc "));
    assert!(!explanation.suggestion.is_empty());
}

#[test]
fn get_netlist_returns_schematic_nets_for_schematic_project() {
    let mut engine = Engine::new().expect("engine should initialize");
    engine
        .import(&fixture_path("simple-demo.kicad_sch"))
        .expect("fixture should import");
    let nets = engine.get_netlist().expect("netlist query should succeed");
    assert_eq!(nets.len(), 4);
    let vcc = nets
        .iter()
        .find(|net| net.name == "VCC")
        .expect("VCC net should exist");
    assert_eq!(vcc.semantic_class.as_deref(), Some("power"));
    assert!(vcc.routed_pct.is_none());
    assert_eq!(vcc.labels, Some(1));
}
