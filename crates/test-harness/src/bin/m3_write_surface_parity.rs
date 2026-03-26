use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use eda_engine::api::Engine;
use eda_test_harness::canonical_json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Cli {
    json: bool,
    allow_deferred: bool,
    repo_root: PathBuf,
    roundtrip_board_fixture_path: PathBuf,
    track_uuid: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Passed,
    Failed,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurfaceCheck {
    surface: String,
    status: Status,
    evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Report {
    schema_version: u32,
    overall_status: Status,
    checks: Vec<SurfaceCheck>,
}

#[derive(Debug, Deserialize)]
struct CliModifyReport {
    actions: Vec<String>,
    last_result: Option<CliOperationResult>,
    saved_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CliOperationResult {
    description: String,
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("m3_write_surface_parity: {err:#}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32> {
    let cli = parse_args()?;
    let report = build_report(&cli)?;

    if cli.json {
        println!("{}", canonical_json(&report)?);
    } else {
        print_human(&report);
    }

    let code = match report.overall_status {
        Status::Passed => 0,
        Status::Failed => 1,
        Status::Deferred => {
            if cli.allow_deferred {
                0
            } else {
                3
            }
        }
    };
    Ok(code)
}

fn parse_args() -> Result<Cli> {
    let mut json = false;
    let mut allow_deferred = false;
    let repo_root = detect_repo_root()?;
    let mut roundtrip_board_fixture_path =
        repo_root.join("crates/engine/testdata/import/kicad/partial-route-demo.kicad_pcb");
    let mut track_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--allow-deferred" => allow_deferred = true,
            "--roundtrip-board-fixture-path" => {
                let value = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--roundtrip-board-fixture-path requires a path argument")
                })?;
                roundtrip_board_fixture_path = PathBuf::from(value);
            }
            "--track-uuid" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--track-uuid requires a UUID argument"))?;
                track_uuid = Uuid::parse_str(&value)?;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            unknown => bail!("unknown argument {unknown}"),
        }
    }

    Ok(Cli {
        json,
        allow_deferred,
        repo_root,
        roundtrip_board_fixture_path,
        track_uuid,
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p eda-test-harness --bin m3_write_surface_parity -- [options]\n\
         Options:\n\
           --json                          Emit canonical JSON\n\
           --allow-deferred                Exit 0 when status is deferred\n\
           --roundtrip-board-fixture-path <p>  KiCad board fixture used for delete/undo/redo/save parity\n\
           --track-uuid <uuid>             Track UUID used for delete/undo/redo parity\n\
           -h, --help                      Show this help"
    );
}

fn detect_repo_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .context("failed to resolve repository root")
}

fn build_report(cli: &Cli) -> Result<Report> {
    let checks = vec![
        check_engine_surface(cli),
        check_daemon_surface(cli),
        check_mcp_surface(cli),
        check_cli_surface(cli),
    ];
    let overall_status = if checks.iter().any(|c| c.status == Status::Failed) {
        Status::Failed
    } else if checks.iter().all(|c| c.status == Status::Passed) {
        Status::Passed
    } else {
        Status::Deferred
    };

    Ok(Report {
        schema_version: 1,
        overall_status,
        checks,
    })
}

fn check_engine_surface(cli: &Cli) -> SurfaceCheck {
    match engine_surface_result(cli) {
        Ok(evidence) => SurfaceCheck {
            surface: "engine_write_surface".to_string(),
            status: Status::Passed,
            evidence,
        },
        Err(err) => SurfaceCheck {
            surface: "engine_write_surface".to_string(),
            status: Status::Failed,
            evidence: err.to_string(),
        },
    }
}

