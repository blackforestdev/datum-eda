use super::*;

#[test]
fn execute_pool_search_command_returns_matches() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "pool",
        "search",
        "SOT23",
        "--library",
        eagle_fixture_path("simple-opamp.lbr").to_str().unwrap(),
        "--library",
        eagle_fixture_path("bjt-sot23.lbr").to_str().unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("pool search should succeed");
    assert!(output.contains("\"package\": \"SOT23\""));
    assert!(output.contains("\"package\": \"SOT23-5\""));
}

#[test]
fn execute_query_summary_command_returns_board_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_pcb")
            .to_str()
            .unwrap(),
        "summary",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("board summary query should succeed");
    assert!(output.contains("\"domain\": \"board\""));
    assert!(output.contains("\"name\": \"simple-demo\""));
    assert!(output.contains("\"components\": 1"));
}

#[test]
fn execute_query_netlist_command_returns_board_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_pcb")
            .to_str()
            .unwrap(),
        "netlist",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("board netlist query should succeed");
    assert!(output.contains("\"domain\": \"board\""));
    assert!(output.contains("\"name\": \"GND\""));
    assert!(output.contains("\"routed_pct\""));
}

#[test]
fn execute_query_nets_command_returns_schematic_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "nets",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("schematic net query should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"name\": \"SCL\""));
    assert!(output.contains("\"name\": \"VCC\""));
}

#[test]
fn execute_query_schematic_nets_command_returns_schematic_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "schematic-nets",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("schematic-nets query should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"name\": \"SCL\""));
    assert!(output.contains("\"semantic_class\": \"power\""));
}

#[test]
fn execute_query_components_command_rejects_schematic_inputs() {
    let cli = Cli::try_parse_from([
        "eda",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "components",
    ])
    .expect("CLI should parse");

    let err = execute(cli).expect_err("schematic components query must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("only implemented for boards in M1"), "{msg}");
}

#[test]
fn execute_query_sheets_command_returns_schematic_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "sheets",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("schematic sheets query should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"name\": \"Root\""));
    assert!(output.contains("\"symbols\": 1"));
}

#[test]
fn execute_query_symbols_command_returns_schematic_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "symbols",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("schematic symbols query should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"reference\": \"R1\""));
}

#[test]
fn execute_query_labels_command_returns_schematic_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "labels",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("schematic labels query should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"name\": \"SCL\""));
    assert!(output.contains("\"name\": \"VCC\""));
    assert!(output.contains("\"name\": \"SUB_IN\""));
}

#[test]
fn execute_query_buses_command_returns_schematic_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "buses",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("schematic buses query should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"buses\""));
    assert!(output.contains("\"members\""));
}

#[test]
fn execute_query_bus_entries_command_returns_schematic_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "bus-entries",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("schematic bus-entries query should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"bus_entries\""));
}

#[test]
fn execute_query_noconnects_command_returns_schematic_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "noconnects",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("schematic noconnects query should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"noconnects\""));
    assert!(output.contains("\"pin\""));
}

#[test]
fn execute_query_ports_command_returns_schematic_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "ports",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("schematic ports query should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"name\": \"SUB_IN\""));
}

#[test]
fn execute_query_hierarchy_command_returns_schematic_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "hierarchy",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("schematic hierarchy query should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"name\": \"Sub\""));
}

#[test]
fn execute_query_diagnostics_command_returns_schematic_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "diagnostics",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("schematic diagnostics query should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"kind\": \"dangling_component_pin\""));
}

#[test]
fn execute_query_diagnostics_command_returns_board_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("simple-demo.kicad_pcb")
            .to_str()
            .unwrap(),
        "diagnostics",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("board diagnostics query should succeed");
    assert!(output.contains("\"domain\": \"board\""));
    assert!(output.contains("\"kind\": \"net_without_copper\""));
}

#[test]
fn execute_query_diagnostics_command_returns_partial_route_board_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("partial-route-demo.kicad_pcb")
            .to_str()
            .unwrap(),
        "diagnostics",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("partial-route diagnostics query should succeed");
    assert!(output.contains("\"domain\": \"board\""));
    assert!(output.contains("\"kind\": \"partially_routed_net\""));
}

#[test]
fn execute_query_unrouted_command_returns_board_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        kicad_fixture_path("airwire-demo.kicad_pcb")
            .to_str()
            .unwrap(),
        "unrouted",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("board unrouted query should succeed");
    assert!(output.contains("\"domain\": \"board\""));
    assert!(output.contains("\"net_name\": \"SIG\""));
    assert!(output.contains("\"component\": \"R1\""));
    assert!(output.contains("\"component\": \"R2\""));
}

#[test]
fn execute_query_labels_command_rejects_board_inputs() {
    let cli = Cli::try_parse_from([
        "eda",
        "query",
        kicad_fixture_path("simple-demo.kicad_pcb")
            .to_str()
            .unwrap(),
        "labels",
    ])
    .expect("CLI should parse");

    let err = execute(cli).expect_err("board labels query must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("only implemented for schematics"), "{msg}");
}

#[test]
fn execute_query_unrouted_command_rejects_schematic_inputs() {
    let cli = Cli::try_parse_from([
        "eda",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "unrouted",
    ])
    .expect("CLI should parse");

    let err = execute(cli).expect_err("schematic unrouted query must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("only implemented for boards in M1"), "{msg}");
}

#[test]
fn execute_query_design_rules_command_returns_board_rules() {
    let source = kicad_fixture_path("simple-demo.kicad_pcb");
    let target = std::env::temp_dir().join(format!(
        "{}-cli-query-design-rules.kicad_pcb",
        Uuid::new_v4()
    ));
    modify_board(
        &source,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        Some(125_000),
        0,
        0,
        Some(&target),
        false,
    )
    .expect("modify rule save should succeed");

    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "query",
        target.to_str().unwrap(),
        "design-rules",
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("design-rules query should succeed");
    let payload: serde_json::Value =
        serde_json::from_str(&output).expect("output should be valid JSON");
    assert_eq!(payload["domain"], "board");
    let rules = payload["rules"]
        .as_array()
        .expect("rules should be an array");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0]["name"], "default clearance");

    let sidecar = target.with_file_name(format!(
        "{}.rules.json",
        target.file_name().unwrap().to_string_lossy()
    ));
    let _ = std::fs::remove_file(target);
    let _ = std::fs::remove_file(sidecar);
}

#[test]
fn execute_query_design_rules_command_rejects_schematic_inputs() {
    let cli = Cli::try_parse_from([
        "eda",
        "query",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "design-rules",
    ])
    .expect("CLI should parse");

    let err = execute(cli).expect_err("schematic design-rules query must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("only implemented for boards in M3"), "{msg}");
}
