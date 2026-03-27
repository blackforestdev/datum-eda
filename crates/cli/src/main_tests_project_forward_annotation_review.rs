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
fn project_forward_annotation_review_persists_defer_and_reject_by_action_id() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-review");
    create_native_project(&root, Some("Forward Annotation Review Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let missing_symbol_uuid = Uuid::new_v4();
    let matched_symbol_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    write_native_sheet(
        &root,
        sheet_uuid,
        "Main",
        BTreeMap::from([
            (
                missing_symbol_uuid.to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: missing_symbol_uuid,
                    part: Some(part_uuid),
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
                matched_symbol_uuid.to_string(),
                serde_json::to_value(PlacedSymbol {
                    uuid: matched_symbol_uuid,
                    part: Some(part_uuid),
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

    let orphan_component_uuid = Uuid::new_v4();
    let matched_component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Forward Annotation Review Demo Board",
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
                    matched_component_uuid.to_string(): serde_json::to_value(PlacedPackage {
                        uuid: matched_component_uuid,
                        part: part_uuid,
                        package: Uuid::new_v4(),
                        reference: "R1".into(),
                        value: "22k".into(),
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
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "forward-annotation-proposal",
    ])
    .expect("CLI should parse");
    let proposal_output = execute(proposal_cli).expect("proposal should succeed");
    let proposal: serde_json::Value =
        serde_json::from_str(&proposal_output).expect("proposal JSON");
    let defer_action_id = proposal["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "add_component")
        .unwrap()["action_id"]
        .as_str()
        .unwrap()
        .to_string();
    let reject_action_id = proposal["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "remove_component")
        .unwrap()["action_id"]
        .as_str()
        .unwrap()
        .to_string();

    let defer_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "defer-forward-annotation-action",
        root.to_str().unwrap(),
        "--action-id",
        &defer_action_id,
    ])
    .expect("CLI should parse");
    let defer_output = execute(defer_cli).expect("defer should succeed");
    let deferred: serde_json::Value = serde_json::from_str(&defer_output).expect("defer JSON");
    assert_eq!(deferred["decision"], "deferred");
    assert_eq!(deferred["proposal_action"], "add_component");

    let reject_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "reject-forward-annotation-action",
        root.to_str().unwrap(),
        "--action-id",
        &reject_action_id,
    ])
    .expect("CLI should parse");
    let reject_output = execute(reject_cli).expect("reject should succeed");
    let rejected: serde_json::Value = serde_json::from_str(&reject_output).expect("reject JSON");
    assert_eq!(rejected["decision"], "rejected");
    assert_eq!(rejected["proposal_action"], "remove_component");

    let review_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "forward-annotation-review",
    ])
    .expect("CLI should parse");
    let review_output = execute(review_cli).expect("review query should succeed");
    let review: serde_json::Value = serde_json::from_str(&review_output).expect("review JSON");
    assert_eq!(review["total_reviews"], 2);
    assert_eq!(review["deferred_actions"], 1);
    assert_eq!(review["rejected_actions"], 1);
    let actions = review["actions"].as_array().unwrap();
    assert!(
        actions.iter().any(|entry| {
            entry["action_id"] == defer_action_id && entry["decision"] == "deferred"
        })
    );
    assert!(actions.iter().any(|entry| {
        entry["action_id"] == reject_action_id && entry["decision"] == "rejected"
    }));

    let clear_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "clear-forward-annotation-action-review",
        root.to_str().unwrap(),
        "--action-id",
        &defer_action_id,
    ])
    .expect("CLI should parse");
    let clear_output = execute(clear_cli).expect("clear should succeed");
    let cleared: serde_json::Value = serde_json::from_str(&clear_output).expect("clear JSON");
    assert_eq!(cleared["action"], "clear_forward_annotation_action_review");
    assert_eq!(cleared["action_id"], defer_action_id);
    assert_eq!(cleared["decision"], "deferred");

    let review_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "forward-annotation-review",
    ])
    .expect("CLI should parse");
    let review_output = execute(review_cli).expect("review query should succeed");
    let review: serde_json::Value = serde_json::from_str(&review_output).expect("review JSON");
    assert_eq!(review["total_reviews"], 1);
    assert_eq!(review["deferred_actions"], 0);
    assert_eq!(review["rejected_actions"], 1);
    let actions = review["actions"].as_array().unwrap();
    assert!(
        !actions
            .iter()
            .any(|entry| entry["action_id"] == defer_action_id)
    );
    assert!(
        actions
            .iter()
            .any(|entry| entry["action_id"] == reject_action_id)
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_forward_annotation_review_rejects_unknown_action_id() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-review-missing");
    create_native_project(&root, Some("Forward Annotation Review Demo".to_string()))
        .expect("initial scaffold should succeed");

    let reject_cli = Cli::try_parse_from([
        "eda",
        "project",
        "reject-forward-annotation-action",
        root.to_str().unwrap(),
        "--action-id",
        "sha256:missing",
    ])
    .expect("CLI should parse");
    let err = execute(reject_cli).expect_err("unknown action should fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("forward-annotation proposal action not found"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_forward_annotation_review_rejects_unknown_clear_action_id() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-review-clear-missing");
    create_native_project(&root, Some("Forward Annotation Review Demo".to_string()))
        .expect("initial scaffold should succeed");

    let clear_cli = Cli::try_parse_from([
        "eda",
        "project",
        "clear-forward-annotation-action-review",
        root.to_str().unwrap(),
        "--action-id",
        "sha256:missing",
    ])
    .expect("CLI should parse");
    let err = execute(clear_cli).expect_err("unknown review clear should fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("forward-annotation review action not found"));

    let _ = std::fs::remove_dir_all(&root);
}
