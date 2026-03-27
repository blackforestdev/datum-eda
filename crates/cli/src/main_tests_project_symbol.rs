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
fn project_place_symbol_updates_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-symbol");
    create_native_project(&root, Some("Symbol Demo".to_string()))
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
        "U1",
        "--value",
        "LM358",
        "--lib-id",
        "device:opamp",
        "--x-nm",
        "100",
        "--y-nm",
        "200",
        "--rotation-deg",
        "90",
        "--mirrored",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();
    assert_eq!(placed["reference"], "U1");
    assert_eq!(placed["value"], "LM358");
    assert_eq!(placed["lib_id"], "device:opamp");
    assert_eq!(placed["rotation_deg"], 90);
    assert_eq!(placed["mirrored"], true);

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
    assert_eq!(symbols[0]["sheet"], sheet_uuid.to_string());
    assert_eq!(symbols[0]["reference"], "U1");
    assert_eq!(symbols[0]["value"], "LM358");
    assert_eq!(symbols[0]["position"]["x"], 100);
    assert_eq!(symbols[0]["position"]["y"], 200);
    assert_eq!(symbols[0]["rotation"], 90);
    assert_eq!(symbols[0]["mirrored"], true);
    assert!(symbols[0]["part_uuid"].is_null());
    assert!(symbols[0]["entity_uuid"].is_null());
    assert!(symbols[0]["gate_uuid"].is_null());

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_symbols: 1"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_move_rotate_and_delete_symbol_update_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-symbol-transform");
    create_native_project(&root, Some("Symbol Transform Demo".to_string()))
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
        "R1",
        "--value",
        "10k",
        "--x-nm",
        "10",
        "--y-nm",
        "20",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();

    let move_cli = Cli::try_parse_from([
        "eda",
        "project",
        "move-symbol",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--x-nm",
        "300",
        "--y-nm",
        "400",
    ])
    .expect("CLI should parse");
    let move_output = execute(move_cli).expect("project move-symbol should succeed");
    assert!(move_output.contains("action: move_symbol"));
    assert!(move_output.contains("x_nm: 300"));
    assert!(move_output.contains("y_nm: 400"));

    let rotate_cli = Cli::try_parse_from([
        "eda",
        "project",
        "rotate-symbol",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--rotation-deg",
        "180",
    ])
    .expect("CLI should parse");
    let rotate_output = execute(rotate_cli).expect("project rotate-symbol should succeed");
    assert!(rotate_output.contains("action: rotate_symbol"));
    assert!(rotate_output.contains("rotation_deg: 180"));

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
    assert_eq!(symbols[0]["position"]["x"], 300);
    assert_eq!(symbols[0]["position"]["y"], 400);
    assert_eq!(symbols[0]["rotation"], 180);
    assert_eq!(symbols[0]["reference"], "R1");
    assert_eq!(symbols[0]["value"], "10k");

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-symbol",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("project delete-symbol should succeed");
    assert!(delete_output.contains("action: delete_symbol"));
    assert!(delete_output.contains("reference: R1"));

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
    assert_eq!(symbols.as_array().unwrap().len(), 0);

    let summary_cli = Cli::try_parse_from([
        "eda",
        "project",
        "query",
        root.to_str().unwrap(),
        "summary",
    ])
    .expect("CLI should parse");
    let summary_output = execute(summary_cli).expect("project query summary should succeed");
    assert!(summary_output.contains("schematic_symbols: 0"));

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_symbol_reference_and_value_update_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-symbol-annotation");
    create_native_project(&root, Some("Symbol Annotation Demo".to_string()))
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
        "U3",
        "--value",
        "OPA2134",
        "--x-nm",
        "15",
        "--y-nm",
        "25",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();

    let set_reference_cli = Cli::try_parse_from([
        "eda",
        "project",
        "set-symbol-reference",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--reference",
        "U7",
    ])
    .expect("CLI should parse");
    let set_reference_output =
        execute(set_reference_cli).expect("project set-symbol-reference should succeed");
    assert!(set_reference_output.contains("action: set_symbol_reference"));
    assert!(set_reference_output.contains("reference: U7"));

    let set_value_cli = Cli::try_parse_from([
        "eda",
        "project",
        "set-symbol-value",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--value",
        "OPA1642",
    ])
    .expect("CLI should parse");
    let set_value_output =
        execute(set_value_cli).expect("project set-symbol-value should succeed");
    assert!(set_value_output.contains("action: set_symbol_value"));
    assert!(set_value_output.contains("value: OPA1642"));

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
    assert_eq!(symbols[0]["reference"], "U7");
    assert_eq!(symbols[0]["value"], "OPA1642");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_and_clear_symbol_lib_id_update_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-symbol-lib-id");
    create_native_project(&root, Some("Symbol LibId Demo".to_string()))
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
        "U4",
        "--value",
        "LM2904",
        "--x-nm",
        "15",
        "--y-nm",
        "25",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();
    assert!(placed["lib_id"].is_null());

    let set_lib_id_cli = Cli::try_parse_from([
        "eda",
        "project",
        "set-symbol-lib-id",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--lib-id",
        "device:dual_opamp",
    ])
    .expect("CLI should parse");
    let set_lib_id_output =
        execute(set_lib_id_cli).expect("project set-symbol-lib-id should succeed");
    assert!(set_lib_id_output.contains("action: set_symbol_lib_id"));
    assert!(set_lib_id_output.contains("lib_id: device:dual_opamp"));

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
    assert_eq!(symbols[0]["lib_id"], "device:dual_opamp");

    let clear_lib_id_cli = Cli::try_parse_from([
        "eda",
        "project",
        "clear-symbol-lib-id",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let clear_lib_id_output =
        execute(clear_lib_id_cli).expect("project clear-symbol-lib-id should succeed");
    assert!(clear_lib_id_output.contains("action: clear_symbol_lib_id"));

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
    assert!(symbols[0]["lib_id"].is_null());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_mirror_symbol_updates_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-symbol-mirror");
    create_native_project(&root, Some("Symbol Mirror Demo".to_string()))
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
        "U9",
        "--value",
        "TL072",
        "--x-nm",
        "12",
        "--y-nm",
        "34",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();
    assert_eq!(placed["mirrored"], false);

    let mirror_cli = Cli::try_parse_from([
        "eda",
        "project",
        "mirror-symbol",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let mirror_output = execute(mirror_cli).expect("project mirror-symbol should succeed");
    assert!(mirror_output.contains("action: mirror_symbol"));
    assert!(mirror_output.contains("mirrored: true"));

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
    assert_eq!(symbols[0]["mirrored"], true);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_add_edit_and_delete_symbol_field_updates_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-symbol-field");
    create_native_project(&root, Some("Symbol Field Demo".to_string()))
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
        "U2",
        "--value",
        "NE5532",
        "--x-nm",
        "50",
        "--y-nm",
        "60",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();

    let add_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "add-symbol-field",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--key",
        "Footprint",
        "--value",
        "Package_SO:SOIC-8",
        "--hidden",
        "--x-nm",
        "500",
        "--y-nm",
        "600",
    ])
    .expect("CLI should parse");
    let add_output = execute(add_cli).expect("project add-symbol-field should succeed");
    let added: serde_json::Value =
        serde_json::from_str(&add_output).expect("add-symbol-field JSON should parse");
    let field_uuid = added["field_uuid"].as_str().unwrap().to_string();
    assert_eq!(added["key"], "Footprint");
    assert_eq!(added["value"], "Package_SO:SOIC-8");
    assert_eq!(added["visible"], false);
    assert_eq!(added["x_nm"], 500);
    assert_eq!(added["y_nm"], 600);

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbol-fields",
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let fields_output = execute(query_cli).expect("project query symbol-fields should succeed");
    let fields: serde_json::Value =
        serde_json::from_str(&fields_output).expect("symbol-fields JSON should parse");
    assert_eq!(fields.as_array().unwrap().len(), 1);
    assert_eq!(fields[0]["uuid"], field_uuid);
    assert_eq!(fields[0]["symbol"], symbol_uuid);
    assert_eq!(fields[0]["key"], "Footprint");
    assert_eq!(fields[0]["value"], "Package_SO:SOIC-8");
    assert_eq!(fields[0]["visible"], false);
    assert_eq!(fields[0]["position"]["x"], 500);
    assert_eq!(fields[0]["position"]["y"], 600);

    let edit_cli = Cli::try_parse_from([
        "eda",
        "project",
        "edit-symbol-field",
        root.to_str().unwrap(),
        "--field",
        &field_uuid,
        "--value",
        "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm",
        "--visible",
        "true",
    ])
    .expect("CLI should parse");
    let edit_output = execute(edit_cli).expect("project edit-symbol-field should succeed");
    assert!(edit_output.contains("action: edit_symbol_field"));
    assert!(edit_output.contains("visible: true"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbol-fields",
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let fields_output = execute(query_cli).expect("project query symbol-fields should succeed");
    let fields: serde_json::Value =
        serde_json::from_str(&fields_output).expect("symbol-fields JSON should parse");
    assert_eq!(fields.as_array().unwrap().len(), 1);
    assert_eq!(fields[0]["value"], "Package_SO:SOIC-8_3.9x4.9mm_P1.27mm");
    assert_eq!(fields[0]["visible"], true);

    let delete_cli = Cli::try_parse_from([
        "eda",
        "project",
        "delete-symbol-field",
        root.to_str().unwrap(),
        "--field",
        &field_uuid,
    ])
    .expect("CLI should parse");
    let delete_output = execute(delete_cli).expect("project delete-symbol-field should succeed");
    assert!(delete_output.contains("action: delete_symbol_field"));
    assert!(delete_output.contains("key: Footprint"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbol-fields",
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let fields_output = execute(query_cli).expect("project query symbol-fields should succeed");
    let fields: serde_json::Value =
        serde_json::from_str(&fields_output).expect("symbol-fields JSON should parse");
    assert_eq!(fields.as_array().unwrap().len(), 0);

    let _ = std::fs::remove_dir_all(&root);
}
