use super::*;
use eda_engine::ir::geometry::Point;
use eda_engine::ir::serialization::to_json_deterministic;
use eda_engine::schematic::{
    HiddenPowerBehavior, PinDisplayOverride, PinElectricalType, PlacedSymbol, SymbolDisplayMode,
    SymbolPin,
};

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

fn seed_native_symbol_with_pins(root: &Path, sheet_uuid: Uuid) -> Uuid {
    let sheet_path = root
        .join("schematic/sheets")
        .join(format!("{sheet_uuid}.json"));
    let mut sheet_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&sheet_path).expect("sheet file should read"),
    )
    .expect("sheet file should parse");

    let symbol_uuid = Uuid::new_v4();
    let pin_a_uuid = Uuid::new_v4();
    let pin_b_uuid = Uuid::new_v4();
    let symbol = PlacedSymbol {
        uuid: symbol_uuid,
        part: None,
        entity: None,
        gate: None,
        lib_id: Some("device:opamp".to_string()),
        reference: "U11".to_string(),
        value: "TL072".to_string(),
        fields: Vec::new(),
        pins: vec![
            SymbolPin {
                uuid: pin_a_uuid,
                number: "1".to_string(),
                name: "OUT_A".to_string(),
                electrical_type: PinElectricalType::Output,
                position: Point { x: 10, y: 20 },
            },
            SymbolPin {
                uuid: pin_b_uuid,
                number: "2".to_string(),
                name: "IN-_A".to_string(),
                electrical_type: PinElectricalType::Input,
                position: Point { x: 30, y: 40 },
            },
        ],
        position: Point { x: 100, y: 200 },
        rotation: 0,
        mirrored: false,
        unit_selection: Some("A".to_string()),
        display_mode: SymbolDisplayMode::LibraryDefault,
        pin_overrides: vec![PinDisplayOverride {
            pin: pin_b_uuid,
            visible: false,
            position: Some(Point { x: 300, y: 400 }),
        }],
        hidden_power_behavior: HiddenPowerBehavior::SourceDefinedImplicit,
    };

    sheet_value["symbols"][symbol_uuid.to_string()] =
        serde_json::to_value(symbol).expect("symbol serialization should succeed");
    std::fs::write(
        &sheet_path,
        format!(
            "{}\n",
            to_json_deterministic(&sheet_value).expect("canonical serialization should succeed")
        ),
    )
    .expect("sheet file should write");

    symbol_uuid
}

