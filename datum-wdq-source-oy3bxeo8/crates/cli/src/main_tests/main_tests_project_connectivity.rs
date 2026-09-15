use std::collections::BTreeMap;

use super::*;
use eda_engine::ir::geometry::Point;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::schematic::{
    HiddenPowerBehavior, HierarchicalPort, LabelKind, NetLabel, PinElectricalType, PlacedSymbol,
    PortDirection, SchematicWire, SymbolDisplayMode, SymbolPin,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_native_sheet(
    root: &Path,
    sheet_uuid: Uuid,
    sheet_name: &str,
    symbols: BTreeMap<String, serde_json::Value>,
    wires: BTreeMap<String, serde_json::Value>,
    labels: BTreeMap<String, serde_json::Value>,
    ports: BTreeMap<String, serde_json::Value>,
) {
    let sheet_path = root
        .join("schematic/sheets")
        .join(format!("{sheet_uuid}.json"));
    std::fs::write(
        &sheet_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": sheet_uuid,
                "name": sheet_name,
                "frame": null,
                "symbols": symbols,
                "wires": wires,
                "junctions": {},
                "labels": labels,
                "buses": {},
                "bus_entries": {},
                "ports": ports,
                "noconnects": {},
                "texts": {},
                "drawings": {}
            }))
            .expect("sheet JSON should serialize")
        ),
    )
    .expect("sheet file should write");

    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    schematic_value["sheets"] = serde_json::json!({
        sheet_uuid.to_string(): format!("sheets/{sheet_uuid}.json")
    });
    std::fs::write(
        &schematic_json,
        format!(
            "{}\n",
            to_json_deterministic(&schematic_value)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("schematic.json should write");
}

#[test]
fn project_query_nets_reports_native_connectivity_inventory() {
    let root = unique_project_root("datum-eda-cli-project-query-nets");
    create_native_project(&root, Some("Connectivity Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let pin_uuid = Uuid::new_v4();
    let wire_uuid = Uuid::new_v4();
    let label_uuid = Uuid::new_v4();

    write_native_sheet(
        &root,
        sheet_uuid,
        "Main",
        BTreeMap::from([(
            symbol_uuid.to_string(),
            serde_json::to_value(PlacedSymbol {
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
                    position: Point::new(10, 10),
                }],
                position: Point::new(0, 0),
                rotation: 0,
                mirrored: false,
                unit_selection: None,
                display_mode: SymbolDisplayMode::LibraryDefault,
                pin_overrides: Vec::new(),
                hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
            })
            .expect("symbol should serialize"),
        )]),
        BTreeMap::from([(
            wire_uuid.to_string(),
            serde_json::to_value(SchematicWire {
                uuid: wire_uuid,
                from: Point::new(10, 10),
                to: Point::new(20, 10),
            })
            .expect("wire should serialize"),
        )]),
        BTreeMap::from([(
            label_uuid.to_string(),
            serde_json::to_value(NetLabel {
                uuid: label_uuid,
                kind: LabelKind::Local,
                name: "SIG".into(),
                position: Point::new(15, 10),
            })
            .expect("label should serialize"),
        )]),
        BTreeMap::new(),
    );

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "nets",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query nets should succeed");
    let nets: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    let nets = nets.as_array().expect("nets should be an array");
    assert_eq!(nets.len(), 1);
    assert_eq!(nets[0]["name"], "SIG");
    assert_eq!(nets[0]["labels"], 1);
    assert_eq!(nets[0]["ports"], 0);
    assert_eq!(nets[0]["pins"].as_array().unwrap().len(), 1);
    assert_eq!(nets[0]["pins"][0]["uuid"], pin_uuid.to_string());
    assert_eq!(nets[0]["sheets"], serde_json::json!(["Main"]));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_diagnostics_reports_native_connectivity_findings() {
    let root = unique_project_root("datum-eda-cli-project-query-diagnostics");
    create_native_project(&root, Some("Diagnostics Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let pin_a_uuid = Uuid::new_v4();
    let pin_b_uuid = Uuid::new_v4();
    let pin_c_uuid = Uuid::new_v4();
    let port_uuid = Uuid::new_v4();

    let symbol_a_uuid = Uuid::new_v4();
    let symbol_b_uuid = Uuid::new_v4();
    let symbol_c_uuid = Uuid::new_v4();

    write_native_sheet(
        &root,
        sheet_uuid,
        "Root",
        BTreeMap::from([
            (
                symbol_a_uuid.to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: symbol_a_uuid,
                    part: None,
                    entity: None,
                    gate: None,
                    lib_id: Some("Device:R".into()),
                    reference: "R1".into(),
                    value: "10k".into(),
                    fields: Vec::new(),
                    pins: vec![SymbolPin {
                        uuid: pin_a_uuid,
                        number: "1".into(),
                        name: "~".into(),
                        electrical_type: PinElectricalType::Passive,
                        position: Point::new(5, 5),
                    }],
                    position: Point::new(0, 0),
                    rotation: 0,
                    mirrored: false,
                    unit_selection: None,
                    display_mode: SymbolDisplayMode::LibraryDefault,
                    pin_overrides: Vec::new(),
                    hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
                })
                .expect("symbol should serialize"),
            ),
            (
                symbol_b_uuid.to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: symbol_b_uuid,
                    part: None,
                    entity: None,
                    gate: None,
                    lib_id: Some("Device:R".into()),
                    reference: "R2".into(),
                    value: "10k".into(),
                    fields: Vec::new(),
                    pins: vec![SymbolPin {
                        uuid: pin_b_uuid,
                        number: "1".into(),
                        name: "~".into(),
                        electrical_type: PinElectricalType::Passive,
                        position: Point::new(20, 20),
                    }],
                    position: Point::new(0, 0),
                    rotation: 0,
                    mirrored: false,
                    unit_selection: None,
                    display_mode: SymbolDisplayMode::LibraryDefault,
                    pin_overrides: Vec::new(),
                    hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
                })
                .expect("symbol should serialize"),
            ),
            (
                symbol_c_uuid.to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: symbol_c_uuid,
                    part: None,
                    entity: None,
                    gate: None,
                    lib_id: Some("Device:R".into()),
                    reference: "R3".into(),
                    value: "10k".into(),
                    fields: Vec::new(),
                    pins: vec![SymbolPin {
                        uuid: pin_c_uuid,
                        number: "1".into(),
                        name: "~".into(),
                        electrical_type: PinElectricalType::Passive,
                        position: Point::new(20, 20),
                    }],
                    position: Point::new(0, 0),
                    rotation: 0,
                    mirrored: false,
                    unit_selection: None,
                    display_mode: SymbolDisplayMode::LibraryDefault,
                    pin_overrides: Vec::new(),
                    hidden_power_behavior: HiddenPowerBehavior::PreservedAsImportedMetadata,
                })
                .expect("symbol should serialize"),
            ),
        ]),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(
            port_uuid.to_string(),
            serde_json::to_value(HierarchicalPort {
                uuid: port_uuid,
                name: "SUB_IN".into(),
                direction: PortDirection::Input,
                position: Point::new(60, 15),
            })
            .expect("port should serialize"),
        )]),
    );

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "diagnostics",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query diagnostics should succeed");
    let diagnostics: serde_json::Value =
        serde_json::from_str(&output).expect("query JSON should parse");
    let diagnostics = diagnostics
        .as_array()
        .expect("diagnostics should be an array");
    assert_eq!(diagnostics.len(), 3);
    assert!(diagnostics.iter().any(|entry| {
        entry["kind"] == "dangling_component_pin"
            && entry["objects"] == serde_json::json!([pin_a_uuid.to_string()])
    }));
    assert!(diagnostics.iter().any(|entry| {
        entry["kind"] == "dangling_interface_port"
            && entry["objects"] == serde_json::json!([port_uuid.to_string()])
    }));
    let mut expected_multi_pin = vec![pin_b_uuid.to_string(), pin_c_uuid.to_string()];
    expected_multi_pin.sort();
    let expected_multi_pin = serde_json::json!(expected_multi_pin);
    assert!(diagnostics.iter().any(|entry| {
        entry["kind"] == "anonymous_multi_pin_net" && entry["objects"] == expected_multi_pin
    }));

    let _ = std::fs::remove_dir_all(&root);
}
