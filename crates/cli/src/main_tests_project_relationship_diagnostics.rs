use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_query_relationships_reports_component_instance_handoff_gaps() {
    let root = unique_project_root("datum-eda-cli-project-relationship-diagnostics");
    create_native_project(&root, Some("Relationship Diagnostics Demo".to_string()))
        .expect("initial scaffold should succeed");
    let project: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("project.json")).unwrap()).unwrap();
    let project_id = Uuid::parse_str(project["uuid"].as_str().unwrap()).unwrap();
    let mut schematic: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("schematic/schematic.json")).unwrap())
            .unwrap();
    let sheet_id = Uuid::new_v4();
    let sheet_path = format!("sheets/{sheet_id}.json");
    schematic["sheets"][sheet_id.to_string()] = serde_json::Value::String(sheet_path.clone());
    std::fs::write(
        root.join("schematic/schematic.json"),
        format!("{}\n", to_json_deterministic(&schematic).unwrap()),
    )
    .unwrap();
    let symbol_id = Uuid::new_v4();
    let symbol_part_id = Uuid::new_v4();
    let sheet_file = root.join("schematic").join(&sheet_path);
    std::fs::create_dir_all(sheet_file.parent().unwrap()).unwrap();
    std::fs::write(
        &sheet_file,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": sheet_id,
                "name": "Main",
                "symbols": {
                    symbol_id.to_string(): {
                        "uuid": symbol_id,
                        "part": symbol_part_id,
                        "entity": Uuid::new_v5(&project_id, b"entity"),
                        "gate": Uuid::new_v5(&project_id, b"gate"),
                        "lib_id": "test:R",
                        "reference": "R1",
                        "value": "10k",
                        "fields": [],
                        "pins": [],
                        "position": { "x": 0, "y": 0 },
                        "rotation": 0,
                        "mirrored": false,
                        "unit_selection": null,
                        "display_mode": "LibraryDefault",
                        "pin_overrides": [],
                        "hidden_power_behavior": "SourceDefinedImplicit"
                    }
                },
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
            .unwrap()
        ),
    )
    .unwrap();

    let board_path = root.join("board/board.json");
    let mut board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&board_path).unwrap()).unwrap();
    let board_package_id = Uuid::new_v4();
    board["packages"] = serde_json::json!({
        board_package_id.to_string(): {
            "uuid": board_package_id,
            "part": Uuid::new_v4(),
            "package": Uuid::new_v4(),
            "reference": "J99",
            "value": "Board only",
            "position": { "x": 0, "y": 0 },
            "rotation": 0,
            "layer": 1,
            "locked": false
        }
    });
    std::fs::write(
        &board_path,
        format!("{}\n", to_json_deterministic(&board).unwrap()),
    )
    .unwrap();

    let component_instance_id = Uuid::new_v4();
    let component_instance_path = root.join(format!(
        ".datum/component_instances/{component_instance_id}.json"
    ));
    std::fs::create_dir_all(component_instance_path.parent().unwrap()).unwrap();
    std::fs::write(
        &component_instance_path,
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "component_instance": {
                    "uuid": component_instance_id,
                    "object_revision": 0,
                    "placed_symbol_refs": [{
                        "object_id": symbol_id,
                        "object_revision": 0
                    }],
                    "placed_package_refs": []
                }
            }))
            .unwrap()
        ),
    )
    .unwrap();
    let component_instance_before = std::fs::read(&component_instance_path).unwrap();

    let relationships_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "relationships",
        ])
        .expect("CLI should parse"),
    )
    .expect("relationships query should succeed");
    let relationships: serde_json::Value = serde_json::from_str(&relationships_output).unwrap();
    let diagnostics = &relationships["component_instance_diagnostics"];
    assert_eq!(relationships["contract"], "relationships_query_v1");
    assert_eq!(diagnostics["schematic_symbol_count"], 1);
    assert_eq!(diagnostics["authored_component_instance_count"], 1);
    assert_eq!(diagnostics["board_package_count"], 1);
    assert_eq!(diagnostics["unplaced_component_instance_count"], 1);
    assert_eq!(
        diagnostics["unplaced_component_instances"][0],
        component_instance_id.to_string()
    );
    assert_eq!(diagnostics["unmatched_package_count"], 1);
    assert_eq!(
        diagnostics["unmatched_packages"][0]["code"],
        "component_instance_unmatched_package"
    );
    assert_eq!(diagnostics["unmatched_symbol_count"], 0);
    assert_eq!(diagnostics["stale_or_missing_ref_count"], 0);
    assert_eq!(diagnostics["ambiguous_join_count"], 0);
    assert_eq!(
        std::fs::read(&component_instance_path).unwrap(),
        component_instance_before
    );
}
