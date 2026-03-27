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
fn project_import_forward_annotation_artifact_review_imports_only_live_matching_actions() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-import-review");
    create_native_project(&root, Some("Forward Annotation Import Review Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let c1_symbol_uuid = Uuid::new_v4();
    let r1_symbol_uuid = Uuid::new_v4();
    let c1_part_uuid = Uuid::new_v4();
    let r1_part_uuid = Uuid::new_v4();
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
        ]),
    );

    let r1_component_uuid = Uuid::new_v4();
    let orphan_component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Forward Annotation Import Review Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
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
    let add_action_id = proposal["actions"].as_array().unwrap().iter()
        .find(|entry| entry["action"] == "add_component").unwrap()["action_id"]
        .as_str().unwrap().to_string();
    let remove_action_id = proposal["actions"].as_array().unwrap().iter()
        .find(|entry| entry["action"] == "remove_component").unwrap()["action_id"]
        .as_str().unwrap().to_string();

    let defer_cli = Cli::try_parse_from([
        "eda", "project", "defer-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &add_action_id,
    ]).expect("CLI should parse");
    let _ = execute(defer_cli).expect("defer should succeed");

    let reject_cli = Cli::try_parse_from([
        "eda", "project", "reject-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &remove_action_id,
    ]).expect("CLI should parse");
    let _ = execute(reject_cli).expect("reject should succeed");

    let artifact_path = root.join("forward-annotation-proposal.json");
    let export_cli = Cli::try_parse_from([
        "eda", "project", "export-forward-annotation-proposal",
        root.to_str().unwrap(), "--out", artifact_path.to_str().unwrap(),
    ]).expect("CLI should parse");
    let _ = execute(export_cli).expect("export should succeed");

    let clear_add_cli = Cli::try_parse_from([
        "eda", "project", "clear-forward-annotation-action-review",
        root.to_str().unwrap(), "--action-id", &add_action_id,
    ]).expect("CLI should parse");
    let _ = execute(clear_add_cli).expect("clear add should succeed");

    let clear_remove_cli = Cli::try_parse_from([
        "eda", "project", "clear-forward-annotation-action-review",
        root.to_str().unwrap(), "--action-id", &remove_action_id,
    ]).expect("CLI should parse");
    let _ = execute(clear_remove_cli).expect("clear remove should succeed");

    let delete_orphan_cli = Cli::try_parse_from([
        "eda", "project", "delete-board-component",
        root.to_str().unwrap(), "--component", &orphan_component_uuid.to_string(),
    ]).expect("CLI should parse");
    let _ = execute(delete_orphan_cli).expect("delete orphan should succeed");

    let import_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "import-forward-annotation-artifact-review",
        root.to_str().unwrap(), "--artifact", artifact_path.to_str().unwrap(),
    ]).expect("CLI should parse");
    let import_output = execute(import_cli).expect("import should succeed");
    let report: serde_json::Value = serde_json::from_str(&import_output).expect("import JSON");
    assert_eq!(report["action"], "import_forward_annotation_artifact_review");
    assert_eq!(report["total_artifact_reviews"], 2);
    assert_eq!(report["imported_reviews"], 1);
    assert_eq!(report["skipped_missing_live_actions"], 1);

    let review_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "query", root.to_str().unwrap(),
        "forward-annotation-review",
    ]).expect("CLI should parse");
    let review_output = execute(review_cli).expect("review query should succeed");
    let review: serde_json::Value = serde_json::from_str(&review_output).expect("review JSON");
    let actions = review["actions"].as_array().unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0]["action_id"], add_action_id);
    assert_eq!(actions[0]["decision"], "deferred");

    let _ = std::fs::remove_dir_all(&root);
}
