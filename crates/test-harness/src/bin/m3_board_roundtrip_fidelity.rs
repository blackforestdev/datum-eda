use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use eda_engine::api::{
    Engine, MoveComponentInput, RotateComponentInput, SetReferenceInput, SetValueInput,
};
use eda_test_harness::canonical_json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Cli {
    json: bool,
    allow_deferred: bool,
    board_fixture_path: PathBuf,
    simple_board_fixture_path: PathBuf,
    component_uuid: Uuid,
    track_uuid: Uuid,
    via_uuid: Uuid,
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
            eprintln!("m3_board_roundtrip_fidelity: {err:#}");
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
    let mut simple_board_fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../engine/testdata/import/kicad/simple-demo.kicad_pcb")
        .canonicalize()
        .map_err(|err| anyhow::anyhow!("failed to resolve default simple board fixture path: {err}"))?;
    let mut component_uuid =
        Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").expect("uuid should parse");
    let mut track_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
    let mut via_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
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
            "--simple-board-fixture-path" => {
                let value = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--simple-board-fixture-path requires a path argument")
                })?;
                simple_board_fixture_path = PathBuf::from(value);
            }
            "--component-uuid" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--component-uuid requires a UUID argument"))?;
                component_uuid = Uuid::parse_str(&value)?;
            }
            "--track-uuid" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--track-uuid requires a UUID argument"))?;
                track_uuid = Uuid::parse_str(&value)?;
            }
            "--via-uuid" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--via-uuid requires a UUID argument"))?;
                via_uuid = Uuid::parse_str(&value)?;
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
        simple_board_fixture_path,
        component_uuid,
        track_uuid,
        via_uuid,
        target_x_nm,
        target_y_nm,
        target_rotation_deg,
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p eda-test-harness --bin m3_board_roundtrip_fidelity -- [options]\n\
         Options:\n\
           --json                    Emit canonical JSON\n\
           --allow-deferred          Exit 0 when status is deferred\n\
           --board-fixture-path <p>  Fixture for component/track board-fidelity checks\n\
           --simple-board-fixture-path <p> Fixture for via board-fidelity checks\n\
           --component-uuid <uuid>   Component UUID used for component board-fidelity checks\n\
           --track-uuid <uuid>       Track UUID used for delete-track board-fidelity checks\n\
           --via-uuid <uuid>         Via UUID used for delete-via board-fidelity checks\n\
           --target-x-nm <nm>        Target X for move-component checks\n\
           --target-y-nm <nm>        Target Y for move-component checks\n\
           --target-rotation-deg <deg> Target rotation for move/rotate checks\n\
           -h, --help                Show this help"
    );
}

