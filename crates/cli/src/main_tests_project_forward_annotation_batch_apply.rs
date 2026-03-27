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
fn project_apply_forward_annotation_reviewed_applies_supported_and_skips_reviewed_or_input_bound_actions(
) {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-batch-apply");
    create_native_project(&root, Some("Forward Annotation Batch Apply Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let c1_symbol_uuid = Uuid::new_v4();
    let r1_symbol_uuid = Uuid::new_v4();
    let q1_symbol_uuid = Uuid::new_v4();
    let c1_part_uuid = Uuid::new_v4();
    let r1_part_uuid = Uuid::new_v4();
    let q1_part_uuid = Uuid::new_v4();
    write_native_sheet(
        &root,
        sheet_uuid,
        "Main",
        BTreeMap::from([
            (
                c1_symbol_uuid.to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: c1_symbol_uuid,
                    part: Some(c1_part_uuid),
                    entity: None,
                    gate: None,
                    lib_id: Some("Device:C".into()),
                    reference: "C1".into(),
                    value: "1u".into(),
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
                r1_symbol_uuid.to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: r1_symbol_uuid,
                    part: Some(r1_part_uuid),
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
                q1_symbol_uuid.to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: q1_symbol_uuid,
                    part: Some(q1_part_uuid),
                    entity: None,
                    gate: None,
                    lib_id: Some("Device:Q".into()),
                    reference: "Q1".into(),
                    value: "BJT".into(),
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
        ]),
    );

    let orphan_component_uuid = Uuid::new_v4();
    let r1_component_uuid = Uuid::new_v4();
    let q1_component_uuid = Uuid::new_v4();
    let q1_board_part_uuid = Uuid::new_v4();
    let q1_board_package_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Forward Annotation Batch Apply Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    orphan_component_uuid.to_string(): serde_json::to_value(PlacedPackage {
                        uuid: orphan_component_uuid,
                        part: Uuid::new_v4(),
                        package: Uuid::new_v4(),
                        reference: "U1".into(),
                        value: "MCU".into(),
                        position: Point::new(0, 0),
                        rotation: 0,
                        layer: 1,
                        locked: false,
                    }).expect("component should serialize"),
                    r1_component_uuid.to_string(): serde_json::to_value(PlacedPackage {
                        uuid: r1_component_uuid,
                        part: r1_part_uuid,
                        package: Uuid::new_v4(),
                        reference: "R1".into(),
                        value: "22k".into(),
                        position: Point::new(0, 0),
                        rotation: 0,
                        layer: 1,
                        locked: false,
                    }).expect("component should serialize"),
                    q1_component_uuid.to_string(): serde_json::to_value(PlacedPackage {
                        uuid: q1_component_uuid,
                        part: q1_board_part_uuid,
                        package: q1_board_package_uuid,
                        reference: "Q1".into(),
                        value: "BJT".into(),
                        position: Point::new(0, 0),
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

    let proposal_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "query", root.to_str().unwrap(),
        "forward-annotation-proposal",
    ])
    .expect("CLI should parse");
    let proposal_output = execute(proposal_cli).expect("proposal should succeed");
    let proposal: serde_json::Value = serde_json::from_str(&proposal_output).expect("proposal JSON");
    let add_action_id = proposal["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "add_component")
        .unwrap()["action_id"]
        .as_str()
        .unwrap()
        .to_string();
    let remove_action_id = proposal["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "remove_component")
        .unwrap()["action_id"]
        .as_str()
        .unwrap()
        .to_string();

    let defer_cli = Cli::try_parse_from([
        "eda", "project", "defer-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &add_action_id,
    ])
    .expect("CLI should parse");
    let _ = execute(defer_cli).expect("defer should succeed");

    let reject_cli = Cli::try_parse_from([
        "eda", "project", "reject-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &remove_action_id,
    ])
    .expect("CLI should parse");
    let _ = execute(reject_cli).expect("reject should succeed");

    let batch_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "apply-forward-annotation-reviewed",
        root.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let batch_output = execute(batch_cli).expect("batch apply should succeed");
    let batch: serde_json::Value = serde_json::from_str(&batch_output).expect("batch JSON");
    assert_eq!(batch["action"], "apply_forward_annotation_reviewed");
    assert_eq!(batch["proposal_actions"], 4);
    assert_eq!(batch["applied_actions"], 1);
    assert_eq!(batch["skipped_deferred_actions"], 1);
    assert_eq!(batch["skipped_rejected_actions"], 1);
    assert_eq!(batch["skipped_requires_input_actions"], 1);

    let applied = batch["applied"].as_array().unwrap();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0]["proposal_action"], "update_component");
    assert_eq!(applied[0]["reason"], "value_mismatch");
    assert_eq!(applied[0]["component_report"]["reference"], "R1");
    assert_eq!(applied[0]["component_report"]["value"], "10k");

    let skipped = batch["skipped"].as_array().unwrap();
    assert!(skipped.iter().any(|entry| {
        entry["proposal_action"] == "add_component"
            && entry["reference"] == "C1"
            && entry["skip_reason"] == "deferred_by_review"
    }));
    assert!(skipped.iter().any(|entry| {
        entry["proposal_action"] == "remove_component"
            && entry["reference"] == "U1"
            && entry["skip_reason"] == "rejected_by_review"
    }));
    assert!(skipped.iter().any(|entry| {
        entry["proposal_action"] == "update_component"
            && entry["reference"] == "Q1"
            && entry["reason"] == "part_mismatch"
            && entry["skip_reason"] == "requires_explicit_input"
    }));

    let components_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "query", root.to_str().unwrap(), "board-components",
    ])
    .expect("CLI should parse");
    let components_output = execute(components_cli).expect("components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("components parse");
    assert_eq!(components.len(), 3);
    let r1 = components.iter().find(|component| component.reference == "R1").unwrap();
    assert_eq!(r1.value, "10k");
    let q1 = components.iter().find(|component| component.reference == "Q1").unwrap();
    assert_eq!(q1.part, q1_board_part_uuid);
    assert_eq!(q1.package, q1_board_package_uuid);
    assert!(components.iter().any(|component| component.reference == "U1"));
    assert!(!components.iter().any(|component| component.reference == "C1"));

    let _ = std::fs::remove_dir_all(&root);
}
