use std::collections::HashMap;

use uuid::Uuid;

use crate::erc::{run_prechecks, ErcSeverity};
use crate::ir::geometry::Point;
use crate::schematic::{
    CheckWaiver, HiddenPowerBehavior, LabelKind, NetLabel, PinElectricalType, PlacedSymbol,
    Schematic, Sheet, SymbolDisplayMode, SymbolPin, Variant,
};

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
                            hidden_power_behavior:
                                HiddenPowerBehavior::PreservedAsImportedMetadata,
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
                            hidden_power_behavior:
                                HiddenPowerBehavior::PreservedAsImportedMetadata,
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
