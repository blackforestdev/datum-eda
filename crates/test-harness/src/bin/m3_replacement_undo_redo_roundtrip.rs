use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use eda_engine::api::{
    AssignPartInput, ComponentReplacementPolicy, ComponentReplacementScope, Engine,
    PlannedComponentReplacementInput, PolicyDrivenComponentReplacementInput,
    ReplaceComponentInput, ScopedComponentReplacementPolicyInput, SetPackageWithPartInput,
};
use eda_test_harness::canonical_json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Cli {
    json: bool,
    allow_deferred: bool,
    board_fixture_path: PathBuf,
    library_fixture_path: PathBuf,
    component_uuid: Uuid,
    second_component_uuid: Uuid,
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
            eprintln!("m3_replacement_undo_redo_roundtrip: {err:#}");
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
    let mut board_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/partial-route-demo.kicad_pcb")
        .canonicalize()
        .map_err(|err| anyhow::anyhow!("failed to resolve default board fixture path: {err}"))?;
    let mut library_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/eagle/simple-opamp.lbr")
        .canonicalize()
        .map_err(|err| anyhow::anyhow!("failed to resolve library fixture path: {err}"))?;
    let mut component_uuid =
        Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").expect("uuid should parse");
    let mut second_component_uuid =
        Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").expect("uuid should parse");

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
            "--library-fixture-path" => {
                let value = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--library-fixture-path requires a path argument")
                })?;
                library_fixture_path = PathBuf::from(value);
            }
            "--component-uuid" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--component-uuid requires a UUID argument"))?;
                component_uuid = Uuid::parse_str(&value)?;
            }
            "--second-component-uuid" => {
                let value = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--second-component-uuid requires a UUID argument")
                })?;
                second_component_uuid = Uuid::parse_str(&value)?;
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
        library_fixture_path,
        component_uuid,
        second_component_uuid,
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p eda-test-harness --bin m3_replacement_undo_redo_roundtrip -- [options]\n\
         Options:\n\
           --json                    Emit canonical JSON\n\
           --allow-deferred          Exit 0 when status is deferred\n\
           --board-fixture-path <p>  KiCad board fixture used for roundtrip checks\n\
           --library-fixture-path <p>  Library fixture used for replacement checks\n\
           --component-uuid <uuid>   Primary component UUID used for roundtrip checks\n\
           --second-component-uuid <uuid>  Secondary component UUID used for batched replacement checks\n\
           -h, --help                Show this help"
    );
}

fn build_report(cli: &Cli) -> Result<Report> {
    let checks = vec![
        replacement_roundtrip_check(
            "undo_redo_restores_set_package_with_part_state",
            run_set_package_with_part_roundtrip(cli),
        ),
        replacement_roundtrip_check(
            "undo_redo_restores_replace_component_state",
            run_replace_component_roundtrip(cli),
        ),
        replacement_roundtrip_check(
            "undo_redo_restores_replace_components_state",
            run_replace_components_roundtrip(cli),
        ),
        replacement_roundtrip_check(
            "undo_redo_restores_apply_component_replacement_plan_state",
            run_apply_component_replacement_plan_roundtrip(cli),
        ),
        replacement_roundtrip_check(
            "undo_redo_restores_apply_component_replacement_policy_state",
            run_apply_component_replacement_policy_roundtrip(cli),
        ),
        replacement_roundtrip_check(
            "undo_redo_restores_apply_scoped_component_replacement_policy_state",
            run_apply_scoped_component_replacement_policy_roundtrip(cli),
        ),
        replacement_roundtrip_check(
            "undo_redo_restores_apply_scoped_component_replacement_plan_state",
            run_apply_scoped_component_replacement_plan_roundtrip(cli),
        ),
    ];

    let overall_status = if checks.iter().any(|check| check.status == Status::Failed) {
        Status::Failed
    } else {
        Status::Passed
    };

    Ok(Report {
        schema_version: 1,
        overall_status,
        checks,
    })
}

