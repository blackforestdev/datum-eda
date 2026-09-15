use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use eda_engine::api::Engine;
use eda_engine::erc::ErcSeverity;
use eda_engine::rules::ast::RuleType;
use eda_test_harness::{canonical_json, read_golden, testdata_path};
use serde::{Deserialize, Serialize};

const SPEC_ERC_FP_MAX_PCT: f64 = 5.0;
const SPEC_DRC_FP_MAX_PCT: f64 = 5.0;
const SPEC_ERC_FN_MAX_PCT: f64 = 0.0;
const SPEC_DRC_FN_MAX_PCT: f64 = 0.0;

#[derive(Debug, Clone)]
struct Cli {
    manifest: PathBuf,
    repo_root: Option<PathBuf>,
    json: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Manifest {
    schema_version: u32,
    corpus_min_designs: usize,
    erc_clean: Vec<String>,
    erc_violations: Vec<ViolationCase>,
    drc_clean: Vec<String>,
    drc_violations: Vec<ViolationCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ViolationCase {
    fixture: String,
    required_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DomainStats {
    clean_cases: usize,
    clean_failed: usize,
    expected_codes: usize,
    missing_codes: usize,
    false_positive_rate_pct: f64,
    false_negative_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CorpusStats {
    erc_goldens: usize,
    drc_goldens: usize,
    unique_designs: usize,
    required_min_designs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QualityReport {
    schema_version: u32,
    manifest_path: String,
    corpus: CorpusStats,
    erc: DomainStats,
    drc: DomainStats,
    spec_limits: QualityLimits,
    pass: bool,
    failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QualityLimits {
    erc_fp_max_pct: f64,
    erc_fn_max_pct: f64,
    drc_fp_max_pct: f64,
    drc_fn_max_pct: f64,
}

fn main() {
    match run() {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(err) => {
            eprintln!("m2_quality: {err:#}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32> {
    let cli = parse_args()?;
    let repo_root = match cli.repo_root {
        Some(root) => root,
        None => detect_repo_root()?,
    };
    let manifest = read_manifest(&cli.manifest)?;
    let report = build_report(&repo_root, &cli.manifest, &manifest)?;

    if cli.json {
        println!("{}", canonical_json(&report)?);
    } else {
        print_human_report(&report);
    }

    Ok(if report.pass { 0 } else { 1 })
}

fn parse_args() -> Result<Cli> {
    let mut manifest = testdata_path("quality/m2_quality_manifest.json");
    let mut repo_root = None;
    let mut json = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--manifest requires a path argument"))?;
                manifest = PathBuf::from(value);
            }
            "--repo-root" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--repo-root requires a path argument"))?;
                repo_root = Some(PathBuf::from(value));
            }
            "--json" => json = true,
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            unknown => bail!("unknown argument {unknown}"),
        }
    }

    Ok(Cli {
        manifest,
        repo_root,
        json,
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p eda-test-harness --bin m2_quality -- [options]\n\
         Options:\n\
           --manifest <path>    Quality manifest JSON (default: crate testdata)\n\
           --repo-root <path>   Repo root override\n\
           --json               Emit canonical JSON\n\
           -h, --help           Show this help"
    );
}

fn detect_repo_root() -> Result<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .context("failed to resolve repository root from test-harness crate")?;
    Ok(root)
}

fn read_manifest(path: &Path) -> Result<Manifest> {
    let text = read_golden(path).with_context(|| format!("failed to read {}", path.display()))?;
    let manifest: Manifest = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    if manifest.schema_version != 1 {
        bail!(
            "unsupported manifest schema_version {}; expected 1",
            manifest.schema_version
        );
    }
    Ok(manifest)
}

fn build_report(
    repo_root: &Path,
    manifest_path: &Path,
    manifest: &Manifest,
) -> Result<QualityReport> {
    let corpus = measure_corpus(repo_root, manifest.corpus_min_designs)?;
    let erc = measure_erc(repo_root, manifest)?;
    let drc = measure_drc(repo_root, manifest)?;

    let mut failures = Vec::new();
    if corpus.unique_designs < manifest.corpus_min_designs {
        failures.push(format!(
            "corpus unique designs {} is below required {}",
            corpus.unique_designs, manifest.corpus_min_designs
        ));
    }
    if erc.false_negative_rate_pct > SPEC_ERC_FN_MAX_PCT {
        failures.push(format!(
            "erc false-negative rate {:.2}% exceeds spec {:.2}%",
            erc.false_negative_rate_pct, SPEC_ERC_FN_MAX_PCT
        ));
    }
    if erc.false_positive_rate_pct > SPEC_ERC_FP_MAX_PCT {
        failures.push(format!(
            "erc false-positive rate {:.2}% exceeds spec {:.2}%",
            erc.false_positive_rate_pct, SPEC_ERC_FP_MAX_PCT
        ));
    }
    if drc.false_negative_rate_pct > SPEC_DRC_FN_MAX_PCT {
        failures.push(format!(
            "drc false-negative rate {:.2}% exceeds spec {:.2}%",
            drc.false_negative_rate_pct, SPEC_DRC_FN_MAX_PCT
        ));
    }
    if drc.false_positive_rate_pct > SPEC_DRC_FP_MAX_PCT {
        failures.push(format!(
            "drc false-positive rate {:.2}% exceeds spec {:.2}%",
            drc.false_positive_rate_pct, SPEC_DRC_FP_MAX_PCT
        ));
    }

    Ok(QualityReport {
        schema_version: 1,
        manifest_path: manifest_path.display().to_string(),
        corpus,
        erc,
        drc,
        spec_limits: QualityLimits {
            erc_fp_max_pct: SPEC_ERC_FP_MAX_PCT,
            erc_fn_max_pct: SPEC_ERC_FN_MAX_PCT,
            drc_fp_max_pct: SPEC_DRC_FP_MAX_PCT,
            drc_fn_max_pct: SPEC_DRC_FN_MAX_PCT,
        },
        pass: failures.is_empty(),
        failures,
    })
}

fn measure_corpus(repo_root: &Path, required_min_designs: usize) -> Result<CorpusStats> {
    let erc_dir = repo_root.join("crates/engine/testdata/golden/erc");
    let drc_dir = repo_root.join("crates/engine/testdata/golden/drc");
    let erc_goldens = count_json_files(&erc_dir)?;
    let drc_goldens = count_json_files(&drc_dir)?;

    let mut names = BTreeSet::new();
    for stem in list_fixture_stems(&erc_dir)? {
        names.insert(stem);
    }
    for stem in list_fixture_stems(&drc_dir)? {
        names.insert(stem);
    }

    Ok(CorpusStats {
        erc_goldens,
        drc_goldens,
        unique_designs: names.len(),
        required_min_designs,
    })
}

fn measure_erc(repo_root: &Path, manifest: &Manifest) -> Result<DomainStats> {
    let mut clean_failed = 0usize;
    for fixture in &manifest.erc_clean {
        let findings = run_erc_for_fixture(repo_root, fixture)?;
        let hard_findings = findings
            .iter()
            .filter(|f| matches!(f.severity, ErcSeverity::Error | ErcSeverity::Warning))
            .count();
        if hard_findings > 0 {
            clean_failed += 1;
        }
    }

    let mut expected_codes = 0usize;
    let mut missing_codes = 0usize;
    for case in &manifest.erc_violations {
        let findings = run_erc_for_fixture(repo_root, &case.fixture)?;
        let codes: BTreeSet<&str> = findings.iter().map(|f| f.code).collect();
        for required in &case.required_codes {
            expected_codes += 1;
            if !codes.contains(required.as_str()) {
                missing_codes += 1;
            }
        }
    }

    Ok(DomainStats {
        clean_cases: manifest.erc_clean.len(),
        clean_failed,
        expected_codes,
        missing_codes,
        false_positive_rate_pct: pct(clean_failed, manifest.erc_clean.len()),
        false_negative_rate_pct: pct(missing_codes, expected_codes),
    })
}

fn measure_drc(repo_root: &Path, manifest: &Manifest) -> Result<DomainStats> {
    let mut clean_failed = 0usize;
    for fixture in &manifest.drc_clean {
        let report = run_drc_for_fixture(repo_root, fixture)?;
        if !report.violations.is_empty() {
            clean_failed += 1;
        }
    }

    let mut expected_codes = 0usize;
    let mut missing_codes = 0usize;
    for case in &manifest.drc_violations {
        let report = run_drc_for_fixture(repo_root, &case.fixture)?;
        let codes: BTreeSet<&str> = report.violations.iter().map(|v| v.code.as_str()).collect();
        for required in &case.required_codes {
            expected_codes += 1;
            if !codes.contains(required.as_str()) {
                missing_codes += 1;
            }
        }
    }

    Ok(DomainStats {
        clean_cases: manifest.drc_clean.len(),
        clean_failed,
        expected_codes,
        missing_codes,
        false_positive_rate_pct: pct(clean_failed, manifest.drc_clean.len()),
        false_negative_rate_pct: pct(missing_codes, expected_codes),
    })
}

fn run_erc_for_fixture(
    repo_root: &Path,
    fixture: &str,
) -> Result<Vec<eda_engine::erc::ErcFinding>> {
    let path = repo_root
        .join("crates/engine/testdata/import/kicad")
        .join(fixture);
    let mut engine = Engine::new().context("failed to initialize engine for ERC run")?;
    engine
        .import(&path)
        .with_context(|| format!("failed to import schematic fixture {}", path.display()))?;
    engine
        .run_erc_prechecks()
        .with_context(|| format!("failed to run ERC on {}", path.display()))
}

fn run_drc_for_fixture(repo_root: &Path, fixture: &str) -> Result<eda_engine::drc::DrcReport> {
    let path = repo_root
        .join("crates/engine/testdata/import/kicad")
        .join(fixture);
    let mut engine = Engine::new().context("failed to initialize engine for DRC run")?;
    engine
        .import(&path)
        .with_context(|| format!("failed to import board fixture {}", path.display()))?;
    engine
        .run_drc(&[
            RuleType::Connectivity,
            RuleType::ClearanceCopper,
            RuleType::TrackWidth,
            RuleType::ViaHole,
            RuleType::ViaAnnularRing,
            RuleType::SilkClearance,
        ])
        .with_context(|| format!("failed to run DRC on {}", path.display()))
}

fn count_json_files(dir: &Path) -> Result<usize> {
    let mut count = 0usize;
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read entry in {}", dir.display()))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            count += 1;
        }
    }
    Ok(count)
}

fn list_fixture_stems(dir: &Path) -> Result<Vec<String>> {
    let mut stems = Vec::new();
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read entry in {}", dir.display()))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            stems.push(stem.to_string());
        }
    }
    Ok(stems)
}

fn pct(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        (numerator as f64) * 100.0 / (denominator as f64)
    }
}

