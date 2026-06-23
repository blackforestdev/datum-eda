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
    std::fs::write(&sheet_path, format!("{}\n", to_json_deterministic(&serde_json::json!({"schema_version": 1, "uuid": sheet_uuid, "name": "Main", "frame": null, "symbols": {}, "wires": {}, "junctions": {}, "labels": {}, "buses": {}, "bus_entries": {}, "ports": {}, "noconnects": {}, "texts": {}, "drawings": {}})).expect("sheet JSON should serialize"))).expect("sheet file should write");
    let schematic_json = root.join("schematic/schematic.json");
    let mut schematic_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&schematic_json).expect("schematic.json should read"),
    )
    .expect("schematic.json should parse");
    schematic_value["sheets"] =
        serde_json::json!({ sheet_uuid.to_string(): format!("sheets/{sheet_uuid}.json") });
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
fn project_place_symbol_materializes_pins_from_pool_symbol_uuid_lib_id() {
    let root = unique_project_root("datum-eda-cli-project-symbol-library-materialization");
    create_native_project(&root, Some("Symbol Library Materialization".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let unit_id = Uuid::new_v4();
    let pin_id = Uuid::new_v4();
    let symbol_id = Uuid::new_v4();
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-unit",
            root.to_str().unwrap(),
            "--unit",
            &unit_id.to_string(),
            "--name",
            "OpAmpUnit",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool unit create should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-unit-pin",
            root.to_str().unwrap(),
            "--unit",
            &unit_id.to_string(),
            "--pin",
            &pin_id.to_string(),
            "--name",
            "OUT",
            "--direction",
            "Output",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool unit pin set should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "create-pool-symbol",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--unit",
            &unit_id.to_string(),
            "--name",
            "OpAmpSymbol",
        ])
        .expect("CLI should parse"),
    )
    .expect("typed pool symbol create should succeed");
    execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "set-pool-symbol-pin-anchor",
            root.to_str().unwrap(),
            "--symbol",
            &symbol_id.to_string(),
            "--pin",
            &pin_id.to_string(),
            "--x-nm",
            "100",
            "--y-nm",
            "200",
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol pin anchor set should succeed");
    let place_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "place-symbol",
            root.to_str().unwrap(),
            "--sheet",
            &sheet_uuid.to_string(),
            "--reference",
            "U1",
            "--value",
            "AMP",
            "--lib-id",
            &symbol_id.to_string(),
            "--x-nm",
            "1000",
            "--y-nm",
            "2000",
        ])
        .expect("CLI should parse"),
    )
    .expect("place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let placed_symbol =
        Uuid::parse_str(placed["symbol_uuid"].as_str().unwrap()).expect("symbol uuid should parse");
    let pins_output = execute(
        Cli::try_parse_from([
            "eda",
            "--format",
            "json",
            "project",
            "query",
            root.to_str().unwrap(),
            "symbol-pins",
            "--symbol",
            &placed_symbol.to_string(),
        ])
        .expect("CLI should parse"),
    )
    .expect("symbol-pins query should succeed");
    let pins: serde_json::Value =
        serde_json::from_str(&pins_output).expect("symbol pins JSON should parse");
    assert_eq!(pins.as_array().expect("pins should be array").len(), 1);
    assert_eq!(pins[0]["pin_uuid"], pin_id.to_string());
    assert_eq!(pins[0]["number"], "OUT");
    assert_eq!(pins[0]["electrical_type"], "Output");
    assert_eq!(pins[0]["x_nm"], 100);
    assert_eq!(pins[0]["y_nm"], 200);
    let _ = std::fs::remove_dir_all(&root);
}
