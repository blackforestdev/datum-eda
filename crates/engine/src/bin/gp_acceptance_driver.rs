//! GP-CM05E engine-surface acceptance driver.
//!
//! The acceptance contract makes some engine states reachable only inside the
//! engine — managed provider generation states (resolver-level) and committed-store
//! recovery flows — with no CLI/MCP/GUI adapter able to drive them. This thin binary
//! is the `engine` surface the production acceptance corpus pins as
//! `candidate.binaries["engine"]`: it builds a TEST PRECONDITION through
//! [`acceptance_support`] and prints the honest observation booleans for one coverage
//! row as a bare JSON object on stdout. The committed engine recipe
//! (`scripts/global_preferences_acceptance/recipes/preferences_engine_case.py`)
//! invokes it for `surface == "engine"` and wraps the output into the unchanged
//! `datum.preferences.case-observations.v2` record — the binary emits only the
//! `{assertion_id: bool}` dict.
//!
//! Two phases: the default `observe` phase computes and prints the observation dict;
//! a write family's `--phase setup` establishes its on-disk precondition under
//! `DATUM_EDA_WORK_ROOT` (so the producer can snapshot the real `before` state) and
//! prints nothing. The managed-offline family is a pure resolver read (no store);
//! the backup-restore family goes through the engine's OWN committed-store path
//! (product set_user + the repository recovery API) — never a private writer.
//! Observations are reported as data; a false result is never massaged to true.

use std::path::PathBuf;
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

/// The acceptance work root a write family operates in (producer-provided, as the
/// CLI families receive it).
fn work_root() -> Result<PathBuf, String> {
    std::env::var("DATUM_EDA_WORK_ROOT")
        .map(PathBuf::from)
        .map_err(|_| "DATUM_EDA_WORK_ROOT must be set for a write-family phase".to_owned())
}

fn main() -> ExitCode {
    let case = arg_value("--case").unwrap_or_default();
    let variant = arg_value("--variant").unwrap_or_default();
    let phase = arg_value("--phase").unwrap_or_else(|| "observe".to_owned());

    // Setup phase (write families only): establish the on-disk precondition and
    // print nothing; the producer snapshots the resulting store as `before`.
    if phase == "setup" {
        let result = match case.as_str() {
            "backup-restore" => {
                match acceptance_support::backup_restore::BackupRestoreVariant::parse(&variant) {
                    Some(backup_variant) => work_root().and_then(|root| {
                        acceptance_support::backup_restore::backup_restore_setup(
                            backup_variant,
                            &root,
                        )
                    }),
                    None => {
                        eprintln!("gp_acceptance_driver: no setup phase for '{case}/{variant}'");
                        return ExitCode::FAILURE;
                    }
                }
            }
            "downgrade-preservation" => {
                match acceptance_support::backup_restore::DowngradeVariant::parse(&variant) {
                    Some(downgrade_variant) => work_root().and_then(|root| {
                        acceptance_support::backup_restore::downgrade_setup(
                            downgrade_variant,
                            &root,
                        )
                    }),
                    None => {
                        eprintln!("gp_acceptance_driver: no setup phase for '{case}/{variant}'");
                        return ExitCode::FAILURE;
                    }
                }
            }
            _ => {
                eprintln!("gp_acceptance_driver: no setup phase for '{case}/{variant}'");
                return ExitCode::FAILURE;
            }
        };
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("gp_acceptance_driver: setup '{case}/{variant}' failed: {error}");
                ExitCode::FAILURE
            }
        };
    }

    // Observe phase: compute the observation booleans for the coverage row.
    let observations: Vec<(&'static str, bool)> = match case.as_str() {
        "managed-offline" => match acceptance_support::ManagedOfflineVariant::parse(&variant) {
            Some(managed_variant) => {
                let scenario = acceptance_support::managed_offline_scenario(managed_variant);
                let explanation = scenario.resolve();
                acceptance_support::observe_managed_offline(&explanation, &scenario)
            }
            None => {
                eprintln!("gp_acceptance_driver: unimplemented engine coverage '{case}/{variant}'");
                return ExitCode::FAILURE;
            }
        },
        "backup-restore" => {
            match acceptance_support::backup_restore::BackupRestoreVariant::parse(&variant) {
                Some(backup_variant) => {
                    match work_root().and_then(|root| {
                        acceptance_support::backup_restore::backup_restore_observe(
                            backup_variant,
                            &root,
                        )
                    }) {
                        Ok(observed) => observed,
                        Err(error) => {
                            eprintln!(
                                "gp_acceptance_driver: observe '{case}/{variant}' failed: {error}"
                            );
                            return ExitCode::FAILURE;
                        }
                    }
                }
                None => {
                    eprintln!(
                        "gp_acceptance_driver: unimplemented engine coverage '{case}/{variant}'"
                    );
                    return ExitCode::FAILURE;
                }
            }
        }
        "downgrade-preservation" => {
            match acceptance_support::backup_restore::DowngradeVariant::parse(&variant) {
                Some(downgrade_variant) => {
                    match work_root().and_then(|root| {
                        acceptance_support::backup_restore::downgrade_observe(
                            downgrade_variant,
                            &root,
                        )
                    }) {
                        Ok(observed) => observed,
                        Err(error) => {
                            eprintln!(
                                "gp_acceptance_driver: observe '{case}/{variant}' failed: {error}"
                            );
                            return ExitCode::FAILURE;
                        }
                    }
                }
                None => {
                    eprintln!(
                        "gp_acceptance_driver: unimplemented engine coverage '{case}/{variant}'"
                    );
                    return ExitCode::FAILURE;
                }
            }
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
