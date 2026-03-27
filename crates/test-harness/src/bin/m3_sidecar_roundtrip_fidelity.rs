use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use eda_engine::api::{
    AssignPartInput, ComponentReplacementPolicy, ComponentReplacementScope, Engine,
    PlannedComponentReplacementInput, PolicyDrivenComponentReplacementInput, ReplaceComponentInput,
    ScopedComponentReplacementPolicyInput, SetDesignRuleInput, SetNetClassInput, SetPackageInput,
    SetPackageWithPartInput,
};
use eda_engine::import::{
    net_classes_sidecar, package_assignments_sidecar, part_assignments_sidecar, rules_sidecar,
};
use eda_engine::rules::ast::{RuleParams, RuleScope, RuleType};
use eda_test_harness::canonical_json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Cli {
    json: bool,
    allow_deferred: bool,
    roundtrip_board_fixture_path: PathBuf,
    simple_board_fixture_path: PathBuf,
    library_fixture_path: PathBuf,
    component_uuid: Uuid,
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
            eprintln!("m3_sidecar_roundtrip_fidelity: {err:#}");
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
    let mut roundtrip_board_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/partial-route-demo.kicad_pcb")
        .canonicalize()
        .map_err(|err| {
            anyhow::anyhow!("failed to resolve default roundtrip board fixture path: {err}")
        })?;
    let mut simple_board_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_pcb")
        .canonicalize()
        .map_err(|err| {
            anyhow::anyhow!("failed to resolve default simple board fixture path: {err}")
        })?;
    let mut library_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/eagle/simple-opamp.lbr")
        .canonicalize()
        .map_err(|err| anyhow::anyhow!("failed to resolve default library fixture path: {err}"))?;
    let mut component_uuid =
        Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").expect("uuid should parse");

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
            "--simple-board-fixture-path" => {
                let value = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--simple-board-fixture-path requires a path argument")
                })?;
                simple_board_fixture_path = PathBuf::from(value);
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
        roundtrip_board_fixture_path,
        simple_board_fixture_path,
        library_fixture_path,
        component_uuid,
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p eda-test-harness --bin m3_sidecar_roundtrip_fidelity -- [options]\n\
         Options:\n\
           --json                          Emit canonical JSON\n\
           --allow-deferred                Exit 0 when status is deferred\n\
           --roundtrip-board-fixture-path <p>  Fixture for assign/package fidelity checks\n\
           --simple-board-fixture-path <p>     Fixture for rule/net-class fidelity checks\n\
           --library-fixture-path <p>      Pool library fixture for assign/package checks\n\
           --component-uuid <uuid>         Component UUID used for assign/package checks\n\
           -h, --help                      Show this help"
    );
}

