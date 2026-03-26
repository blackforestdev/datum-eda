use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use eda_engine::api::{AssignPartInput, Engine, SetNetClassInput, SetPackageInput};
use eda_test_harness::canonical_json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[path = "m3_write_surface_parity/m3_write_surface_cli.rs"]
mod m3_write_surface_cli;
#[path = "m3_write_surface_parity/m3_write_surface_common.rs"]
mod m3_write_surface_common;
#[path = "m3_write_surface_parity/m3_write_surface_daemon.rs"]
mod m3_write_surface_daemon;
#[path = "m3_write_surface_parity/m3_write_surface_engine.rs"]
mod m3_write_surface_engine;
#[path = "m3_write_surface_parity/m3_write_surface_mcp.rs"]
mod m3_write_surface_mcp;

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
    m3_write_surface_engine::engine_surface_result(cli)
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
    m3_write_surface_daemon::daemon_surface_result(cli)
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
    m3_write_surface_mcp::mcp_surface_result(cli)
}

fn check_cli_surface(cli: &Cli) -> SurfaceCheck {
    m3_write_surface_cli::check_cli_surface(cli)
}

fn after_delete_state(cli: &Cli) -> Result<Vec<eda_engine::board::BoardNetInfo>> {
    m3_write_surface_common::after_delete_state(cli)
}

fn after_delete_via_state(
    fixture: &Path,
    via_uuid: Uuid,
) -> Result<Vec<eda_engine::board::BoardNetInfo>> {
    m3_write_surface_common::after_delete_via_state(fixture, via_uuid)
}

fn cli_unrouted_distance(repo_root: &Path, board_path: &Path) -> Result<i64> {
    m3_write_surface_common::cli_unrouted_distance(repo_root, board_path)
}

fn unique_temp_path(prefix: &str, extension: &str) -> PathBuf {
    m3_write_surface_common::unique_temp_path(prefix, extension)
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
