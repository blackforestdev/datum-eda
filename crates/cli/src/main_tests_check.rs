use super::*;

#[test]
fn run_erc_supports_kicad_schematic() {
    let findings =
        run_erc(&kicad_fixture_path("simple-demo.kicad_sch")).expect("erc should succeed");
    assert_eq!(findings.len(), 2);
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "unconnected_component_pin")
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.code == "undriven_power_net")
    );
}

#[test]
fn run_erc_rejects_non_schematic_inputs() {
    let err = run_erc(&eagle_fixture_path("simple-opamp.lbr"))
        .expect_err("non schematic input must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("only accepts KiCad .kicad_sch"), "{msg}");
}

#[test]
fn run_drc_supports_kicad_board() {
    let report =
        run_drc(&kicad_fixture_path("partial-route-demo.kicad_pcb")).expect("drc should run");
    assert!(!report.passed);
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.code == "connectivity_unrouted_net")
    );
}

#[test]
fn run_drc_rejects_non_board_inputs() {
    let err =
        run_drc(&kicad_fixture_path("simple-demo.kicad_sch")).expect_err("non board must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("only accepts KiCad .kicad_pcb"), "{msg}");
}

#[test]
fn run_check_supports_board_and_schematic_inputs() {
    let board = run_check(&kicad_fixture_path("simple-demo.kicad_pcb"))
        .expect("board check should succeed");
    match board {
        CheckReport::Board {
            summary,
            diagnostics,
        } => {
            assert_eq!(summary.status, eda_engine::api::CheckStatus::Info);
            assert_eq!(summary.infos, 1);
            assert_eq!(summary.by_code.len(), 1);
            assert_eq!(summary.by_code[0].code, "net_without_copper");
            assert_eq!(summary.by_code[0].count, 1);
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].kind, "net_without_copper");
        }
        other => panic!("expected board report, got {other:?}"),
    }

    let partial_board = run_check(&kicad_fixture_path("partial-route-demo.kicad_pcb"))
        .expect("partial-route board check should succeed");
    match partial_board {
        CheckReport::Board {
            summary,
            diagnostics,
        } => {
            assert_eq!(summary.status, eda_engine::api::CheckStatus::Warning);
            assert_eq!(summary.warnings, 1);
            assert_eq!(summary.infos, 1);
            assert!(
                summary
                    .by_code
                    .iter()
                    .any(|entry| entry.code == "partially_routed_net" && entry.count == 1)
            );
            assert!(
                diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.kind == "partially_routed_net")
            );
        }
        other => panic!("expected board report, got {other:?}"),
    }

    let schematic = run_check(&kicad_fixture_path("simple-demo.kicad_sch"))
        .expect("schematic check should succeed");
    match schematic {
        CheckReport::Schematic {
            summary,
            diagnostics,
            erc,
            drc,
        } => {
            assert_eq!(summary.status, eda_engine::api::CheckStatus::Warning);
            assert_eq!(summary.warnings, 3);
            assert_eq!(summary.by_code.len(), 3);
            assert!(drc.is_empty());
            assert!(
                summary
                    .by_code
                    .iter()
                    .any(|entry| entry.code == "dangling_component_pin" && entry.count == 1)
            );
            assert!(
                summary
                    .by_code
                    .iter()
                    .any(|entry| entry.code == "unconnected_component_pin" && entry.count == 1)
            );
            assert!(
                summary
                    .by_code
                    .iter()
                    .any(|entry| entry.code == "undriven_power_net" && entry.count == 1)
            );
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(erc.len(), 2);
        }
        other => panic!("expected schematic report, got {other:?}"),
    }
}

#[test]
fn execute_erc_command_returns_finding_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "erc",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("erc command should succeed");
    assert!(output.contains("\"code\": \"unconnected_component_pin\""));
    assert!(output.contains("\"code\": \"undriven_power_net\""));
    assert!(output.contains("\"waived\": false"));
}

#[test]
fn execute_drc_command_returns_report_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "drc",
        kicad_fixture_path("partial-route-demo.kicad_pcb")
            .to_str()
            .unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("drc command should succeed");
    assert!(output.contains("\"passed\": false"));
    assert!(output.contains("\"code\": \"connectivity_unrouted_net\""));
}

#[test]
fn execute_check_command_returns_schematic_report_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "check",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("check command should succeed");
    assert!(output.contains("\"domain\": \"schematic\""));
    assert!(output.contains("\"status\": \"warning\""));
    assert!(output.contains("\"kind\": \"dangling_component_pin\""));
    assert!(output.contains("\"code\": \"unconnected_component_pin\""));
}