fn build_report(cli: &Cli) -> Result<Report> {
    let checks = vec![
        check("set_design_rule_sidecar_roundtrip_fidelity", || {
            set_design_rule_fidelity_evidence(cli)
        }),
        check("assign_part_sidecar_roundtrip_fidelity", || {
            assign_part_fidelity_evidence(cli)
        }),
        check("set_package_sidecar_roundtrip_fidelity", || {
            set_package_fidelity_evidence(cli)
        }),
        check("set_package_with_part_sidecar_roundtrip_fidelity", || {
            set_package_with_part_fidelity_evidence(cli)
        }),
        check("replace_component_sidecar_roundtrip_fidelity", || {
            replace_component_fidelity_evidence(cli)
        }),
        check("replace_components_sidecar_roundtrip_fidelity", || {
            replace_components_fidelity_evidence(cli)
        }),
        check(
            "apply_component_replacement_plan_sidecar_roundtrip_fidelity",
            || apply_component_replacement_plan_fidelity_evidence(cli),
        ),
        check(
            "apply_component_replacement_policy_sidecar_roundtrip_fidelity",
            || apply_component_replacement_policy_fidelity_evidence(cli),
        ),
        check(
            "apply_scoped_component_replacement_policy_sidecar_roundtrip_fidelity",
            || apply_scoped_component_replacement_policy_fidelity_evidence(cli),
        ),
        check(
            "apply_scoped_component_replacement_plan_sidecar_roundtrip_fidelity",
            || apply_scoped_component_replacement_plan_fidelity_evidence(cli),
        ),
        check("set_net_class_sidecar_roundtrip_fidelity", || {
            set_net_class_fidelity_evidence(cli)
        }),
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

fn check<F>(name: &str, build_evidence: F) -> Check
where
    F: FnOnce() -> Result<String>,
{
    match build_evidence() {
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

fn set_design_rule_fidelity_evidence(cli: &Cli) -> Result<String> {
    let fixture_bytes = fs::read(&cli.simple_board_fixture_path)?;
    let first_board = unique_temp_path("m3-sidecar-rule-first", "kicad_pcb");
    let second_board = unique_temp_path("m3-sidecar-rule-second", "kicad_pcb");

    let mut engine = Engine::new()?;
    engine.import(&cli.simple_board_fixture_path)?;
    engine.set_design_rule(SetDesignRuleInput {
        rule_type: RuleType::ClearanceCopper,
        scope: RuleScope::All,
        parameters: RuleParams::Clearance { min: 125_000 },
        priority: 10,
        name: Some("default clearance".to_string()),
    })?;
    engine.save(&first_board)?;

    let first_board_bytes = fs::read(&first_board)?;
    if first_board_bytes != fixture_bytes {
        bail!(
            "set_design_rule mutated KiCad board bytes; expected authored rule state to live in sidecar only"
        );
    }

    let first_rules_sidecar = rules_sidecar::sidecar_path_for_source(&first_board);
    let first_rules = rules_sidecar::read_sidecar(&first_rules_sidecar)
        .context("failed to decode first rule sidecar")?;

    let mut reloaded = Engine::new()?;
    reloaded.import(&first_board)?;
    if reloaded.get_design_rules()?.len() != 1 {
        bail!("reimported set_design_rule save did not restore expected rule sidecar state");
    }
    reloaded.save(&second_board)?;

    let second_board_bytes = fs::read(&second_board)?;
    let second_rules_sidecar = rules_sidecar::sidecar_path_for_source(&second_board);
    let second_rules = rules_sidecar::read_sidecar(&second_rules_sidecar)
        .context("failed to decode second rule sidecar")?;
    if first_board_bytes != second_board_bytes {
        bail!("set_design_rule save→reimport→save changed KiCad board bytes");
    }
    if first_rules.schema_version != second_rules.schema_version
        || first_rules.source_hash != second_rules.source_hash
        || first_rules.rules != second_rules.rules
    {
        bail!("set_design_rule save→reimport→save changed semantic rule sidecar content");
    }

    cleanup_paths(&[
        first_board,
        second_board,
        first_rules_sidecar,
        second_rules_sidecar,
    ]);

    Ok("board_bytes_unchanged=true, rules_sidecar_roundtrip_stable=true, reimport_restored_rule=true".to_string())
}

fn assign_part_fidelity_evidence(cli: &Cli) -> Result<String> {
    let first_board = unique_temp_path("m3-sidecar-assign-first", "kicad_pcb");
    let second_board = unique_temp_path("m3-sidecar-assign-second", "kicad_pcb");

    let mut engine = Engine::new()?;
    engine.import_eagle_library(&cli.library_fixture_path)?;
    engine.import(&cli.roundtrip_board_fixture_path)?;
    let part_uuid = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .context("ALTAMP part missing for assign_part fidelity probe")?
        .uuid;
    engine.assign_part(AssignPartInput {
        uuid: cli.component_uuid,
        part_uuid,
    })?;
    engine.save(&first_board)?;

    let first_board_bytes = fs::read(&first_board)?;
    let first_parts_sidecar = part_assignments_sidecar::sidecar_path_for_source(&first_board);
    let first_parts = part_assignments_sidecar::read_sidecar(&first_parts_sidecar)
        .context("failed to decode first part-assignment sidecar")?;

    let mut reloaded = Engine::new()?;
    reloaded.import_eagle_library(&cli.library_fixture_path)?;
    reloaded.import(&first_board)?;
    let target_component = reloaded
        .get_components()?
        .into_iter()
        .find(|component| component.uuid == cli.component_uuid)
        .context("reimported assign_part save missing target component")?;
    if target_component.value != "ALTAMP" {
        bail!("reimported assign_part save did not restore expected part assignment");
    }
    reloaded.save(&second_board)?;

    let second_board_bytes = fs::read(&second_board)?;
    let second_parts_sidecar = part_assignments_sidecar::sidecar_path_for_source(&second_board);
    let second_parts = part_assignments_sidecar::read_sidecar(&second_parts_sidecar)
        .context("failed to decode second part-assignment sidecar")?;
    if first_board_bytes != second_board_bytes {
        bail!("assign_part save→reimport→save changed KiCad board bytes");
    }
    if first_parts.schema_version != second_parts.schema_version
        || first_parts.source_hash != second_parts.source_hash
        || first_parts.assignments != second_parts.assignments
    {
        bail!("assign_part save→reimport→save changed semantic part-assignment sidecar content");
    }

    cleanup_paths(&[
        first_board,
        second_board,
        first_parts_sidecar,
        second_parts_sidecar,
    ]);

    Ok("board_roundtrip_stable=true, part_sidecar_roundtrip_stable=true, reimport_restored_part_assignment=true".to_string())
}

fn set_package_fidelity_evidence(cli: &Cli) -> Result<String> {
    let first_board = unique_temp_path("m3-sidecar-package-first", "kicad_pcb");
    let second_board = unique_temp_path("m3-sidecar-package-second", "kicad_pcb");

    let mut engine = Engine::new()?;
    engine.import_eagle_library(&cli.library_fixture_path)?;
    engine.import(&cli.roundtrip_board_fixture_path)?;
    let package_uuid = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .context("ALTAMP package missing for set_package fidelity probe")?
        .package_uuid;
    engine.set_package(SetPackageInput {
        uuid: cli.component_uuid,
        package_uuid,
    })?;
    engine.save(&first_board)?;

    let first_board_bytes = fs::read(&first_board)?;
    let first_packages_sidecar = package_assignments_sidecar::sidecar_path_for_source(&first_board);
    let first_packages = package_assignments_sidecar::read_sidecar(&first_packages_sidecar)
        .context("failed to decode first package-assignment sidecar")?;

    let mut reloaded = Engine::new()?;
    reloaded.import_eagle_library(&cli.library_fixture_path)?;
    reloaded.import(&first_board)?;
    let target_component = reloaded
        .get_components()?
        .into_iter()
        .find(|component| component.uuid == cli.component_uuid)
        .context("reimported set_package save missing target component")?;
    if target_component.package_uuid != package_uuid {
        bail!("reimported set_package save did not restore expected package assignment");
    }
    reloaded.save(&second_board)?;

    let second_board_bytes = fs::read(&second_board)?;
    let second_packages_sidecar =
        package_assignments_sidecar::sidecar_path_for_source(&second_board);
    let second_packages = package_assignments_sidecar::read_sidecar(&second_packages_sidecar)
        .context("failed to decode second package-assignment sidecar")?;
    if first_board_bytes != second_board_bytes {
        bail!("set_package save→reimport→save changed KiCad board bytes");
    }
    if first_packages.schema_version != second_packages.schema_version
        || first_packages.source_hash != second_packages.source_hash
        || first_packages.assignments != second_packages.assignments
    {
        bail!("set_package save→reimport→save changed semantic package-assignment sidecar content");
    }

    cleanup_paths(&[
        first_board,
        second_board,
        first_packages_sidecar,
        second_packages_sidecar,
    ]);

    Ok("board_roundtrip_stable=true, package_sidecar_roundtrip_stable=true, reimport_restored_package_assignment=true".to_string())
}

fn set_net_class_fidelity_evidence(cli: &Cli) -> Result<String> {
    let fixture_bytes = fs::read(&cli.simple_board_fixture_path)?;
    let first_board = unique_temp_path("m3-sidecar-net-class-first", "kicad_pcb");
    let second_board = unique_temp_path("m3-sidecar-net-class-second", "kicad_pcb");

    let mut engine = Engine::new()?;
    engine.import(&cli.simple_board_fixture_path)?;
    let gnd_uuid = engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "GND")
        .context("GND net missing for set_net_class fidelity probe")?
        .uuid;
    engine.set_net_class(SetNetClassInput {
        net_uuid: gnd_uuid,
        class_name: "power".to_string(),
        clearance: 125_000,
        track_width: 250_000,
        via_drill: 300_000,
        via_diameter: 600_000,
        diffpair_width: 0,
        diffpair_gap: 0,
    })?;
    engine.save(&first_board)?;

    let first_board_bytes = fs::read(&first_board)?;
    if first_board_bytes != fixture_bytes {
        bail!(
            "set_net_class mutated KiCad board bytes; expected authored net-class state to live in sidecar only"
        );
    }

    let first_net_classes_sidecar = net_classes_sidecar::sidecar_path_for_source(&first_board);
    let first_net_classes = net_classes_sidecar::read_sidecar(&first_net_classes_sidecar)
        .context("failed to decode first net-class sidecar")?;

    let mut reloaded = Engine::new()?;
    reloaded.import(&first_board)?;
    let reloaded_gnd = reloaded
        .get_net_info()?
        .into_iter()
        .find(|net| net.uuid == gnd_uuid)
        .context("reimported set_net_class save missing GND net")?;
    if reloaded_gnd.class != "power" {
        bail!("reimported set_net_class save did not restore expected net class sidecar state");
    }
    reloaded.save(&second_board)?;

    let second_board_bytes = fs::read(&second_board)?;
    let second_net_classes_sidecar = net_classes_sidecar::sidecar_path_for_source(&second_board);
    let second_net_classes = net_classes_sidecar::read_sidecar(&second_net_classes_sidecar)
        .context("failed to decode second net-class sidecar")?;
    if first_board_bytes != second_board_bytes {
        bail!("set_net_class save→reimport→save changed KiCad board bytes");
    }
    if first_net_classes.schema_version != second_net_classes.schema_version
        || first_net_classes.source_hash != second_net_classes.source_hash
        || first_net_classes.classes != second_net_classes.classes
        || first_net_classes.assignments != second_net_classes.assignments
    {
        bail!("set_net_class save→reimport→save changed semantic net-class sidecar content");
    }

    cleanup_paths(&[
        first_board,
        second_board,
        first_net_classes_sidecar,
        second_net_classes_sidecar,
    ]);

    Ok("board_bytes_unchanged=true, net_class_sidecar_roundtrip_stable=true, reimport_restored_net_class=true".to_string())
}

fn set_package_with_part_fidelity_evidence(cli: &Cli) -> Result<String> {
    let first_board = unique_temp_path("m3-sidecar-package-with-part-first", "kicad_pcb");
    let second_board = unique_temp_path("m3-sidecar-package-with-part-second", "kicad_pcb");

    let mut engine = Engine::new()?;
    engine.import_eagle_library(&cli.library_fixture_path)?;
    engine.import(&cli.roundtrip_board_fixture_path)?;
    let lmv321_part_uuid = engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .context("LMV321 part missing for set_package_with_part fidelity probe")?
        .uuid;
    let altamp = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .context("ALTAMP part missing for set_package_with_part fidelity probe")?;
    engine.assign_part(AssignPartInput {
        uuid: cli.component_uuid,
        part_uuid: lmv321_part_uuid,
    })?;
    engine.set_package_with_part(SetPackageWithPartInput {
        uuid: cli.component_uuid,
        package_uuid: altamp.package_uuid,
        part_uuid: altamp.uuid,
    })?;
    engine.save(&first_board)?;

    let first_board_bytes = fs::read(&first_board)?;
    let first_parts_sidecar = part_assignments_sidecar::sidecar_path_for_source(&first_board);
    let first_parts = part_assignments_sidecar::read_sidecar(&first_parts_sidecar)
        .context("failed to decode first explicit part-assignment sidecar")?;
    let first_packages_sidecar = package_assignments_sidecar::sidecar_path_for_source(&first_board);
    let first_packages = package_assignments_sidecar::read_sidecar(&first_packages_sidecar)
        .context("failed to decode first explicit package-assignment sidecar")?;

    let mut reloaded = Engine::new()?;
    reloaded.import_eagle_library(&cli.library_fixture_path)?;
    reloaded.import(&first_board)?;
    let target_component = reloaded
        .get_components()?
        .into_iter()
        .find(|component| component.uuid == cli.component_uuid)
        .context("reimported set_package_with_part save missing target component")?;
    if target_component.package_uuid != altamp.package_uuid {
        bail!("reimported set_package_with_part save did not restore expected package assignment");
    }
    if target_component.value != "ALTAMP" {
        bail!(
            "reimported set_package_with_part save did not restore expected explicit part assignment"
        );
    }
    let sig = reloaded
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .context("reimported set_package_with_part save missing SIG net")?;
    if sig.pins.len() != 2 {
        bail!(
            "reimported set_package_with_part save did not preserve expected logical net mapping"
        );
    }
    reloaded.save(&second_board)?;

    let second_board_bytes = fs::read(&second_board)?;
    let second_parts_sidecar = part_assignments_sidecar::sidecar_path_for_source(&second_board);
    let second_parts = part_assignments_sidecar::read_sidecar(&second_parts_sidecar)
        .context("failed to decode second explicit part-assignment sidecar")?;
    let second_packages_sidecar =
        package_assignments_sidecar::sidecar_path_for_source(&second_board);
    let second_packages = package_assignments_sidecar::read_sidecar(&second_packages_sidecar)
        .context("failed to decode second explicit package-assignment sidecar")?;

    if first_board_bytes != second_board_bytes {
        bail!("set_package_with_part save→reimport→save changed KiCad board bytes");
    }
    if first_parts.schema_version != second_parts.schema_version
        || first_parts.source_hash != second_parts.source_hash
        || first_parts.assignments != second_parts.assignments
    {
        bail!(
            "set_package_with_part save→reimport→save changed semantic part-assignment sidecar content"
        );
    }
    if first_packages.schema_version != second_packages.schema_version
        || first_packages.source_hash != second_packages.source_hash
        || first_packages.assignments != second_packages.assignments
    {
        bail!(
            "set_package_with_part save→reimport→save changed semantic package-assignment sidecar content"
        );
    }

    cleanup_paths(&[
        first_board,
        second_board,
        first_parts_sidecar,
        second_parts_sidecar,
        first_packages_sidecar,
        second_packages_sidecar,
    ]);

    Ok("board_roundtrip_stable=true, explicit_part_sidecar_roundtrip_stable=true, explicit_package_sidecar_roundtrip_stable=true, reimport_restored_explicit_package_and_part=true".to_string())
}

fn replace_component_fidelity_evidence(cli: &Cli) -> Result<String> {
    run_replacement_roundtrip(
        cli,
        "m3-sidecar-replace-component",
        |engine, component_uuid, altamp, _lmv321_uuid| {
            engine.replace_component(ReplaceComponentInput {
                uuid: component_uuid,
                package_uuid: altamp.package_uuid,
                part_uuid: altamp.uuid,
            })?;
            Ok(())
        },
        |engine, component_uuid, altamp| {
            let target = component_by_uuid(engine, component_uuid)?;
            if target.package_uuid != altamp.package_uuid || target.value != "ALTAMP" {
                bail!(
                    "reimported replace_component save did not restore expected replacement state"
                );
            }
            Ok("board_roundtrip_stable=true, replace_component_sidecars_roundtrip_stable=true, reimport_restored_replacement=true".to_string())
        },
    )
}

fn replace_components_fidelity_evidence(cli: &Cli) -> Result<String> {
    let second_uuid =
        Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").expect("uuid should parse");
    run_replacement_roundtrip(
        cli,
        "m3-sidecar-replace-components",
        |engine, component_uuid, altamp, _lmv321_uuid| {
            engine.replace_components(vec![
                ReplaceComponentInput {
                    uuid: component_uuid,
                    package_uuid: altamp.package_uuid,
                    part_uuid: altamp.uuid,
                },
                ReplaceComponentInput {
                    uuid: second_uuid,
                    package_uuid: altamp.package_uuid,
                    part_uuid: altamp.uuid,
                },
            ])?;
            Ok(())
        },
        |_engine, _component_uuid, _altamp| {
            Ok("board_roundtrip_stable=true, replace_components_sidecars_roundtrip_stable=true, reimport_restored_batch_replacement=true".to_string())
        },
    )
}

fn apply_component_replacement_plan_fidelity_evidence(cli: &Cli) -> Result<String> {
    run_replacement_roundtrip(
        cli,
        "m3-sidecar-apply-replacement-plan",
        |engine, component_uuid, altamp, _lmv321_uuid| {
            let lmv321_part_uuid = engine
                .search_pool("LMV321")?
                .into_iter()
                .next()
                .context("LMV321 part missing for replacement-plan fidelity probe")?
                .uuid;
            engine.assign_part(AssignPartInput {
                uuid: component_uuid,
                part_uuid: lmv321_part_uuid,
            })?;
            engine.apply_component_replacement_plan(vec![PlannedComponentReplacementInput {
                uuid: component_uuid,
                package_uuid: Some(altamp.package_uuid),
                part_uuid: None,
            }])?;
            Ok(())
        },
        |engine, component_uuid, altamp| {
            let target = component_by_uuid(engine, component_uuid)?;
            if target.package_uuid != altamp.package_uuid || target.value != "ALTAMP" {
                bail!(
                    "reimported apply_component_replacement_plan save did not restore expected replacement state"
                );
            }
            Ok("board_roundtrip_stable=true, replacement_plan_sidecars_roundtrip_stable=true, reimport_restored_planned_replacement=true".to_string())
        },
    )
}

fn apply_component_replacement_policy_fidelity_evidence(cli: &Cli) -> Result<String> {
    run_replacement_roundtrip(
        cli,
        "m3-sidecar-apply-replacement-policy",
        |engine, component_uuid, _altamp, lmv321_uuid| {
            engine.assign_part(AssignPartInput {
                uuid: component_uuid,
                part_uuid: lmv321_uuid,
            })?;
            engine.apply_component_replacement_policy(vec![
                PolicyDrivenComponentReplacementInput {
                    uuid: component_uuid,
                    policy: ComponentReplacementPolicy::BestCompatiblePackage,
                },
            ])?;
            Ok(())
        },
        |engine, component_uuid, _altamp| {
            let target = component_by_uuid(engine, component_uuid)?;
            if target.value.is_empty() {
                bail!(
                    "reimported apply_component_replacement_policy save missing replacement value"
                );
            }
            Ok("board_roundtrip_stable=true, replacement_policy_sidecars_roundtrip_stable=true, reimport_restored_policy_replacement=true".to_string())
        },
    )
}

fn apply_scoped_component_replacement_policy_fidelity_evidence(cli: &Cli) -> Result<String> {
    run_replacement_roundtrip(
        cli,
        "m3-sidecar-apply-scoped-policy",
        |engine, component_uuid, _altamp, lmv321_uuid| {
            engine.assign_part(AssignPartInput {
                uuid: component_uuid,
                part_uuid: lmv321_uuid,
            })?;
            engine.apply_scoped_component_replacement_policy(
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
            Ok(())
        },
        |_engine, _component_uuid, _altamp| {
            Ok("board_roundtrip_stable=true, scoped_policy_sidecars_roundtrip_stable=true, reimport_restored_scoped_policy_replacement=true".to_string())
        },
    )
}

fn apply_scoped_component_replacement_plan_fidelity_evidence(cli: &Cli) -> Result<String> {
    run_replacement_roundtrip(
        cli,
        "m3-sidecar-apply-scoped-plan",
        |engine, component_uuid, _altamp, lmv321_uuid| {
            engine.assign_part(AssignPartInput {
                uuid: component_uuid,
                part_uuid: lmv321_uuid,
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
            engine.apply_scoped_component_replacement_plan(plan)?;
            Ok(())
        },
        |_engine, _component_uuid, _altamp| {
            Ok("board_roundtrip_stable=true, scoped_plan_sidecars_roundtrip_stable=true, reimport_restored_scoped_plan_replacement=true".to_string())
        },
    )
}

fn run_replacement_roundtrip<M, V>(
    cli: &Cli,
    prefix: &str,
    mutate: M,
    validate: V,
) -> Result<String>
where
    M: Fn(&mut Engine, Uuid, &eda_engine::pool::PartSummary, Uuid) -> Result<()>,
    V: Fn(&Engine, Uuid, &eda_engine::pool::PartSummary) -> Result<String>,
{
    let first_board = unique_temp_path(&format!("{prefix}-first"), "kicad_pcb");
    let second_board = unique_temp_path(&format!("{prefix}-second"), "kicad_pcb");

    let mut engine = Engine::new()?;
    engine.import_eagle_library(&cli.library_fixture_path)?;
    engine.import(&cli.roundtrip_board_fixture_path)?;
    let lmv321_part_uuid = engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .context("LMV321 part missing for replacement fidelity probe")?
        .uuid;
    let altamp = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .context("ALTAMP part missing for replacement fidelity probe")?;

    mutate(&mut engine, cli.component_uuid, &altamp, lmv321_part_uuid)?;
    engine.save(&first_board)?;

    let first_board_bytes = fs::read(&first_board)?;
    let first_parts_sidecar = part_assignments_sidecar::sidecar_path_for_source(&first_board);
    let first_parts = part_assignments_sidecar::read_sidecar(&first_parts_sidecar)
        .context("failed to decode first replacement part-assignment sidecar")?;
    let first_packages_sidecar = package_assignments_sidecar::sidecar_path_for_source(&first_board);
    let first_packages = package_assignments_sidecar::read_sidecar(&first_packages_sidecar)
        .context("failed to decode first replacement package-assignment sidecar")?;

    let mut reloaded = Engine::new()?;
    reloaded.import_eagle_library(&cli.library_fixture_path)?;
    reloaded.import(&first_board)?;
    let evidence = validate(&reloaded, cli.component_uuid, &altamp)?;
    reloaded.save(&second_board)?;

    let second_board_bytes = fs::read(&second_board)?;
    let second_parts_sidecar = part_assignments_sidecar::sidecar_path_for_source(&second_board);
    let second_parts = part_assignments_sidecar::read_sidecar(&second_parts_sidecar)
        .context("failed to decode second replacement part-assignment sidecar")?;
    let second_packages_sidecar =
        package_assignments_sidecar::sidecar_path_for_source(&second_board);
    let second_packages = package_assignments_sidecar::read_sidecar(&second_packages_sidecar)
        .context("failed to decode second replacement package-assignment sidecar")?;

    if first_board_bytes != second_board_bytes {
        bail!("{prefix} save→reimport→save changed KiCad board bytes");
    }
    if first_parts.schema_version != second_parts.schema_version
        || first_parts.source_hash != second_parts.source_hash
        || first_parts.assignments != second_parts.assignments
    {
        bail!("{prefix} save→reimport→save changed semantic part-assignment sidecar content");
    }
    if first_packages.schema_version != second_packages.schema_version
        || first_packages.source_hash != second_packages.source_hash
        || first_packages.assignments != second_packages.assignments
    {
        bail!("{prefix} save→reimport→save changed semantic package-assignment sidecar content");
    }

    cleanup_paths(&[
        first_board,
        second_board,
        first_parts_sidecar,
        second_parts_sidecar,
        first_packages_sidecar,
        second_packages_sidecar,
    ]);

    Ok(evidence)
}

fn component_by_uuid(
    engine: &Engine,
    component_uuid: Uuid,
) -> Result<eda_engine::board::ComponentInfo> {
    engine
        .get_components()?
        .into_iter()
        .find(|component| component.uuid == component_uuid)
        .context("target component missing after reimport")
}

fn cleanup_paths(paths: &[PathBuf]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}

fn unique_temp_path(prefix: &str, extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{prefix}-{}.{extension}", Uuid::new_v4()))
}

fn print_human(report: &Report) {
    println!("m3 sidecar roundtrip fidelity:");
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
    fn sidecar_roundtrip_smoke_passes() {
        let cli = Cli {
            json: false,
            allow_deferred: false,
            roundtrip_board_fixture_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../engine/testdata/import/kicad/partial-route-demo.kicad_pcb")
                .canonicalize()
                .expect("fixture path should resolve"),
            simple_board_fixture_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../engine/testdata/import/kicad/simple-demo.kicad_pcb")
                .canonicalize()
                .expect("fixture path should resolve"),
            library_fixture_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../engine/testdata/import/eagle/simple-opamp.lbr")
                .canonicalize()
                .expect("library fixture should resolve"),
            component_uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
                .expect("uuid should parse"),
        };

        let report = build_report(&cli).expect("report should build");
        assert_eq!(report.overall_status, Status::Passed);
    }
}
