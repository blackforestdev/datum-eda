use std::fs;
use std::path::PathBuf;

use anyhow::{Result, bail};
use eda_engine::api::{Engine, MoveComponentInput};
use eda_test_harness::canonical_json;
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
    let mut engine = Engine::new()?;
    engine.import(&cli.board_fixture_path)?;
    let moved = engine.move_component(MoveComponentInput {
        uuid: cli.component_uuid,
        position: eda_engine::ir::geometry::Point::new(cli.target_x_nm, cli.target_y_nm),
        rotation: Some(cli.target_rotation_deg),
    })?;

    let second_probe_path = cli.save_probe_path.with_file_name(format!(
        "{}-second.kicad_pcb",
        cli.save_probe_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("datum-eda-m3-save-probe")
    ));

    let save_probe = match save_and_compare(&engine, &cli.save_probe_path, &second_probe_path) {
        Ok(_) => GateResult {
            gate: "save_byte_determinism".to_string(),
            status: GateStatus::Passed,
            evidence: format!(
                "{}; Engine::save wrote byte-identical KiCad board output to {} and {}",
                moved.description,
                cli.save_probe_path.display(),
                second_probe_path.display()
            ),
        },
        Err(err) => GateResult {
            gate: "save_byte_determinism".to_string(),
            status: GateStatus::Failed,
            evidence: format!("KiCad board save determinism probe failed: {err}"),
        },
    };

    let _ = fs::remove_file(&cli.save_probe_path);
    let _ = fs::remove_file(&second_probe_path);

    let gates = vec![save_probe];
    let overall_status = if gates.iter().any(|g| g.status == GateStatus::Failed) {
        GateStatus::Failed
    } else {
        GateStatus::Passed
    };

    let summary = match overall_status {
        GateStatus::Passed => {
            "M3 determinism preflight passed for the current move_component/save write slice"
                .to_string()
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

fn save_and_compare(engine: &Engine, first: &PathBuf, second: &PathBuf) -> Result<()> {
    engine.save(first)?;
    engine.save(second)?;

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
