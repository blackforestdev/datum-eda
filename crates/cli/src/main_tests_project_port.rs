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
            to_json_deterministic(&schematic_value)
                .expect("canonical serialization should succeed")
        ),
    )
    .expect("schematic.json should write");

    sheet_uuid
}

#[test]
fn project_place_edit_and_delete_port_update_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-port");
    create_native_project(&root, Some("Port Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-port",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--name",
        "SUB_IN",
        "--direction",
        "input",
        "--x-nm",
        "11",
        "--y-nm",
        "22",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-port should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-port JSON should parse");
    let port_uuid = placed["port_uuid"].as_str().unwrap().to_string();

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "ports",
    ])
    .expect("CLI should parse");
    let ports_output = execute(query_cli).expect("project query ports should succeed");
    let ports: serde_json::Value =
        serde_json::from_str(&ports_output).expect("ports JSON should parse");
    assert_eq!(ports.as_array().unwrap().len(), 1);
    assert_eq!(ports[0]["uuid"], port_uuid);
    assert_eq!(ports[0]["name"], "SUB_IN");
    assert_eq!(ports[0]["direction"], "Input");

    let edit_cli = Cli::try_parse_from([
        "eda",
        "project",
        "edit-port",
        root.to_str().unwrap(),
        "--port",
        &port_uuid,
        "--name",
        "SUB_IO",
        "--direction",
        "bidirectional",
        "--x-nm",
        "33",
        "--y-nm",
        "44",
    ])
    .expect("CLI should parse");
    let edit_output = execute(edit_cli).expect("project edit-port should succeed");
    assert!(edit_output.contains("action: edit_port"));
    assert!(edit_output.contains("name: SUB_IO"));
    assert!(edit_output.contains("direction: bidirectional"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "ports",
    ])
    .expect("CLI should parse");
    let edited_output = execute(query_cli).expect("project query ports should succeed");
    let edited: serde_json::Value =
        serde_json::from_str(&edited_output).expect("ports JSON should parse");
    assert_eq!(edited.as_array().unwrap().len(), 1);
    assert_eq!(edited[0]["name"], "SUB_IO");
    assert_eq!(edited[0]["direction"], "Bidirectional");
    assert_eq!(edited[0]["position"]["x"], 33);
    assert_eq!(edited[0]["position"]["y"], 44);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_ports: 1"));

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-port",
        root.to_str().unwrap(),
        "--port",
        &port_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("project delete-port should succeed");
    assert!(delete_output.contains("action: delete_port"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "ports",
    ])
    .expect("CLI should parse");
    let deleted_output = execute(query_cli).expect("project query ports should succeed");
    let deleted: serde_json::Value =
        serde_json::from_str(&deleted_output).expect("ports JSON should parse");
    assert_eq!(deleted.as_array().unwrap().len(), 0);

    let summary_cli =
        Cli::try_parse_from(["eda", "project", "query", root.to_str().unwrap(), "summary"])
            .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_ports: 0"));

    let _ = std::fs::remove_dir_all(&root);
}
