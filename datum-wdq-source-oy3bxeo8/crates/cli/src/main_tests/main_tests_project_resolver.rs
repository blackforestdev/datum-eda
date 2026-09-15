use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_query_resolve_debug_omits_compatibility_derived_component_instance_join() {
    let root = unique_project_root("datum-eda-cli-project-resolve-debug-component-instance");
    create_native_project(&root, Some("Component Instance Demo".to_string()))
        .expect("initial scaffold should succeed");
    let project_id = Uuid::new_v4();
    let sheet_id = Uuid::new_v5(&project_id, b"sheet");
    let symbol_id = Uuid::new_v4();
    let package_id = Uuid::new_v4();
    let part_id = Uuid::new_v4();
    std::fs::write(
        root.join("schematic/schematic.json"),
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": Uuid::new_v5(&project_id, b"schematic"),
                "sheets": { sheet_id.to_string(): "sheets/main.json" },
                "definitions": {},
                "instances": [],
                "variants": {},
                "waivers": []
            }))
            .expect("schematic should serialize")
        ),
    )
    .expect("schematic should write");
    std::fs::write(
        root.join("schematic/sheets/main.json"),
        format!(
            "{}\n",
            to_json_deterministic(&serde_json::json!({
                "schema_version": 1,
                "uuid": sheet_id,
                "name": "Main",
                "symbols": {
                    symbol_id.to_string(): {
                        "uuid": symbol_id,
                        "part": part_id,
                        "entity": null,
                        "gate": null,
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
            .expect("sheet should serialize")
        ),
    )
    .expect("sheet should write");
    let board_path = root.join("board/board.json");
    let mut board: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&board_path).expect("board should read"))
            .expect("board should parse");
    board["packages"] = serde_json::json!({
        package_id.to_string(): {
            "uuid": package_id,
            "part": part_id,
            "package": Uuid::new_v4(),
            "reference": "R1",
            "value": "10k",
            "position": { "x": 0, "y": 0 },
            "rotation": 0,
            "layer": 1,
            "locked": false
        }
    });
    std::fs::write(
        &board_path,
        format!(
            "{}\n",
            to_json_deterministic(&board).expect("board should serialize")
        ),
    )
    .expect("board should write");

    let output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "resolve-debug",
        ])
        .expect("CLI should parse"),
    )
    .expect("project query resolve-debug should succeed");
    let report: serde_json::Value =
        serde_json::from_str(&output).expect("resolve-debug JSON should parse");
    assert_eq!(report["component_instance_count"], 0);

    let component_instances_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "component-instances",
        ])
        .expect("CLI should parse"),
    )
    .expect("component-instances query should succeed");
    let component_instances: serde_json::Value =
        serde_json::from_str(&component_instances_output).expect("component-instances JSON");
    assert_eq!(component_instances["component_instance_count"], 0);
    assert_eq!(
        component_instances["component_instances"]
            .as_object()
            .expect("component_instances should be an object")
            .len(),
        0
    );

    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("project.json")).expect("project read"))
            .expect("project parse");
    let manifest_id = Uuid::parse_str(manifest["uuid"].as_str().expect("project uuid"))
        .expect("project uuid parses");
    let derived_component_instance_id = Uuid::new_v5(
        &manifest_id,
        format!("datum-eda:component-instance:{symbol_id}:{package_id}").as_bytes(),
    );
    let set_error = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-component-instance",
            root.to_str().unwrap(),
            "--component-instance",
            &derived_component_instance_id.to_string(),
            "--symbol",
            &symbol_id.to_string(),
            "--package",
            &package_id.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect_err("legacy derived component instance id must not be mutable");
    assert!(format!("{set_error:#}").contains("was not found"));

    let _ = std::fs::remove_dir_all(&root);
}
