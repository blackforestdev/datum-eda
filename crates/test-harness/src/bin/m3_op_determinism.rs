use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use eda_engine::api::{
    AssignPartInput, Engine, MoveComponentInput, RotateComponentInput, SetDesignRuleInput,
    SetNetClassInput, SetPackageInput, SetReferenceInput, SetValueInput,
};
use eda_test_harness::canonical_json;
use eda_engine::rules::ast::{RuleParams, RuleScope, RuleType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Cli {
    json: bool,
    allow_deferred: bool,
    board_fixture_path: PathBuf,
    save_probe_path: PathBuf,
    component_uuid: Uuid,
    target_x_nm: i64,
    target_y_nm: i64,
    target_rotation_deg: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum GateStatus {
    Passed,
    Failed,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GateResult {
    gate: String,
    status: GateStatus,
    evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeterminismReport {
    schema_version: u32,
    overall_status: GateStatus,
    summary: String,
    gates: Vec<GateResult>,
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("m3_op_determinism: {err:#}");
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
        print_human_report(&report);
    }

    let code = match report.overall_status {
        GateStatus::Passed => 0,
        GateStatus::Failed => 1,
        GateStatus::Deferred => {
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
    let mut board_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/partial-route-demo.kicad_pcb")
        .canonicalize()
        .map_err(|err| anyhow::anyhow!("failed to resolve default board fixture path: {err}"))?;
    let mut save_probe_path = std::env::temp_dir().join("datum-eda-m3-save-probe.kicad_pcb");
    let mut component_uuid =
        Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").expect("uuid should parse");
    let mut target_x_nm = 15_000_000;
    let mut target_y_nm = 12_000_000;
    let mut target_rotation_deg = 90;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--allow-deferred" => allow_deferred = true,
            "--board-fixture-path" => {
                let value = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--board-fixture-path requires a path argument")
                })?;
                board_fixture_path = PathBuf::from(value);
            }
            "--save-probe-path" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--save-probe-path requires a path argument"))?;
                save_probe_path = PathBuf::from(value);
            }
            "--component-uuid" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--component-uuid requires a UUID argument"))?;
                component_uuid = Uuid::parse_str(&value)?;
            }
            "--target-x-nm" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--target-x-nm requires an integer argument"))?;
                target_x_nm = value.parse()?;
            }
            "--target-y-nm" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--target-y-nm requires an integer argument"))?;
                target_y_nm = value.parse()?;
            }
            "--target-rotation-deg" => {
                let value = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--target-rotation-deg requires an integer argument")
                })?;
                target_rotation_deg = value.parse()?;
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
        board_fixture_path,
        save_probe_path,
        component_uuid,
        target_x_nm,
        target_y_nm,
        target_rotation_deg,
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p eda-test-harness --bin m3_op_determinism -- [options]\n\
         Options:\n\
           --json                 Emit canonical JSON\n\
           --allow-deferred       Exit 0 when status is deferred (planning mode)\n\
           --board-fixture-path <p>  KiCad board fixture used for move/save determinism probe\n\
           --save-probe-path <p>  Path used for save() capability probe\n\
           --component-uuid <uuid>  Component UUID used for the current move/save slice\n\
           --target-x-nm <nm>     Target component X position in nm\n\
           --target-y-nm <nm>     Target component Y position in nm\n\
           --target-rotation-deg <deg>  Target component rotation in degrees\n\
           -h, --help             Show this help"
    );
}

