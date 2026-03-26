use std::collections::HashMap;

use uuid::Uuid;

use crate::connectivity::schematic_net_info;
use crate::ir::geometry::Point;
use crate::schematic::{
    Bus, CheckWaiver, Junction, LabelKind, NetLabel, NoConnectMarker, Schematic, SchematicWire,
    Sheet, SheetDefinition, SheetInstance, Variant,
};

#[test]
fn groups_wire_and_local_label_into_single_named_net() {
    let sheet_uuid = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(
            sheet_uuid,
            Sheet {
                uuid: sheet_uuid,
                name: "Root".into(),
                frame: None,
                symbols: HashMap::new(),
                wires: HashMap::from([(
                    Uuid::new_v4(),
                    SchematicWire {
                        uuid: Uuid::new_v4(),
                        from: Point::new(20, 20),
                        to: Point::new(30, 20),
                    },
                )]),
                junctions: HashMap::from([(
                    Uuid::new_v4(),
                    Junction {
                        uuid: Uuid::new_v4(),
                        position: Point::new(30, 20),
                    },
                )]),
                labels: HashMap::from([(
                    Uuid::new_v4(),
                    NetLabel {
                        uuid: Uuid::new_v4(),
                        kind: LabelKind::Local,
                        name: "SCL".into(),
                        position: Point::new(20, 20),
                    },
                )]),
                buses: HashMap::<Uuid, Bus>::new(),
                bus_entries: HashMap::new(),
                ports: HashMap::new(),
                noconnects: HashMap::<Uuid, NoConnectMarker>::new(),
                texts: HashMap::new(),
                drawings: HashMap::new(),
            },
        )]),
        sheet_definitions: HashMap::<Uuid, SheetDefinition>::new(),
        sheet_instances: HashMap::<Uuid, SheetInstance>::new(),
        variants: HashMap::<Uuid, Variant>::new(),
        waivers: Vec::<CheckWaiver>::new(),
    };

    let nets = schematic_net_info(&schematic);
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0].name, "SCL");
    assert_eq!(nets[0].labels, 1);
    assert_eq!(nets[0].ports, 0);
    assert_eq!(nets[0].sheets, vec!["Root".to_string()]);
}

#[test]
fn attaches_midwire_label_to_connected_pin_net() {
    let sheet_uuid = Uuid::new_v4();
    let pin_uuid = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(
            sheet_uuid,
            Sheet {
                uuid: sheet_uuid,
                name: "Root".into(),
                frame: None,
                symbols: HashMap::from([(
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
                            uuid: pin_uuid,
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
                )]),
                wires: HashMap::from([(
                    Uuid::new_v4(),
                    SchematicWire {
                        uuid: Uuid::new_v4(),
                        from: Point::new(10, 10),
                        to: Point::new(20, 10),
                    },
                )]),
                junctions: HashMap::new(),
                labels: HashMap::from([(
                    Uuid::new_v4(),
                    NetLabel {
                        uuid: Uuid::new_v4(),
                        kind: LabelKind::Local,
                        name: "SIG".into(),
                        position: Point::new(15, 10),
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
    };

    let nets = schematic_net_info(&schematic);
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0].name, "SIG");
    assert_eq!(nets[0].pins.len(), 1);
    assert_eq!(nets[0].pins[0].uuid, pin_uuid);
}

#[test]
fn merges_global_labels_by_name_across_sheets() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([
            (
                a,
                Sheet {
                    uuid: a,
                    name: "A".into(),
                    frame: None,
                    symbols: HashMap::new(),
                    wires: HashMap::new(),
                    junctions: HashMap::new(),
                    labels: HashMap::from([(
                        Uuid::new_v4(),
                        NetLabel {
                            uuid: Uuid::new_v4(),
                            kind: LabelKind::Global,
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
            ),
            (
                b,
                Sheet {
                    uuid: b,
                    name: "B".into(),
                    frame: None,
                    symbols: HashMap::new(),
                    wires: HashMap::new(),
                    junctions: HashMap::new(),
                    labels: HashMap::from([(
                        Uuid::new_v4(),
                        NetLabel {
                            uuid: Uuid::new_v4(),
                            kind: LabelKind::Global,
                            name: "VCC".into(),
                            position: Point::new(10, 0),
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
    assert_eq!(nets[0].name, "VCC");
    assert_eq!(nets[0].labels, 2);
    assert_eq!(nets[0].semantic_class.as_deref(), Some("power"));
    assert_eq!(nets[0].sheets, vec!["A".to_string(), "B".to_string()]);
}

#[test]
fn infers_power_semantics_for_local_supply_labels() {
    let sheet_uuid = Uuid::new_v4();
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
                labels: HashMap::from([
                    (
                        Uuid::new_v4(),
                        NetLabel {
                            uuid: Uuid::new_v4(),
                            kind: LabelKind::Local,
                            name: "VCC".into(),
                            position: Point::new(0, 0),
                        },
                    ),
                    (
                        Uuid::new_v4(),
                        NetLabel {
                            uuid: Uuid::new_v4(),
                            kind: LabelKind::Local,
                            name: "VEE".into(),
                            position: Point::new(10, 0),
                        },
                    ),
                ]),
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
    assert!(
        nets.iter()
            .any(|net| net.name == "VCC" && net.semantic_class.as_deref() == Some("power"))
    );
    assert!(
        nets.iter()
            .any(|net| net.name == "VEE" && net.semantic_class.as_deref() == Some("power"))
    );
}
