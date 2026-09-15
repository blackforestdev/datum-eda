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
fn project_set_and_clear_symbol_entity_updates_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-symbol-entity");
    create_native_project(&root, Some("Symbol Entity Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-symbol",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--reference",
        "U21",
        "--value",
        "BUF",
        "--x-nm",
        "11",
        "--y-nm",
        "22",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();
    let entity_uuid = Uuid::new_v4().to_string();

    let set_entity_cli = Cli::try_parse_from([
        "eda",
        "project",
        "set-symbol-entity",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--entity",
        &entity_uuid,
    ])
    .expect("CLI should parse");
    let set_entity_output =
        execute(set_entity_cli).expect("project set-symbol-entity should succeed");
    assert!(set_entity_output.contains("action: set_symbol_entity"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbols",
    ])
    .expect("CLI should parse");
    let symbols_output = execute(query_cli).expect("project query symbols should succeed");
    let symbols: serde_json::Value =
        serde_json::from_str(&symbols_output).expect("symbols JSON should parse");
    assert_eq!(symbols.as_array().unwrap().len(), 1);
    assert_eq!(symbols[0]["uuid"], symbol_uuid);
    assert_eq!(symbols[0]["entity_uuid"], entity_uuid);
    assert!(symbols[0]["part_uuid"].is_null());

    let clear_entity_cli = Cli::try_parse_from([
        "eda",
        "project",
        "clear-symbol-entity",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let clear_entity_output =
        execute(clear_entity_cli).expect("project clear-symbol-entity should succeed");
    assert!(clear_entity_output.contains("action: clear_symbol_entity"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbols",
    ])
    .expect("CLI should parse");
    let symbols_output = execute(query_cli).expect("project query symbols should succeed");
    let symbols: serde_json::Value =
        serde_json::from_str(&symbols_output).expect("symbols JSON should parse");
    assert!(symbols[0]["entity_uuid"].is_null());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_and_clear_symbol_part_updates_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-symbol-part");
    create_native_project(&root, Some("Symbol Part Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);

    let place_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "place-symbol",
        root.to_str().unwrap(),
        "--sheet",
        &sheet_uuid.to_string(),
        "--reference",
        "U22",
        "--value",
        "AMP",
        "--x-nm",
        "33",
        "--y-nm",
        "44",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();
    let entity_uuid = Uuid::new_v4().to_string();
    let part_uuid = Uuid::new_v4().to_string();

    let set_entity_cli = Cli::try_parse_from([
        "eda",
        "project",
        "set-symbol-entity",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--entity",
        &entity_uuid,
    ])
    .expect("CLI should parse");
    execute(set_entity_cli).expect("project set-symbol-entity should succeed");

    let set_part_cli = Cli::try_parse_from([
        "eda",
        "project",
        "set-symbol-part",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--part",
        &part_uuid,
    ])
    .expect("CLI should parse");
    let set_part_output = execute(set_part_cli).expect("project set-symbol-part should succeed");
    assert!(set_part_output.contains("action: set_symbol_part"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbols",
    ])
    .expect("CLI should parse");
    let symbols_output = execute(query_cli).expect("project query symbols should succeed");
    let symbols: serde_json::Value =
        serde_json::from_str(&symbols_output).expect("symbols JSON should parse");
    assert_eq!(symbols.as_array().unwrap().len(), 1);
    assert_eq!(symbols[0]["uuid"], symbol_uuid);
    assert_eq!(symbols[0]["part_uuid"], part_uuid);
    assert!(symbols[0]["entity_uuid"].is_null());

    let clear_part_cli = Cli::try_parse_from([
        "eda",
        "project",
        "clear-symbol-part",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let clear_part_output =
        execute(clear_part_cli).expect("project clear-symbol-part should succeed");
    assert!(clear_part_output.contains("action: clear_symbol_part"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbols",
    ])
    .expect("CLI should parse");
    let symbols_output = execute(query_cli).expect("project query symbols should succeed");
    let symbols: serde_json::Value =
        serde_json::from_str(&symbols_output).expect("symbols JSON should parse");
    assert!(symbols[0]["part_uuid"].is_null());
    assert!(symbols[0]["entity_uuid"].is_null());

    let _ = std::fs::remove_dir_all(&root);
}
