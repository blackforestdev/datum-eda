use super::*;

#[test]
fn schematic_round_trip() {
    let sheet_uuid = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(
            sheet_uuid,
            Sheet {
                uuid: sheet_uuid,
                name: "Sheet1".into(),
                frame: None,
                symbols: HashMap::new(),
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

    let json = serde_json::to_string(&schematic).unwrap();
    let restored: Schematic = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, schematic);
}

#[test]
fn schematic_summary_counts_objects_across_sheets() {
    let sheet_a = Uuid::new_v4();
    let sheet_b = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([
            (
                sheet_a,
                Sheet {
                    uuid: sheet_a,
                    name: "Top".into(),
                    frame: None,
                    symbols: HashMap::from([(
                        Uuid::new_v4(),
                        PlacedSymbol {
                            uuid: Uuid::new_v4(),
                            part: None,
                            entity: None,
                            gate: None,
                            lib_id: None,
                            reference: "U1".into(),
                            value: "MCU".into(),
                            fields: Vec::new(),
                            pins: Vec::new(),
                            position: Point::new(0, 0),
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
                sheet_b,
                Sheet {
                    uuid: sheet_b,
                    name: "Sub".into(),
                    frame: None,
                    symbols: HashMap::new(),
                    wires: HashMap::new(),
                    junctions: HashMap::new(),
                    labels: HashMap::new(),
                    buses: HashMap::new(),
                    bus_entries: HashMap::new(),
                    ports: HashMap::from([(
                        Uuid::new_v4(),
                        HierarchicalPort {
                            uuid: Uuid::new_v4(),
                            name: "SCL".into(),
                            direction: PortDirection::Bidirectional,
                            position: Point::new(0, 0),
                        },
                    )]),
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

    let summary = schematic.summary();
    assert_eq!(summary.sheet_count, 2);
    assert_eq!(summary.symbol_count, 1);
    assert_eq!(summary.net_label_count, 1);
    assert_eq!(summary.port_count, 1);
    let symbols = schematic.symbols(None);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].reference, "U1");
    let ports = schematic.ports(None);
    assert_eq!(ports.len(), 1);
    assert_eq!(ports[0].name, "SCL");
}

#[test]
fn hierarchy_reports_sorted_instances() {
    let top_sheet = Uuid::new_v4();
    let def_a = Uuid::new_v4();
    let def_b = Uuid::new_v4();
    let inst_b = Uuid::new_v4();
    let inst_a = Uuid::new_v4();
    let schematic = Schematic {
        uuid: Uuid::new_v4(),
        sheets: HashMap::from([(
            top_sheet,
            Sheet {
                uuid: top_sheet,
                name: "Top".into(),
                frame: None,
                symbols: HashMap::new(),
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
        sheet_definitions: HashMap::from([
            (
                def_a,
                SheetDefinition {
                    uuid: def_a,
                    root_sheet: top_sheet,
                    name: "Amplifier".into(),
                },
            ),
            (
                def_b,
                SheetDefinition {
                    uuid: def_b,
                    root_sheet: top_sheet,
                    name: "Bias".into(),
                },
            ),
        ]),
        sheet_instances: HashMap::from([
            (
                inst_b,
                SheetInstance {
                    uuid: inst_b,
                    definition: def_b,
                    parent_sheet: Some(top_sheet),
                    position: Point::new(20, 30),
                    name: "Bias".into(),
                    ports: Vec::new(),
                },
            ),
            (
                inst_a,
                SheetInstance {
                    uuid: inst_a,
                    definition: def_a,
                    parent_sheet: Some(top_sheet),
                    position: Point::new(10, 15),
                    name: "Amplifier".into(),
                    ports: Vec::new(),
                },
            ),
        ]),
        variants: HashMap::new(),
        waivers: Vec::new(),
    };

    let hierarchy = schematic.hierarchy();
    assert_eq!(hierarchy.instances.len(), 2);
    assert!(hierarchy.links.is_empty());
    assert_eq!(hierarchy.instances[0].name, "Amplifier");
    assert_eq!(hierarchy.instances[1].name, "Bias");
    assert_eq!(hierarchy.instances[0].parent_sheet, Some(top_sheet));
}
