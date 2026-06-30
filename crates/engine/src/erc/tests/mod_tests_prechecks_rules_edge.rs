use std::collections::HashMap;

use uuid::Uuid;

use super::super::electrical::{
    PIN_ELECTRICAL_TAXONOMY_REVISION, canonical_pin_electrical_type_name, is_conflicting_output,
    is_explicit_driver, is_input, is_no_connect, is_passive, is_power_input,
};
use crate::erc::{ErcSeverity, run_prechecks};
use crate::ir::geometry::Point;
use crate::pool::LibraryPinElectricalType;
use crate::schematic::{
    CheckWaiver, HiddenPowerBehavior, LabelKind, NetLabel, PinElectricalType, PlacedSymbol,
    Schematic, Sheet, SymbolDisplayMode, SymbolPin, Variant,
};

#[test]
fn erc_pin_taxonomy_is_bound_to_library_alias_and_canonical_values() {
    let schematic_pin_type: PinElectricalType = LibraryPinElectricalType::OpenEmitter;
    assert_eq!(
        canonical_pin_electrical_type_name(&schematic_pin_type),
        "open_emitter"
    );
    assert_eq!(
        PIN_ELECTRICAL_TAXONOMY_REVISION,
        "LibraryPinElectricalType:v1"
    );

    let cases = [
        (LibraryPinElectricalType::Input, "input"),
        (LibraryPinElectricalType::Output, "output"),
        (LibraryPinElectricalType::Bidirectional, "bidirectional"),
        (LibraryPinElectricalType::Passive, "passive"),
        (LibraryPinElectricalType::PowerIn, "power_in"),
        (LibraryPinElectricalType::PowerOut, "power_out"),
        (LibraryPinElectricalType::OpenCollector, "open_collector"),
        (LibraryPinElectricalType::OpenEmitter, "open_emitter"),
        (LibraryPinElectricalType::TriState, "tri_state"),
        (LibraryPinElectricalType::NoConnect, "no_connect"),
    ];

    for (electrical_type, canonical_name) in cases {
        let schematic_pin_type: PinElectricalType = electrical_type;
        assert_eq!(
            canonical_pin_electrical_type_name(&schematic_pin_type),
            canonical_name
        );
    }
}

#[test]
fn erc_preserves_richer_library_electrical_roles() {
    let cases = [
        (
            LibraryPinElectricalType::OpenCollector,
            "open_collector",
            true,
            false,
        ),
        (
            LibraryPinElectricalType::OpenEmitter,
            "open_emitter",
            true,
            false,
        ),
        (LibraryPinElectricalType::TriState, "tri_state", true, false),
        (LibraryPinElectricalType::PowerOut, "power_out", true, true),
    ];

    for (electrical_type, canonical_name, explicit_driver, conflicting_output) in cases {
        let schematic_pin_type: PinElectricalType = electrical_type;
        assert_eq!(
            canonical_pin_electrical_type_name(&schematic_pin_type),
            canonical_name
        );
        assert_eq!(is_explicit_driver(&schematic_pin_type), explicit_driver);
        assert_eq!(
            is_conflicting_output(&schematic_pin_type),
            conflicting_output
        );
        assert!(!is_input(&schematic_pin_type));
        assert!(!is_passive(&schematic_pin_type));
        assert!(!is_power_input(&schematic_pin_type));
        assert!(!is_no_connect(&schematic_pin_type));
    }
}

