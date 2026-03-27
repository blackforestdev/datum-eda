use std::collections::HashMap;

use uuid::Uuid;

use crate::connectivity::{schematic_diagnostics, schematic_net_info};
use crate::ir::geometry::Point;
use crate::schematic::{HierarchicalPort, LabelKind, NetLabel, PortDirection, Schematic, Sheet};

#[test]
fn reports_connectivity_diagnostics_for_dangling_and_anonymous_nets() {
    let sheet_uuid = Uuid::new_v4();
    let pin_a_uuid = Uuid::new_v4();
    let pin_b_uuid = Uuid::new_v4();
    let pin_c_uuid = Uuid::new_v4();
    let port_uuid = Uuid::new_v4();
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
                        Uuid::new_v4(),
                        crate::schematic::PlacedSymbol {
                            uuid: Uuid::new_v4(),
                            part: None,
                            entity: None,
                            gate: None,
                            lib_id: Some("Device:R".into()),
                            reference: "R1".into(),
                            value: "10k".into(),
                            fields: Vec::new(),
                            pins: vec![
                                crate::schematic::SymbolPin {
                                    uuid: pin_a_uuid,
                                    number: "1".into(),
                                    name: "~".into(),
                                    electrical_type: crate::schematic::PinElectricalType::Passive,
                                    position: Point::new(5, 5),
                                },
                                crate::schematic::SymbolPin {
                                    uuid: pin_b_uuid,
                                    number: "2".into(),
                                    name: "~".into(),
                                    electrical_type: crate::schematic::PinElectricalType::Passive,
                                    position: Point::new(20, 20),
                                },
                            ],
                            position: Point::new(0, 0),
                            rotation: 0,
                            mirrored: false,
                            unit_selection: None,
                            display_mode: crate::schematic::SymbolDisplayMode::LibraryDefault,
                            pin_overrides: Vec::new(),
                            hidden_power_behavior:
                                crate::schematic::HiddenPowerBehavior::PreservedAsImportedMetadata,
                        },
                    ),
                    (
                        Uuid::new_v4(),
                        crate::schematic::PlacedSymbol {
                            uuid: Uuid::new_v4(),
                            part: None,
                            entity: None,
                            gate: None,
                            lib_id: Some("Device:R".into()),
                            reference: "R2".into(),
                            value: "10k".into(),
                            fields: Vec::new(),
                            pins: vec![crate::schematic::SymbolPin {
                                uuid: pin_c_uuid,
                                number: "1".into(),
                                name: "~".into(),
                                electrical_type: crate::schematic::PinElectricalType::Passive,
                                position: Point::new(20, 20),
                            }],
                            position: Point::new(0, 0),
                            rotation: 0,
                            mirrored: false,
                            unit_selection: None,
                            display_mode: crate::schematic::SymbolDisplayMode::LibraryDefault,
                            pin_overrides: Vec::new(),
                            hidden_power_behavior:
                                crate::schematic::HiddenPowerBehavior::PreservedAsImportedMetadata,
                        },
                    ),
                ]),
                wires: HashMap::new(),
                junctions: HashMap::new(),
                labels: HashMap::new(),
                buses: HashMap::new(),
                bus_entries: HashMap::new(),
                ports: HashMap::from([(
                    port_uuid,
                    HierarchicalPort {
                        uuid: port_uuid,
                        name: "SUB_IN".into(),
                        direction: PortDirection::Input,
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
        variants: HashMap::new(),
        waivers: Vec::new(),
    };

    let diagnostics = schematic_diagnostics(&schematic);
    assert_eq!(diagnostics.len(), 3);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.kind == "dangling_component_pin" && d.objects == vec![pin_a_uuid])
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d.kind == "dangling_interface_port" && d.objects == vec![port_uuid])
    );
    assert!(diagnostics.iter().any(|d| {
        let mut expected = vec![pin_b_uuid, pin_c_uuid];
        expected.sort();
        d.kind == "anonymous_multi_pin_net" && d.objects == expected
    }));
}

#[test]
fn creates_named_net_for_standalone_port() {
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
                    HierarchicalPort {
                        uuid: port_uuid,
                        name: "SUB_IN".into(),
                        direction: PortDirection::Input,
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
        variants: HashMap::new(),
        waivers: Vec::new(),
    };

    let nets = schematic_net_info(&schematic);
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0].name, "SUB_IN");
    assert_eq!(nets[0].ports, 1);
    assert_eq!(nets[0].labels, 0);
}