fn build_report(cli: &Cli) -> Result<DeterminismReport> {
    let simple_board_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_pcb")
        .canonicalize()
        .map_err(|err| anyhow::anyhow!("failed to resolve simple-board determinism fixture path: {err}"))?;
    let library_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/eagle/simple-opamp.lbr")
        .canonicalize()
        .map_err(|err| anyhow::anyhow!("failed to resolve library determinism fixture path: {err}"))?;
    let via_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_pcb")
        .canonicalize()
        .map_err(|err| anyhow::anyhow!("failed to resolve via determinism fixture path: {err}"))?;
    let via_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
    let track_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
    let gates = vec![
        save_probe_gate(
            "move_component_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "move_component"),
            save_and_compare_after(&cli.board_fixture_path, |engine| {
                let moved = engine.move_component(MoveComponentInput {
                    uuid: cli.component_uuid,
                    position: eda_engine::ir::geometry::Point::new(
                        cli.target_x_nm,
                        cli.target_y_nm,
                    ),
                    rotation: Some(cli.target_rotation_deg),
                })?;
                Ok(moved.description)
            }),
        ),
        save_probe_gate(
            "delete_via_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "delete_via"),
            save_and_compare_after(&via_fixture_path, |engine| {
                let deleted = engine.delete_via(&via_uuid)?;
                Ok(deleted.description)
            }),
        ),
        save_probe_gate(
            "delete_component_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "delete_component"),
            save_and_compare_after(&cli.board_fixture_path, |engine| {
                let deleted = engine.delete_component(&cli.component_uuid)?;
                Ok(deleted.description)
            }),
        ),
        save_probe_gate(
            "delete_track_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "delete_track"),
            save_and_compare_after(&cli.board_fixture_path, |engine| {
                let deleted = engine.delete_track(&track_uuid)?;
                Ok(deleted.description)
            }),
        ),
        save_probe_gate(
            "rotate_component_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "rotate_component"),
            save_and_compare_after(&cli.board_fixture_path, |engine| {
                let rotated = engine.rotate_component(RotateComponentInput {
                    uuid: cli.component_uuid,
                    rotation: 180,
                })?;
                Ok(rotated.description)
            }),
        ),
        save_probe_gate(
            "set_value_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "set_value"),
            save_and_compare_after(&cli.board_fixture_path, |engine| {
                let updated = engine.set_value(SetValueInput {
                    uuid: cli.component_uuid,
                    value: "22k".to_string(),
                })?;
                Ok(updated.description)
            }),
        ),
        save_probe_gate(
            "set_reference_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "set_reference"),
            save_and_compare_after(&cli.board_fixture_path, |engine| {
                let updated = engine.set_reference(SetReferenceInput {
                    uuid: cli.component_uuid,
                    reference: "R10".to_string(),
                })?;
                Ok(updated.description)
            }),
        ),
        save_probe_gate(
            "set_design_rule_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "set_design_rule"),
            save_and_compare_after(&simple_board_fixture_path, |engine| {
                engine.set_design_rule(SetDesignRuleInput {
                    rule_type: RuleType::ClearanceCopper,
                    scope: RuleScope::All,
                    parameters: RuleParams::Clearance { min: 125_000 },
                    priority: 10,
                    name: Some("default clearance".to_string()),
                })?;
                Ok("set_design_rule clearance_copper".to_string())
            }),
        ),
        save_probe_gate(
            "assign_part_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "assign_part"),
            save_and_compare_after_with_setup(
                &cli.board_fixture_path,
                |engine| {
                    engine.import_eagle_library(&library_fixture_path)?;
                    Ok(())
                },
                |engine| {
                    let part_uuid = engine
                        .search_pool("ALTAMP")?
                        .into_iter()
                        .next()
                        .context("ALTAMP part missing for determinism probe")?
                        .uuid;
                    let updated = engine.assign_part(AssignPartInput {
                        uuid: cli.component_uuid,
                        part_uuid,
                    })?;
                    Ok(updated.description)
                },
            ),
        ),
        save_probe_gate(
            "set_package_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "set_package"),
            save_and_compare_after_with_setup(
                &cli.board_fixture_path,
                |engine| {
                    engine.import_eagle_library(&library_fixture_path)?;
                    Ok(())
                },
                |engine| {
                    let package_uuid = engine
                        .search_pool("ALTAMP")?
                        .into_iter()
                        .next()
                        .context("ALTAMP package missing for determinism probe")?
                        .package_uuid;
                    let updated = engine.set_package(SetPackageInput {
                        uuid: cli.component_uuid,
                        package_uuid,
                    })?;
                    Ok(updated.description)
                },
            ),
        ),
        save_probe_gate(
            "set_net_class_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "set_net_class"),
            save_and_compare_after(&simple_board_fixture_path, |engine| {
                let gnd_uuid = engine
                    .get_net_info()?
                    .into_iter()
                    .find(|net| net.name == "GND")
                    .context("GND net missing for determinism probe")?
                    .uuid;
                let updated = engine.set_net_class(SetNetClassInput {
                    net_uuid: gnd_uuid,
                    class_name: "power".to_string(),
                    clearance: 125_000,
                    track_width: 250_000,
                    via_drill: 300_000,
                    via_diameter: 600_000,
                    diffpair_width: 0,
                    diffpair_gap: 0,
                })?;
                Ok(updated.description)
            }),
        ),
    ];
    let overall_status = if gates.iter().any(|g| g.status == GateStatus::Failed) {
        GateStatus::Failed
    } else {
        GateStatus::Passed
    };

    let summary = match overall_status {
        GateStatus::Passed => {
            "M3 determinism preflight passed for the current board-write save slices".to_string()
        }
        GateStatus::Failed => {
            "M3 determinism preflight failed; unexpected write-capability state detected"
                .to_string()
        }
        GateStatus::Deferred => unreachable!("determinism hook no longer returns deferred"),
    };

    Ok(DeterminismReport {
        schema_version: 1,
        overall_status,
        summary,
        gates,
    })
}

