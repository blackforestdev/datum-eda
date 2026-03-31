use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use std::fs;
use std::path::PathBuf;

#[test]
fn imported_query_golden_simple_demo_board_summary_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_pcb", "summary");
}

#[test]
fn imported_query_golden_simple_demo_board_netlist_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_pcb", "netlist");
}

#[test]
fn imported_query_golden_simple_demo_schematic_nets_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_sch", "nets");
}

#[test]
fn imported_query_golden_simple_demo_schematic_net_inventory_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_sch", "schematic-nets");
}

#[test]
fn imported_query_golden_simple_demo_sheets_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_sch", "sheets");
}

#[test]
fn imported_query_golden_simple_demo_symbols_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_sch", "symbols");
}

#[test]
fn imported_query_golden_simple_demo_buses_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_sch", "buses");
}

#[test]
fn imported_query_golden_simple_demo_bus_entries_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_sch", "bus-entries");
}

#[test]
fn imported_query_golden_simple_demo_noconnects_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_sch", "noconnects");
}

#[test]
fn imported_query_golden_simple_demo_labels_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_sch", "labels");
}

#[test]
fn imported_query_golden_simple_demo_ports_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_sch", "ports");
}

#[test]
fn imported_query_golden_simple_demo_hierarchy_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_sch", "hierarchy");
}

#[test]
fn imported_query_golden_simple_demo_diagnostics_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("simple-demo.kicad_sch", "diagnostics");
}

#[test]
fn imported_query_golden_airwire_demo_unrouted_matches_checked_in_fixture() {
    assert_cli_query_matches_golden("airwire-demo.kicad_pcb", "unrouted");
}

fn assert_cli_query_matches_golden(fixture: &str, subcommand: &str) {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path(fixture).to_str().unwrap(),
        subcommand,
    ])
    .expect("CLI should parse");

    let (output, exit_code) =
        execute_with_exit_code(cli).expect("imported query command should succeed");
    assert_eq!(
        exit_code, 0,
        "query command should succeed for {fixture} {subcommand}"
    );

    let normalized = normalize_cli_json(&output);
    let actual = to_json_deterministic(&normalized)
        .unwrap_or_else(|err| panic!("failed to serialize normalized query output: {err}"));

    let golden = golden_path_for_query_fixture(fixture, subcommand);
    if std::env::var_os("UPDATE_GOLDENS").is_some() {
        if let Some(parent) = golden.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|err| {
                panic!(
                    "failed to create golden directory {}: {err}",
                    parent.display()
                )
            });
        }
        fs::write(&golden, format!("{actual}\n"))
            .unwrap_or_else(|err| panic!("failed to write golden {}: {err}", golden.display()));
        return;
    }

    let expected = fs::read_to_string(&golden).unwrap_or_else(|err| {
        panic!(
            "failed to read golden {} (set UPDATE_GOLDENS=1 to regenerate): {err}",
            golden.display()
        )
    });
    assert_eq!(
        format!("{actual}\n"),
        expected,
        "CLI query golden mismatch for fixture {fixture} subcommand {subcommand}"
    );
}

fn normalize_cli_json(output: &str) -> serde_json::Value {
    let mut value: serde_json::Value =
        serde_json::from_str(output).expect("CLI query JSON output should parse");
    normalize_query_json_value(&mut value, None);
    value
}

fn normalize_query_json_value(value: &mut serde_json::Value, key: Option<&str>) {
    match value {
        serde_json::Value::Object(map) => {
            for (child_key, child_value) in map {
                normalize_query_json_value(child_value, Some(child_key.as_str()));
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                normalize_query_json_value(item, key);
            }
        }
        serde_json::Value::String(text) => {
            if is_uuid_like(text) {
                if matches!(
                    key,
                    Some("uuid")
                        | Some("sheet")
                        | Some("parent_sheet")
                        | Some("definition")
                        | Some("symbol")
                        | Some("pin")
                        | Some("bus")
                        | Some("objects")
                        | Some("port_uuids")
                ) {
                    *text = "<uuid>".to_string();
                    return;
                }
                if key.is_some_and(|name| name.ends_with("_uuid")) {
                    *text = "<uuid>".to_string();
                    return;
                }
            }
            if key == Some("name") && text.starts_with("N$") {
                *text = "N$<anon>".to_string();
            }
        }
        _ => {}
    }
}

fn is_uuid_like(text: &str) -> bool {
    if text.len() != 36 {
        return false;
    }
    text.chars().enumerate().all(|(index, ch)| match index {
        8 | 13 | 18 | 23 => ch == '-',
        _ => ch.is_ascii_hexdigit(),
    })
}

fn golden_path_for_query_fixture(fixture: &str, subcommand: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/golden/query")
        .join(format!("{fixture}.{subcommand}.json"))
}
