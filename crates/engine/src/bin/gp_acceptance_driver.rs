//! GP-CM05E engine-surface acceptance driver.
//!
//! The acceptance contract makes some resolver states (managed provider
//! generation states) reachable only inside the engine — no CLI/MCP/GUI adapter
//! can inject them. This thin binary is the `engine` surface the production
//! acceptance corpus pins as `candidate.binaries["engine"]`: it builds a TEST
//! PRECONDITION through [`acceptance_support`], resolves it through the ordinary
//! engine resolver, and prints the honest observation booleans for one coverage
//! row as a bare JSON object on stdout. The committed Python recipe
//! (`scripts/global_preferences_acceptance/recipes/preferences_case.py`) invokes
//! this binary for `surface == "engine"` and wraps its output into the unchanged
//! `datum.preferences.case-observations.v2` record — the engine binary emits only
//! the `{assertion_id: bool}` dict.
//!
//! It writes nothing: the managed-offline family is a pure resolver evaluation,
//! so no store is created or mutated and the one-mutation-path law is not engaged.
//! Observations are reported as data; a false result is never massaged to true —
//! the governed per-case validator adjudicates it.

use std::process::ExitCode;

use eda_engine::preferences::acceptance_support;
use serde_json::{Map, Value};

/// Read the value of a `--flag value` or `--flag=value` option from argv.
fn arg_value(flag: &str) -> Option<String> {
    let mut args = std::env::args();
    let eq_prefix = format!("{flag}=");
    while let Some(arg) = args.next() {
        if arg == flag {
            return args.next();
        }
        if let Some(rest) = arg.strip_prefix(&eq_prefix) {
            return Some(rest.to_owned());
        }
    }
    None
}

fn main() -> ExitCode {
    let case = arg_value("--case").unwrap_or_default();
    let variant = arg_value("--variant").unwrap_or_default();

    let observations = match (case.as_str(), variant.as_str()) {
        ("managed-offline", "inactive") => {
            let scenario = acceptance_support::managed_offline_inactive_scenario();
            let explanation = scenario.resolve();
            acceptance_support::observe_managed_offline(&explanation, &scenario)
        }
        _ => {
            eprintln!("gp_acceptance_driver: unimplemented engine coverage '{case}/{variant}'");
            return ExitCode::FAILURE;
        }
    };

    let mut object = Map::new();
    for (id, value) in &observations {
        object.insert((*id).to_owned(), Value::Bool(*value));
    }
    println!(
        "{}",
        serde_json::to_string(&Value::Object(object)).expect("observations serialize to JSON")
    );
    eprintln!("Ran {} observations", observations.len());
    ExitCode::SUCCESS
}
