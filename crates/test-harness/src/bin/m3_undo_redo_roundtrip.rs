use std::path::PathBuf;

use anyhow::{Result, bail};
use eda_engine::api::{
    AssignPartInput, Engine, MoveComponentInput, SetDesignRuleInput, SetNetClassInput,
    SetPackageInput,
};
use eda_engine::rules::ast::{RuleParams, RuleScope, RuleType};
use eda_test_harness::canonical_json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Cli {
    json: bool,
    allow_deferred: bool,
    board_fixture_path: PathBuf,
    track_uuid: Uuid,
    component_uuid: Uuid,
    target_x_nm: i64,
    target_y_nm: i64,
    target_rotation_deg: i32,
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
            eprintln!("m3_undo_redo_roundtrip: {err:#}");
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
    let mut track_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
    let mut component_uuid =
        Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").expect("uuid should parse");
    let mut target_x_nm = 15_000_000;
    let mut target_y_nm = 12_000_000;
    let mut target_rotation_deg = 90;

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
            "--track-uuid" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--track-uuid requires a UUID argument"))?;
                track_uuid = Uuid::parse_str(&value)?;
            }
            "--component-uuid" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--component-uuid requires a UUID argument"))?;
                component_uuid = Uuid::parse_str(&value)?;
            }
            "--target-x-nm" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--target-x-nm requires an integer argument"))?;
                target_x_nm = value.parse()?;
            }
            "--target-y-nm" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--target-y-nm requires an integer argument"))?;
                target_y_nm = value.parse()?;
            }
            "--target-rotation-deg" => {
                let value = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--target-rotation-deg requires an integer argument")
                })?;
                target_rotation_deg = value.parse()?;
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
        track_uuid,
        component_uuid,
        target_x_nm,
        target_y_nm,
        target_rotation_deg,
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p eda-test-harness --bin m3_undo_redo_roundtrip -- [options]\n\
         Options:\n\
           --json             Emit canonical JSON\n\
           --allow-deferred   Exit 0 when status is deferred\n\
           --board-fixture-path <p>  KiCad board fixture used for roundtrip check\n\
           --track-uuid <uuid>  Track UUID used for delete/undo/redo roundtrip\n\
           --component-uuid <uuid>  Component UUID used for move/undo/redo roundtrip\n\
           --target-x-nm <nm>   Target component X position in nm\n\
           --target-y-nm <nm>   Target component Y position in nm\n\
           --target-rotation-deg <deg>  Target component rotation in degrees\n\
           -h, --help         Show this help"
    );
}