fn engine_surface_result(cli: &Cli) -> Result<String> {
    let mut engine = Engine::new()?;
    engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline = engine.get_net_info()?;
    let baseline_check = engine.get_check_report()?;
    let delete = engine.delete_track(&cli.track_uuid)?;
    let after_delete = engine.get_net_info()?;
    let after_delete_check = engine.get_check_report()?;
    let undo = engine.undo()?;
    let after_undo = engine.get_net_info()?;
    let redo = engine.redo()?;
    let after_redo = engine.get_net_info()?;

    if after_delete == baseline {
        bail!("delete_track did not change engine net state");
    }
    if after_undo != baseline {
        bail!("undo did not restore engine baseline state");
    }
    if after_redo != after_delete {
        bail!("redo did not restore engine deleted state");
    }
    let baseline_diagnostics = match baseline_check {
        eda_engine::api::CheckReport::Board { diagnostics, .. } => diagnostics,
        _ => bail!("engine baseline check report was not a board report"),
    };
    let after_delete_diagnostics = match after_delete_check {
        eda_engine::api::CheckReport::Board { diagnostics, .. } => diagnostics,
        _ => bail!("engine delete_track check report was not a board report"),
    };
    if !baseline_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == "partially_routed_net")
    {
        bail!("engine baseline check report missing partially_routed_net");
    }
    if !after_delete_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == "net_without_copper")
    {
        bail!("engine delete_track follow-up check report missing net_without_copper");
    }

    let target = unique_temp_path("engine-surface-save", "kicad_pcb");
    engine.save(&target)?;
    let mut reloaded = Engine::new()?;
    reloaded.import(&target)?;
    let reloaded_after_save = reloaded.get_net_info()?;
    if reloaded_after_save != after_redo {
        bail!("saved engine board did not reimport to the current deleted state");
    }

    let via_fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let via_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
    let mut via_engine = Engine::new()?;
    via_engine.import(&via_fixture)?;
    let baseline_via_state = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&via_fixture)?;
        baseline_engine.get_net_info()?
    };
    let delete_via = via_engine.delete_via(&via_uuid)?;
    let via_deleted_state = via_engine.get_net_info()?;
    let via_target = unique_temp_path("engine-surface-via-save", "kicad_pcb");
    via_engine.save(&via_target)?;
    let mut reloaded_via = Engine::new()?;
    reloaded_via.import(&via_target)?;
    if reloaded_via.get_net_info()? != via_deleted_state {
        bail!("saved engine via-deleted board did not reimport to the current deleted state");
    }
    let baseline_via_gnd = baseline_via_state
        .iter()
        .find(|net| net.name == "GND")
        .ok_or_else(|| anyhow::anyhow!("baseline via state missing GND"))?;
    let after_via_gnd = via_deleted_state
        .iter()
        .find(|net| net.name == "GND")
        .ok_or_else(|| anyhow::anyhow!("via-deleted state missing GND"))?;
    if baseline_via_gnd.vias == after_via_gnd.vias {
        bail!("delete_via did not change engine follow-up net-info state");
    }

    let mut rule_engine = Engine::new()?;
    rule_engine.import(&via_fixture)?;
    let baseline_rules = rule_engine.get_design_rules()?;
    let set_rule = rule_engine.set_design_rule(eda_engine::api::SetDesignRuleInput {
        rule_type: eda_engine::rules::ast::RuleType::ClearanceCopper,
        scope: eda_engine::rules::ast::RuleScope::All,
        parameters: eda_engine::rules::ast::RuleParams::Clearance { min: 125_000 },
        priority: 10,
        name: Some("default clearance".to_string()),
    })?;
    let rule_target = unique_temp_path("engine-surface-rule-save", "kicad_pcb");
    rule_engine.save(&rule_target)?;
    let mut reloaded_rule = Engine::new()?;
    reloaded_rule.import(&rule_target)?;
    if reloaded_rule.get_design_rules()?.len() != 1 {
        bail!("saved engine rule-mutated board did not reimport one design rule");
    }
    if baseline_rules.len() == reloaded_rule.get_design_rules()?.len() {
        bail!("set_design_rule did not change engine follow-up design-rules state");
    }

    let mut move_engine = Engine::new()?;
    move_engine.import(&cli.roundtrip_board_fixture_path)?;
    let moved = move_engine.move_component(eda_engine::api::MoveComponentInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        position: eda_engine::ir::geometry::Point::new(15_000_000, 12_000_000),
        rotation: Some(90),
    })?;
    let move_target = unique_temp_path("engine-surface-move-save", "kicad_pcb");
    move_engine.save(&move_target)?;
    let mut reloaded_move = Engine::new()?;
    reloaded_move.import(&move_target)?;
    let moved_component = reloaded_move
        .get_components()?
        .into_iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("reloaded moved component missing R1"))?;
    if moved_component.position.x != 15_000_000
        || moved_component.position.y != 12_000_000
        || moved_component.rotation != 90
    {
        bail!("saved moved component did not reimport to the expected position");
    }
    let baseline_move_airwires = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_unrouted()?
    };
    let moved_airwires = move_engine.get_unrouted()?;
    if moved_airwires.len() != baseline_move_airwires.len() {
        bail!("move_component changed engine airwire count unexpectedly");
    }
    if moved_airwires.first().map(|airwire| airwire.distance_nm)
        == baseline_move_airwires
            .first()
            .map(|airwire| airwire.distance_nm)
    {
        bail!("move_component did not change engine unrouted derived state");
    }

    let mut value_engine = Engine::new()?;
    value_engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline_value_components = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_components()?
    };
    let set_value = value_engine.set_value(eda_engine::api::SetValueInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        value: "22k".to_string(),
    })?;
    let value_target = unique_temp_path("engine-surface-value-save", "kicad_pcb");
    value_engine.save(&value_target)?;
    let mut reloaded_value = Engine::new()?;
    reloaded_value.import(&value_target)?;
    let reloaded_value_components = reloaded_value.get_components()?;
    let baseline_r1 = baseline_value_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("baseline value components missing R1"))?;
    let updated_r1 = reloaded_value_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("reloaded value components missing R1"))?;
    if updated_r1.value != "22k" {
        bail!("saved set_value component did not reimport expected value");
    }
    if baseline_r1.value == updated_r1.value {
        bail!("set_value did not change engine follow-up components state");
    }

    let mut reference_engine = Engine::new()?;
    reference_engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline_reference_components = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_components()?
    };
    let set_reference = reference_engine.set_reference(eda_engine::api::SetReferenceInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        reference: "R10".to_string(),
    })?;
    let reference_target = unique_temp_path("engine-surface-reference-save", "kicad_pcb");
    reference_engine.save(&reference_target)?;
    let mut reloaded_reference = Engine::new()?;
    reloaded_reference.import(&reference_target)?;
    let reloaded_reference_components = reloaded_reference.get_components()?;
    let baseline_reference_r1 = baseline_reference_components
        .iter()
        .find(|component| component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
        .ok_or_else(|| anyhow::anyhow!("baseline reference components missing target component"))?;
    let updated_reference = reloaded_reference_components
        .iter()
        .find(|component| component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
        .ok_or_else(|| anyhow::anyhow!("reloaded reference components missing target component"))?;
    if updated_reference.reference != "R10" {
        bail!("saved set_reference component did not reimport expected reference");
    }
    if baseline_reference_r1.reference == updated_reference.reference {
        bail!("set_reference did not change engine follow-up components state");
    }

    Ok(format!(
        "delete={}, undo={}, redo={}, saved={}, reimported_deleted_state=true, delete_followup_check_changed=true, delete_via={}, via_saved={}, via_reimported_deleted_state=true, delete_via_followup_net_info_changed=true, set_rule={}, rule_saved={}, rule_reimported=true, rule_followup_query_changed=true, move_component={}, moved_saved={}, move_reimported=true, move_followup_unrouted_changed=true, set_value={}, value_saved={}, value_followup_components_changed=true, set_reference={}, reference_saved={}, reference_followup_components_changed=true",
        delete.description,
        undo.description,
        redo.description,
        target.display(),
        delete_via.description,
        via_target.display(),
        set_rule.description,
        rule_target.display(),
        moved.description,
        move_target.display(),
        set_value.description,
        value_target.display(),
        set_reference.description,
        reference_target.display()
    ))
}

fn check_daemon_surface(cli: &Cli) -> SurfaceCheck {
    match daemon_surface_result(cli) {
        Ok(evidence) => SurfaceCheck {
            surface: "daemon_write_surface".to_string(),
            status: Status::Passed,
            evidence,
        },
        Err(err) => SurfaceCheck {
            surface: "daemon_write_surface".to_string(),
            status: Status::Failed,
            evidence: err.to_string(),
        },
    }
}

fn daemon_surface_result(cli: &Cli) -> Result<String> {
    let save_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "save_dispatch_writes_current_m3_slice_to_requested_path",
            ])
            .current_dir(&cli.repo_root),
        "daemon save dispatch parity probe",
    )?;
    let roundtrip_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_track_undo_and_redo_dispatch_round_trip",
            ])
            .current_dir(&cli.repo_root),
        "daemon roundtrip dispatch parity probe",
    )?;
    let delete_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_track_dispatch_updates_followup_check_report",
            ])
            .current_dir(&cli.repo_root),
        "daemon delete-track derived-state parity probe",
    )?;
    let via_roundtrip_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_via_undo_and_redo_dispatch_round_trip",
            ])
            .current_dir(&cli.repo_root),
        "daemon via roundtrip dispatch parity probe",
    )?;
    let via_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_via_dispatch_updates_followup_net_info_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon delete-via derived-state parity probe",
    )?;
    let rule_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_design_rule_dispatch_persists_rule_in_memory",
            ])
            .current_dir(&cli.repo_root),
        "daemon rule dispatch parity probe",
    )?;
    let rule_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_design_rule_dispatch_updates_followup_design_rules_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon rule derived-state parity probe",
    )?;
    let value_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_value_dispatch_updates_component_value",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-value parity probe",
    )?;
    let value_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_value_dispatch_updates_followup_components_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-value derived-state parity probe",
    )?;
    let reference_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_reference_dispatch_updates_component_reference",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-reference parity probe",
    )?;
    let reference_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_reference_dispatch_updates_followup_components_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-reference derived-state parity probe",
    )?;
    let move_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "move_component_dispatch_updates_component_position",
            ])
            .current_dir(&cli.repo_root),
        "daemon move-component parity probe",
    )?;
    let move_derived_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "move_component_dispatch_updates_followup_unrouted_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon move-component derived-state parity probe",
    )?;

    Ok(format!(
        "behavioral dispatch tests passed: save_dispatch_writes_current_m3_slice_to_requested_path, delete_track_undo_and_redo_dispatch_round_trip, delete_track_dispatch_updates_followup_check_report, delete_via_undo_and_redo_dispatch_round_trip, delete_via_dispatch_updates_followup_net_info_query, set_design_rule_dispatch_persists_rule_in_memory, set_design_rule_dispatch_updates_followup_design_rules_query, set_value_dispatch_updates_component_value, set_value_dispatch_updates_followup_components_query, set_reference_dispatch_updates_component_reference, set_reference_dispatch_updates_followup_components_query, move_component_dispatch_updates_component_position, move_component_dispatch_updates_followup_unrouted_query (outputs: {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
        save_test,
        roundtrip_test,
        delete_followup_test,
        via_roundtrip_test,
        via_followup_test,
        rule_test,
        rule_followup_test,
        value_test,
        value_followup_test,
        reference_test,
        reference_followup_test,
        move_test,
        move_derived_test
    ))
}

