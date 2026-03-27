use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use eda_test_harness::canonical_json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
struct Cli {
    json: bool,
    allow_deferred: bool,
    repo_root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Passed,
    Failed,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Check {
    name: String,
    status: Status,
    evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Report {
    schema_version: u32,
    overall_status: Status,
    checks: Vec<Check>,
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("m3_acceptance_gate: {err:#}");
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
    let mut repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .context("failed to resolve repository root")?;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--allow-deferred" => allow_deferred = true,
            "--repo-root" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--repo-root requires a path argument"))?;
                repo_root = PathBuf::from(value);
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
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p eda-test-harness --bin m3_acceptance_gate -- [options]\n\
         Options:\n\
           --json             Emit canonical JSON\n\
           --allow-deferred   Exit 0 when status is deferred\n\
           --repo-root <p>    Repository root used for sub-harness execution\n\
           -h, --help         Show this help"
    );
}

fn build_report(cli: &Cli) -> Result<Report> {
    let checks = vec![
        run_subharness(
            cli,
            "m3_op_determinism",
            "determinism",
            "current board-write save slices remain byte-deterministic",
        ),
        run_subharness(
            cli,
            "m3_replacement_op_determinism",
            "replacement_determinism",
            "current replacement-family save slices remain byte-deterministic",
        ),
        run_subharness(
            cli,
            "m3_undo_redo_roundtrip",
            "undo_redo",
            "current transaction stack remains round-trip safe for covered M3 ops",
        ),
        run_subharness(
            cli,
            "m3_replacement_undo_redo_roundtrip",
            "replacement_undo_redo",
            "current replacement-family transactions remain round-trip safe",
        ),
        run_subharness(
            cli,
            "m3_board_roundtrip_fidelity",
            "board_roundtrip_fidelity",
            "current pure board-write save paths remain stable across save→reimport→save",
        ),
        run_subharness(
            cli,
            "m3_sidecar_roundtrip_fidelity",
            "sidecar_roundtrip_fidelity",
            "current sidecar-backed save paths remain stable across save→reimport→save",
        ),
        run_subharness(
            cli,
            "m3_write_surface_parity",
            "write_surface_parity",
            "current engine/daemon/MCP/CLI write surfaces remain behaviorally aligned",
        ),
    ];

    let overall_status = if checks.iter().any(|check| check.status == Status::Failed) {
        Status::Failed
    } else if checks.iter().any(|check| check.status == Status::Deferred) {
        Status::Deferred
    } else {
        Status::Passed
    };

    Ok(Report {
        schema_version: 1,
        overall_status,
        checks,
    })
}

fn run_subharness(cli: &Cli, bin: &str, name: &str, success_summary: &str) -> Check {
    match run_subharness_inner(cli, bin, success_summary) {
        Ok(evidence) => Check {
            name: name.to_string(),
            status: Status::Passed,
            evidence,
        },
        Err(err) => Check {
            name: name.to_string(),
            status: Status::Failed,
            evidence: err.to_string(),
        },
    }
}

fn run_subharness_inner(cli: &Cli, bin: &str, success_summary: &str) -> Result<String> {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "eda-test-harness",
            "--bin",
            bin,
            "--",
            "--json",
        ])
        .current_dir(&cli.repo_root)
        .output()
        .with_context(|| format!("failed to execute {bin}"))?;

    if !output.status.success() {
        bail!(
            "{bin} failed with status {:?}: {}{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim(),
            if output.stdout.is_empty() {
                "".to_string()
            } else {
                format!(
                    " | stdout: {}",
                    String::from_utf8_lossy(&output.stdout).trim()
                )
            }
        );
    }

    let payload: Value = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("failed to parse {bin} JSON output"))?;
    let overall = payload
        .get("overall_status")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{bin} JSON missing overall_status"))?;
    if overall != "passed" {
        bail!("{bin} returned non-passed overall_status={overall}");
    }

    let passed_count = payload
        .get("checks")
        .or_else(|| payload.get("gates"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| item.get("status").and_then(Value::as_str) == Some("passed"))
                .count()
        })
        .unwrap_or(0);

    Ok(format!(
        "{success_summary}; subchecks_passed={passed_count}"
    ))
}

fn print_human(report: &Report) {
    println!("m3 acceptance gate:");
    println!("  overall: {:?}", report.overall_status);
    for check in &report.checks {
        println!("  - {}: {:?}", check.name, check.status);
        println!("    {}", check.evidence);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acceptance_gate_passes_against_current_checkpoint() {
        let cli = Cli {
            json: false,
            allow_deferred: false,
            repo_root: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../")
                .canonicalize()
                .expect("repo root should resolve"),
        };

        let report = build_report(&cli).expect("report should build");
        assert_eq!(report.overall_status, Status::Passed);
        assert_eq!(report.checks.len(), 7);
    }
}
