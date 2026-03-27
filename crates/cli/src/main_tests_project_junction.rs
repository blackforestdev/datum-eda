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
fn project_place_and_delete_junction_update_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-junction");
    create_native_project(&root, Some("Junction Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-junction",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--x-nm",
        "50",
        "--y-nm",
        "60",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-junction should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-junction JSON should parse");
    let junction_uuid = placed["junction_uuid"].as_str().unwrap().to_string();

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "junctions",
    ])
    .expect("CLI should parse");
    let junctions_output = execute(query_cli).expect("project query junctions should succeed");
    let junctions: serde_json::Value =
        serde_json::from_str(&junctions_output).expect("junctions JSON should parse");
    assert_eq!(junctions.as_array().unwrap().len(), 1);
    assert_eq!(junctions[0]["uuid"], junction_uuid);
    assert_eq!(junctions[0]["position"]["x"], 50);
    assert_eq!(junctions[0]["position"]["y"], 60);

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_junctions: 1"));

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-junction",
        root.to_str().unwrap(),
        "--junction",
        &junction_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("project delete-junction should succeed");
    assert!(delete_output.contains("action: delete_junction"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "junctions",
    ])
    .expect("CLI should parse");
    let deleted_output = execute(query_cli).expect("project query junctions should succeed");
    let deleted: serde_json::Value =
        serde_json::from_str(&deleted_output).expect("junctions JSON should parse");
    assert_eq!(deleted.as_array().unwrap().len(), 0);

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_junctions: 0"));

    let _ = std::fs::remove_dir_all(&root);
}