fn check_mcp_surface(cli: &Cli) -> SurfaceCheck {
    match mcp_surface_result(cli) {
        Ok(evidence) => SurfaceCheck {
            surface: "mcp_write_surface".to_string(),
            status: Status::Passed,
            evidence,
        },
        Err(err) => SurfaceCheck {
            surface: "mcp_write_surface".to_string(),
            status: Status::Failed,
            evidence: err.to_string(),
        },
    }
}

fn mcp_surface_result(cli: &Cli) -> Result<String> {
    let script = r#"
import importlib.util
import pathlib
import sys
import unittest

repo = pathlib.Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("datum_mcp_server", repo / "mcp-server" / "server.py")
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
sys.modules[spec.name] = module
spec.loader.exec_module(module)
suite = unittest.TestSuite()
loader = unittest.defaultTestLoader
for name in [
    "test_tools_call_dispatches_save",
    "test_tools_call_dispatches_delete_track",
    "test_tools_call_delete_track_changes_followup_check_report",
    "test_tools_call_dispatches_delete_via",
    "test_tools_call_delete_via_changes_followup_net_info_response",
    "test_tools_call_dispatches_move_component",
    "test_tools_call_move_component_changes_followup_unrouted_response",
    "test_tools_call_dispatches_set_design_rule",
    "test_tools_call_set_design_rule_changes_followup_design_rules_response",
    "test_tools_call_dispatches_set_value",
    "test_tools_call_set_value_changes_followup_components_response",
    "test_tools_call_dispatches_set_reference",
    "test_tools_call_set_reference_changes_followup_components_response",
    "test_tools_call_dispatches_undo_and_redo",
]:
    suite.addTests(loader.loadTestsFromName(f"ServerTests.{name}", module))
result = unittest.TextTestRunner(verbosity=0).run(suite)
if not result.wasSuccessful():
    sys.exit(1)
print("selected MCP write-surface dispatch tests passed")
"#;

    let output = run_command_checked(
        Command::new("python3")
            .arg("-c")
            .arg(script)
            .arg(&cli.repo_root)
            .current_dir(&cli.repo_root),
        "mcp write-surface parity probe",
    )?;

    Ok(output)
}

