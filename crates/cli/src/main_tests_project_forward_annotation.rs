use std::collections::BTreeMap;

use super::*;
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::schematic::{
    HiddenPowerBehavior, PinElectricalType, PlacedSymbol, SymbolDisplayMode, SymbolPin,
};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_native_sheet(
    root: &Path,
    sheet_uuid: Uuid,
    sheet_name: &str,
    symbols: BTreeMap<String, serde_json::Value>,
) {
    let sheet_path = root.join("schematic/sheets").join(format!("{sheet_uuid}.json"));
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
                "labels": {},
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
            to_json_deterministic(&schematic_value).expect("canonical serialization should succeed")
        ),
    )
    .expect("schematic.json should write");
}

#[test]
fn project_query_forward_annotation_audit_reports_native_reference_alignment() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-audit");
    create_native_project(&root, Some("Forward Annotation Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let matched_symbol_uuid = Uuid::new_v4();
    let missing_symbol_uuid = Uuid::new_v4();
    let unresolved_symbol_uuid = Uuid::new_v4();
    let matched_part_uuid = Uuid::new_v4();
    let missing_part_uuid = Uuid::new_v4();

    write_native_sheet(
        &root,
        sheet_uuid,
        "Main",
        BTreeMap::from([
            (
                matched_symbol_uuid.to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: matched_symbol_uuid,
                    part: Some(matched_part_uuid),
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
                        position: Point::new(0, 0),
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
                missing_symbol_uuid.to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: missing_symbol_uuid,
                    part: Some(missing_part_uuid),
                    entity: None,
                    gate: None,
                    lib_id: Some("Device:C".into()),
                    reference: "C1".into(),
                    value: "1u".into(),
                    fields: Vec::new(),
                    pins: Vec::new(),
                    position: Point::new(10, 0),
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
                unresolved_symbol_uuid.to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: unresolved_symbol_uuid,
                    part: None,
                    entity: None,
                    gate: None,
                    lib_id: Some("Device:LED".into()),
                    reference: "D1".into(),
                    value: "GREEN".into(),
                    fields: Vec::new(),
                    pins: Vec::new(),
                    position: Point::new(20, 0),
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
    );

    let matched_component_uuid = Uuid::new_v4();
    let orphan_component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Forward Annotation Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    matched_component_uuid.to_string(): serde_json::to_value(PlacedPackage {
                        uuid: matched_component_uuid,
                        part: matched_part_uuid,
                        package: Uuid::new_v4(),
                        reference: "R1".into(),
                        value: "22k".into(),
                        position: Point::new(0, 0),
                        rotation: 0,
                        layer: 1,
                        locked: false,
                    }).expect("component should serialize"),
                    orphan_component_uuid.to_string(): serde_json::to_value(PlacedPackage {
                        uuid: orphan_component_uuid,
                        part: Uuid::new_v4(),
                        package: Uuid::new_v4(),
                        reference: "U1".into(),
                        value: "MCU".into(),
                        position: Point::new(100, 0),
                        rotation: 0,
                        layer: 1,
                        locked: false,
                    }).expect("component should serialize")
                },
                "pads": {},
                "tracks": {},
                "vias": {},
                "zones": {},
                "nets": {},
                "net_classes": {},
                "rules": [],
                "keepouts": [],
                "dimensions": [],
                "texts": []
            }))
            .expect("canonical serialization should succeed")
        ),
    )
    .expect("board file should write");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "forward-annotation-audit",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("forward annotation audit should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    assert_eq!(report["domain"], "native_project");
    assert_eq!(report["schematic_symbol_count"], 3);
    assert_eq!(report["board_component_count"], 2);
    assert_eq!(report["matched_count"], 1);
    assert_eq!(report["unresolved_symbol_count"], 1);
    assert_eq!(report["missing_on_board"].as_array().unwrap().len(), 2);
    assert!(report["missing_on_board"].as_array().unwrap().iter().any(|entry| {
        entry["reference"] == "C1" && entry["part_uuid"] == missing_part_uuid.to_string()
    }));
    assert!(report["missing_on_board"].as_array().unwrap().iter().any(|entry| {
        entry["reference"] == "D1" && entry["part_uuid"].is_null()
    }));
    assert_eq!(report["orphaned_on_board"].as_array().unwrap().len(), 1);
    assert_eq!(report["orphaned_on_board"][0]["reference"], "U1");
    assert_eq!(report["value_mismatches"].as_array().unwrap().len(), 1);
    assert_eq!(report["value_mismatches"][0]["reference"], "R1");
    assert_eq!(report["value_mismatches"][0]["schematic_value"], "10k");
    assert_eq!(report["value_mismatches"][0]["board_value"], "22k");
    assert!(report["part_mismatches"].as_array().unwrap().is_empty());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_forward_annotation_audit_reports_text_summary() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-audit-text");
    create_native_project(&root, Some("Forward Annotation Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
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
                pins: Vec::new(),
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
    );

    let cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "forward-annotation-audit",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("forward annotation audit should succeed");
    assert!(output.contains("schematic_symbol_count: 1"));
    assert!(output.contains("board_component_count: 0"));
    assert!(output.contains("unresolved_symbol_count: 1"));
    assert!(output.contains("missing_on_board_count: 1"));
    assert!(output.contains("missing_on_board:"));
    assert!(output.contains("R1 value=10k part_uuid=none"));

    let _ = std::fs::remove_dir_all(&root);
}
