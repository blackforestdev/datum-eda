//! Generator/drift-gate for `mcp-server/datum_tool_catalog.json`.
//!
//! `--write` regenerates the checked-in catalog; `--check` diffs the registry
//! projection against the committed file in memory and exits 1 on drift
//! (registered in `scripts/run_drift_gates.sh`).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use datum_verb_registry::catalog_string;

fn catalog_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mcp-server/datum_tool_catalog.json")
}

fn check(expected: &str, path: &Path) -> ExitCode {
    let committed = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!(
                "datum-verb-catalog --check: cannot read {}: {err}\nrun with --write to generate it",
                path.display()
            );
            return ExitCode::FAILURE;
        }
    };
    if committed == expected {
        println!(
            "datum-verb-catalog --check: {} matches the verb registry",
            path.display()
        );
        return ExitCode::SUCCESS;
    }
    eprintln!(
        "datum-verb-catalog --check: {} drifted from the verb registry:",
        path.display()
    );
    let committed_lines: Vec<&str> = committed.lines().collect();
    let expected_lines: Vec<&str> = expected.lines().collect();
    let mut shown = 0usize;
    for (index, (committed_line, expected_line)) in committed_lines
        .iter()
        .zip(expected_lines.iter())
        .enumerate()
    {
        if committed_line != expected_line {
            eprintln!("  line {}:", index + 1);
            eprintln!("    committed: {committed_line}");
            eprintln!("    registry:  {expected_line}");
            shown += 1;
            if shown >= 10 {
                eprintln!("  ... (further differences elided)");
                break;
            }
        }
    }
    if committed_lines.len() != expected_lines.len() {
        eprintln!(
            "  line count: committed {} vs registry {}",
            committed_lines.len(),
            expected_lines.len()
        );
    }
    eprintln!(
        "run `cargo run -p datum-verb-registry --bin datum-verb-catalog -- --write` to regenerate"
    );
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    let mode = std::env::args().nth(1);
    let path = catalog_path();
    let rendered = catalog_string();
    match mode.as_deref() {
        Some("--write") => {
            if let Err(err) = std::fs::write(&path, rendered) {
                eprintln!(
                    "datum-verb-catalog --write: cannot write {}: {err}",
                    path.display()
                );
                return ExitCode::FAILURE;
            }
            println!("datum-verb-catalog --write: wrote {}", path.display());
            ExitCode::SUCCESS
        }
        Some("--check") => check(&rendered, &path),
        _ => {
            eprintln!("usage: datum-verb-catalog --write | --check");
            ExitCode::FAILURE
        }
    }
}
