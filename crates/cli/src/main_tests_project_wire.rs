use super::*;
use eda_engine::ir::serialization::to_json_deterministic;

fn unique_project_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-{}", label, Uuid::new_v4()))
}

fn seed_native_sheet(root: &Path) -> Uuid {
    let sheet_uuid = Uuid::new_v4();
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
                "name": "Main",
                "frame": null,
                "symbols": {},
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

    sheet_uuid
}

#[test]
fn project_draw_and_delete_wire_update_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-wire");
    create_native_project(&root, Some("Wire Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);

    let draw_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "draw-wire",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--from-x-nm",
        "10",
        "--from-y-nm",
        "20",
        "--to-x-nm",
        "30",
        "--to-y-nm",
        "40",
    ])
    .expect("CLI should parse");
    let draw_output = execute(draw_cli).expect("project draw-wire should succeed");
    let drawn: serde_json::Value =
        serde_json::from_str(&draw_output).expect("draw-wire JSON should parse");
    let wire_uuid = drawn["wire_uuid"].as_str().unwrap().to_string();

    let wires_query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "wires",
    ])
    .expect("CLI should parse");
    let wires_output = execute(wires_query_cli).expect("project query wires should succeed");
    let wires: serde_json::Value =
        serde_json::from_str(&wires_output).expect("wires JSON should parse");
    assert_eq!(wires.as_array().unwrap().len(), 1);
    assert_eq!(wires[0]["uuid"], wire_uuid);
    assert_eq!(wires[0]["from"]["x"], 10);
    assert_eq!(wires[0]["from"]["y"], 20);
    assert_eq!(wires[0]["to"]["x"], 30);
    assert_eq!(wires[0]["to"]["y"], 40);

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_wires: 1"));

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-wire",
        root.to_str().unwrap(),
        "--wire",
        &wire_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("project delete-wire should succeed");
    assert!(delete_output.contains("action: delete_wire"));

    let wires_query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "wires",
    ])
    .expect("CLI should parse");
    let deleted_wires_output =
        execute(wires_query_cli).expect("project query wires should succeed");
    let deleted_wires: serde_json::Value =
        serde_json::from_str(&deleted_wires_output).expect("wires JSON should parse");
    assert_eq!(deleted_wires.as_array().unwrap().len(), 0);

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_wires: 0"));

    let _ = std::fs::remove_dir_all(&root);
}
