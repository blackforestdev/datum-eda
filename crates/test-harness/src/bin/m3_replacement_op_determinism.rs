use std::fs;
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
    save_probe_path: PathBuf,
    library_fixture_path: PathBuf,
    component_uuid: Uuid,
    second_component_uuid: Uuid,
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
            eprintln!("m3_replacement_op_determinism: {err:#}");
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
    let mut save_probe_path =
        std::env::temp_dir().join("datum-eda-m3-replacement-save-probe.kicad_pcb");
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
            "--save-probe-path" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--save-probe-path requires a path argument"))?;
                save_probe_path = PathBuf::from(value);
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
        save_probe_path,
        library_fixture_path,
        component_uuid,
        second_component_uuid,
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p eda-test-harness --bin m3_replacement_op_determinism -- [options]\n\
         Options:\n\
           --json                    Emit canonical JSON\n\
           --allow-deferred          Exit 0 when status is deferred\n\
           --board-fixture-path <p>  KiCad board fixture used for replacement determinism probes\n\
           --save-probe-path <p>     Path used for save() capability probes\n\
           --library-fixture-path <p> Pool library fixture for replacement checks\n\
           --component-uuid <uuid>   Primary component UUID used for replacement checks\n\
           --second-component-uuid <uuid> Secondary component UUID used for batched replacement checks\n\
           -h, --help                Show this help"
    );
}

