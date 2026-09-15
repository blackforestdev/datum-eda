use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use eda_engine::api::Engine;
use eda_engine::rules::ast::RuleType;
use eda_test_harness::{canonical_json, write_golden};
use serde::{Deserialize, Serialize};

#[path = "m2_perf/m2_perf_helpers.rs"]
mod m2_perf_helpers;

use m2_perf_helpers::{
    compare_against_baseline, detect_repo_root, elapsed_ms, median_u64, parse_args,
    print_human_comparison, print_human_report, read_baseline, resolve_board_path,
    resolve_schematic_path,
};

const SPEC_LIMIT_ERC_MS: u64 = 3_000;
const SPEC_LIMIT_DRC_MS: u64 = 5_000;

#[derive(Debug, Clone)]
struct Cli {
    board_path: Option<PathBuf>,
    schematic_path: Option<PathBuf>,
    iterations: usize,
    compare_baseline: Option<PathBuf>,
    write_baseline: Option<PathBuf>,
    max_regression_pct: u64,
    json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerfSample {
    import_board_ms: u64,
    import_schematic_ms: u64,
    erc_ms: u64,
    drc_ms: u64,
    erc_findings: usize,
    drc_violations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerfMedians {
    import_board_ms: u64,
    import_schematic_ms: u64,
    erc_ms: u64,
    drc_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerfLimits {
    erc_ms: u64,
    drc_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerfReport {
    schema_version: u32,
    fixture: String,
    board_path: String,
    schematic_path: String,
    iterations: usize,
    max_regression_pct: u64,
    medians_ms: PerfMedians,
    limits_ms: PerfLimits,
    samples: Vec<PerfSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerfComparison {
    against: String,
    allowed_erc_ms: u64,
    allowed_drc_ms: u64,
    current_erc_ms: u64,
    current_drc_ms: u64,
    pass: bool,
    failures: Vec<String>,
}

fn main() {
    match run() {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(err) => {
            eprintln!("m2_perf: {err:#}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32> {
    let cli = parse_args()?;
    let repo_root = detect_repo_root()?;
    let board_path = resolve_board_path(cli.board_path.as_deref(), &repo_root)
        .context("unable to locate DOA2526 board fixture")?;
    let schematic_path = resolve_schematic_path(cli.schematic_path.as_deref(), &repo_root)
        .context("unable to locate DOA2526 schematic fixture")?;

    let mut samples = Vec::with_capacity(cli.iterations);
    for _ in 0..cli.iterations {
        samples.push(run_single_iteration(&board_path, &schematic_path)?);
    }

    let report = PerfReport {
        schema_version: 1,
        fixture: "DOA2526".into(),
        board_path: board_path.display().to_string(),
        schematic_path: schematic_path.display().to_string(),
        iterations: cli.iterations,
        max_regression_pct: cli.max_regression_pct,
        medians_ms: PerfMedians {
            import_board_ms: median_u64(samples.iter().map(|s| s.import_board_ms).collect()),
            import_schematic_ms: median_u64(
                samples.iter().map(|s| s.import_schematic_ms).collect(),
            ),
            erc_ms: median_u64(samples.iter().map(|s| s.erc_ms).collect()),
            drc_ms: median_u64(samples.iter().map(|s| s.drc_ms).collect()),
        },
        limits_ms: PerfLimits {
            erc_ms: SPEC_LIMIT_ERC_MS,
            drc_ms: SPEC_LIMIT_DRC_MS,
        },
        samples,
    };

    if cli.json {
        println!("{}", canonical_json(&report)?);
    } else {
        print_human_report(&report);
    }

    if let Some(path) = cli.write_baseline.as_deref() {
        let encoded = canonical_json(&report)?;
        write_golden(path, &encoded)
            .with_context(|| format!("failed to write baseline file {}", path.display()))?;
    }

    if let Some(path) = cli.compare_baseline.as_deref() {
        let baseline = read_baseline(path)?;
        let comparison = compare_against_baseline(&report, &baseline, cli.max_regression_pct, path);
        if cli.json {
            println!("{}", canonical_json(&comparison)?);
        } else {
            print_human_comparison(&comparison);
        }
        return Ok(if comparison.pass { 0 } else { 1 });
    }

    let mut failures = Vec::new();
    if report.medians_ms.erc_ms > SPEC_LIMIT_ERC_MS {
        failures.push(format!(
            "erc median {}ms exceeds spec limit {}ms",
            report.medians_ms.erc_ms, SPEC_LIMIT_ERC_MS
        ));
    }
    if report.medians_ms.drc_ms > SPEC_LIMIT_DRC_MS {
        failures.push(format!(
            "drc median {}ms exceeds spec limit {}ms",
            report.medians_ms.drc_ms, SPEC_LIMIT_DRC_MS
        ));
    }
    if failures.is_empty() {
        Ok(0)
    } else {
        for failure in failures {
            eprintln!("m2_perf: {failure}");
        }
        Ok(1)
    }
}

fn run_single_iteration(board_path: &Path, schematic_path: &Path) -> Result<PerfSample> {
    let mut board_engine = Engine::new().context("failed to create engine for board run")?;
    let board_import_start = Instant::now();
    board_engine
        .import(board_path)
        .with_context(|| format!("failed to import board {}", board_path.display()))?;
    let import_board_ms = elapsed_ms(board_import_start);

    let drc_start = Instant::now();
    let drc = board_engine
        .run_drc(&[RuleType::Connectivity, RuleType::ClearanceCopper])
        .context("failed to run DRC")?;
    let drc_ms = elapsed_ms(drc_start);

    let mut schematic_engine =
        Engine::new().context("failed to create engine for schematic run")?;
    let schematic_import_start = Instant::now();
    schematic_engine
        .import(schematic_path)
        .with_context(|| format!("failed to import schematic {}", schematic_path.display()))?;
    let import_schematic_ms = elapsed_ms(schematic_import_start);

    let erc_start = Instant::now();
    let erc_findings = schematic_engine
        .run_erc_prechecks()
        .context("failed to run ERC")?;
    let erc_ms = elapsed_ms(erc_start);

    Ok(PerfSample {
        import_board_ms,
        import_schematic_ms,
        erc_ms,
        drc_ms,
        erc_findings: erc_findings.len(),
        drc_violations: drc.violations.len(),
    })
}
