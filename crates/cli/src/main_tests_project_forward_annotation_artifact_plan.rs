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
fn project_plan_forward_annotation_artifact_apply_reports_self_sufficient_and_input_bound_actions()
{
    let root = unique_project_root("datum-eda-cli-project-forward-annotation-plan-artifact");
    create_native_project(&root, Some("Forward Annotation Plan Demo".to_string()))
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

    let r1_component_uuid = Uuid::new_v4();
    let q1_board_part_uuid = Uuid::new_v4();
    let q1_board_package_uuid = Uuid::new_v4();
    let orphan_component_uuid = Uuid::new_v4();
    let board_json = root.join("board/board.json");
    std::fs::write(
        &board_json,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v4(),
                "name": "Forward Annotation Plan Demo Board",
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
                    q1_board_package_uuid.to_string(): serde_json::to_value(PlacedPackage {
                        uuid: q1_board_package_uuid,
                        part: q1_board_part_uuid,
                        package: Uuid::new_v4(),
                        reference: "Q1".into(),
                        value: "BJT".into(),
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
        "eda",
        "project",
        "defer-forward-annotation-action",
        root.to_str().unwrap(),
        "--action-id",
        &add_action_id,
    ])
    .expect("CLI should parse");
    let _ = execute(defer_cli).expect("defer should succeed");

    let artifact_path = root.join("forward-annotation-proposal.json");
    let export_cli = Cli::try_parse_from([
        "eda",
        "project",
        "export-forward-annotation-proposal",
        root.to_str().unwrap(),
        "--out",
        artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let _ = execute(export_cli).expect("export should succeed");

    let plan_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "plan-forward-annotation-proposal-artifact-apply",
        root.to_str().unwrap(),
        "--artifact",
        artifact_path.to_str().unwrap(),
    ])
    .expect("CLI should parse");
    let plan_output = execute(plan_cli).expect("plan should succeed");
    let plan: serde_json::Value = serde_json::from_str(&plan_output).expect("plan JSON");
    assert_eq!(
        plan["action"],
        "plan_forward_annotation_proposal_artifact_apply"
    );
    assert_eq!(plan["artifact_actions"], 4);
    assert_eq!(plan["self_sufficient_actions"], 2);
    assert_eq!(plan["requires_input_actions"], 2);
    assert_eq!(plan["not_applicable_actions"], 0);

    let actions = plan["actions"].as_array().unwrap();
    assert!(actions.iter().any(|entry| {
        entry["proposal_action"] == "remove_component" && entry["execution"] == "self_sufficient"
    }));
    assert!(actions.iter().any(|entry| {
        entry["proposal_action"] == "update_component"
            && entry["reason"] == "value_mismatch"
            && entry["execution"] == "self_sufficient"
    }));
    assert!(actions.iter().any(|entry| {
        entry["proposal_action"] == "add_component"
            && entry["execution"] == "requires_explicit_input"
            && entry["review_decision"] == "deferred"
            && entry["required_inputs"].as_array().unwrap().len() == 4
    }));
    assert!(actions.iter().any(|entry| {
        entry["proposal_action"] == "update_component"
            && entry["reason"] == "part_mismatch"
            && entry["execution"] == "requires_explicit_input"
            && entry["required_inputs"] == serde_json::json!(["package_uuid"])
    }));

    let _ = std::fs::remove_dir_all(&root);
}
