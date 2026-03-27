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
fn project_export_forward_annotation_proposal_writes_versioned_artifact_with_reviews() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-export");
    create_native_project(&root, Some("Forward Annotation Export Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    write_native_sheet(
        &root,
        sheet_uuid,
        "Main",
        BTreeMap::from([(
            symbol_uuid.to_string(),
            serde_json::to_value(PlacedSymbol {
                uuid: symbol_uuid,
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
        )]),
    );

    let orphan_component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Forward Annotation Export Demo Board",
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

    let defer_cli = Cli::try_parse_from([
        "eda", "project", "defer-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &add_action_id,
    ])
    .expect("CLI should parse");
    let _ = execute(defer_cli).expect("defer should succeed");

    let artifact_path = root.join("forward-annotation-proposal.json");
    let export_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "export-forward-annotation-proposal",
        root.to_str().unwrap(), "--out", artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let export_output = execute(export_cli).expect("export should succeed");
    let report: serde_json::Value = serde_json::from_str(&export_output).expect("export JSON");
    assert_eq!(report["action"], "export_forward_annotation_proposal");
    assert_eq!(report["kind"], "native_forward_annotation_proposal_artifact");
    assert_eq!(report["version"], 1);
    assert_eq!(report["actions"], 2);
    assert_eq!(report["reviews"], 1);

    let artifact_text = std::fs::read_to_string(&artifact_path).expect("artifact should read");
    let artifact: serde_json::Value =
        serde_json::from_str(&artifact_text).expect("artifact should parse");
    assert_eq!(artifact["kind"], "native_forward_annotation_proposal_artifact");
    assert_eq!(artifact["version"], 1);
    assert_eq!(artifact["project_name"], "Forward Annotation Export Demo");
    assert_eq!(artifact["actions"].as_array().unwrap().len(), 2);
    assert_eq!(artifact["reviews"].as_array().unwrap().len(), 1);
    assert_eq!(artifact["reviews"][0]["action_id"], add_action_id);
    assert_eq!(artifact["reviews"][0]["decision"], "deferred");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_inspect_forward_annotation_proposal_artifact_reports_counts() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-inspect-artifact");
    create_native_project(&root, Some("Forward Annotation Inspect Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let part_uuid = Uuid::new_v4();
    write_native_sheet(
        &root,
        sheet_uuid,
        "Main",
        BTreeMap::from([(
            symbol_uuid.to_string(),
            serde_json::to_value(PlacedSymbol {
                uuid: symbol_uuid,
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
        )]),
    );

    let orphan_component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Forward Annotation Inspect Demo Board",
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

    let defer_cli = Cli::try_parse_from([
        "eda", "project", "defer-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &add_action_id,
    ])
    .expect("CLI should parse");
    let _ = execute(defer_cli).expect("defer should succeed");

    let artifact_path = root.join("forward-annotation-proposal.json");
    let export_cli = Cli::try_parse_from([
        "eda", "project", "export-forward-annotation-proposal",
        root.to_str().unwrap(), "--out", artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let _ = execute(export_cli).expect("export should succeed");

    let inspect_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "inspect-forward-annotation-proposal-artifact",
        artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let inspect_output = execute(inspect_cli).expect("inspect should succeed");
    let inspection: serde_json::Value =
        serde_json::from_str(&inspect_output).expect("inspection JSON");
    assert_eq!(inspection["kind"], "native_forward_annotation_proposal_artifact");
    assert_eq!(inspection["source_version"], 1);
    assert_eq!(inspection["version"], 1);
    assert_eq!(inspection["migration_applied"], false);
    assert_eq!(inspection["project_name"], "Forward Annotation Inspect Demo");
    assert_eq!(inspection["actions"], 2);
    assert_eq!(inspection["reviews"], 1);
    assert_eq!(inspection["add_component_actions"], 1);
    assert_eq!(inspection["remove_component_actions"], 1);
    assert_eq!(inspection["update_component_actions"], 0);
    assert_eq!(inspection["deferred_reviews"], 1);
    assert_eq!(inspection["rejected_reviews"], 0);

    let _ = std::fs::remove_dir_all(&root);
}