fn replacement_roundtrip_check(name: &str, probe: Result<String>) -> Check {
    match probe {
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

fn run_set_package_with_part_roundtrip(cli: &Cli) -> Result<String> {
    run_component_roundtrip(cli, |engine, component_uuid, altamp, lmv321_uuid| {
        engine.assign_part(AssignPartInput {
            uuid: component_uuid,
            part_uuid: lmv321_uuid,
        })?;
        let baseline = engine.get_components()?;
        let result = engine.set_package_with_part(SetPackageWithPartInput {
            uuid: component_uuid,
            package_uuid: altamp.package_uuid,
            part_uuid: altamp.uuid,
        })?;
        Ok((baseline, result.description))
    })
}

fn run_replace_component_roundtrip(cli: &Cli) -> Result<String> {
    run_component_roundtrip(cli, |engine, component_uuid, altamp, lmv321_uuid| {
        engine.assign_part(AssignPartInput {
            uuid: component_uuid,
            part_uuid: lmv321_uuid,
        })?;
        let baseline = engine.get_components()?;
        let result = engine.replace_component(ReplaceComponentInput {
            uuid: component_uuid,
            package_uuid: altamp.package_uuid,
            part_uuid: altamp.uuid,
        })?;
        Ok((baseline, result.description))
    })
}

fn run_replace_components_roundtrip(cli: &Cli) -> Result<String> {
    run_component_roundtrip(cli, |engine, component_uuid, altamp, _lmv321_uuid| {
        let baseline = engine.get_components()?;
        let result = engine.replace_components(vec![
            ReplaceComponentInput {
                uuid: component_uuid,
                package_uuid: altamp.package_uuid,
                part_uuid: altamp.uuid,
            },
            ReplaceComponentInput {
                uuid: cli.second_component_uuid,
                package_uuid: altamp.package_uuid,
                part_uuid: altamp.uuid,
            },
        ])?;
        Ok((baseline, result.description))
    })
}

fn run_apply_component_replacement_plan_roundtrip(cli: &Cli) -> Result<String> {
    run_component_roundtrip(cli, |engine, component_uuid, altamp, lmv321_uuid| {
        engine.assign_part(AssignPartInput {
            uuid: component_uuid,
            part_uuid: lmv321_uuid,
        })?;
        let baseline = engine.get_components()?;
        let result = engine.apply_component_replacement_plan(vec![
            PlannedComponentReplacementInput {
                uuid: component_uuid,
                package_uuid: Some(altamp.package_uuid),
                part_uuid: None,
            },
        ])?;
        Ok((baseline, result.description))
    })
}

fn run_apply_component_replacement_policy_roundtrip(cli: &Cli) -> Result<String> {
    run_component_roundtrip(cli, |engine, component_uuid, _altamp, lmv321_uuid| {
        engine.assign_part(AssignPartInput {
            uuid: component_uuid,
            part_uuid: lmv321_uuid,
        })?;
        let baseline = engine.get_components()?;
        let result = engine.apply_component_replacement_policy(vec![
            PolicyDrivenComponentReplacementInput {
                uuid: component_uuid,
                policy: ComponentReplacementPolicy::BestCompatiblePackage,
            },
        ])?;
        Ok((baseline, result.description))
    })
}

fn run_apply_scoped_component_replacement_policy_roundtrip(cli: &Cli) -> Result<String> {
    run_component_roundtrip(cli, |engine, component_uuid, _altamp, lmv321_uuid| {
        engine.assign_part(AssignPartInput {
            uuid: component_uuid,
            part_uuid: lmv321_uuid,
        })?;
        let baseline = engine.get_components()?;
        let result = engine.apply_scoped_component_replacement_policy(
            ScopedComponentReplacementPolicyInput {
                scope: ComponentReplacementScope {
                    reference_prefix: Some("R1".to_string()),
                    value_equals: None,
                    current_package_uuid: None,
                    current_part_uuid: None,
                },
                policy: ComponentReplacementPolicy::BestCompatiblePackage,
            },
        )?;
        Ok((baseline, result.description))
    })
}

fn run_apply_scoped_component_replacement_plan_roundtrip(cli: &Cli) -> Result<String> {
    run_component_roundtrip(cli, |engine, component_uuid, _altamp, lmv321_uuid| {
        engine.assign_part(AssignPartInput {
            uuid: component_uuid,
            part_uuid: lmv321_uuid,
        })?;
        let baseline = engine.get_components()?;
        let plan =
            engine.get_scoped_component_replacement_plan(ScopedComponentReplacementPolicyInput {
                scope: ComponentReplacementScope {
                    reference_prefix: Some("R1".to_string()),
                    value_equals: None,
                    current_package_uuid: None,
                    current_part_uuid: None,
                },
                policy: ComponentReplacementPolicy::BestCompatiblePackage,
            })?;
        let result = engine.apply_scoped_component_replacement_plan(plan)?;
        Ok((baseline, result.description))
    })
}

fn run_component_roundtrip<F>(cli: &Cli, mutate: F) -> Result<String>
where
    F: FnOnce(
        &mut Engine,
        Uuid,
        &eda_engine::pool::PartSummary,
        Uuid,
    ) -> Result<(Vec<eda_engine::board::ComponentInfo>, String)>,
{
    let mut engine = Engine::new()?;
    engine.import_eagle_library(&cli.library_fixture_path)?;
    engine.import(&cli.board_fixture_path)?;
    let lmv321_uuid = engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .context("LMV321 part missing for replacement undo/redo probe")?
        .uuid;
    let altamp = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .context("ALTAMP part missing for replacement undo/redo probe")?;

    let (baseline, operation_description) = mutate(&mut engine, cli.component_uuid, &altamp, lmv321_uuid)?;
    let after_apply = engine.get_components()?;
    if after_apply == baseline {
        bail!("target operation did not change component state");
    }

    let undo_result = engine.undo()?;
    let after_undo = engine.get_components()?;
    let redo_result = engine.redo()?;
    let after_redo = engine.get_components()?;

    if after_undo != baseline {
        bail!("undo did not restore baseline component state");
    }
    if after_redo != after_apply {
        bail!("redo did not restore applied component state");
    }
    if !engine.can_undo() || engine.can_redo() {
        bail!("undo/redo stack flags were not restored after redo");
    }

    Ok(format!(
        "op={}, undo={}, redo={}, apply_changed_state=true, undo_restored=true, redo_restored=true, can_undo={}, can_redo={}",
        operation_description,
        undo_result.description,
        redo_result.description,
        engine.can_undo(),
        engine.can_redo()
    ))
}

fn print_human(report: &Report) {
    println!("m3 replacement undo/redo roundtrip:");
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
    fn replacement_roundtrip_report_passes_for_current_slice() {
        let cli = Cli {
            json: false,
            allow_deferred: false,
            board_fixture_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../engine/testdata/import/kicad/partial-route-demo.kicad_pcb")
                .canonicalize()
                .expect("fixture path should resolve"),
            library_fixture_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../engine/testdata/import/eagle/simple-opamp.lbr")
                .canonicalize()
                .expect("library path should resolve"),
            component_uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
                .expect("uuid should parse"),
            second_component_uuid: Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb")
                .expect("uuid should parse"),
        };
        let report = build_report(&cli).expect("report should build");
        assert_eq!(report.overall_status, Status::Passed);
    }
}
