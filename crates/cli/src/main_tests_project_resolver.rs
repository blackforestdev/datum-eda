use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

#[test]
fn project_query_resolve_debug_reports_component_instance_join() {
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
    assert_eq!(report["component_instance_count"], 1);

    let _ = std::fs::remove_dir_all(&root);
}
