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
fn project_apply_forward_annotation_artifact_applies_only_self_sufficient_actions_and_honors_reviews() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-artifact");
    create_native_project(&root, Some("Forward Annotation Artifact Apply Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let r1_symbol_uuid = Uuid::new_v4();
    let r1_part_uuid = Uuid::new_v4();
    write_native_sheet(
        &root,
        sheet_uuid,
        "Main",
        BTreeMap::from([(
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
        )]),
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
                "name": "Forward Annotation Artifact Apply Demo Board",
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
    let remove_action_id = proposal["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "remove_component")
        .unwrap()["action_id"]
        .as_str()
        .unwrap()
        .to_string();

    let reject_cli = Cli::try_parse_from([
        "eda", "project", "reject-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &remove_action_id,
    ])
    .expect("CLI should parse");
    let _ = execute(reject_cli).expect("reject should succeed");

    let artifact_path = root.join("forward-annotation-proposal.json");
    let export_cli = Cli::try_parse_from([
        "eda", "project", "export-forward-annotation-proposal",
        root.to_str().unwrap(), "--out", artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let _ = execute(export_cli).expect("export should succeed");

    let filtered_artifact_path = root.join("forward-annotation-proposal-filtered.json");
    let filter_cli = Cli::try_parse_from([
        "eda", "project", "filter-forward-annotation-proposal-artifact",
        root.to_str().unwrap(), "--artifact", artifact_path.to_str().unwrap(),
        "--out", filtered_artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let _ = execute(filter_cli).expect("filter should succeed");

    let apply_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "apply-forward-annotation-proposal-artifact",
        root.to_str().unwrap(), "--artifact", filtered_artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("artifact apply should succeed");
    let apply: serde_json::Value = serde_json::from_str(&apply_output).expect("apply JSON");
    assert_eq!(apply["action"], "apply_forward_annotation_proposal_artifact");
    assert_eq!(apply["artifact_actions"], 2);
    assert_eq!(apply["applied_actions"], 1);
    assert_eq!(apply["skipped_deferred_actions"], 0);
    assert_eq!(apply["skipped_rejected_actions"], 1);
    assert_eq!(apply["applied"][0]["proposal_action"], "update_component");
    assert_eq!(apply["applied"][0]["component_report"]["reference"], "R1");
    assert_eq!(apply["applied"][0]["component_report"]["value"], "10k");
    assert!(apply["skipped"].as_array().unwrap().iter().any(|entry| {
        entry["proposal_action"] == "remove_component"
            && entry["reference"] == "U1"
            && entry["skip_reason"] == "rejected_by_review"
    }));

    let components_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "query", root.to_str().unwrap(), "board-components",
    ])
    .expect("CLI should parse");
    let components_output = execute(components_cli).expect("components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("components parse");
    assert_eq!(components.len(), 2);
    let r1 = components.iter().find(|component| component.reference == "R1").unwrap();
    assert_eq!(r1.value, "10k");
    assert!(components.iter().any(|component| component.reference == "U1"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_apply_forward_annotation_artifact_rejects_input_bound_actions() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-artifact-input");
    create_native_project(&root, Some("Forward Annotation Artifact Apply Input Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let c1_symbol_uuid = Uuid::new_v4();
    let c1_part_uuid = Uuid::new_v4();
    write_native_sheet(
        &root,
        sheet_uuid,
        "Main",
        BTreeMap::from([(
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
        )]),
    );

    let artifact_path = root.join("forward-annotation-proposal.json");
    let export_cli = Cli::try_parse_from([
        "eda", "project", "export-forward-annotation-proposal",
        root.to_str().unwrap(), "--out", artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let _ = execute(export_cli).expect("export should succeed");

    let filtered_artifact_path = root.join("forward-annotation-proposal-filtered.json");
    let filter_cli = Cli::try_parse_from([
        "eda", "project", "filter-forward-annotation-proposal-artifact",
        root.to_str().unwrap(), "--artifact", artifact_path.to_str().unwrap(),
        "--out", filtered_artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let _ = execute(filter_cli).expect("filter should succeed");

    let apply_cli = Cli::try_parse_from([
        "eda", "project", "apply-forward-annotation-proposal-artifact",
        root.to_str().unwrap(), "--artifact", filtered_artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let err = execute(apply_cli).expect_err("artifact apply should fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("forward-annotation artifact apply requires only self-sufficient actions"));

    let _ = std::fs::remove_dir_all(&root);
}
