use std::collections::BTreeMap;

use super::*;
use eda_engine::ir::geometry::Point;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::schematic::{
    CheckDomain, HiddenPowerBehavior, LabelKind, NetLabel, PinElectricalType, PlacedSymbol,
    SymbolDisplayMode, SymbolPin, WaiverTarget,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_native_sheet(
    root: &Path,
    sheet_uuid: Uuid,
    sheet_name: &str,
    symbols: BTreeMap<String, serde_json::Value>,
    labels: BTreeMap<String, serde_json::Value>,
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
                "wires": {},
                "junctions": {},
                "labels": labels,
                "buses": {},
                "bus_entries": {},
                "ports": {},
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

fn write_native_waivers(root: &Path, waivers: &[serde_json::Value]) {
    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    schematic_value["waivers"] = serde_json::Value::Array(waivers.to_vec());
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
fn project_query_erc_reports_native_precheck_findings() {
    let root = unique_project_root("datum-eda-cli-project-query-erc");
    create_native_project(&root, Some("ERC Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let passive_pin_uuid = Uuid::new_v4();
    let power_pin_uuid = Uuid::new_v4();
    let power_label_uuid = Uuid::new_v4();

    write_native_sheet(
        &root,
        sheet_uuid,
        "Root",
        BTreeMap::from([
            (
                Uuid::new_v4().to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: Uuid::new_v4(),
                    part: None,
                    entity: None,
                    gate: None,
                    lib_id: Some("Device:R".into()),
                    reference: "R1".into(),
                    value: "10k".into(),
                    fields: Vec::new(),
                    pins: vec![SymbolPin {
                        uuid: passive_pin_uuid,
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
                Uuid::new_v4().to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: Uuid::new_v4(),
                    part: None,
                    entity: None,
                    gate: None,
                    lib_id: Some("power:VCC".into()),
                    reference: "PWR1".into(),
                    value: "VCC".into(),
                    fields: Vec::new(),
                    pins: vec![SymbolPin {
                        uuid: power_pin_uuid,
                        number: "1".into(),
                        name: "VCC".into(),
                        electrical_type: PinElectricalType::PowerIn,
                        position: Point::new(20, 20),
                    }],
                    position: Point::new(0, 0),
                    rotation: 0,
                    mirrored: false,
                    unit_selection: None,
                    display_mode: SymbolDisplayMode::LibraryDefault,
                    pin_overrides: Vec::new(),
                    hidden_power_behavior: HiddenPowerBehavior::ExplicitPowerObject,
                })
                .expect("symbol should serialize"),
            ),
        ]),
        BTreeMap::from([(
            power_label_uuid.to_string(),
            serde_json::to_value(NetLabel {
                uuid: power_label_uuid,
                kind: LabelKind::Power,
                name: "VCC".into(),
                position: Point::new(20, 20),
            })
            .expect("label should serialize"),
        )]),
    );

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "erc",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query erc should succeed");
    let findings: serde_json::Value =
        serde_json::from_str(&output).expect("query JSON should parse");
    let findings = findings.as_array().expect("findings should be an array");
    assert!(findings.iter().any(|entry| {
        entry["code"] == "unconnected_component_pin"
            && entry["object_uuids"] == serde_json::json!([passive_pin_uuid.to_string()])
    }));
    assert!(findings.iter().any(|entry| {
        entry["code"] == "power_in_without_source"
            && entry["object_uuids"] == serde_json::json!([power_pin_uuid.to_string()])
    }));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_erc_honors_native_authored_waiver() {
    let root = unique_project_root("datum-eda-cli-project-query-erc-waiver");
    create_native_project(&root, Some("ERC Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let passive_pin_uuid = Uuid::new_v4();

    write_native_sheet(
        &root,
        sheet_uuid,
        "Root",
        BTreeMap::from([(
            Uuid::new_v4().to_string(),
            serde_json::to_value(PlacedSymbol {
                uuid: Uuid::new_v4(),
                part: None,
                entity: None,
                gate: None,
                lib_id: Some("Device:R".into()),
                reference: "R1".into(),
                value: "10k".into(),
                fields: Vec::new(),
                pins: vec![SymbolPin {
                    uuid: passive_pin_uuid,
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
        )]),
        BTreeMap::new(),
    );

    write_native_waivers(
        &root,
        &[serde_json::to_value(serde_json::json!({
            "uuid": Uuid::new_v4(),
            "domain": CheckDomain::ERC,
            "target": WaiverTarget::Object(passive_pin_uuid),
            "rationale": "Intentional dangling passive pin",
            "created_by": "cli-test"
        }))
        .expect("waiver should serialize")],
    );

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "erc",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("project query erc should succeed");
    let findings: serde_json::Value =
        serde_json::from_str(&output).expect("query JSON should parse");
    let findings = findings.as_array().expect("findings should be an array");
    let waived = findings
        .iter()
        .find(|entry| entry["code"] == "unconnected_component_pin")
        .expect("unconnected_component_pin finding should exist");
    assert_eq!(
        waived["object_uuids"],
        serde_json::json!([passive_pin_uuid.to_string()])
    );
    assert_eq!(waived["waived"], true);

    let _ = std::fs::remove_dir_all(&root);
}
