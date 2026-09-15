use std::collections::HashMap;

use uuid::Uuid;

use crate::erc::{ErcSeverity, run_prechecks};
use crate::ir::geometry::Point;
use crate::schematic::{
    CheckWaiver, HiddenPowerBehavior, LabelKind, NetLabel, PinElectricalType, PlacedSymbol,
    Schematic, Sheet, SymbolDisplayMode, SymbolPin, Variant,
};

#[test]
fn reports_undriven_named_net() {
    let sheet_uuid = Uuid::new_v4();
    let label_uuid = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(
            sheet_uuid,
            Sheet {
                uuid: sheet_uuid,
                name: "Root".into(),
                frame: None,
                symbols: HashMap::new(),
                wires: HashMap::new(),
                junctions: HashMap::new(),
                labels: HashMap::from([(
                    label_uuid,
                    NetLabel {
                        uuid: label_uuid,
                        kind: LabelKind::Local,
                        name: "SCL".into(),
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
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "undriven_named_net");
    assert_eq!(findings[0].net_name.as_deref(), Some("SCL"));
}

#[test]
fn reports_undriven_power_net() {
    let sheet_uuid = Uuid::new_v4();
    let label_uuid = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(
            sheet_uuid,
            Sheet {
                uuid: sheet_uuid,
                name: "Root".into(),
                frame: None,
                symbols: HashMap::new(),
                wires: HashMap::new(),
                junctions: HashMap::new(),
                labels: HashMap::from([(
                    label_uuid,
                    NetLabel {
                        uuid: label_uuid,
                        kind: LabelKind::Global,
                        name: "VCC".into(),
                        position: Point::new(40, 20),
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
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "undriven_power_net");
    assert_eq!(findings[0].net_name.as_deref(), Some("VCC"));
}

#[test]
fn reports_output_to_output_conflict() {
    let sheet_uuid = Uuid::new_v4();
    let a_uuid = Uuid::new_v4();
    let b_uuid = Uuid::new_v4();
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
                        a_uuid,
                        PlacedSymbol {
                            uuid: a_uuid,
                            part: None,
                            entity: None,
                            gate: None,
                            lib_id: Some("Device:BUF".into()),
                            reference: "U1".into(),
                            value: "BUF".into(),
                            fields: Vec::new(),
                            pins: vec![SymbolPin {
                                uuid: Uuid::new_v4(),
                                number: "1".into(),
                                name: "OUT".into(),
                                electrical_type: PinElectricalType::Output,
                                position: Point::new(5, 5),
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
                        b_uuid,
                        PlacedSymbol {
                            uuid: b_uuid,
                            part: None,
                            entity: None,
                            gate: None,
                            lib_id: Some("Device:BUF".into()),
                            reference: "U2".into(),
                            value: "BUF".into(),
                            fields: Vec::new(),
                            pins: vec![SymbolPin {
                                uuid: Uuid::new_v4(),
                                number: "1".into(),
                                name: "OUT".into(),
                                electrical_type: PinElectricalType::Output,
                                position: Point::new(5, 5),
                            }],
                            position: Point::new(20, 10),
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
                        name: "CLK".into(),
                        position: Point::new(5, 5),
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
    let conflict = findings
        .iter()
        .find(|finding| finding.code == "output_to_output_conflict")
        .expect("output conflict should be reported");
    assert_eq!(conflict.severity, ErcSeverity::Error);
    assert_eq!(conflict.net_name.as_deref(), Some("CLK"));
}

#[test]
fn reports_power_in_without_source() {
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
                        lib_id: Some("Device:IC".into()),
                        reference: "U1".into(),
                        value: "IC".into(),
                        fields: Vec::new(),
                        pins: vec![SymbolPin {
                            uuid: Uuid::new_v4(),
                            number: "8".into(),
                            name: "VCC".into(),
                            electrical_type: PinElectricalType::PowerIn,
                            position: Point::new(40, 20),
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
                labels: HashMap::from([(
                    Uuid::new_v4(),
                    NetLabel {
                        uuid: Uuid::new_v4(),
                        kind: LabelKind::Global,
                        name: "VCC".into(),
                        position: Point::new(40, 20),
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
        .find(|finding| finding.code == "power_in_without_source")
        .expect("power-input source finding should exist");
    assert_eq!(finding.net_name.as_deref(), Some("VCC"));
}

#[test]
fn reports_undriven_input_pin() {
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
                labels: HashMap::from([(
                    Uuid::new_v4(),
                    NetLabel {
                        uuid: Uuid::new_v4(),
                        kind: LabelKind::Local,
                        name: "SCL".into(),
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
        .find(|finding| finding.code == "undriven_input_pin")
        .expect("undriven input finding should exist");
    assert_eq!(finding.net_name.as_deref(), Some("SCL"));
}