#[test]
fn passive_biased_input_net_becomes_info_not_hard_undriven_warning() {
    let sheet_uuid = Uuid::new_v4();
    let input_uuid = Uuid::new_v4();
    let passive_uuid = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(
            sheet_uuid,
            Sheet {
                uuid: sheet_uuid,
                name: "Root".into(),
                frame: None,
                symbols: HashMap::from([
                    (
                        input_uuid,
                        PlacedSymbol {
                            uuid: input_uuid,
                            part: None,
                            entity: None,
                            gate: None,
                            lib_id: Some("Device:Q".into()),
                            reference: "Q1".into(),
                            value: "Q".into(),
                            fields: Vec::new(),
                            pins: vec![SymbolPin {
                                uuid: Uuid::new_v4(),
                                number: "1".into(),
                                name: "B".into(),
                                electrical_type: PinElectricalType::Input,
                                position: Point::new(20, 20),
                            }],
                            position: Point::new(10, 10),
                            rotation: 0,
                            mirrored: false,
                            unit_selection: None,
                            display_mode: SymbolDisplayMode::LibraryDefault,
                            pin_overrides: Vec::new(),
                            hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
                        },
                    ),
                    (
                        passive_uuid,
                        PlacedSymbol {
                            uuid: passive_uuid,
                            part: None,
                            entity: None,
                            gate: None,
                            lib_id: Some("Device:R".into()),
                            reference: "R1".into(),
                            value: "10k".into(),
                            fields: Vec::new(),
                            pins: vec![SymbolPin {
                                uuid: Uuid::new_v4(),
                                number: "1".into(),
                                name: "~".into(),
                                electrical_type: PinElectricalType::Passive,
                                position: Point::new(20, 20),
                            }],
                            position: Point::new(30, 10),
                            rotation: 0,
                            mirrored: false,
                            unit_selection: None,
                            display_mode: SymbolDisplayMode::LibraryDefault,
                            pin_overrides: Vec::new(),
                            hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
                        },
                    ),
                ]),
                wires: HashMap::new(),
                junctions: HashMap::new(),
                labels: HashMap::from([(
                    Uuid::new_v4(),
                    NetLabel {
                        uuid: Uuid::new_v4(),
                        kind: LabelKind::Local,
                        name: "IN_P".into(),
                        position: Point::new(20, 20),
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
        variants: HashMap::<Uuid, Variant>::new(),
        waivers: Vec::<CheckWaiver>::new(),
    };

    let findings = run_prechecks(&schematic);
    let finding = findings
        .iter()
        .find(|finding| finding.code == "input_without_explicit_driver")
        .expect("passive-biased input finding should exist");
    assert_eq!(finding.severity, ErcSeverity::Info);
    assert_eq!(finding.net_name.as_deref(), Some("IN_P"));
    assert!(
        !findings
            .iter()
            .any(|finding| finding.code == "undriven_input_pin")
    );
}

#[test]
fn does_not_duplicate_dangling_input_pin_with_unconnected_component_pin() {
    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(
            sheet_uuid,
            Sheet {
                uuid: sheet_uuid,
                name: "Root".into(),
                frame: None,
                symbols: HashMap::from([(
                    symbol_uuid,
                    PlacedSymbol {
                        uuid: symbol_uuid,
                        part: None,
                        entity: None,
                        gate: None,
                        lib_id: Some("Device:MCU".into()),
                        reference: "U1".into(),
                        value: "MCU".into(),
                        fields: Vec::new(),
                        pins: vec![SymbolPin {
                            uuid: Uuid::new_v4(),
                            number: "3".into(),
                            name: "SCL".into(),
                            electrical_type: PinElectricalType::Input,
                            position: Point::new(20, 20),
                        }],
                        position: Point::new(10, 10),
                        rotation: 0,
                        mirrored: false,
                        unit_selection: None,
                        display_mode: SymbolDisplayMode::LibraryDefault,
                        pin_overrides: Vec::new(),
                        hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
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
        variants: HashMap::<Uuid, Variant>::new(),
        waivers: Vec::<CheckWaiver>::new(),
    };

    let findings = run_prechecks(&schematic);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "unconnected_component_pin");
}

#[test]
fn reports_noconnect_connected_when_marker_pin_is_on_connected_net() {
    let pin_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let marker_uuid = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(
            Uuid::new_v4(),
            Sheet {
                uuid: Uuid::new_v4(),
                name: "Root".into(),
                frame: None,
                symbols: HashMap::from([(
                    symbol_uuid,
                    PlacedSymbol {
                        uuid: symbol_uuid,
                        part: None,
                        entity: None,
                        gate: None,
                        lib_id: Some("Device:R".into()),
                        reference: "R1".into(),
                        value: "10k".into(),
                        fields: Vec::new(),
                        pins: vec![SymbolPin {
                            uuid: pin_uuid,
                            number: "1".into(),
                            name: "~".into(),
                            electrical_type: PinElectricalType::Passive,
                            position: Point::new(10_000_000, 10_000_000),
                        }],
                        position: Point::new(10_000_000, 10_000_000),
                        rotation: 0,
                        mirrored: false,
                        unit_selection: Some("1".into()),
                        display_mode: SymbolDisplayMode::LibraryDefault,
                        pin_overrides: Vec::new(),
                        hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
                    },
                )]),
                wires: HashMap::new(),
                junctions: HashMap::new(),
                labels: HashMap::from([(
                    Uuid::new_v4(),
                    NetLabel {
                        uuid: Uuid::new_v4(),
                        kind: LabelKind::Local,
                        name: "NC_SIG".into(),
                        position: Point::new(10_000_000, 10_000_000),
                    },
                )]),
                buses: HashMap::new(),
                bus_entries: HashMap::new(),
                ports: HashMap::new(),
                noconnects: HashMap::from([(
                    marker_uuid,
                    crate::schematic::NoConnectMarker {
                        uuid: marker_uuid,
                        symbol: symbol_uuid,
                        pin: pin_uuid,
                        position: Point::new(10_000_000, 10_000_000),
                    },
                )]),
                texts: HashMap::new(),
                drawings: HashMap::new(),
            },
        )]),
        sheet_definitions: HashMap::new(),
        sheet_instances: HashMap::new(),
        variants: HashMap::new(),
        waivers: Vec::new(),
    };

    let findings = run_prechecks(&schematic);
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "noconnect_connected")
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.code == "unconnected_component_pin")
    );
}

#[test]
fn open_collector_and_tri_state_are_explicit_drivers_without_push_pull_conflict() {
    let schematic = schematic_with_pins_on_named_net(
        "BUS",
        vec![
            ("U1", "1", "OC", PinElectricalType::OpenCollector),
            ("U2", "1", "OC", PinElectricalType::OpenCollector),
            ("U3", "1", "TS", PinElectricalType::TriState),
            ("U4", "1", "IN", PinElectricalType::Input),
        ],
    );

    let findings = run_prechecks(&schematic);
    assert!(
        !findings
            .iter()
            .any(|finding| finding.code == "undriven_input_pin")
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.code == "input_without_explicit_driver")
    );
    assert!(
        !findings
            .iter()
            .any(|finding| finding.code == "output_to_output_conflict")
    );
}

#[test]
fn no_connect_electrical_type_drives_noconnect_erc_without_marker() {
    let connected = schematic_with_pins_on_named_net(
        "NC_SIG",
        vec![
            ("J1", "1", "NC", PinElectricalType::NoConnect),
            ("R1", "1", "~", PinElectricalType::Passive),
        ],
    );
    let connected_findings = run_prechecks(&connected);
    assert!(
        connected_findings
            .iter()
            .any(|finding| finding.code == "noconnect_connected")
    );

    let dangling = schematic_with_dangling_pin("J1", "1", "NC", PinElectricalType::NoConnect);
    let dangling_findings = run_prechecks(&dangling);
    assert!(
        !dangling_findings
            .iter()
            .any(|finding| finding.code == "unconnected_component_pin")
    );
}

fn schematic_with_pins_on_named_net(
    net_name: &str,
    pins: Vec<(&str, &str, &str, PinElectricalType)>,
) -> Schematic {
    let sheet_uuid = Uuid::new_v4();
    let point = Point::new(20, 20);
    Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(
            sheet_uuid,
            Sheet {
                uuid: sheet_uuid,
                name: "Root".into(),
                frame: None,
                symbols: pins
                    .into_iter()
                    .map(|(reference, number, name, electrical_type)| {
                        let symbol_uuid = Uuid::new_v4();
                        (
                            symbol_uuid,
                            PlacedSymbol {
                                uuid: symbol_uuid,
                                part: None,
                                entity: None,
                                gate: None,
                                lib_id: Some("Device:T".into()),
                                reference: reference.into(),
                                value: "T".into(),
                                fields: Vec::new(),
                                pins: vec![SymbolPin {
                                    uuid: Uuid::new_v4(),
                                    number: number.into(),
                                    name: name.into(),
                                    electrical_type,
                                    position: point,
                                }],
                                position: point,
                                rotation: 0,
                                mirrored: false,
                                unit_selection: None,
                                display_mode: SymbolDisplayMode::LibraryDefault,
                                pin_overrides: Vec::new(),
                                hidden_power_behavior:
                                    HiddenPowerBehavior::PreservedAsImportedMetadata,
                            },
                        )
                    })
                    .collect(),
                wires: HashMap::new(),
                junctions: HashMap::new(),
                labels: HashMap::from([(
                    Uuid::new_v4(),
                    NetLabel {
                        uuid: Uuid::new_v4(),
                        kind: LabelKind::Local,
                        name: net_name.into(),
                        position: point,
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
        variants: HashMap::<Uuid, Variant>::new(),
        waivers: Vec::<CheckWaiver>::new(),
    }
}

fn schematic_with_dangling_pin(
    reference: &str,
    number: &str,
    name: &str,
    electrical_type: PinElectricalType,
) -> Schematic {
    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let point = Point::new(20, 20);
    Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(
            sheet_uuid,
            Sheet {
                uuid: sheet_uuid,
                name: "Root".into(),
                frame: None,
                symbols: HashMap::from([(
                    symbol_uuid,
                    PlacedSymbol {
                        uuid: symbol_uuid,
                        part: None,
                        entity: None,
                        gate: None,
                        lib_id: Some("Device:T".into()),
                        reference: reference.into(),
                        value: "T".into(),
                        fields: Vec::new(),
                        pins: vec![SymbolPin {
                            uuid: Uuid::new_v4(),
                            number: number.into(),
                            name: name.into(),
                            electrical_type,
                            position: point,
                        }],
                        position: point,
                        rotation: 0,
                        mirrored: false,
                        unit_selection: None,
                        display_mode: SymbolDisplayMode::LibraryDefault,
                        pin_overrides: Vec::new(),
                        hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
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
        variants: HashMap::<Uuid, Variant>::new(),
        waivers: Vec::<CheckWaiver>::new(),
    }
}