fn build_report(cli: &Cli) -> Result<Report> {
    let checks = vec![
        check("unmodified_board_writeback_identity", unmodified_board_identity(cli)),
        check("delete_track_board_roundtrip_fidelity", delete_track_fidelity(cli)),
        check("delete_via_board_roundtrip_fidelity", delete_via_fidelity(cli)),
        check(
            "delete_component_board_roundtrip_fidelity",
            delete_component_fidelity(cli),
        ),
        check("move_component_board_roundtrip_fidelity", move_component_fidelity(cli)),
        check(
            "rotate_component_board_roundtrip_fidelity",
            rotate_component_fidelity(cli),
        ),
        check("set_value_board_roundtrip_fidelity", set_value_fidelity(cli)),
        check("set_reference_board_roundtrip_fidelity", set_reference_fidelity(cli)),
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

fn check(name: &str, probe: Result<String>) -> Check {
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

fn unmodified_board_identity(cli: &Cli) -> Result<String> {
    let fixture_bytes = fs::read(&cli.board_fixture_path)?;
    let first_board = unique_temp_path("m3-board-unmodified-first", "kicad_pcb");
    let second_board = unique_temp_path("m3-board-unmodified-second", "kicad_pcb");

    let mut engine = Engine::new()?;
    engine.import(&cli.board_fixture_path)?;
    engine.save(&first_board)?;
    let first_bytes = fs::read(&first_board)?;
    if first_bytes != fixture_bytes {
        bail!("unmodified save was not byte-identical to imported KiCad board");
    }

    let mut reloaded = Engine::new()?;
    reloaded.import(&first_board)?;
    reloaded.save(&second_board)?;
    let second_bytes = fs::read(&second_board)?;
    if second_bytes != first_bytes {
        bail!("unmodified save→reimport→save changed KiCad board bytes");
    }

    cleanup_paths(&[first_board, second_board]);
    Ok("unmodified_board_bytes_identical=true, reimported_unmodified_board_stable=true".to_string())
}

fn delete_track_fidelity(cli: &Cli) -> Result<String> {
    run_netinfo_roundtrip(
        "m3-board-delete-track",
        &cli.board_fixture_path,
        |engine| {
            let deleted = engine.delete_track(&cli.track_uuid)?;
            Ok((engine.get_net_info()?, deleted.description))
        },
    )
}

fn delete_via_fidelity(cli: &Cli) -> Result<String> {
    run_netinfo_roundtrip(
        "m3-board-delete-via",
        &cli.simple_board_fixture_path,
        |engine| {
            let deleted = engine.delete_via(&cli.via_uuid)?;
            Ok((engine.get_net_info()?, deleted.description))
        },
    )
}

fn delete_component_fidelity(cli: &Cli) -> Result<String> {
    run_components_roundtrip(
        "m3-board-delete-component",
        &cli.board_fixture_path,
        |engine| {
            let deleted = engine.delete_component(&cli.component_uuid)?;
            Ok((engine.get_components()?, deleted.description))
        },
        |components| {
            if components.iter().any(|component| component.uuid == cli.component_uuid) {
                bail!("reimported delete_component save still contains deleted component");
            }
            Ok(())
        },
    )
}

fn move_component_fidelity(cli: &Cli) -> Result<String> {
    run_components_roundtrip(
        "m3-board-move-component",
        &cli.board_fixture_path,
        |engine| {
            let moved = engine.move_component(MoveComponentInput {
                uuid: cli.component_uuid,
                position: eda_engine::ir::geometry::Point::new(cli.target_x_nm, cli.target_y_nm),
                rotation: Some(cli.target_rotation_deg),
            })?;
            Ok((engine.get_components()?, moved.description))
        },
        |components| {
            let component = components
                .iter()
                .find(|component| component.uuid == cli.component_uuid)
                .context("reimported move_component save missing target component")?;
            if component.position.x != cli.target_x_nm
                || component.position.y != cli.target_y_nm
                || component.rotation != cli.target_rotation_deg
            {
                bail!("reimported move_component save did not restore expected placement");
            }
            Ok(())
        },
    )
}

fn rotate_component_fidelity(cli: &Cli) -> Result<String> {
    run_components_roundtrip(
        "m3-board-rotate-component",
        &cli.board_fixture_path,
        |engine| {
            let rotated = engine.rotate_component(RotateComponentInput {
                uuid: cli.component_uuid,
                rotation: 180,
            })?;
            Ok((engine.get_components()?, rotated.description))
        },
        |components| {
            let component = components
                .iter()
                .find(|component| component.uuid == cli.component_uuid)
                .context("reimported rotate_component save missing target component")?;
            if component.rotation != 180 {
                bail!("reimported rotate_component save did not restore expected rotation");
            }
            Ok(())
        },
    )
}

fn set_value_fidelity(cli: &Cli) -> Result<String> {
    run_components_roundtrip(
        "m3-board-set-value",
        &cli.board_fixture_path,
        |engine| {
            let updated = engine.set_value(SetValueInput {
                uuid: cli.component_uuid,
                value: "22k".to_string(),
            })?;
            Ok((engine.get_components()?, updated.description))
        },
        |components| {
            let component = components
                .iter()
                .find(|component| component.uuid == cli.component_uuid)
                .context("reimported set_value save missing target component")?;
            if component.value != "22k" {
                bail!("reimported set_value save did not restore expected value");
            }
            Ok(())
        },
    )
}

fn set_reference_fidelity(cli: &Cli) -> Result<String> {
    run_components_roundtrip(
        "m3-board-set-reference",
        &cli.board_fixture_path,
        |engine| {
            let updated = engine.set_reference(SetReferenceInput {
                uuid: cli.component_uuid,
                reference: "R10".to_string(),
            })?;
            Ok((engine.get_components()?, updated.description))
        },
        |components| {
            let component = components
                .iter()
                .find(|component| component.uuid == cli.component_uuid)
                .context("reimported set_reference save missing target component")?;
            if component.reference != "R10" {
                bail!("reimported set_reference save did not restore expected reference");
            }
            Ok(())
        },
    )
}

fn run_netinfo_roundtrip<M>(
    prefix: &str,
    fixture: &PathBuf,
    mutate: M,
) -> Result<String>
where
    M: Fn(&mut Engine) -> Result<(Vec<eda_engine::board::BoardNetInfo>, String)>,
{
    let first_board = unique_temp_path(&format!("{prefix}-first"), "kicad_pcb");
    let second_board = unique_temp_path(&format!("{prefix}-second"), "kicad_pcb");

    let mut engine = Engine::new()?;
    engine.import(fixture)?;
    let (expected_after_apply, operation_description) = mutate(&mut engine)?;
    engine.save(&first_board)?;

    let first_bytes = fs::read(&first_board)?;
    let mut reloaded = Engine::new()?;
    reloaded.import(&first_board)?;
    let reloaded_state = reloaded.get_net_info()?;
    if reloaded_state != expected_after_apply {
        bail!("{prefix} reimported net-info state did not match saved state");
    }
    reloaded.save(&second_board)?;
    let second_bytes = fs::read(&second_board)?;
    if first_bytes != second_bytes {
        bail!("{prefix} save→reimport→save changed KiCad board bytes");
    }

    cleanup_paths(&[first_board, second_board]);
    Ok(format!(
        "board_roundtrip_stable=true, reimport_restored_expected_netinfo=true, op={operation_description}"
    ))
}

fn run_components_roundtrip<M, V>(
    prefix: &str,
    fixture: &PathBuf,
    mutate: M,
    validate: V,
) -> Result<String>
where
    M: Fn(&mut Engine) -> Result<(Vec<eda_engine::board::ComponentInfo>, String)>,
    V: Fn(&Vec<eda_engine::board::ComponentInfo>) -> Result<()>,
{
    let first_board = unique_temp_path(&format!("{prefix}-first"), "kicad_pcb");
    let second_board = unique_temp_path(&format!("{prefix}-second"), "kicad_pcb");

    let mut engine = Engine::new()?;
    engine.import(fixture)?;
    let (expected_after_apply, operation_description) = mutate(&mut engine)?;
    engine.save(&first_board)?;

    let first_bytes = fs::read(&first_board)?;
    let mut reloaded = Engine::new()?;
    reloaded.import(&first_board)?;
    let reloaded_state = reloaded.get_components()?;
    if reloaded_state != expected_after_apply {
        bail!("{prefix} reimported component state did not match saved state");
    }
    validate(&reloaded_state)?;
    reloaded.save(&second_board)?;
    let second_bytes = fs::read(&second_board)?;
    if first_bytes != second_bytes {
        bail!("{prefix} save→reimport→save changed KiCad board bytes");
    }

    cleanup_paths(&[first_board, second_board]);
    Ok(format!(
        "board_roundtrip_stable=true, reimport_restored_expected_components=true, op={operation_description}"
    ))
}

fn unique_temp_path(prefix: &str, extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{prefix}-{}.{}", Uuid::new_v4(), extension))
}

fn cleanup_paths(paths: &[PathBuf]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}

fn print_human(report: &Report) {
    println!("m3 board roundtrip fidelity:");
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
    fn board_roundtrip_fidelity_passes_for_current_slice() {
        let cli = Cli {
            json: false,
            allow_deferred: false,
            board_fixture_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../engine/testdata/import/kicad/partial-route-demo.kicad_pcb")
                .canonicalize()
                .expect("fixture path should resolve"),
            simple_board_fixture_path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../engine/testdata/import/kicad/simple-demo.kicad_pcb")
                .canonicalize()
                .expect("fixture path should resolve"),
            component_uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
                .expect("uuid should parse"),
            track_uuid: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc")
                .expect("uuid should parse"),
            via_uuid: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc")
                .expect("uuid should parse"),
            target_x_nm: 15_000_000,
            target_y_nm: 12_000_000,
            target_rotation_deg: 90,
        };
        let report = build_report(&cli).expect("report should build");
        assert_eq!(report.overall_status, Status::Passed);
    }
}
