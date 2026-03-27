use std::collections::BTreeMap;

use super::*;
use eda_engine::board::PlacedPackage;
use eda_engine::ir::geometry::Point;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::schematic::{HiddenPowerBehavior, PlacedSymbol, SymbolDisplayMode};

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn write_native_sheet(
    root: &Path,
    sheet_uuid: Uuid,
    sheet_name: &str,
    symbols: BTreeMap<String, serde_json::Value>,
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
            to_json_deterministic(&schematic_value)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("schematic.json should write");
}

#[test]
fn project_query_forward_annotation_proposal_reports_deterministic_actions() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-proposal");
    create_native_project(&root, Some("Forward Annotation Proposal Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let matched_symbol_uuid = Uuid::new_v4();
    let missing_symbol_uuid = Uuid::new_v4();
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
                "name": "Forward Annotation Proposal Demo Board",
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
        "forward-annotation-proposal",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("forward annotation proposal should succeed");
    let report: serde_json::Value = serde_json::from_str(&output).expect("query JSON should parse");
    assert_eq!(report["domain"], "native_project");
    assert_eq!(report["total_actions"], 3);
    assert_eq!(report["add_component_actions"], 1);
    assert_eq!(report["remove_component_actions"], 1);
    assert_eq!(report["update_component_actions"], 1);
    assert_eq!(report["add_component_group"].as_array().unwrap().len(), 1);
    assert_eq!(
        report["remove_component_group"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        report["update_component_group"].as_array().unwrap().len(),
        1
    );
    assert!(report["actions"].as_array().unwrap().iter().any(|entry| {
        entry["action"] == "add_component"
            && entry["reference"] == "C1"
            && entry["reason"] == "symbol_missing_on_board"
            && entry["action_id"].as_str().unwrap().starts_with("sha256:")
    }));
    assert!(report["actions"].as_array().unwrap().iter().any(|entry| {
        entry["action"] == "remove_component"
            && entry["reference"] == "U1"
            && entry["reason"] == "board_component_missing_in_schematic"
            && entry["action_id"].as_str().unwrap().starts_with("sha256:")
    }));
    assert!(report["actions"].as_array().unwrap().iter().any(|entry| {
        entry["action"] == "update_component"
            && entry["reference"] == "R1"
            && entry["reason"] == "value_mismatch"
            && entry["schematic_value"] == "10k"
            && entry["board_value"] == "22k"
            && entry["action_id"].as_str().unwrap().starts_with("sha256:")
    }));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_forward_annotation_proposal_reports_text_summary() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-proposal-text");
    create_native_project(&root, Some("Forward Annotation Proposal Demo".to_string()))
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
        "forward-annotation-proposal",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("forward annotation proposal should succeed");
    assert!(output.contains("total_actions: 1"));
    assert!(output.contains("add_component_actions: 1"));
    assert!(output.contains("actions:"));
    assert!(output.contains("add_component R1 id=sha256:"));
    assert!(output.contains("reason=symbol_missing_on_board_unresolved_part"));

    let _ = std::fs::remove_dir_all(&root);
}