#[test]
fn disconnected_anonymous_nets_get_distinct_names_and_ids() {
    let sheet_uuid = Uuid::new_v4();
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
                        Uuid::new_v4(),
                        crate::schematic::PlacedSymbol {
                            uuid: Uuid::new_v4(),
                            part: None,
                            entity: None,
                            gate: None,
                            lib_id: Some("Device:R".into()),
                            reference: "R1".into(),
                            value: "10k".into(),
                            fields: Vec::new(),
                            pins: vec![crate::schematic::SymbolPin {
                                uuid: Uuid::new_v4(),
                                number: "1".into(),
                                name: "~".into(),
                                electrical_type: crate::schematic::PinElectricalType::Passive,
                                position: Point::new(10, 10),
                            }],
                            position: Point::new(0, 0),
                            rotation: 0,
                            mirrored: false,
                            unit_selection: None,
                            display_mode: crate::schematic::SymbolDisplayMode::LibraryDefault,
                            pin_overrides: Vec::new(),
                            hidden_power_behavior:
                                crate::schematic::HiddenPowerBehavior::PreservedAsImportedMetadata,
                        },
                    ),
                    (
                        Uuid::new_v4(),
                        crate::schematic::PlacedSymbol {
                            uuid: Uuid::new_v4(),
                            part: None,
                            entity: None,
                            gate: None,
                            lib_id: Some("Device:R".into()),
                            reference: "R2".into(),
                            value: "10k".into(),
                            fields: Vec::new(),
                            pins: vec![crate::schematic::SymbolPin {
                                uuid: Uuid::new_v4(),
                                number: "1".into(),
                                name: "~".into(),
                                electrical_type: crate::schematic::PinElectricalType::Passive,
                                position: Point::new(40, 10),
                            }],
                            position: Point::new(0, 0),
                            rotation: 0,
                            mirrored: false,
                            unit_selection: None,
                            display_mode: crate::schematic::SymbolDisplayMode::LibraryDefault,
                            pin_overrides: Vec::new(),
                            hidden_power_behavior:
                                crate::schematic::HiddenPowerBehavior::PreservedAsImportedMetadata,
                        },
                    ),
                ]),
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
    };

    let nets = schematic_net_info(&schematic);
    assert_eq!(nets.len(), 2);
    assert!(nets.iter().all(|net| net.name.starts_with("N$")));
    assert_ne!(nets[0].name, nets[1].name);
    assert_ne!(nets[0].uuid, nets[1].uuid);
}

#[test]
fn merges_hierarchical_label_with_matching_port_name() {
    let root_sheet = Uuid::new_v4();
    let child_sheet = Uuid::new_v4();
    let port_uuid = Uuid::new_v4();
    let label_uuid = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([
            (
                root_sheet,
                Sheet {
                    uuid: root_sheet,
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
                        HierarchicalPort {
                            uuid: port_uuid,
                            name: "SUB_IN".into(),
                            direction: PortDirection::Input,
                            position: Point::new(60, 15),
                        },
                    )]),
                    noconnects: HashMap::new(),
                    texts: HashMap::new(),
                    drawings: HashMap::new(),
                },
            ),
            (
                child_sheet,
                Sheet {
                    uuid: child_sheet,
                    name: "Child".into(),
                    frame: None,
                    symbols: HashMap::new(),
                    wires: HashMap::new(),
                    junctions: HashMap::new(),
                    labels: HashMap::from([(
                        label_uuid,
                        NetLabel {
                            uuid: label_uuid,
                            kind: LabelKind::Hierarchical,
                            name: "SUB_IN".into(),
                            position: Point::new(10, 10),
                        },
                    )]),
                    buses: HashMap::new(),
                    bus_entries: HashMap::new(),
                    ports: HashMap::new(),
                    noconnects: HashMap::new(),
                    texts: HashMap::new(),
                    drawings: HashMap::new(),
                },
            ),
        ]),
        sheet_definitions: HashMap::new(),
        sheet_instances: HashMap::new(),
        variants: HashMap::new(),
        waivers: Vec::new(),
    };

    let nets = schematic_net_info(&schematic);
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0].name, "SUB_IN");
    assert_eq!(nets[0].labels, 1);
    assert_eq!(nets[0].ports, 1);
    assert_eq!(
        nets[0].sheets,
        vec!["Child".to_string(), "Root".to_string()]
    );
}
