use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::connectivity;
use crate::schematic::{CheckDomain, CheckWaiver, PinElectricalType, Schematic, WaiverTarget};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErcSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ErcConfig {
    pub severity_overrides: BTreeMap<String, ErcSeverity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErcFinding {
    pub id: Uuid,
    pub code: &'static str,
    pub severity: ErcSeverity,
    pub message: String,
    pub net_name: Option<String>,
    pub component: Option<String>,
    pub pin: Option<String>,
    pub objects: Vec<ErcObjectRef>,
    pub object_uuids: Vec<Uuid>,
    pub waived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ErcObjectRef {
    pub kind: &'static str,
    pub key: String,
}

pub fn run_prechecks(schematic: &Schematic) -> Vec<ErcFinding> {
    run_prechecks_with_config_and_waivers(schematic, &ErcConfig::default(), &schematic.waivers)
}

pub fn run_prechecks_with_config(schematic: &Schematic, config: &ErcConfig) -> Vec<ErcFinding> {
    run_prechecks_with_config_and_waivers(schematic, config, &schematic.waivers)
}

pub fn run_prechecks_with_config_and_waivers(
    schematic: &Schematic,
    config: &ErcConfig,
    waivers: &[CheckWaiver],
) -> Vec<ErcFinding> {
    let mut findings = Vec::new();
    findings.extend(hierarchical_mismatch_findings(schematic, config));
    let nets = connectivity::schematic_net_info(schematic);
    let noconnect_pins = noconnect_pin_uuids(schematic);

    for net in nets {
        let net_name = net.name.clone();
        let output_pins: Vec<_> = net
            .pins
            .iter()
            .filter(|pin| {
                matches!(
                    pin.electrical_type,
                    PinElectricalType::Output | PinElectricalType::PowerOut
                )
            })
            .collect();
        let input_pins: Vec<_> = net
            .pins
            .iter()
            .filter(|pin| matches!(pin.electrical_type, PinElectricalType::Input))
            .collect();
        let passive_pins: Vec<_> = net
            .pins
            .iter()
            .filter(|pin| matches!(pin.electrical_type, PinElectricalType::Passive))
            .collect();
        let power_in_pins: Vec<_> = net
            .pins
            .iter()
            .filter(|pin| matches!(pin.electrical_type, PinElectricalType::PowerIn))
            .collect();
        let noconnect_marked_pins: Vec<_> = net
            .pins
            .iter()
            .filter(|pin| noconnect_pins.contains(&pin.uuid))
            .collect();

        if output_pins.len() > 1 {
            findings.push(build_finding(
                "output_to_output_conflict",
                severity_for(config, "output_to_output_conflict", ErcSeverity::Error),
                format!(
                    "net {} has multiple driving outputs: {}",
                    net_name,
                    output_pins
                        .iter()
                        .map(|pin| format!("{}.{}", pin.component, pin.pin))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Some(net_name.clone()),
                None,
                None,
                output_pins
                    .iter()
                    .map(|pin| object_ref("pin", format!("{}.{}", pin.component, pin.pin)))
                    .collect(),
                output_pins.iter().map(|pin| pin.uuid).collect(),
            ));
        }

        if !power_in_pins.is_empty() && output_pins.is_empty() {
            findings.push(build_finding(
                "power_in_without_source",
                severity_for(config, "power_in_without_source", ErcSeverity::Warning),
                format!(
                    "power-input pins on net {} have no driving source: {}",
                    net_name,
                    power_in_pins
                        .iter()
                        .map(|pin| format!("{}.{}", pin.component, pin.pin))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Some(net_name.clone()),
                None,
                None,
                power_in_pins
                    .iter()
                    .map(|pin| object_ref("pin", format!("{}.{}", pin.component, pin.pin)))
                    .collect(),
                power_in_pins.iter().map(|pin| pin.uuid).collect(),
            ));
        }

        let is_single_dangling_pin = net.pins.len() == 1 && net.labels == 0 && net.ports == 0;
        let has_connected_noconnect_pin = !noconnect_marked_pins.is_empty()
            && (net.pins.len() > 1 || net.labels > 0 || net.ports > 0);
        if has_connected_noconnect_pin {
            findings.push(build_finding(
                "noconnect_connected",
                severity_for(config, "noconnect_connected", ErcSeverity::Warning),
                format!(
                    "no_connect-marked pins are connected on net {}: {}",
                    net_name,
                    noconnect_marked_pins
                        .iter()
                        .map(|pin| format!("{}.{}", pin.component, pin.pin))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Some(net_name.clone()),
                None,
                None,
                noconnect_marked_pins
                    .iter()
                    .map(|pin| object_ref("pin", format!("{}.{}", pin.component, pin.pin)))
                    .collect(),
                noconnect_marked_pins.iter().map(|pin| pin.uuid).collect(),
            ));
        }

        let dangling_pin_is_noconnect =
            is_single_dangling_pin && noconnect_pins.contains(&net.pins[0].uuid);
        if !input_pins.is_empty() && output_pins.is_empty() && !is_single_dangling_pin {
            let has_passive_biasing = !passive_pins.is_empty();
            let is_named = !net_name.starts_with("N$");
            let (code, severity, message) = if has_passive_biasing && is_named {
                (
                    "input_without_explicit_driver",
                    severity_for(config, "input_without_explicit_driver", ErcSeverity::Info),
                    format!(
                        "input pins on net {} have no explicit driver, but the net includes passive biasing/components: {}",
                        net_name,
                        input_pins
                            .iter()
                            .map(|pin| format!("{}.{}", pin.component, pin.pin))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
            } else {
                (
                    "undriven_input_pin",
                    severity_for(config, "undriven_input_pin", ErcSeverity::Warning),
                    format!(
                        "input pins on net {} have no driving source: {}",
                        net_name,
                        input_pins
                            .iter()
                            .map(|pin| format!("{}.{}", pin.component, pin.pin))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
            };
            findings.push(build_finding(
                code,
                severity,
                message,
                Some(net_name.clone()),
                None,
                None,
                input_pins
                    .iter()
                    .map(|pin| object_ref("pin", format!("{}.{}", pin.component, pin.pin)))
                    .collect(),
                input_pins.iter().map(|pin| pin.uuid).collect(),
            ));
        }

        if is_single_dangling_pin && !dangling_pin_is_noconnect {
            let pin = &net.pins[0];
            findings.push(build_finding(
                "unconnected_component_pin",
                severity_for(config, "unconnected_component_pin", ErcSeverity::Warning),
                format!(
                    "component pin {}.{} is not connected to any labeled net or interface",
                    pin.component, pin.pin
                ),
                Some(net_name.clone()),
                Some(pin.component.clone()),
                Some(pin.pin.clone()),
                vec![object_ref("pin", format!("{}.{}", pin.component, pin.pin))],
                vec![pin.uuid],
            ));
        }

        if net.ports == 1 && net.pins.is_empty() && net.labels == 0 {
            findings.push(build_finding(
                "unconnected_interface_port",
                severity_for(config, "unconnected_interface_port", ErcSeverity::Warning),
                format!(
                    "interface port {} is not connected to any component pin or labeled net",
                    net_name
                ),
                Some(net_name.clone()),
                None,
                None,
                vec![object_ref("port", net_name.clone())],
                net.port_uuids.clone(),
            ));
        }

        let is_named = !net_name.starts_with("N$");
        let is_power_like = net.semantic_class.is_some();
        if is_named && net.pins.is_empty() && net.labels > 0 && net.ports == 0 {
            let code = if is_power_like {
                "undriven_power_net"
            } else {
                "undriven_named_net"
            };
            findings.push(build_finding(
                code,
                severity_for(config, code, ErcSeverity::Warning),
                if is_power_like {
                    format!("power net {} has no connected component pins", net_name)
                } else {
                    format!("named net {} has no connected component pins", net_name)
                },
                Some(net_name.clone()),
                None,
                None,
                vec![object_ref("net", net_name)],
                vec![net.uuid],
            ));
        }
    }

    apply_waivers(&mut findings, waivers);
    findings.sort_by(|a, b| {
        a.code
            .cmp(b.code)
            .then_with(|| a.component.cmp(&b.component))
            .then_with(|| a.pin.cmp(&b.pin))
            .then_with(|| a.net_name.cmp(&b.net_name))
    });
    findings
}

fn hierarchical_mismatch_findings(schematic: &Schematic, config: &ErcConfig) -> Vec<ErcFinding> {
    let mut findings = Vec::new();
    for sheet in schematic.sheets.values() {
        let mut label_names: BTreeMap<String, Vec<Uuid>> = BTreeMap::new();
        for label in sheet.labels.values() {
            if matches!(label.kind, crate::schematic::LabelKind::Hierarchical) {
                label_names
                    .entry(label.name.clone())
                    .or_default()
                    .push(label.uuid);
            }
        }
        let mut port_names: BTreeMap<String, Vec<Uuid>> = BTreeMap::new();
        for port in sheet.ports.values() {
            port_names
                .entry(port.name.clone())
                .or_default()
                .push(port.uuid);
        }

        let missing_ports: Vec<_> = label_names
            .keys()
            .filter(|name| !port_names.contains_key(*name))
            .cloned()
            .collect();
        let missing_labels: Vec<_> = port_names
            .keys()
            .filter(|name| !label_names.contains_key(*name))
            .cloned()
            .collect();

        if missing_ports.is_empty() && missing_labels.is_empty() {
            continue;
        }

        let mut object_uuids = Vec::new();
        for name in &missing_ports {
            if let Some(uuids) = label_names.get(name) {
                object_uuids.extend(uuids.iter().copied());
            }
        }
        for name in &missing_labels {
            if let Some(uuids) = port_names.get(name) {
                object_uuids.extend(uuids.iter().copied());
            }
        }
        object_uuids.sort();
        object_uuids.dedup();

        let mut objects = vec![object_ref("sheet", sheet.name.clone())];
        for name in &missing_ports {
            objects.push(object_ref("hierarchical_label", name.clone()));
        }
        for name in &missing_labels {
            objects.push(object_ref("port", name.clone()));
        }

        let mut message_parts = Vec::new();
        if !missing_ports.is_empty() {
            message_parts.push(format!(
                "labels without matching ports: {}",
                missing_ports.join(", ")
            ));
        }
        if !missing_labels.is_empty() {
            message_parts.push(format!(
                "ports without matching labels: {}",
                missing_labels.join(", ")
            ));
        }
        findings.push(build_finding(
            "hierarchical_connectivity_mismatch",
            severity_for(
                config,
                "hierarchical_connectivity_mismatch",
                ErcSeverity::Warning,
            ),
            format!(
                "sheet {} has hierarchical interface mismatch ({})",
                sheet.name,
                message_parts.join("; ")
            ),
            None,
            None,
            None,
            objects,
            object_uuids,
        ));
    }
    findings
}

fn severity_for(config: &ErcConfig, code: &str, default: ErcSeverity) -> ErcSeverity {
    config
        .severity_overrides
        .get(code)
        .cloned()
        .unwrap_or(default)
}

#[allow(clippy::too_many_arguments)]
fn build_finding(
    code: &'static str,
    severity: ErcSeverity,
    message: String,
    net_name: Option<String>,
    component: Option<String>,
    pin: Option<String>,
    mut objects: Vec<ErcObjectRef>,
    mut object_uuids: Vec<Uuid>,
) -> ErcFinding {
    objects.sort();
    objects.dedup();
    object_uuids.sort();
    object_uuids.dedup();
    let id = stable_finding_id(
        code,
        net_name.as_deref(),
        component.as_deref(),
        pin.as_deref(),
        &objects,
    );
    ErcFinding {
        id,
        code,
        severity,
        message,
        net_name,
        component,
        pin,
        objects,
        object_uuids,
        waived: false,
    }
}

fn object_ref(kind: &'static str, key: String) -> ErcObjectRef {
    ErcObjectRef { kind, key }
}

fn stable_finding_id(
    code: &str,
    net_name: Option<&str>,
    component: Option<&str>,
    pin: Option<&str>,
    objects: &[ErcObjectRef],
) -> Uuid {
    let mut material = vec![format!("code={code}")];
    if let Some(net_name) = net_name {
        material.push(format!("net={net_name}"));
    }
    if let Some(component) = component {
        material.push(format!("component={component}"));
    }
    if let Some(pin) = pin {
        material.push(format!("pin={pin}"));
    }
    for object in objects {
        material.push(format!("obj:{}={}", object.kind, object.key));
    }
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, material.join("|").as_bytes())
}

fn apply_waivers(findings: &mut [ErcFinding], waivers: &[CheckWaiver]) {
    for finding in findings {
        finding.waived = waivers.iter().any(|waiver| waiver_matches(waiver, finding));
    }
}

fn noconnect_pin_uuids(schematic: &Schematic) -> BTreeSet<Uuid> {
    let mut pins = BTreeSet::new();
    for sheet in schematic.sheets.values() {
        for marker in sheet.noconnects.values() {
            if marker.pin != Uuid::nil() {
                pins.insert(marker.pin);
            }
        }
    }
    pins
}

fn waiver_matches(waiver: &CheckWaiver, finding: &ErcFinding) -> bool {
    if !matches!(waiver.domain, CheckDomain::ERC) {
        return false;
    }

    match &waiver.target {
        WaiverTarget::Object(uuid) => finding.object_uuids.contains(uuid),
        WaiverTarget::RuleObject { rule, object } => {
            *rule == finding.code && finding.object_uuids.contains(object)
        }
        WaiverTarget::RuleObjects { rule, objects } => {
            if *rule != finding.code {
                return false;
            }
            let mut actual = finding.object_uuids.clone();
            actual.sort();
            let mut expected = objects.clone();
            expected.sort();
            actual == expected
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use uuid::Uuid;

    use super::*;
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
}