fn check_cli_surface(cli: &Cli) -> SurfaceCheck {
    match (
        cli_surface_result(cli),
        cli_via_surface_result(cli),
        cli_move_surface_result(cli),
        cli_rule_surface_result(cli),
        cli_value_surface_result(cli),
        cli_reference_surface_result(cli),
    ) {
        (
            Ok(track_evidence),
            Ok(via_evidence),
            Ok(move_evidence),
            Ok(rule_evidence),
            Ok(value_evidence),
            Ok(reference_evidence),
        ) => SurfaceCheck {
            surface: "cli_modify_surface".to_string(),
            status: Status::Passed,
            evidence: format!(
                "{track_evidence}; {via_evidence}; {move_evidence}; {rule_evidence}; {value_evidence}; {reference_evidence}"
            ),
        },
        (Err(err), _, _, _, _, _)
        | (_, Err(err), _, _, _, _)
        | (_, _, Err(err), _, _, _)
        | (_, _, _, Err(err), _, _)
        | (_, _, _, _, Err(err), _)
        | (_, _, _, _, _, Err(err)) => SurfaceCheck {
            surface: "cli_modify_surface".to_string(),
            status: Status::Failed,
            evidence: err.to_string(),
        },
    }
}

fn cli_surface_result(cli: &Cli) -> Result<String> {
    let roundtrip_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--delete-track")
        .arg(cli.track_uuid.to_string())
        .arg("--undo")
        .arg("1")
        .arg("--redo")
        .arg("1")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI modify parity probe")?;
    if !roundtrip_output.status.success() {
        bail!(
            "CLI roundtrip parity probe failed with status {:?}: {}",
            roundtrip_output.status.code(),
            String::from_utf8_lossy(&roundtrip_output.stderr).trim()
        );
    }
    let roundtrip: CliModifyReport = serde_json::from_slice(&roundtrip_output.stdout)
        .context("failed to parse CLI roundtrip JSON output")?;

    let expected_actions = vec![
        format!("delete_track {}", cli.track_uuid),
        "undo".to_string(),
        "redo".to_string(),
    ];
    if roundtrip.actions != expected_actions {
        bail!(
            "CLI roundtrip actions mismatch: expected {:?}, got {:?}",
            expected_actions,
            roundtrip.actions
        );
    }
    let expected_last_description = format!("redo delete_track {}", cli.track_uuid);
    if roundtrip
        .last_result
        .as_ref()
        .map(|result| result.description.as_str())
        != Some(expected_last_description.as_str())
    {
        bail!("CLI roundtrip last_result description mismatch");
    }

    let target = unique_temp_path("cli-surface-save", "kicad_pcb");
    let save_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--delete-track")
        .arg(cli.track_uuid.to_string())
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI save parity probe")?;
    if !save_output.status.success() {
        bail!(
            "CLI save parity probe failed with status {:?}: {}",
            save_output.status.code(),
            String::from_utf8_lossy(&save_output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&save_output.stdout)
        .context("failed to parse CLI save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    let reloaded_after_save = reloaded.get_net_info()?;
    if reloaded_after_save != after_delete_state(cli)? {
        bail!("CLI save did not persist the current deleted board state");
    }

    let check_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "check",
        ])
        .arg(saved_path)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI delete-track follow-up check")?;
    if !check_output.status.success() {
        bail!(
            "CLI delete-track follow-up check failed with status {:?}: {}",
            check_output.status.code(),
            String::from_utf8_lossy(&check_output.stderr).trim()
        );
    }
    let check_payload: Value = serde_json::from_slice(&check_output.stdout)
        .context("failed to parse CLI delete-track follow-up check JSON")?;
    let diagnostics = check_payload["diagnostics"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI delete-track follow-up check missing diagnostics"))?;
    if !diagnostics
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "net_without_copper")
    {
        bail!("CLI delete-track follow-up check missing net_without_copper");
    }

    Ok(format!(
        "roundtrip_last={}, saved={}, delete_then_save_persisted=true, delete_track_followup_check_changed=true",
        expected_last_description, saved_path
    ))
}

fn cli_via_surface_result(cli: &Cli) -> Result<String> {
    let via_fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let via_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
    let target = unique_temp_path("cli-surface-via-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&via_fixture)
        .arg("--delete-via")
        .arg(via_uuid.to_string())
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI via save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI via save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI via save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI via save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    let expected = after_delete_via_state(&via_fixture, via_uuid)?;
    if reloaded.get_net_info()? != expected {
        bail!("CLI via save did not persist the current deleted board state");
    }
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI delete-via follow-up net query")?;
    if !query_output.status.success() {
        bail!(
            "CLI delete-via follow-up net query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI delete-via follow-up net JSON")?;
    let nets = payload["nets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI delete-via follow-up net query missing nets"))?;
    let gnd = nets
        .iter()
        .find(|net| net["name"] == "GND")
        .ok_or_else(|| anyhow::anyhow!("CLI delete-via follow-up net query missing GND"))?;
    if gnd["vias"] != 0 {
        bail!("CLI delete-via follow-up net query did not reflect removed via");
    }
    Ok(format!(
        "via_saved={}, delete_via_then_save_persisted=true, delete_via_followup_net_info_changed=true",
        saved_path
    ))
}

fn cli_rule_surface_result(cli: &Cli) -> Result<String> {
    let fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let target = unique_temp_path("cli-surface-rule-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&fixture)
        .arg("--set-clearance-min-nm")
        .arg("125000")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI rule save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI rule save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI rule save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI rule save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    if reloaded.get_design_rules()?.len() != 1 {
        bail!("CLI rule save did not persist one design rule");
    }
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("design-rules")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI rule follow-up query")?;
    if !query_output.status.success() {
        bail!(
            "CLI rule follow-up query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI rule follow-up query JSON")?;
    let rules = payload["rules"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI rule follow-up query missing rules array"))?;
    if rules.len() != 1 || rules[0]["name"] != "default clearance" {
        bail!("CLI rule follow-up query did not reflect current design-rule state");
    }
    Ok(format!(
        "rule_saved={}, set_design_rule_then_save_persisted=true, rule_followup_query_changed=true",
        saved_path
    ))
}

fn cli_value_surface_result(cli: &Cli) -> Result<String> {
    let target = unique_temp_path("cli-surface-value-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--set-value")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:22k")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-value save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-value save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-value save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-value save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-value follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-value follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-value follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-value follow-up query missing components"))?;
    let r1 = components
        .iter()
        .find(|component| component["reference"] == "R1")
        .ok_or_else(|| anyhow::anyhow!("CLI set-value follow-up query missing R1"))?;
    if r1["value"] != "22k" {
        bail!("CLI set-value follow-up query did not reflect updated component value");
    }
    Ok(format!(
        "value_saved={}, set_value_then_save_persisted=true, set_value_followup_components_changed=true",
        saved_path
    ))
}

fn cli_reference_surface_result(cli: &Cli) -> Result<String> {
    let target = unique_temp_path("cli-surface-reference-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--set-reference")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:R10")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-reference save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-reference save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-reference save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-reference save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-reference follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-reference follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-reference follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-reference follow-up query missing components"))?;
    let target_component = components
        .iter()
        .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .ok_or_else(|| anyhow::anyhow!("CLI set-reference follow-up query missing target component"))?;
    if target_component["reference"] != "R10" {
        bail!("CLI set-reference follow-up query did not reflect updated component reference");
    }
    Ok(format!(
        "reference_saved={}, set_reference_then_save_persisted=true, set_reference_followup_components_changed=true",
        saved_path
    ))
}

fn cli_move_surface_result(cli: &Cli) -> Result<String> {
    let baseline_distance =
        cli_unrouted_distance(&cli.repo_root, &cli.roundtrip_board_fixture_path)?;
    let target = unique_temp_path("cli-surface-move-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--move-component")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:15:12:90")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI move save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI move save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI move save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI move save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    let moved = reloaded
        .get_components()?
        .into_iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("CLI move save missing R1 after reimport"))?;
    if moved.position.x != 15_000_000 || moved.position.y != 12_000_000 || moved.rotation != 90 {
        bail!("CLI move save did not persist expected moved component state");
    }
    let moved_distance = cli_unrouted_distance(&cli.repo_root, Path::new(saved_path))?;
    if moved_distance == baseline_distance {
        bail!("CLI follow-up query did not reflect moved-component derived state");
    }
    Ok(format!(
        "move_saved={}, move_component_then_save_persisted=true, cli_followup_unrouted_changed=true",
        saved_path
    ))
}