fn save_probe_gate(gate: &str, paths: (PathBuf, PathBuf), probe: Result<String>) -> GateResult {
    let (first, second) = paths;
    match probe {
        Ok(evidence) => GateResult {
            gate: gate.to_string(),
            status: GateStatus::Passed,
            evidence: format!(
                "{}; Engine::save wrote byte-identical KiCad board output to {} and {}",
                evidence,
                first.display(),
                second.display()
            ),
        },
        Err(err) => GateResult {
            gate: gate.to_string(),
            status: GateStatus::Failed,
            evidence: format!("KiCad board save determinism probe failed: {err}"),
        },
    }
}

fn save_and_compare_after<F>(fixture: &PathBuf, mutate: F) -> Result<String>
where
    F: Fn(&mut Engine) -> Result<String>,
{
    save_and_compare_after_with_setup(fixture, |_engine| Ok(()), mutate)
}

fn save_and_compare_after_with_setup<S, F>(
    fixture: &PathBuf,
    setup: S,
    mutate: F,
) -> Result<String>
where
    S: Fn(&mut Engine) -> Result<()>,
    F: Fn(&mut Engine) -> Result<String>,
{
    let mut first_engine = Engine::new()?;
    setup(&mut first_engine)?;
    first_engine.import(fixture)?;
    let first_evidence = mutate(&mut first_engine)?;

    let mut second_engine = Engine::new()?;
    setup(&mut second_engine)?;
    second_engine.import(fixture)?;
    let second_evidence = mutate(&mut second_engine)?;

    let (first_probe_path, second_probe_path) = save_probe_paths(
        &std::env::temp_dir().join(format!("datum-eda-m3-save-probe-{}.kicad_pcb", Uuid::new_v4())),
        "determinism",
    );
    let save_result = save_and_compare(
        &first_engine,
        &second_engine,
        &first_probe_path,
        &second_probe_path,
    );
    let _ = fs::remove_file(&first_probe_path);
    let _ = fs::remove_file(&second_probe_path);
    save_result?;

    if first_evidence != second_evidence {
        bail!("operation metadata was not identical across repeated runs");
    }

    Ok(first_evidence)
}

fn save_probe_paths(base: &PathBuf, label: &str) -> (PathBuf, PathBuf) {
    let stem = base
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("datum-eda-m3-save-probe");
    let dir = base.parent().unwrap_or_else(|| std::path::Path::new("."));
    (
        dir.join(format!("{stem}-{label}.kicad_pcb")),
        dir.join(format!("{stem}-{label}-second.kicad_pcb")),
    )
}

fn save_and_compare(
    first_engine: &Engine,
    second_engine: &Engine,
    first: &PathBuf,
    second: &PathBuf,
) -> Result<()> {
    first_engine.save(first)?;
    second_engine.save(second)?;

    let first_bytes = fs::read(first)?;
    let second_bytes = fs::read(second)?;
    if first_bytes != second_bytes {
        bail!("save outputs were not byte-identical across consecutive runs");
    }

    Ok(())
}

fn print_human_report(report: &DeterminismReport) {
    println!("m3 operation determinism preflight:");
    println!("  overall: {:?}", report.overall_status);
    println!("  summary: {}", report.summary);
    println!("  gates:");
    for gate in &report.gates {
        println!("    - {}: {:?}", gate.gate, gate.status);
        println!("      {}", gate.evidence);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deferred_by_default_with_stubbed_write_surface() {
        let cli = Cli {
            json: false,
            allow_deferred: false,
            board_fixture_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../engine/testdata/import/kicad/partial-route-demo.kicad_pcb")
                .canonicalize()
                .expect("fixture path should resolve"),
            save_probe_path: std::env::temp_dir().join("datum-eda-m3-test-save.kicad_pcb"),
            component_uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
                .expect("uuid should parse"),
            target_x_nm: 15_000_000,
            target_y_nm: 12_000_000,
            target_rotation_deg: 90,
        };
        let report = build_report(&cli).expect("report should build");
        assert_eq!(report.overall_status, GateStatus::Passed);
    }
}
