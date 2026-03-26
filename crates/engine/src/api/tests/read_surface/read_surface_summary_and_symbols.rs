use super::super::*;
use crate::board::{Net, PlacedPackage, Stackup, StackupLayer, StackupLayerType};
use crate::ir::geometry::{Point, Polygon};
use crate::schematic::{LabelKind, NetLabel, Sheet};
use std::collections::HashMap;

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