#[test]
fn execute_check_command_returns_board_report_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "check",
        kicad_fixture_path("simple-demo.kicad_pcb")
            .to_str()
            .unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("check command should succeed");
    assert!(output.contains("\"domain\": \"board\""));
    assert!(output.contains("\"status\": \"info\""));
    assert!(output.contains("\"by_code\""));
    assert!(output.contains("\"kind\": \"net_without_copper\""));
}

#[test]
fn execute_check_command_returns_partial_route_board_report_output() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "check",
        kicad_fixture_path("partial-route-demo.kicad_pcb")
            .to_str()
            .unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("check command should succeed");
    assert!(output.contains("\"domain\": \"board\""));
    assert!(output.contains("\"status\": \"warning\""));
    assert!(output.contains("\"kind\": \"partially_routed_net\""));
}

#[test]
fn execute_check_command_text_output_is_compact_for_schematic() {
    let cli = Cli::try_parse_from([
        "eda",
        "check",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("check command should succeed");
    assert!(output.contains("schematic check: status=warning"));
    assert!(output.contains("counts:"));
    assert!(output.contains("dangling_component_pin x1"));
    assert!(output.contains("erc:"));
    assert!(output.contains("[warning] unconnected_component_pin:"));
}

#[test]
fn execute_check_command_text_output_is_compact_for_board() {
    let cli = Cli::try_parse_from([
        "eda",
        "check",
        kicad_fixture_path("simple-demo.kicad_pcb")
            .to_str()
            .unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("check command should succeed");
    assert!(output.contains("board check: status=info"));
    assert!(output.contains("counts:"));
    assert!(output.contains("net_without_copper x1"));
    assert!(output.contains("diagnostics:"));
}

#[test]
fn execute_check_command_text_output_is_compact_for_partial_route_board() {
    let cli = Cli::try_parse_from([
        "eda",
        "check",
        kicad_fixture_path("partial-route-demo.kicad_pcb")
            .to_str()
            .unwrap(),
    ])
    .expect("CLI should parse");

    let output = execute(cli).expect("check command should succeed");
    assert!(output.contains("board check: status=warning"));
    assert!(output.contains("partially_routed_net x1"));
    assert!(output.contains("net_without_copper x1"));
}

#[test]
fn render_check_report_text_includes_input_without_explicit_driver() {
    let test_uuid =
        eda_engine::ir::ids::import_uuid(&eda_engine::ir::ids::namespace_kicad(), "test-pin");
    let report = CheckReport::Schematic {
        summary: eda_engine::api::CheckSummary {
            status: CheckStatus::Info,
            errors: 0,
            warnings: 0,
            infos: 1,
            waived: 0,
            by_code: vec![eda_engine::api::CheckCodeCount {
                code: "input_without_explicit_driver".into(),
                count: 1,
            }],
        },
        diagnostics: Vec::new(),
        erc: vec![ErcFinding {
            id: test_uuid,
            code: "input_without_explicit_driver",
            severity: eda_engine::erc::ErcSeverity::Info,
            message: "input pins on net IN_P have no explicit driver".into(),
            net_name: Some("IN_P".into()),
            component: None,
            pin: None,
            objects: vec![eda_engine::erc::ErcObjectRef {
                kind: "pin",
                key: "Q1.1".into(),
            }],
            object_uuids: vec![test_uuid],
            waived: false,
        }],
        drc: Vec::new(),
    };

    let output = render_check_report_text(&report);
    assert!(output.contains("schematic check: status=info"));
    assert!(output.contains("input_without_explicit_driver x1"));
    assert!(output.contains("[info] input_without_explicit_driver:"));
}

#[test]
fn execute_check_command_honors_fail_on_threshold() {
    let cli = Cli::try_parse_from([
        "eda",
        "check",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "--fail-on",
        "warning",
    ])
    .expect("CLI should parse");

    let (_output, exit_code) =
        execute_with_exit_code(cli).expect("check command should run successfully");
    assert_eq!(exit_code, 1);
}

#[test]
fn execute_check_command_allows_higher_fail_on_threshold() {
    let cli = Cli::try_parse_from([
        "eda",
        "check",
        kicad_fixture_path("simple-demo.kicad_sch")
            .to_str()
            .unwrap(),
        "--fail-on",
        "error",
    ])
    .expect("CLI should parse");

    let (output, exit_code) =
        execute_with_exit_code(cli).expect("check command should run successfully");
    assert_eq!(exit_code, 0);
    assert!(output.contains("schematic check: status=warning"));
}

#[test]
fn execute_drc_command_uses_violation_exit_code() {
    let cli = Cli::try_parse_from([
        "eda",
        "--format",
        "json",
        "drc",
        kicad_fixture_path("partial-route-demo.kicad_pcb")
            .to_str()
            .unwrap(),
    ])
    .expect("CLI should parse");

    let (_output, exit_code) =
        execute_with_exit_code(cli).expect("drc command should run successfully");
    assert_eq!(exit_code, 1);
}
