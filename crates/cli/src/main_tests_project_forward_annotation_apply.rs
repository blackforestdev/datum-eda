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
fn project_apply_forward_annotation_action_applies_value_mismatch_update() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-update");
    create_native_project(&root, Some("Forward Annotation Apply Demo".to_string()))
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

    let component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Forward Annotation Apply Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    component_uuid.to_string(): serde_json::to_value(PlacedPackage {
                        uuid: component_uuid,
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
        "eda", "--format", "json", "project", "query", root.to_str().unwrap(),
        "forward-annotation-proposal",
    ])
    .expect("CLI should parse");
    let proposal_output = execute(proposal_cli).expect("proposal should succeed");
    let proposal: serde_json::Value = serde_json::from_str(&proposal_output).expect("proposal JSON");
    let action_id = proposal["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "update_component" && entry["reason"] == "value_mismatch")
        .unwrap()["action_id"]
        .as_str()
        .unwrap()
        .to_string();

    let apply_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "apply-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &action_id,
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&apply_output).expect("apply JSON");
    assert_eq!(applied["action"], "apply_forward_annotation_action");
    assert_eq!(applied["action_id"], action_id);
    assert_eq!(applied["proposal_action"], "update_component");
    assert_eq!(applied["reason"], "value_mismatch");
    assert_eq!(applied["component_report"]["value"], "10k");

    let components_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "query", root.to_str().unwrap(), "board-components",
    ])
    .expect("CLI should parse");
    let components_output = execute(components_cli).expect("components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("components parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].value, "10k");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_apply_forward_annotation_action_applies_remove_component() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-remove");
    create_native_project(&root, Some("Forward Annotation Apply Demo".to_string()))
        .expect("initial scaffold should succeed");

    let component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Forward Annotation Apply Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    component_uuid.to_string(): serde_json::to_value(PlacedPackage {
                        uuid: component_uuid,
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
    let action_id = proposal["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "remove_component")
        .unwrap()["action_id"]
        .as_str()
        .unwrap()
        .to_string();

    let apply_cli = Cli::try_parse_from([
        "eda", "project", "apply-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &action_id,
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("apply should succeed");
    assert!(apply_output.contains("proposal_action: remove_component"));
    assert!(apply_output.contains("reference: U1"));

    let components_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "query", root.to_str().unwrap(), "board-components",
    ])
    .expect("CLI should parse");
    let components_output = execute(components_cli).expect("components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("components parse");
    assert!(components.is_empty());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_apply_forward_annotation_action_applies_add_component_with_explicit_resolution() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-add");
    create_native_project(&root, Some("Forward Annotation Apply Demo".to_string()))
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

    let proposal_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "query", root.to_str().unwrap(),
        "forward-annotation-proposal",
    ])
    .expect("CLI should parse");
    let proposal_output = execute(proposal_cli).expect("proposal should succeed");
    let proposal: serde_json::Value = serde_json::from_str(&proposal_output).expect("proposal JSON");
    let action_id = proposal["actions"][0]["action_id"].as_str().unwrap().to_string();
    let package_uuid = Uuid::new_v4();

    let apply_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "apply-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &action_id,
        "--package", &package_uuid.to_string(),
        "--x-nm", "2500000",
        "--y-nm", "3500000",
        "--layer", "1",
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("add_component should succeed");
    let applied: serde_json::Value = serde_json::from_str(&apply_output).expect("apply JSON");
    assert_eq!(applied["proposal_action"], "add_component");
    assert_eq!(applied["reason"], "symbol_missing_on_board");
    assert_eq!(applied["component_report"]["reference"], "C1");
    assert_eq!(applied["component_report"]["part_uuid"], part_uuid.to_string());
    assert_eq!(applied["component_report"]["package_uuid"], package_uuid.to_string());
    assert_eq!(applied["component_report"]["x_nm"], 2500000);
    assert_eq!(applied["component_report"]["y_nm"], 3500000);

    let components_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "query", root.to_str().unwrap(), "board-components",
    ])
    .expect("CLI should parse");
    let components_output = execute(components_cli).expect("components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("components parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].reference, "C1");
    assert_eq!(components[0].value, "1u");
    assert_eq!(components[0].part, part_uuid);
    assert_eq!(components[0].package, package_uuid);
    assert_eq!(components[0].position, Point::new(2500000, 3500000));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_apply_forward_annotation_action_rejects_unresolved_add_without_part_override() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-add-unresolved");
    create_native_project(&root, Some("Forward Annotation Apply Demo".to_string()))
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
                entity: Some(Uuid::new_v4()),
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

    let proposal_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "query", root.to_str().unwrap(),
        "forward-annotation-proposal",
    ])
    .expect("CLI should parse");
    let proposal_output = execute(proposal_cli).expect("proposal should succeed");
    let proposal: serde_json::Value = serde_json::from_str(&proposal_output).expect("proposal JSON");
    let action_id = proposal["actions"][0]["action_id"].as_str().unwrap().to_string();

    let apply_cli = Cli::try_parse_from([
        "eda", "project", "apply-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &action_id,
        "--package", &Uuid::new_v4().to_string(),
        "--x-nm", "2500000",
        "--y-nm", "3500000",
        "--layer", "1",
    ])
    .expect("CLI should parse");
    let err = execute(apply_cli).expect_err("unresolved add_component should fail without part");
    let msg = format!("{err:#}");
    assert!(msg.contains("requires --part <uuid>"), "{msg}");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_apply_forward_annotation_action_applies_part_mismatch_with_explicit_package() {
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-apply-part-mismatch");
    create_native_project(&root, Some("Forward Annotation Apply Demo".to_string()))
        .expect("initial scaffold should succeed");

    let sheet_uuid = Uuid::new_v4();
    let symbol_uuid = Uuid::new_v4();
    let schematic_part_uuid = Uuid::new_v4();
    write_native_sheet(
        &root,
        sheet_uuid,
        "Main",
        BTreeMap::from([(
            symbol_uuid.to_string(),
            serde_json::to_value(PlacedSymbol {
                uuid: symbol_uuid,
                part: Some(schematic_part_uuid),
                entity: None,
                gate: None,
                lib_id: Some("Device:U".into()),
                reference: "U1".into(),
                value: "MCU".into(),
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

    let component_uuid = Uuid::new_v4();
    let board_part_uuid = Uuid::new_v4();
    let board_package_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Forward Annotation Apply Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    component_uuid.to_string(): serde_json::to_value(PlacedPackage {
                        uuid: component_uuid,
                        part: board_part_uuid,
                        package: board_package_uuid,
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
    let action_id = proposal["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "update_component" && entry["reason"] == "part_mismatch")
        .unwrap()["action_id"]
        .as_str()
        .unwrap()
        .to_string();
    let replacement_package_uuid = Uuid::new_v4();

    let apply_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "apply-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &action_id,
        "--package", &replacement_package_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let apply_output = execute(apply_cli).expect("part_mismatch apply should succeed");
    let applied: serde_json::Value = serde_json::from_str(&apply_output).expect("apply JSON");
    assert_eq!(applied["proposal_action"], "update_component");
    assert_eq!(applied["reason"], "part_mismatch");
    assert_eq!(applied["component_report"]["component_uuid"], component_uuid.to_string());
    assert_eq!(applied["component_report"]["part_uuid"], schematic_part_uuid.to_string());
    assert_eq!(applied["component_report"]["package_uuid"], replacement_package_uuid.to_string());

    let components_cli = Cli::try_parse_from([
        "eda", "--format", "json", "project", "query", root.to_str().unwrap(), "board-components",
    ])
    .expect("CLI should parse");
    let components_output = execute(components_cli).expect("components query should succeed");
    let components: Vec<PlacedPackage> =
        serde_json::from_str(&components_output).expect("components parse");
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].part, schematic_part_uuid);
    assert_eq!(components[0].package, replacement_package_uuid);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_apply_forward_annotation_action_rejects_part_mismatch_without_package() {
    let root =
        unique_project_root("datum-eda-cli-project-forward-annotation-apply-part-mismatch-missing");
    create_native_project(&root, Some("Forward Annotation Apply Demo".to_string()))
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
                part: Some(Uuid::new_v4()),
                entity: None,
                gate: None,
                lib_id: Some("Device:U".into()),
                reference: "U1".into(),
                value: "MCU".into(),
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

    let component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Forward Annotation Apply Demo Board",
                "stackup": { "layers": [] },
                "outline": { "vertices": [], "closed": true },
                "packages": {
                    component_uuid.to_string(): serde_json::to_value(PlacedPackage {
                        uuid: component_uuid,
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
    let action_id = proposal["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["action"] == "update_component" && entry["reason"] == "part_mismatch")
        .unwrap()["action_id"]
        .as_str()
        .unwrap()
        .to_string();

    let apply_cli = Cli::try_parse_from([
        "eda", "project", "apply-forward-annotation-action",
        root.to_str().unwrap(), "--action-id", &action_id,
    ])
    .expect("CLI should parse");
    let err = execute(apply_cli).expect_err("part_mismatch apply should require package");
    let msg = format!("{err:#}");
    assert!(msg.contains("requires --package <uuid>"), "{msg}");

    let _ = std::fs::remove_dir_all(&root);
}