fn build_report(cli: &Cli) -> Result<DeterminismReport> {
    let gates = vec![
        save_probe_gate(
            "set_package_with_part_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "set_package_with_part"),
            save_and_compare_after_with_setup(
                &cli.board_fixture_path,
                |engine| {
                    engine.import_eagle_library(&cli.library_fixture_path)?;
                    Ok(())
                },
                |engine| {
                    let lmv321_part_uuid = engine
                        .search_pool("LMV321")?
                        .into_iter()
                        .next()
                        .context("LMV321 part missing for explicit package determinism probe")?
                        .uuid;
                    let altamp = engine
                        .search_pool("ALTAMP")?
                        .into_iter()
                        .next()
                        .context("ALTAMP part missing for explicit package determinism probe")?;
                    engine.assign_part(AssignPartInput {
                        uuid: cli.component_uuid,
                        part_uuid: lmv321_part_uuid,
                    })?;
                    let updated = engine.set_package_with_part(SetPackageWithPartInput {
                        uuid: cli.component_uuid,
                        package_uuid: altamp.package_uuid,
                        part_uuid: altamp.uuid,
                    })?;
                    Ok(updated.description)
                },
            ),
        ),
        save_probe_gate(
            "replace_component_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "replace_component"),
            save_and_compare_after_with_setup(
                &cli.board_fixture_path,
                |engine| {
                    engine.import_eagle_library(&cli.library_fixture_path)?;
                    Ok(())
                },
                |engine| {
                    let lmv321_part_uuid = engine
                        .search_pool("LMV321")?
                        .into_iter()
                        .next()
                        .context("LMV321 part missing for replace_component determinism probe")?
                        .uuid;
                    let altamp = engine
                        .search_pool("ALTAMP")?
                        .into_iter()
                        .next()
                        .context("ALTAMP part missing for replace_component determinism probe")?;
                    engine.assign_part(AssignPartInput {
                        uuid: cli.component_uuid,
                        part_uuid: lmv321_part_uuid,
                    })?;
                    let updated = engine.replace_component(ReplaceComponentInput {
                        uuid: cli.component_uuid,
                        package_uuid: altamp.package_uuid,
                        part_uuid: altamp.uuid,
                    })?;
                    Ok(updated.description)
                },
            ),
        ),
        save_probe_gate(
            "replace_components_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "replace_components"),
            save_and_compare_after_with_setup(
                &cli.board_fixture_path,
                |engine| {
                    engine.import_eagle_library(&cli.library_fixture_path)?;
                    Ok(())
                },
                |engine| {
                    let altamp = engine
                        .search_pool("ALTAMP")?
                        .into_iter()
                        .next()
                        .context("ALTAMP part missing for replace_components determinism probe")?;
                    let updated = engine.replace_components(vec![
                        ReplaceComponentInput {
                            uuid: cli.component_uuid,
                            package_uuid: altamp.package_uuid,
                            part_uuid: altamp.uuid,
                        },
                        ReplaceComponentInput {
                            uuid: cli.second_component_uuid,
                            package_uuid: altamp.package_uuid,
                            part_uuid: altamp.uuid,
                        },
                    ])?;
                    Ok(updated.description)
                },
            ),
        ),
        save_probe_gate(
            "apply_component_replacement_plan_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "apply_component_replacement_plan"),
            save_and_compare_after_with_setup(
                &cli.board_fixture_path,
                |engine| {
                    engine.import_eagle_library(&cli.library_fixture_path)?;
                    Ok(())
                },
                |engine| {
                    let lmv321_part_uuid = engine
                        .search_pool("LMV321")?
                        .into_iter()
                        .next()
                        .context("LMV321 part missing for replacement-plan determinism probe")?
                        .uuid;
                    let altamp = engine
                        .search_pool("ALTAMP")?
                        .into_iter()
                        .next()
                        .context("ALTAMP part missing for replacement-plan determinism probe")?;
                    engine.assign_part(AssignPartInput {
                        uuid: cli.component_uuid,
                        part_uuid: lmv321_part_uuid,
                    })?;
                    let updated = engine.apply_component_replacement_plan(vec![
                        PlannedComponentReplacementInput {
                            uuid: cli.component_uuid,
                            package_uuid: Some(altamp.package_uuid),
                            part_uuid: None,
                        },
                    ])?;
                    Ok(updated.description)
                },
            ),
        ),
        save_probe_gate(
            "apply_component_replacement_policy_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "apply_component_replacement_policy"),
            save_and_compare_after_with_setup(
                &cli.board_fixture_path,
                |engine| {
                    engine.import_eagle_library(&cli.library_fixture_path)?;
                    Ok(())
                },
                |engine| {
                    let lmv321_part_uuid = engine
                        .search_pool("LMV321")?
                        .into_iter()
                        .next()
                        .context("LMV321 part missing for replacement-policy determinism probe")?
                        .uuid;
                    engine.assign_part(AssignPartInput {
                        uuid: cli.component_uuid,
                        part_uuid: lmv321_part_uuid,
                    })?;
                    let updated = engine.apply_component_replacement_policy(vec![
                        PolicyDrivenComponentReplacementInput {
                            uuid: cli.component_uuid,
                            policy: ComponentReplacementPolicy::BestCompatiblePackage,
                        },
                    ])?;
                    Ok(updated.description)
                },
            ),
        ),
        save_probe_gate(
            "apply_scoped_component_replacement_policy_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "apply_scoped_component_replacement_policy"),
            save_and_compare_after_with_setup(
                &cli.board_fixture_path,
                |engine| {
                    engine.import_eagle_library(&cli.library_fixture_path)?;
                    Ok(())
                },
                |engine| {
                    let lmv321_part_uuid = engine
                        .search_pool("LMV321")?
                        .into_iter()
                        .next()
                        .context("LMV321 part missing for scoped-policy determinism probe")?
                        .uuid;
                    engine.assign_part(AssignPartInput {
                        uuid: cli.component_uuid,
                        part_uuid: lmv321_part_uuid,
                    })?;
                    let updated = engine.apply_scoped_component_replacement_policy(
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
                    Ok(updated.description)
                },
            ),
        ),
        save_probe_gate(
            "apply_scoped_component_replacement_plan_save_byte_determinism",
            save_probe_paths(&cli.save_probe_path, "apply_scoped_component_replacement_plan"),
            save_and_compare_after_with_setup(
                &cli.board_fixture_path,
                |engine| {
                    engine.import_eagle_library(&cli.library_fixture_path)?;
                    Ok(())
                },
                |engine| {
                    let lmv321_part_uuid = engine
                        .search_pool("LMV321")?
                        .into_iter()
                        .next()
                        .context("LMV321 part missing for scoped-plan determinism probe")?
                        .uuid;
                    engine.assign_part(AssignPartInput {
                        uuid: cli.component_uuid,
                        part_uuid: lmv321_part_uuid,
                    })?;
                    let plan = engine.get_scoped_component_replacement_plan(
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
                    let updated = engine.apply_scoped_component_replacement_plan(plan)?;
                    Ok(updated.description)
                },
            ),
        ),
    ];

    let overall_status = if gates.iter().any(|g| g.status == GateStatus::Failed) {
        GateStatus::Failed
    } else {
        GateStatus::Passed
    };

    let summary = match overall_status {
        GateStatus::Passed => {
            "M3 replacement determinism preflight passed for the current replacement save slices"
                .to_string()
        }
        GateStatus::Failed => {
            "M3 replacement determinism preflight failed; unexpected write-capability state detected"
                .to_string()
        }
        GateStatus::Deferred => unreachable!("replacement determinism hook no longer returns deferred"),
    };

    Ok(DeterminismReport {
        schema_version: 1,
        overall_status,
        summary,
        gates,
    })
}

fn save_probe_gate(gate: &str, paths: (PathBuf, PathBuf), probe: Result<String>) -> GateResult {
    let (first, second) = paths;
    match probe {
        Ok(evidence) => GateResult {
            gate: gate.to_string(),
            status: GateStatus::Passed,
            evidence: format!(
                "{}; Engine::save wrote byte-identical KiCad board output to {} and {}",
                evidence,
                first.display(),
                second.display()
            ),
        },
        Err(err) => GateResult {
            gate: gate.to_string(),
            status: GateStatus::Failed,
            evidence: format!("KiCad board save determinism probe failed: {err}"),
        },
    }
}

fn save_and_compare_after_with_setup<S, F>(
    fixture: &PathBuf,
    setup: S,
    mutate: F,
) -> Result<String>
where
    S: Fn(&mut Engine) -> Result<()>,
    F: Fn(&mut Engine) -> Result<String>,
{
    let mut first_engine = Engine::new()?;
    setup(&mut first_engine)?;
    first_engine.import(fixture)?;
    let first_evidence = mutate(&mut first_engine)?;

    let mut second_engine = Engine::new()?;
    setup(&mut second_engine)?;
    second_engine.import(fixture)?;
    let second_evidence = mutate(&mut second_engine)?;

    let (first_probe_path, second_probe_path) = save_probe_paths(
        &std::env::temp_dir()
            .join(format!("datum-eda-m3-replacement-save-probe-{}.kicad_pcb", Uuid::new_v4())),
        "determinism",
    );
    let save_result = save_and_compare(
        &first_engine,
        &second_engine,
        &first_probe_path,
        &second_probe_path,
    );
    let _ = fs::remove_file(&first_probe_path);
    let _ = fs::remove_file(&second_probe_path);
    save_result?;

    if first_evidence != second_evidence {
        bail!("operation metadata was not identical across repeated runs");
    }

    Ok(first_evidence)
}

fn save_probe_paths(base: &PathBuf, label: &str) -> (PathBuf, PathBuf) {
    let stem = base
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("datum-eda-m3-replacement-save-probe");
    let dir = base.parent().unwrap_or_else(|| std::path::Path::new("."));
    (
        dir.join(format!("{stem}-{label}.kicad_pcb")),
        dir.join(format!("{stem}-{label}-second.kicad_pcb")),
    )
}

fn save_and_compare(
    first_engine: &Engine,
    second_engine: &Engine,
    first: &PathBuf,
    second: &PathBuf,
) -> Result<()> {
    first_engine.save(first)?;
    second_engine.save(second)?;

    let first_bytes = fs::read(first)?;
    let second_bytes = fs::read(second)?;
    if first_bytes != second_bytes {
        bail!("save outputs were not byte-identical across consecutive runs");
    }

    Ok(())
}

fn print_human_report(report: &DeterminismReport) {
    println!("m3 replacement operation determinism preflight:");
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
    fn replacement_determinism_report_passes_for_current_slice() {
        let cli = Cli {
            json: false,
            allow_deferred: false,
            board_fixture_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../engine/testdata/import/kicad/partial-route-demo.kicad_pcb")
                .canonicalize()
                .expect("fixture path should resolve"),
            save_probe_path: std::env::temp_dir()
                .join("datum-eda-m3-replacement-test-save.kicad_pcb"),
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
        assert_eq!(report.overall_status, GateStatus::Passed);
    }
}