#[test]
fn project_set_and_clear_symbol_semantics_updates_native_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-symbol-semantics");
    create_native_project(&root, Some("Symbol Semantics Demo".to_string()))
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
        "U7",
        "--value",
        "LM324",
        "--x-nm",
        "700",
        "--y-nm",
        "800",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();
    let gate_uuid = Uuid::new_v4().to_string();

    let set_unit_cli = Cli::try_parse_from([
        "eda",
        "project",
        "set-symbol-unit",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--unit",
        "A",
    ])
    .expect("CLI should parse");
    let set_unit_output = execute(set_unit_cli).expect("project set-symbol-unit should succeed");
    assert!(set_unit_output.contains("action: set_symbol_unit"));
    assert!(set_unit_output.contains("unit_selection: A"));

    let set_gate_cli = Cli::try_parse_from([
        "eda",
        "project",
        "set-symbol-gate",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--gate",
        &gate_uuid,
    ])
    .expect("CLI should parse");
    let set_gate_output = execute(set_gate_cli).expect("project set-symbol-gate should succeed");
    assert!(set_gate_output.contains("action: set_symbol_gate"));
    assert!(set_gate_output.contains(&format!("gate_uuid: {gate_uuid}")));
    assert!(set_gate_output.contains("unit_selection: A"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbol-semantics",
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let semantics_output =
        execute(query_cli).expect("project query symbol-semantics should succeed");
    let semantics: serde_json::Value =
        serde_json::from_str(&semantics_output).expect("symbol-semantics JSON should parse");
    assert_eq!(semantics["symbol_uuid"], symbol_uuid);
    assert_eq!(semantics["unit_selection"], "A");
    assert_eq!(semantics["gate_uuid"], gate_uuid);

    let clear_unit_cli = Cli::try_parse_from([
        "eda",
        "project",
        "clear-symbol-unit",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let clear_unit_output =
        execute(clear_unit_cli).expect("project clear-symbol-unit should succeed");
    assert!(clear_unit_output.contains("action: clear_symbol_unit"));

    let clear_gate_cli = Cli::try_parse_from([
        "eda",
        "project",
        "clear-symbol-gate",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let clear_gate_output =
        execute(clear_gate_cli).expect("project clear-symbol-gate should succeed");
    assert!(clear_gate_output.contains("action: clear_symbol_gate"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbol-semantics",
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let semantics_output =
        execute(query_cli).expect("project query symbol-semantics should succeed");
    let semantics: serde_json::Value =
        serde_json::from_str(&semantics_output).expect("symbol-semantics JSON should parse");
    assert_eq!(semantics["symbol_uuid"], symbol_uuid);
    assert!(semantics["unit_selection"].is_null());
    assert!(semantics["gate_uuid"].is_null());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_query_symbol_pins_reports_override_state() {
    let root = unique_project_root("datum-eda-cli-project-symbol-pins");
    create_native_project(&root, Some("Symbol Pins Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let symbol_uuid = seed_native_symbol_with_pins(&root, sheet_uuid);

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbol-pins",
        "--symbol",
        &symbol_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let pins_output = execute(query_cli).expect("project query symbol-pins should succeed");
    let pins: serde_json::Value =
        serde_json::from_str(&pins_output).expect("symbol-pins JSON should parse");
    assert_eq!(pins.as_array().unwrap().len(), 2);
    assert_eq!(pins[0]["symbol_uuid"], symbol_uuid.to_string());
    assert_eq!(pins[0]["number"], "1");
    assert_eq!(pins[0]["name"], "OUT_A");
    assert_eq!(pins[0]["electrical_type"], "Output");
    assert_eq!(pins[0]["x_nm"], 10);
    assert_eq!(pins[0]["y_nm"], 20);
    assert!(pins[0]["visible_override"].is_null());
    assert!(pins[0]["override_x_nm"].is_null());
    assert!(pins[0]["override_y_nm"].is_null());
    assert_eq!(pins[1]["number"], "2");
    assert_eq!(pins[1]["name"], "IN-_A");
    assert_eq!(pins[1]["electrical_type"], "Input");
    assert_eq!(pins[1]["visible_override"], false);
    assert_eq!(pins[1]["override_x_nm"], 300);
    assert_eq!(pins[1]["override_y_nm"], 400);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_symbol_display_mode_updates_symbol_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-symbol-display-mode");
    create_native_project(&root, Some("Symbol Display Mode Demo".to_string()))
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
        "U13",
        "--value",
        "LM339",
        "--x-nm",
        "120",
        "--y-nm",
        "240",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();
    assert_eq!(placed["display_mode"], "LibraryDefault");

    let display_cli = Cli::try_parse_from([
        "eda",
        "project",
        "set-symbol-display-mode",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--mode",
        "show-hidden-pins",
    ])
    .expect("CLI should parse");
    let display_output =
        execute(display_cli).expect("project set-symbol-display-mode should succeed");
    assert!(display_output.contains("action: set_symbol_display_mode"));
    assert!(display_output.contains("display_mode: ShowHiddenPins"));

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
    assert_eq!(symbols[0]["gate_uuid"], serde_json::Value::Null);

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_symbol_hidden_power_behavior_updates_symbol_semantics_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-hidden-power");
    create_native_project(&root, Some("Hidden Power Demo".to_string()))
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
        "PWR1",
        "--value",
        "VCC",
        "--x-nm",
        "40",
        "--y-nm",
        "80",
    ])
    .expect("CLI should parse");
    let place_output = execute(place_cli).expect("project place-symbol should succeed");
    let placed: serde_json::Value =
        serde_json::from_str(&place_output).expect("place-symbol JSON should parse");
    let symbol_uuid = placed["symbol_uuid"].as_str().unwrap().to_string();
    assert_eq!(placed["hidden_power_behavior"], "SourceDefinedImplicit");

    let behavior_cli = Cli::try_parse_from([
        "eda",
        "project",
        "set-symbol-hidden-power-behavior",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid,
        "--behavior",
        "explicit-power-object",
    ])
    .expect("CLI should parse");
    let behavior_output = execute(behavior_cli)
        .expect("project set-symbol-hidden-power-behavior should succeed");
    assert!(behavior_output.contains("action: set_symbol_hidden_power_behavior"));
    assert!(behavior_output.contains("hidden_power_behavior: ExplicitPowerObject"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbol-semantics",
        "--symbol",
        &symbol_uuid,
    ])
    .expect("CLI should parse");
    let semantics_output =
        execute(query_cli).expect("project query symbol-semantics should succeed");
    let semantics: serde_json::Value =
        serde_json::from_str(&semantics_output).expect("symbol-semantics JSON should parse");
    assert_eq!(semantics["symbol_uuid"], symbol_uuid);
    assert_eq!(semantics["hidden_power_behavior"], "ExplicitPowerObject");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn project_set_and_clear_pin_override_updates_symbol_pin_query_surface() {
    let root = unique_project_root("datum-eda-cli-project-pin-override");
    create_native_project(&root, Some("Pin Override Demo".to_string()))
        .expect("initial scaffold should succeed");
    let sheet_uuid = seed_native_sheet(&root);
    let symbol_uuid = seed_native_symbol_with_pins(&root, sheet_uuid);

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbol-pins",
        "--symbol",
        &symbol_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let pins_output = execute(query_cli).expect("project query symbol-pins should succeed");
    let pins: serde_json::Value =
        serde_json::from_str(&pins_output).expect("symbol-pins JSON should parse");
    let pin_uuid = pins[0]["pin_uuid"].as_str().unwrap().to_string();

    let set_cli = Cli::try_parse_from([
        "eda",
        "project",
        "set-pin-override",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid.to_string(),
        "--pin",
        &pin_uuid,
        "--visible",
        "false",
        "--x-nm",
        "910",
        "--y-nm",
        "920",
    ])
    .expect("CLI should parse");
    let set_output = execute(set_cli).expect("project set-pin-override should succeed");
    assert!(set_output.contains("action: set_pin_override"));
    assert!(set_output.contains("visible: false"));
    assert!(set_output.contains("x_nm: 910"));
    assert!(set_output.contains("y_nm: 920"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbol-pins",
        "--symbol",
        &symbol_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let pins_output = execute(query_cli).expect("project query symbol-pins should succeed");
    let pins: serde_json::Value =
        serde_json::from_str(&pins_output).expect("symbol-pins JSON should parse");
    assert_eq!(pins[0]["visible_override"], false);
    assert_eq!(pins[0]["override_x_nm"], 910);
    assert_eq!(pins[0]["override_y_nm"], 920);

    let clear_cli = Cli::try_parse_from([
        "eda",
        "project",
        "clear-pin-override",
        root.to_str().unwrap(),
        "--symbol",
        &symbol_uuid.to_string(),
        "--pin",
        &pin_uuid,
    ])
    .expect("CLI should parse");
    let clear_output = execute(clear_cli).expect("project clear-pin-override should succeed");
    assert!(clear_output.contains("action: clear_pin_override"));

    let query_cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "project",
        "query",
        root.to_str().unwrap(),
        "symbol-pins",
        "--symbol",
        &symbol_uuid.to_string(),
    ])
    .expect("CLI should parse");
    let pins_output = execute(query_cli).expect("project query symbol-pins should succeed");
    let pins: serde_json::Value =
        serde_json::from_str(&pins_output).expect("symbol-pins JSON should parse");
    assert!(pins[0]["visible_override"].is_null());
    assert!(pins[0]["override_x_nm"].is_null());
    assert!(pins[0]["override_y_nm"].is_null());

    let _ = std::fs::remove_dir_all(&root);
}
