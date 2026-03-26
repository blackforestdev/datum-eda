use std::collections::HashMap;

use uuid::Uuid;

use crate::erc::{run_prechecks, ErcSeverity};
use crate::ir::geometry::Point;
use crate::schematic::{
    CheckWaiver, HiddenPowerBehavior, HierarchicalPort, LabelKind, NetLabel, PinElectricalType,
    PlacedSymbol, PortDirection, Schematic, Sheet, SymbolDisplayMode, SymbolPin, Variant,
};

#[test]
fn reports_unconnected_component_pin() {
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
                        lib_id: Some("Device:R".into()),
                        reference: "R1".into(),
                        value: "10k".into(),
                        fields: Vec::new(),
                        pins: vec![SymbolPin {
                            uuid: Uuid::new_v4(),
                            number: "1".into(),
                            name: "~".into(),
                            electrical_type: PinElectricalType::Passive,
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
    assert_eq!(findings[0].objects.len(), 1);
    assert_eq!(findings[0].objects[0].kind, "pin");
    assert_eq!(findings[0].objects[0].key, "R1.1");
    assert_eq!(findings[0].component.as_deref(), Some("R1"));
    assert_eq!(findings[0].pin.as_deref(), Some("1"));
}

#[test]
fn reports_hierarchical_connectivity_mismatch_when_sheet_labels_and_ports_differ() {
    let sheet_uuid = Uuid::new_v4();
    let mismatched_label_uuid = Uuid::new_v4();
    let mismatched_port_uuid = Uuid::new_v4();
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
                    mismatched_label_uuid,
                    NetLabel {
                        uuid: mismatched_label_uuid,
                        kind: LabelKind::Hierarchical,
                        name: "SUB_IN".into(),
                        position: Point::new(20, 20),
                    },
                )]),
                buses: HashMap::new(),
                bus_entries: HashMap::new(),
                ports: HashMap::from([(
                    mismatched_port_uuid,
                    HierarchicalPort {
                        uuid: mismatched_port_uuid,
                        name: "SUB_OUT".into(),
                        direction: PortDirection::Input,
                        position: Point::new(30, 20),
                    },
                )]),
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
    let mismatch = findings
        .iter()
        .find(|finding| finding.code == "hierarchical_connectivity_mismatch")
        .expect("expected hierarchical mismatch finding");
    assert_eq!(mismatch.severity, ErcSeverity::Warning);
    assert!(
        mismatch
            .message
            .contains("labels without matching ports: SUB_IN")
    );
    assert!(
        mismatch
            .message
            .contains("ports without matching labels: SUB_OUT")
    );
}

#[test]
fn finding_ids_are_stable_for_same_input() {
    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let make_schematic = || Schematic {
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
                        lib_id: Some("Device:R".into()),
                        reference: "R1".into(),
                        value: "10k".into(),
                        fields: Vec::new(),
                        pins: vec![SymbolPin {
                            uuid: Uuid::new_v4(),
                            number: "1".into(),
                            name: "~".into(),
                            electrical_type: PinElectricalType::Passive,
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

    let a = run_prechecks(&make_schematic());
    let b = run_prechecks(&make_schematic());
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(a[0].id, b[0].id);
}


#[test]
fn ignores_pin_on_named_net() {
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
                        lib_id: Some("Device:R".into()),
                        reference: "R1".into(),
                        value: "10k".into(),
                        fields: Vec::new(),
                        pins: vec![SymbolPin {
                            uuid: Uuid::new_v4(),
                            number: "1".into(),
                            name: "~".into(),
                            electrical_type: PinElectricalType::Passive,
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
                )]),
                wires: HashMap::new(),
                junctions: HashMap::new(),
                labels: HashMap::from([(
                    Uuid::new_v4(),
                    NetLabel {
                        uuid: Uuid::new_v4(),
                        kind: LabelKind::Local,
                        name: "SCL".into(),
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
    assert!(findings.is_empty());
}

#[test]
fn reports_unconnected_interface_port() {
    let sheet_uuid = Uuid::new_v4();
    let port_uuid = Uuid::new_v4();
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
                labels: HashMap::new(),
                buses: HashMap::new(),
                bus_entries: HashMap::new(),
                ports: HashMap::from([(
                    port_uuid,
                    crate::schematic::HierarchicalPort {
                        uuid: port_uuid,
                        name: "SUB_IN".into(),
                        direction: crate::schematic::PortDirection::Input,
                        position: Point::new(60, 15),
                    },
                )]),
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
    let unconnected = findings
        .iter()
        .find(|finding| finding.code == "unconnected_interface_port")
        .expect("expected unconnected interface-port finding");
    assert_eq!(unconnected.net_name.as_deref(), Some("SUB_IN"));
    assert!(unconnected.component.is_none());
    assert!(unconnected.pin.is_none());
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "hierarchical_connectivity_mismatch")
    );
}
