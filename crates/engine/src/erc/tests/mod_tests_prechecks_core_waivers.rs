use std::collections::HashMap;

use uuid::Uuid;

use crate::erc::{
    run_prechecks, run_prechecks_with_config, run_prechecks_with_config_and_waivers, ErcConfig,
    ErcSeverity,
};
use crate::ir::geometry::Point;
use crate::schematic::{
    CheckDomain, CheckWaiver, HiddenPowerBehavior, LabelKind, NetLabel, PinElectricalType,
    PlacedSymbol, Schematic, Sheet, SymbolDisplayMode, SymbolPin, Variant, WaiverTarget,
};

#[test]
fn severity_override_changes_severity_but_not_identity() {
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

    let baseline = run_prechecks(&schematic);
    let baseline_finding = baseline
        .iter()
        .find(|finding| finding.code == "undriven_power_net")
        .expect("baseline finding should exist");

    let mut config = ErcConfig::default();
    config
        .severity_overrides
        .insert("undriven_power_net".into(), ErcSeverity::Error);
    let overridden = run_prechecks_with_config(&schematic, &config);
    let overridden_finding = overridden
        .iter()
        .find(|finding| finding.code == "undriven_power_net")
        .expect("overridden finding should exist");

    assert_eq!(baseline_finding.id, overridden_finding.id);
    assert_eq!(baseline_finding.severity, ErcSeverity::Warning);
    assert_eq!(overridden_finding.severity, ErcSeverity::Error);
}

#[test]
fn authored_waiver_marks_matching_object_finding_as_waived() {
    let pin_uuid = Uuid::new_v4();
    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let waiver = CheckWaiver {
        uuid: Uuid::new_v4(),
        domain: CheckDomain::ERC,
        target: WaiverTarget::Object(pin_uuid),
        rationale: "Intentional floating pin".into(),
        created_by: Some("test".into()),
    };
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
                            uuid: pin_uuid,
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
        waivers: vec![waiver],
    };

    let findings = run_prechecks(&schematic);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "unconnected_component_pin");
    assert!(findings[0].waived);
}

#[test]
fn extra_waiver_matches_rule_objects_independent_of_order() {
    let pin_a_uuid = Uuid::new_v4();
    let pin_b_uuid = Uuid::new_v4();
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
                                uuid: pin_a_uuid,
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
                            hidden_power_behavior:
                                HiddenPowerBehavior::PreservedAsImportedMetadata,
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
                                uuid: pin_b_uuid,
                                number: "1".into(),
                                name: "OUT".into(),
                                electrical_type: PinElectricalType::PowerOut,
                                position: Point::new(5, 5),
                            }],
                            position: Point::new(20, 10),
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
                        name: "DRV".into(),
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
        waivers: Vec::new(),
    };
    let waivers = vec![CheckWaiver {
        uuid: Uuid::new_v4(),
        domain: CheckDomain::ERC,
        target: WaiverTarget::RuleObjects {
            rule: "output_to_output_conflict".into(),
            objects: vec![pin_b_uuid, pin_a_uuid],
        },
        rationale: "Known tie".into(),
        created_by: None,
    }];

    let findings =
        run_prechecks_with_config_and_waivers(&schematic, &ErcConfig::default(), &waivers);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "output_to_output_conflict");
    assert!(findings[0].waived);
}
