use super::*;
use eda_engine::ir::serialization::to_json_deterministic;
use std::fs;
use std::path::PathBuf;

#[test]
fn imported_check_golden_simple_demo_matches_checked_in_fixture() {
    assert_cli_check_matches_golden("simple-demo.kicad_sch");
}

#[test]
fn imported_check_golden_analog_input_demo_matches_checked_in_fixture() {
    assert_cli_check_matches_golden("analog-input-demo.kicad_sch");
}

#[test]
fn imported_check_golden_analog_input_bias_demo_matches_checked_in_fixture() {
    assert_cli_check_matches_golden("analog-input-bias-demo.kicad_sch");
}

#[test]
fn imported_check_golden_erc_coverage_demo_matches_checked_in_fixture() {
    assert_cli_check_matches_golden("erc-coverage-demo.kicad_sch");
}

#[test]
fn imported_check_golden_hierarchy_mismatch_demo_matches_checked_in_fixture() {
    assert_cli_check_matches_golden("hierarchy-mismatch-demo.kicad_sch");
}

#[test]
fn imported_check_golden_bus_demo_matches_checked_in_fixture() {
    assert_cli_check_matches_golden("bus-demo.kicad_sch");
}

fn assert_cli_check_matches_golden(fixture: &str) {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "check",
        kicad_fixture_path(fixture).to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let (output, _exit_code) =
        execute_with_exit_code(cli).expect("imported check command should run");
    let normalized = normalize_check_json(&output);
    let actual = to_json_deterministic(&normalized)
        .unwrap_or_else(|err| panic!("failed to serialize normalized check output: {err}"));

    let golden = golden_path_for_check_fixture(fixture);
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
        "CLI check golden mismatch for fixture {fixture}"
    );
}

fn normalize_check_json(output: &str) -> serde_json::Value {
    let mut value: serde_json::Value =
        serde_json::from_str(output).expect("CLI check JSON output should parse");
    normalize_check_json_value(&mut value);
    value
}

fn normalize_check_json_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for child in map.values_mut() {
                normalize_check_json_value(child);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                normalize_check_json_value(item);
            }
        }
        serde_json::Value::String(text) => {
            if is_uuid_like(text) {
                *text = "<uuid>".to_string();
            } else if text.starts_with("N$") {
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

fn golden_path_for_check_fixture(fixture: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/golden/check")
        .join(format!("{fixture}.json"))
}