fn print_human_report(report: &QualityReport) {
    println!("m2 quality:");
    println!(
        "  corpus: erc_goldens={} drc_goldens={} unique_designs={} required_min={}",
        report.corpus.erc_goldens,
        report.corpus.drc_goldens,
        report.corpus.unique_designs,
        report.corpus.required_min_designs
    );
    println!(
        "  erc: fp={:.2}% (clean_failed={}/{}) fn={:.2}% (missing={}/{})",
        report.erc.false_positive_rate_pct,
        report.erc.clean_failed,
        report.erc.clean_cases,
        report.erc.false_negative_rate_pct,
        report.erc.missing_codes,
        report.erc.expected_codes
    );
    println!(
        "  drc: fp={:.2}% (clean_failed={}/{}) fn={:.2}% (missing={}/{})",
        report.drc.false_positive_rate_pct,
        report.drc.clean_failed,
        report.drc.clean_cases,
        report.drc.false_negative_rate_pct,
        report.drc.missing_codes,
        report.drc.expected_codes
    );
    if report.failures.is_empty() {
        println!("  pass: true");
    } else {
        println!("  pass: false");
        for failure in &report.failures {
            println!("  failure: {failure}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pct_handles_zero_denominator() {
        assert_eq!(pct(1, 0), 0.0);
        assert_eq!(pct(0, 0), 0.0);
        assert_eq!(pct(1, 4), 25.0);
    }
}