fn build_report(cli: &Cli) -> Result<Report> {
    let mut engine = Engine::new()?;
    engine.import(&cli.board_fixture_path)?;
    let baseline = engine.get_net_info()?;
    let delete_result = engine.delete_track(&cli.track_uuid)?;
    let after_delete = engine.get_net_info()?;
    let undo_result = engine.undo()?;
    let after_undo = engine.get_net_info()?;
    let redo_result = engine.redo()?;
    let after_redo = engine.get_net_info()?;

    let mut move_engine = Engine::new()?;
    move_engine.import(&cli.board_fixture_path)?;
    let baseline_components = move_engine.get_components()?;
    let move_result = move_engine.move_component(MoveComponentInput {
        uuid: cli.component_uuid,
        position: eda_engine::ir::geometry::Point::new(cli.target_x_nm, cli.target_y_nm),
        rotation: Some(cli.target_rotation_deg),
    })?;
    let after_move = move_engine.get_components()?;
    let move_undo_result = move_engine.undo()?;
    let after_move_undo = move_engine.get_components()?;
    let move_redo_result = move_engine.redo()?;
    let after_move_redo = move_engine.get_components()?;

    let mut rule_engine = Engine::new()?;
    rule_engine.import(&cli.board_fixture_path)?;
    let baseline_rules = rule_engine.get_design_rules()?;
    let set_rule_result = rule_engine.set_design_rule(SetDesignRuleInput {
        rule_type: RuleType::ClearanceCopper,
        scope: RuleScope::All,
        parameters: RuleParams::Clearance { min: 125_000 },
        priority: 10,
        name: Some("default clearance".to_string()),
    })?;
    let after_set_rule = rule_engine.get_design_rules()?;
    let rule_undo_result = rule_engine.undo()?;
    let after_rule_undo = rule_engine.get_design_rules()?;
    let rule_redo_result = rule_engine.redo()?;
    let after_rule_redo = rule_engine.get_design_rules()?;

    let library_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/eagle/simple-opamp.lbr")
        .canonicalize()
        .map_err(|err| anyhow::anyhow!("failed to resolve assign-part library fixture: {err}"))?;
    let mut assign_engine = Engine::new()?;
    assign_engine.import_eagle_library(&library_fixture_path)?;
    assign_engine.import(&cli.board_fixture_path)?;
    let assign_part_uuid = assign_engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP part missing for undo/redo roundtrip"))?
        .uuid;
    let baseline_assign_components = assign_engine.get_components()?;
    let assign_part_result = assign_engine.assign_part(AssignPartInput {
        uuid: cli.component_uuid,
        part_uuid: assign_part_uuid,
    })?;
    let after_assign_part = assign_engine.get_components()?;
    let assign_undo_result = assign_engine.undo()?;
    let after_assign_undo = assign_engine.get_components()?;
    let assign_redo_result = assign_engine.redo()?;
    let after_assign_redo = assign_engine.get_components()?;

    let mut package_engine = Engine::new()?;
    package_engine.import_eagle_library(&library_fixture_path)?;
    package_engine.import(&cli.board_fixture_path)?;
    let set_package_uuid = package_engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP package missing for undo/redo roundtrip"))?
        .package_uuid;
    let baseline_package_components = package_engine.get_components()?;
    let set_package_result = package_engine.set_package(SetPackageInput {
        uuid: cli.component_uuid,
        package_uuid: set_package_uuid,
    })?;
    let after_set_package = package_engine.get_components()?;
    let package_undo_result = package_engine.undo()?;
    let after_package_undo = package_engine.get_components()?;
    let package_redo_result = package_engine.redo()?;
    let after_package_redo = package_engine.get_components()?;

    let net_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_pcb")
        .canonicalize()
        .map_err(|err| anyhow::anyhow!("failed to resolve set-net-class fixture: {err}"))?;
    let mut net_class_engine = Engine::new()?;
    net_class_engine.import(&net_fixture_path)?;
    let baseline_net_info = net_class_engine.get_net_info()?;
    let gnd_uuid = baseline_net_info
        .iter()
        .find(|net| net.name == "GND")
        .ok_or_else(|| anyhow::anyhow!("GND net missing from undo/redo fixture"))?
        .uuid;
    let set_net_class_result = net_class_engine.set_net_class(SetNetClassInput {
        net_uuid: gnd_uuid,
        class_name: "power".to_string(),
        clearance: 125_000,
        track_width: 250_000,
        via_drill: 300_000,
        via_diameter: 600_000,
        diffpair_width: 0,
        diffpair_gap: 0,
    })?;
    let after_set_net_class = net_class_engine.get_net_info()?;
    let net_class_undo_result = net_class_engine.undo()?;
    let after_net_class_undo = net_class_engine.get_net_info()?;
    let net_class_redo_result = net_class_engine.redo()?;
    let after_net_class_redo = net_class_engine.get_net_info()?;

    let checks = vec![
        Check {
            name: "undo_restores_deleted_track_state".to_string(),
            status: if after_undo == baseline && after_delete != baseline {
                Status::Passed
            } else {
                Status::Failed
            },
            evidence: format!(
                "delete={}, undo={}, baseline_eq_undo={}, baseline_eq_delete={}",
                delete_result.description,
                undo_result.description,
                after_undo == baseline,
                after_delete == baseline
            ),
        },
        Check {
            name: "redo_reapplies_deleted_track_state".to_string(),
            status: if after_redo == after_delete && engine.can_undo() && !engine.can_redo() {
                Status::Passed
            } else {
                Status::Failed
            },
            evidence: format!(
                "redo={}, delete_eq_redo={}, can_undo={}, can_redo={}",
                redo_result.description,
                after_redo == after_delete,
                engine.can_undo(),
                engine.can_redo()
            ),
        },
        Check {
            name: "undo_restores_moved_component_state".to_string(),
            status: if after_move != baseline_components
                && after_move_undo == baseline_components
                && after_move_redo == after_move
            {
                Status::Passed
            } else {
                Status::Failed
            },
            evidence: format!(
                "move={}, undo={}, redo={}, moved_differs={}, undo_restored={}, redo_restored={}",
                move_result.description,
                move_undo_result.description,
                move_redo_result.description,
                after_move != baseline_components,
                after_move_undo == baseline_components,
                after_move_redo == after_move
            ),
        },
        Check {
            name: "undo_restores_design_rule_state".to_string(),
            status: if after_set_rule != baseline_rules
                && after_rule_undo == baseline_rules
                && after_rule_redo == after_set_rule
            {
                Status::Passed
            } else {
                Status::Failed
            },
            evidence: format!(
                "set_rule={}, undo={}, redo={}, set_differs={}, undo_restored={}, redo_restored={}",
                set_rule_result.description,
                rule_undo_result.description,
                rule_redo_result.description,
                after_set_rule != baseline_rules,
                after_rule_undo == baseline_rules,
                after_rule_redo == after_set_rule
            ),
        },
        Check {
            name: "undo_restores_assigned_part_state".to_string(),
            status: if after_assign_part != baseline_assign_components
                && after_assign_undo == baseline_assign_components
                && after_assign_redo == after_assign_part
            {
                Status::Passed
            } else {
                Status::Failed
            },
            evidence: format!(
                "assign_part={}, undo={}, redo={}, assign_differs={}, undo_restored={}, redo_restored={}",
                assign_part_result.description,
                assign_undo_result.description,
                assign_redo_result.description,
                after_assign_part != baseline_assign_components,
                after_assign_undo == baseline_assign_components,
                after_assign_redo == after_assign_part
            ),
        },
        Check {
            name: "undo_restores_set_package_state".to_string(),
            status: if after_set_package != baseline_package_components
                && after_package_undo == baseline_package_components
                && after_package_redo == after_set_package
            {
                Status::Passed
            } else {
                Status::Failed
            },
            evidence: format!(
                "set_package={}, undo={}, redo={}, package_differs={}, undo_restored={}, redo_restored={}",
                set_package_result.description,
                package_undo_result.description,
                package_redo_result.description,
                after_set_package != baseline_package_components,
                after_package_undo == baseline_package_components,
                after_package_redo == after_set_package
            ),
        },
        Check {
            name: "undo_restores_net_class_state".to_string(),
            status: if after_set_net_class != baseline_net_info
                && after_net_class_undo == baseline_net_info
                && after_net_class_redo == after_set_net_class
            {
                Status::Passed
            } else {
                Status::Failed
            },
            evidence: format!(
                "set_net_class={}, undo={}, redo={}, class_differs={}, undo_restored={}, redo_restored={}",
                set_net_class_result.description,
                net_class_undo_result.description,
                net_class_redo_result.description,
                after_set_net_class != baseline_net_info,
                after_net_class_undo == baseline_net_info,
                after_net_class_redo == after_set_net_class
            ),
        },
    ];

    let overall_status = if checks.iter().any(|c| c.status == Status::Failed) {
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

fn print_human(report: &Report) {
    println!("m3 undo/redo roundtrip preflight:");
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
    fn deferred_by_default() {
        let cli = Cli {
            json: false,
            allow_deferred: false,
            board_fixture_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../engine/testdata/import/kicad/partial-route-demo.kicad_pcb")
                .canonicalize()
                .expect("fixture path should resolve"),
            track_uuid: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc")
                .expect("uuid should parse"),
            component_uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
                .expect("uuid should parse"),
            target_x_nm: 15_000_000,
            target_y_nm: 12_000_000,
            target_rotation_deg: 90,
        };
        let report = build_report(&cli).expect("report should build");
        assert_eq!(report.overall_status, Status::Passed);
    }
}