fn after_delete_state(cli: &Cli) -> Result<Vec<eda_engine::board::BoardNetInfo>> {
    let mut engine = Engine::new()?;
    engine.import(&cli.roundtrip_board_fixture_path)?;
    engine.delete_track(&cli.track_uuid)?;
    Ok(engine.get_net_info()?)
}

fn after_delete_via_state(
    fixture: &Path,
    via_uuid: Uuid,
) -> Result<Vec<eda_engine::board::BoardNetInfo>> {
    let mut engine = Engine::new()?;
    engine.import(fixture)?;
    engine.delete_via(&via_uuid)?;
    Ok(engine.get_net_info()?)
}

fn run_command_checked(command: &mut Command, label: &str) -> Result<String> {
    let output = command
        .output()
        .with_context(|| format!("failed to execute {label}"))?;
    if !output.status.success() {
        bail!(
            "{label} failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn cli_unrouted_distance(repo_root: &Path, board_path: &Path) -> Result<i64> {
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(board_path)
        .arg("unrouted")
        .current_dir(repo_root)
        .output()
        .context("failed to run CLI unrouted query")?;
    if !output.status.success() {
        bail!(
            "CLI unrouted query failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI unrouted JSON output")?;
    payload["airwires"][0]["distance_nm"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("CLI unrouted JSON missing first airwire distance"))
}

fn unique_temp_path(prefix: &str, extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{unique}.{extension}",
        std::process::id()
    ))
}

fn print_human(report: &Report) {
    println!("m3 write-surface parity preflight:");
    println!("  overall: {:?}", report.overall_status);
    for check in &report.checks {
        println!("  - {}: {:?}", check.surface, check.status);
        println!("    {}", check.evidence);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_temp_path_includes_prefix_and_extension() {
        let path = unique_temp_path("parity-test", "json");
        let text = path.display().to_string();
        assert!(text.contains("parity-test"));
        assert!(text.ends_with(".json"));
    }
}
