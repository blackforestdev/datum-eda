use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use eda_engine::api::Engine;
use eda_engine::rules::ast::RuleType;
use eda_test_harness::{canonical_json, read_golden, write_golden};
use serde::{Deserialize, Serialize};

const DEFAULT_ITERATIONS: usize = 3;
const DEFAULT_MAX_REGRESSION_PCT: u64 = 25;
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

fn parse_args() -> Result<Cli> {
    let mut board_path = None;
    let mut schematic_path = None;
    let mut iterations = DEFAULT_ITERATIONS;
    let mut compare_baseline = None;
    let mut write_baseline = None;
    let mut max_regression_pct = DEFAULT_MAX_REGRESSION_PCT;
    let mut json = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--board" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--board requires a path argument"))?;
                board_path = Some(PathBuf::from(value));
            }
            "--schematic" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--schematic requires a path argument"))?;
                schematic_path = Some(PathBuf::from(value));
            }
            "--iterations" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--iterations requires a value"))?;
                iterations = value
                    .parse::<usize>()
                    .with_context(|| format!("invalid --iterations value {value}"))?;
            }
            "--compare-baseline" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--compare-baseline requires a path"))?;
                compare_baseline = Some(PathBuf::from(value));
            }
            "--write-baseline" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--write-baseline requires a path"))?;
                write_baseline = Some(PathBuf::from(value));
            }
            "--max-regression-pct" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--max-regression-pct requires a value"))?;
                max_regression_pct = value
                    .parse::<u64>()
                    .with_context(|| format!("invalid --max-regression-pct value {value}"))?;
            }
            "--json" => json = true,
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            unknown => bail!("unknown argument {unknown}"),
        }
    }

    if iterations == 0 {
        bail!("--iterations must be >= 1");
    }

    Ok(Cli {
        board_path,
        schematic_path,
        iterations,
        compare_baseline,
        write_baseline,
        max_regression_pct,
        json,
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p eda-test-harness --bin m2_perf -- [options]\n\
         \n\
         Options:\n\
           --board <path>                 Board fixture path (.kicad_pcb)\n\
           --schematic <path>             Schematic fixture path (.kicad_sch)\n\
           --iterations <n>               Number of timing runs (default: 3)\n\
           --write-baseline <path>        Write canonical JSON baseline\n\
           --compare-baseline <path>      Compare current run to baseline\n\
           --max-regression-pct <n>       Allowed regression over baseline median (default: 25)\n\
           --json                         Emit canonical JSON\n\
           -h, --help                     Show this help"
    );
}

fn detect_repo_root() -> Result<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .context("failed to resolve repository root")?;
    Ok(path)
}

fn resolve_board_path(explicit: Option<&Path>, repo_root: &Path) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return ensure_exists(path.to_path_buf());
    }
    let env_path = std::env::var("DOA2526_BOARD_PATH").ok().map(PathBuf::from);
    if let Some(path) = env_path {
        return ensure_exists(path);
    }
    for candidate in doa2526_board_candidates(repo_root) {
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    bail!("DOA2526 board fixture not found in default locations")
}

fn resolve_schematic_path(explicit: Option<&Path>, repo_root: &Path) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return ensure_exists(path.to_path_buf());
    }
    let env_path = std::env::var("DOA2526_SCHEMATIC_PATH")
        .ok()
        .map(PathBuf::from);
    if let Some(path) = env_path {
        return ensure_exists(path);
    }
    for candidate in doa2526_schematic_candidates(repo_root) {
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    bail!("DOA2526 schematic fixture not found in default locations")
}

fn doa2526_board_candidates(repo_root: &Path) -> Vec<PathBuf> {
    let mut candidates =
        vec![repo_root.join("kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb")];
    if let Some(parent) = repo_root.parent() {
        candidates.push(parent.join("kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb"));
    }
    candidates
}

fn doa2526_schematic_candidates(repo_root: &Path) -> Vec<PathBuf> {
    let mut candidates =
        vec![repo_root.join("kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_sch")];
    if let Some(parent) = repo_root.parent() {
        candidates.push(parent.join("kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_sch"));
    }
    candidates
}

fn ensure_exists(path: PathBuf) -> Result<PathBuf> {
    if path.exists() {
        Ok(path)
    } else {
        bail!("fixture path does not exist: {}", path.display())
    }
}

fn read_baseline(path: &Path) -> Result<PerfReport> {
    let text = read_golden(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str::<PerfReport>(&text)
        .with_context(|| format!("failed to parse baseline JSON {}", path.display()))
}

fn compare_against_baseline(
    current: &PerfReport,
    baseline: &PerfReport,
    max_regression_pct: u64,
    baseline_path: &Path,
) -> PerfComparison {
    let allowed_erc_ms = allowance_with_regression(baseline.medians_ms.erc_ms, max_regression_pct)
        .max(current.limits_ms.erc_ms);
    let allowed_drc_ms = allowance_with_regression(baseline.medians_ms.drc_ms, max_regression_pct)
        .max(current.limits_ms.drc_ms);

    let mut failures = Vec::new();
    if current.medians_ms.erc_ms > allowed_erc_ms {
        failures.push(format!(
            "erc median {}ms exceeds allowed {}ms",
            current.medians_ms.erc_ms, allowed_erc_ms
        ));
    }
    if current.medians_ms.drc_ms > allowed_drc_ms {
        failures.push(format!(
            "drc median {}ms exceeds allowed {}ms",
            current.medians_ms.drc_ms, allowed_drc_ms
        ));
    }

    PerfComparison {
        against: baseline_path.display().to_string(),
        allowed_erc_ms,
        allowed_drc_ms,
        current_erc_ms: current.medians_ms.erc_ms,
        current_drc_ms: current.medians_ms.drc_ms,
        pass: failures.is_empty(),
        failures,
    }
}

fn allowance_with_regression(base_ms: u64, regression_pct: u64) -> u64 {
    let factor = 100 + regression_pct;
    (base_ms.saturating_mul(factor).saturating_add(99)) / 100
}

fn elapsed_ms(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

fn median_u64(mut values: Vec<u64>) -> u64 {
    values.sort_unstable();
    values[values.len() / 2]
}

fn print_human_report(report: &PerfReport) {
    println!("M2 performance report ({})", report.fixture);
    println!("board: {}", report.board_path);
    println!("schematic: {}", report.schematic_path);
    println!("iterations: {}", report.iterations);
    println!(
        "median ms: import_board={} import_schematic={} erc={} drc={}",
        report.medians_ms.import_board_ms,
        report.medians_ms.import_schematic_ms,
        report.medians_ms.erc_ms,
        report.medians_ms.drc_ms
    );
    println!(
        "spec limits ms: erc<={} drc<={}",
        report.limits_ms.erc_ms, report.limits_ms.drc_ms
    );
}

fn print_human_comparison(comparison: &PerfComparison) {
    println!("baseline: {}", comparison.against);
    println!(
        "allowed ms: erc<={} drc<={}",
        comparison.allowed_erc_ms, comparison.allowed_drc_ms
    );
    println!(
        "current ms: erc={} drc={}",
        comparison.current_erc_ms, comparison.current_drc_ms
    );
    if comparison.pass {
        println!("result: pass");
    } else {
        println!("result: fail");
        for failure in &comparison.failures {
            println!("failure: {failure}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regression_allowance_rounds_up() {
        assert_eq!(allowance_with_regression(1000, 25), 1250);
        assert_eq!(allowance_with_regression(1001, 25), 1252);
    }

    #[test]
    fn median_is_stable_for_sorted_and_unsorted_inputs() {
        assert_eq!(median_u64(vec![3, 2, 1]), 2);
        assert_eq!(median_u64(vec![1, 10, 3, 6, 8]), 6);
    }

    #[test]
    fn doa2526_candidates_include_repo_and_parent_locations() {
        let repo_root = PathBuf::from("/tmp/work/datum-eda");
        let board = doa2526_board_candidates(&repo_root);
        let schematic = doa2526_schematic_candidates(&repo_root);

        assert_eq!(
            board[0],
            PathBuf::from(
                "/tmp/work/datum-eda/kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb"
            )
        );
        assert_eq!(
            board[1],
            PathBuf::from("/tmp/work/kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_pcb")
        );
        assert_eq!(
            schematic[0],
            PathBuf::from(
                "/tmp/work/datum-eda/kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_sch"
            )
        );
        assert_eq!(
            schematic[1],
            PathBuf::from("/tmp/work/kicad_projects/DOA2526/hardware/DOA2526/DOA2526.kicad_sch")
        );
    }
}
