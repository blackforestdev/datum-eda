use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use eda_engine::api::{AssignPartInput, Engine, SetNetClassInput, SetPackageInput};
use eda_test_harness::canonical_json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Cli {
    json: bool,
    allow_deferred: bool,
    repo_root: PathBuf,
    roundtrip_board_fixture_path: PathBuf,
    track_uuid: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Passed,
    Failed,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurfaceCheck {
    surface: String,
    status: Status,
    evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Report {
    schema_version: u32,
    overall_status: Status,
    checks: Vec<SurfaceCheck>,
}

#[derive(Debug, Deserialize)]
struct CliModifyReport {
    actions: Vec<String>,
    last_result: Option<CliOperationResult>,
    saved_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CliOperationResult {
    description: String,
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("m3_write_surface_parity: {err:#}");
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
    let repo_root = detect_repo_root()?;
    let mut roundtrip_board_fixture_path =
        repo_root.join("crates/engine/testdata/import/kicad/partial-route-demo.kicad_pcb");
    let mut track_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");

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
            "--track-uuid" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--track-uuid requires a UUID argument"))?;
                track_uuid = Uuid::parse_str(&value)?;
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
        repo_root,
        roundtrip_board_fixture_path,
        track_uuid,
    })
}

fn print_usage() {
    println!(
        "Usage: cargo run -p eda-test-harness --bin m3_write_surface_parity -- [options]\n\
         Options:\n\
           --json                          Emit canonical JSON\n\
           --allow-deferred                Exit 0 when status is deferred\n\
           --roundtrip-board-fixture-path <p>  KiCad board fixture used for delete/undo/redo/save parity\n\
           --track-uuid <uuid>             Track UUID used for delete/undo/redo parity\n\
           -h, --help                      Show this help"
    );
}

fn detect_repo_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .context("failed to resolve repository root")
}

fn build_report(cli: &Cli) -> Result<Report> {
    let checks = vec![
        check_engine_surface(cli),
        check_daemon_surface(cli),
        check_mcp_surface(cli),
        check_cli_surface(cli),
    ];
    let overall_status = if checks.iter().any(|c| c.status == Status::Failed) {
        Status::Failed
    } else if checks.iter().all(|c| c.status == Status::Passed) {
        Status::Passed
    } else {
        Status::Deferred
    };

    Ok(Report {
        schema_version: 1,
        overall_status,
        checks,
    })
}

fn check_engine_surface(cli: &Cli) -> SurfaceCheck {
    match engine_surface_result(cli) {
        Ok(evidence) => SurfaceCheck {
            surface: "engine_write_surface".to_string(),
            status: Status::Passed,
            evidence,
        },
        Err(err) => SurfaceCheck {
            surface: "engine_write_surface".to_string(),
            status: Status::Failed,
            evidence: err.to_string(),
        },
    }
}

fn engine_surface_result(cli: &Cli) -> Result<String> {
    let mut engine = Engine::new()?;
    engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline = engine.get_net_info()?;
    let baseline_check = engine.get_check_report()?;
    let delete = engine.delete_track(&cli.track_uuid)?;
    let after_delete = engine.get_net_info()?;
    let after_delete_check = engine.get_check_report()?;
    let undo = engine.undo()?;
    let after_undo = engine.get_net_info()?;
    let redo = engine.redo()?;
    let after_redo = engine.get_net_info()?;

    if after_delete == baseline {
        bail!("delete_track did not change engine net state");
    }
    if after_undo != baseline {
        bail!("undo did not restore engine baseline state");
    }
    if after_redo != after_delete {
        bail!("redo did not restore engine deleted state");
    }
    let baseline_diagnostics = match baseline_check {
        eda_engine::api::CheckReport::Board { diagnostics, .. } => diagnostics,
        _ => bail!("engine baseline check report was not a board report"),
    };
    let after_delete_diagnostics = match after_delete_check {
        eda_engine::api::CheckReport::Board { diagnostics, .. } => diagnostics,
        _ => bail!("engine delete_track check report was not a board report"),
    };
    if !baseline_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == "partially_routed_net")
    {
        bail!("engine baseline check report missing partially_routed_net");
    }
    if !after_delete_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.kind == "net_without_copper")
    {
        bail!("engine delete_track follow-up check report missing net_without_copper");
    }

    let target = unique_temp_path("engine-surface-save", "kicad_pcb");
    engine.save(&target)?;
    let mut reloaded = Engine::new()?;
    reloaded.import(&target)?;
    let reloaded_after_save = reloaded.get_net_info()?;
    if reloaded_after_save != after_redo {
        bail!("saved engine board did not reimport to the current deleted state");
    }

    let via_fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let via_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
    let mut via_engine = Engine::new()?;
    via_engine.import(&via_fixture)?;
    let baseline_via_state = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&via_fixture)?;
        baseline_engine.get_net_info()?
    };
    let delete_via = via_engine.delete_via(&via_uuid)?;
    let via_deleted_state = via_engine.get_net_info()?;
    let via_target = unique_temp_path("engine-surface-via-save", "kicad_pcb");
    via_engine.save(&via_target)?;
    let mut reloaded_via = Engine::new()?;
    reloaded_via.import(&via_target)?;
    if reloaded_via.get_net_info()? != via_deleted_state {
        bail!("saved engine via-deleted board did not reimport to the current deleted state");
    }
    let baseline_via_gnd = baseline_via_state
        .iter()
        .find(|net| net.name == "GND")
        .ok_or_else(|| anyhow::anyhow!("baseline via state missing GND"))?;
    let after_via_gnd = via_deleted_state
        .iter()
        .find(|net| net.name == "GND")
        .ok_or_else(|| anyhow::anyhow!("via-deleted state missing GND"))?;
    if baseline_via_gnd.vias == after_via_gnd.vias {
        bail!("delete_via did not change engine follow-up net-info state");
    }

    let mut component_engine = Engine::new()?;
    component_engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline_component_list = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_components()?
    };
    let delete_component = component_engine
        .delete_component(&Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())?;
    let component_target = unique_temp_path("engine-surface-component-save", "kicad_pcb");
    component_engine.save(&component_target)?;
    let mut reloaded_component = Engine::new()?;
    reloaded_component.import(&component_target)?;
    let reloaded_component_list = reloaded_component.get_components()?;
    if reloaded_component_list
        .iter()
        .any(|component| component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
    {
        bail!("saved delete_component board still reimported deleted component");
    }
    if baseline_component_list.len() == reloaded_component_list.len() {
        bail!("delete_component did not change engine follow-up components state");
    }

    let mut rule_engine = Engine::new()?;
    rule_engine.import(&via_fixture)?;
    let baseline_rules = rule_engine.get_design_rules()?;
    let set_rule = rule_engine.set_design_rule(eda_engine::api::SetDesignRuleInput {
        rule_type: eda_engine::rules::ast::RuleType::ClearanceCopper,
        scope: eda_engine::rules::ast::RuleScope::All,
        parameters: eda_engine::rules::ast::RuleParams::Clearance { min: 125_000 },
        priority: 10,
        name: Some("default clearance".to_string()),
    })?;
    let rule_target = unique_temp_path("engine-surface-rule-save", "kicad_pcb");
    rule_engine.save(&rule_target)?;
    let mut reloaded_rule = Engine::new()?;
    reloaded_rule.import(&rule_target)?;
    if reloaded_rule.get_design_rules()?.len() != 1 {
        bail!("saved engine rule-mutated board did not reimport one design rule");
    }
    if baseline_rules.len() == reloaded_rule.get_design_rules()?.len() {
        bail!("set_design_rule did not change engine follow-up design-rules state");
    }

    let mut move_engine = Engine::new()?;
    move_engine.import(&cli.roundtrip_board_fixture_path)?;
    let moved = move_engine.move_component(eda_engine::api::MoveComponentInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        position: eda_engine::ir::geometry::Point::new(15_000_000, 12_000_000),
        rotation: Some(90),
    })?;
    let move_target = unique_temp_path("engine-surface-move-save", "kicad_pcb");
    move_engine.save(&move_target)?;
    let mut reloaded_move = Engine::new()?;
    reloaded_move.import(&move_target)?;
    let moved_component = reloaded_move
        .get_components()?
        .into_iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("reloaded moved component missing R1"))?;
    if moved_component.position.x != 15_000_000
        || moved_component.position.y != 12_000_000
        || moved_component.rotation != 90
    {
        bail!("saved moved component did not reimport to the expected position");
    }
    let baseline_move_airwires = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_unrouted()?
    };
    let moved_airwires = move_engine.get_unrouted()?;
    if moved_airwires.len() != baseline_move_airwires.len() {
        bail!("move_component changed engine airwire count unexpectedly");
    }
    if moved_airwires.first().map(|airwire| airwire.distance_nm)
        == baseline_move_airwires
            .first()
            .map(|airwire| airwire.distance_nm)
    {
        bail!("move_component did not change engine unrouted derived state");
    }

    let mut rotate_engine = Engine::new()?;
    rotate_engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline_rotate_components = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_components()?
    };
    let rotate = rotate_engine.rotate_component(eda_engine::api::RotateComponentInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        rotation: 180,
    })?;
    let rotate_target = unique_temp_path("engine-surface-rotate-save", "kicad_pcb");
    rotate_engine.save(&rotate_target)?;
    let mut reloaded_rotate = Engine::new()?;
    reloaded_rotate.import(&rotate_target)?;
    let rotated_component = reloaded_rotate
        .get_components()?
        .into_iter()
        .find(|component| component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
        .ok_or_else(|| anyhow::anyhow!("reloaded rotated component missing target"))?;
    if rotated_component.rotation != 180 {
        bail!("saved rotated component did not reimport expected rotation");
    }
    let baseline_rotated = baseline_rotate_components
        .iter()
        .find(|component| component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
        .ok_or_else(|| anyhow::anyhow!("baseline rotate components missing target"))?;
    if baseline_rotated.rotation == rotated_component.rotation {
        bail!("rotate_component did not change engine follow-up components state");
    }

    let mut value_engine = Engine::new()?;
    value_engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline_value_components = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_components()?
    };
    let set_value = value_engine.set_value(eda_engine::api::SetValueInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        value: "22k".to_string(),
    })?;
    let value_target = unique_temp_path("engine-surface-value-save", "kicad_pcb");
    value_engine.save(&value_target)?;
    let mut reloaded_value = Engine::new()?;
    reloaded_value.import(&value_target)?;
    let reloaded_value_components = reloaded_value.get_components()?;
    let baseline_r1 = baseline_value_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("baseline value components missing R1"))?;
    let updated_r1 = reloaded_value_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("reloaded value components missing R1"))?;
    if updated_r1.value != "22k" {
        bail!("saved set_value component did not reimport expected value");
    }
    if baseline_r1.value == updated_r1.value {
        bail!("set_value did not change engine follow-up components state");
    }

    let mut reference_engine = Engine::new()?;
    reference_engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline_reference_components = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_components()?
    };
    let set_reference = reference_engine.set_reference(eda_engine::api::SetReferenceInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        reference: "R10".to_string(),
    })?;
    let reference_target = unique_temp_path("engine-surface-reference-save", "kicad_pcb");
    reference_engine.save(&reference_target)?;
    let mut reloaded_reference = Engine::new()?;
    reloaded_reference.import(&reference_target)?;
    let reloaded_reference_components = reloaded_reference.get_components()?;
    let baseline_reference_r1 = baseline_reference_components
        .iter()
        .find(|component| component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
        .ok_or_else(|| anyhow::anyhow!("baseline reference components missing target component"))?;
    let updated_reference = reloaded_reference_components
        .iter()
        .find(|component| component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())
        .ok_or_else(|| anyhow::anyhow!("reloaded reference components missing target component"))?;
    if updated_reference.reference != "R10" {
        bail!("saved set_reference component did not reimport expected reference");
    }
    if baseline_reference_r1.reference == updated_reference.reference {
        bail!("set_reference did not change engine follow-up components state");
    }

    let library_fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/eagle/simple-opamp.lbr");
    let mut assign_engine = Engine::new()?;
    assign_engine.import_eagle_library(&library_fixture)?;
    assign_engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline_assign_components = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import_eagle_library(&library_fixture)?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_components()?
    };
    let part_uuid = assign_engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP part missing from pool"))?
        .uuid;
    let assign_part = assign_engine.assign_part(AssignPartInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        part_uuid,
    })?;
    let assign_target = unique_temp_path("engine-surface-assign-part-save", "kicad_pcb");
    assign_engine.save(&assign_target)?;
    let assign_saved = std::fs::read_to_string(&assign_target)?;
    if !assign_saved.contains("(footprint \"ALT-3\"") {
        bail!("saved assign_part component did not rewrite expected footprint name");
    }
    let mut reloaded_assign = Engine::new()?;
    reloaded_assign.import_eagle_library(&library_fixture)?;
    reloaded_assign.import(&assign_target)?;
    let reloaded_assign_components = reloaded_assign.get_components()?;
    let reloaded_assign_sig = reloaded_assign
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("reloaded assign_part nets missing SIG"))?;
    let baseline_assign_r1 = baseline_assign_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("baseline assign_part components missing R1"))?;
    let updated_assign_r1 = reloaded_assign_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("reloaded assign_part components missing R1"))?;
    if updated_assign_r1.value != "ALTAMP" {
        bail!("saved assign_part component did not reimport expected value");
    }
    if baseline_assign_r1.value == updated_assign_r1.value {
        bail!("assign_part did not change engine follow-up components state");
    }
    if reloaded_assign_sig.pins.len() != 1 {
        bail!("assign_part did not change engine follow-up net-info state");
    }
    let lmv321_part_uuid = assign_engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("LMV321 part missing from pool"))?
        .uuid;
    let mut remap_engine = Engine::new()?;
    remap_engine.import_eagle_library(&library_fixture)?;
    remap_engine.import(&cli.roundtrip_board_fixture_path)?;
    remap_engine.assign_part(AssignPartInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        part_uuid: lmv321_part_uuid,
    })?;
    let remap_intermediate_sig = remap_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("intermediate assign_part nets missing SIG"))?;
    remap_engine.assign_part(AssignPartInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        part_uuid,
    })?;
    let remap_after_sig = remap_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("remapped assign_part nets missing SIG"))?;
    if remap_after_sig.pins.len() != remap_intermediate_sig.pins.len() {
        bail!("assign_part logical remap did not preserve engine follow-up net-info state");
    }

    let mut package_engine = Engine::new()?;
    package_engine.import_eagle_library(&library_fixture)?;
    package_engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline_package_components = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import_eagle_library(&library_fixture)?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_components()?
    };
    let package_uuid = package_engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP package missing from pool"))?
        .package_uuid;
    let set_package = package_engine.set_package(SetPackageInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        package_uuid,
    })?;
    let package_target = unique_temp_path("engine-surface-set-package-save", "kicad_pcb");
    package_engine.save(&package_target)?;
    let package_saved = std::fs::read_to_string(&package_target)?;
    if !package_saved.contains("(footprint \"ALT-3\"") {
        bail!("saved set_package component did not rewrite expected footprint name");
    }
    let mut reloaded_package = Engine::new()?;
    reloaded_package.import_eagle_library(&library_fixture)?;
    reloaded_package.import(&package_target)?;
    let reloaded_package_components = reloaded_package.get_components()?;
    let baseline_package_r1 = baseline_package_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("baseline set_package components missing R1"))?;
    let updated_package_r1 = reloaded_package_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("reloaded set_package components missing R1"))?;
    if updated_package_r1.package_uuid != package_uuid {
        bail!("saved set_package component did not reimport expected package uuid");
    }
    if baseline_package_r1.package_uuid == updated_package_r1.package_uuid {
        bail!("set_package did not change engine follow-up components state");
    }
    let lmv321_part_uuid = package_engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("LMV321 part missing from pool"))?
        .uuid;
    let mut package_remap_engine = Engine::new()?;
    package_remap_engine.import_eagle_library(&library_fixture)?;
    package_remap_engine.import(&cli.roundtrip_board_fixture_path)?;
    package_remap_engine.assign_part(AssignPartInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        part_uuid: lmv321_part_uuid,
    })?;
    let package_remap_intermediate_sig = package_remap_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("intermediate set_package nets missing SIG"))?;
    package_remap_engine.set_package(SetPackageInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        package_uuid,
    })?;
    let package_remap_after_sig = package_remap_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("remapped set_package nets missing SIG"))?;
    if package_remap_after_sig.pins.len() != package_remap_intermediate_sig.pins.len() {
        bail!("set_package logical remap did not preserve engine follow-up net-info state");
    }
    let altamp_part_uuid = package_engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP part missing from pool"))?
        .uuid;
    let mut explicit_package_engine = Engine::new()?;
    explicit_package_engine.import_eagle_library(&library_fixture)?;
    explicit_package_engine.import(&cli.roundtrip_board_fixture_path)?;
    explicit_package_engine.assign_part(AssignPartInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        part_uuid: lmv321_part_uuid,
    })?;
    let explicit_package_intermediate_sig = explicit_package_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("intermediate explicit set_package nets missing SIG"))?;
    let explicit_package = explicit_package_engine.set_package_with_part(
        eda_engine::api::SetPackageWithPartInput {
            uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            package_uuid,
            part_uuid: altamp_part_uuid,
        },
    )?;
    let explicit_package_target =
        unique_temp_path("engine-surface-set-package-with-part-save", "kicad_pcb");
    explicit_package_engine.save(&explicit_package_target)?;
    let mut reloaded_explicit_package = Engine::new()?;
    reloaded_explicit_package.import_eagle_library(&library_fixture)?;
    reloaded_explicit_package.import(&explicit_package_target)?;
    let reloaded_explicit_package_components = reloaded_explicit_package.get_components()?;
    let explicit_package_component = reloaded_explicit_package_components
        .iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("reloaded explicit package components missing R1"))?;
    let explicit_package_after_sig = explicit_package_engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "SIG")
        .ok_or_else(|| anyhow::anyhow!("explicit set_package nets missing SIG"))?;
    if explicit_package_component.package_uuid != package_uuid {
        bail!("set_package_with_part did not persist expected package uuid");
    }
    if explicit_package_component.value != "ALTAMP" {
        bail!("set_package_with_part did not persist expected explicit part value");
    }
    if explicit_package_after_sig.pins.len() != explicit_package_intermediate_sig.pins.len() {
        bail!("set_package_with_part did not preserve engine follow-up net-info state");
    }

    let net_fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let mut net_class_engine = Engine::new()?;
    net_class_engine.import(&net_fixture)?;
    let baseline_net_info = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&net_fixture)?;
        baseline_engine.get_net_info()?
    };
    let gnd_uuid = baseline_net_info
        .iter()
        .find(|net| net.name == "GND")
        .ok_or_else(|| anyhow::anyhow!("baseline net info missing GND"))?
        .uuid;
    let set_net_class = net_class_engine.set_net_class(SetNetClassInput {
        net_uuid: gnd_uuid,
        class_name: "power".to_string(),
        clearance: 125_000,
        track_width: 250_000,
        via_drill: 300_000,
        via_diameter: 600_000,
        diffpair_width: 0,
        diffpair_gap: 0,
    })?;
    let net_class_target = unique_temp_path("engine-surface-net-class-save", "kicad_pcb");
    net_class_engine.save(&net_class_target)?;
    let mut reloaded_net_class = Engine::new()?;
    reloaded_net_class.import(&net_class_target)?;
    let reloaded_net_info = reloaded_net_class.get_net_info()?;
    let baseline_gnd = baseline_net_info
        .iter()
        .find(|net| net.uuid == gnd_uuid)
        .ok_or_else(|| anyhow::anyhow!("baseline net info missing GND uuid"))?;
    let updated_gnd = reloaded_net_info
        .iter()
        .find(|net| net.uuid == gnd_uuid)
        .ok_or_else(|| anyhow::anyhow!("reloaded net info missing GND uuid"))?;
    if updated_gnd.class != "power" {
        bail!("saved set_net_class net did not reimport expected class");
    }
    if baseline_gnd.class == updated_gnd.class {
        bail!("set_net_class did not change engine follow-up net-info state");
    }

    Ok(format!(
        "delete={}, undo={}, redo={}, saved={}, reimported_deleted_state=true, delete_followup_check_changed=true, delete_via={}, via_saved={}, via_reimported_deleted_state=true, delete_via_followup_net_info_changed=true, delete_component={}, component_saved={}, component_followup_components_changed=true, set_rule={}, rule_saved={}, rule_reimported=true, rule_followup_query_changed=true, move_component={}, moved_saved={}, move_reimported=true, move_followup_unrouted_changed=true, rotate_component={}, rotate_saved={}, rotate_followup_components_changed=true, set_value={}, value_saved={}, value_followup_components_changed=true, set_reference={}, reference_saved={}, reference_followup_components_changed=true, assign_part={}, assign_saved={}, assign_part_rewrote_footprint=true, assign_part_followup_components_changed=true, assign_part_followup_net_info_changed=true, assign_part_logical_remap_preserved=true, set_package={}, package_saved={}, set_package_followup_components_changed=true, set_package_followup_net_info_changed=true, set_package_logical_remap_preserved=true, set_package_with_part={}, explicit_package_saved={}, set_package_with_part_followup_net_info_changed=true, set_net_class={}, net_class_saved={}, set_net_class_followup_net_info_changed=true",
        delete.description,
        undo.description,
        redo.description,
        target.display(),
        delete_via.description,
        via_target.display(),
        delete_component.description,
        component_target.display(),
        set_rule.description,
        rule_target.display(),
        moved.description,
        move_target.display(),
        rotate.description,
        rotate_target.display(),
        set_value.description,
        value_target.display(),
        set_reference.description,
        reference_target.display(),
        assign_part.description,
        assign_target.display(),
        set_package.description,
        package_target.display(),
        explicit_package.description,
        explicit_package_target.display(),
        set_net_class.description,
        net_class_target.display()
    ))
}

fn check_daemon_surface(cli: &Cli) -> SurfaceCheck {
    match daemon_surface_result(cli) {
        Ok(evidence) => SurfaceCheck {
            surface: "daemon_write_surface".to_string(),
            status: Status::Passed,
            evidence,
        },
        Err(err) => SurfaceCheck {
            surface: "daemon_write_surface".to_string(),
            status: Status::Failed,
            evidence: err.to_string(),
        },
    }
}

fn daemon_surface_result(cli: &Cli) -> Result<String> {
    let save_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "save_dispatch_writes_current_m3_slice_to_requested_path",
            ])
            .current_dir(&cli.repo_root),
        "daemon save dispatch parity probe",
    )?;
    let roundtrip_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_track_undo_and_redo_dispatch_round_trip",
            ])
            .current_dir(&cli.repo_root),
        "daemon roundtrip dispatch parity probe",
    )?;
    let delete_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_track_dispatch_updates_followup_check_report",
            ])
            .current_dir(&cli.repo_root),
        "daemon delete-track derived-state parity probe",
    )?;
    let via_roundtrip_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_via_undo_and_redo_dispatch_round_trip",
            ])
            .current_dir(&cli.repo_root),
        "daemon via roundtrip dispatch parity probe",
    )?;
    let via_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_via_dispatch_updates_followup_net_info_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon delete-via derived-state parity probe",
    )?;
    let component_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_component_dispatch_updates_component_list",
            ])
            .current_dir(&cli.repo_root),
        "daemon delete-component parity probe",
    )?;
    let component_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "delete_component_dispatch_updates_followup_components_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon delete-component derived-state parity probe",
    )?;
    let rule_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_design_rule_dispatch_persists_rule_in_memory",
            ])
            .current_dir(&cli.repo_root),
        "daemon rule dispatch parity probe",
    )?;
    let rule_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_design_rule_dispatch_updates_followup_design_rules_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon rule derived-state parity probe",
    )?;
    let value_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_value_dispatch_updates_component_value",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-value parity probe",
    )?;
    let value_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_value_dispatch_updates_followup_components_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-value derived-state parity probe",
    )?;
    let reference_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_reference_dispatch_updates_component_reference",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-reference parity probe",
    )?;
    let reference_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_reference_dispatch_updates_followup_components_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-reference derived-state parity probe",
    )?;
    let move_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "move_component_dispatch_updates_component_position",
            ])
            .current_dir(&cli.repo_root),
        "daemon move-component parity probe",
    )?;
    let move_derived_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "move_component_dispatch_updates_followup_unrouted_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon move-component derived-state parity probe",
    )?;
    let rotate_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "rotate_component_dispatch_updates_component_rotation",
            ])
            .current_dir(&cli.repo_root),
        "daemon rotate-component parity probe",
    )?;
    let rotate_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "rotate_component_dispatch_updates_followup_components_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon rotate-component derived-state parity probe",
    )?;
    let assign_part_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "assign_part_dispatch_updates_component_value",
            ])
            .current_dir(&cli.repo_root),
        "daemon assign-part parity probe",
    )?;
    let assign_part_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "assign_part_dispatch_updates_followup_net_info_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon assign-part derived-state parity probe",
    )?;
    let assign_part_remap_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "assign_part_dispatch_preserves_logical_nets_across_known_part_remap",
            ])
            .current_dir(&cli.repo_root),
        "daemon assign-part logical-remap parity probe",
    )?;
    let set_package_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_package_dispatch_updates_component_package",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-package parity probe",
    )?;
    let set_package_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_package_dispatch_updates_followup_net_info_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-package net-info derived-state parity probe",
    )?;
    let set_package_with_part_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_package_with_part_dispatch_preserves_logical_nets_for_explicit_candidate",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-package-with-part parity probe",
    )?;
    let set_package_remap_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_package_dispatch_preserves_logical_nets_across_known_part_remap",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-package logical-remap parity probe",
    )?;
    let set_net_class_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_net_class_dispatch_updates_net_class",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-net-class parity probe",
    )?;
    let set_net_class_followup_test = run_command_checked(
        Command::new("cargo")
            .args([
                "test",
                "-q",
                "-p",
                "eda-engine-daemon",
                "set_net_class_dispatch_updates_followup_net_info_query",
            ])
            .current_dir(&cli.repo_root),
        "daemon set-net-class derived-state parity probe",
    )?;

    Ok(format!(
        "behavioral dispatch tests passed: save_dispatch_writes_current_m3_slice_to_requested_path, delete_track_undo_and_redo_dispatch_round_trip, delete_track_dispatch_updates_followup_check_report, delete_via_undo_and_redo_dispatch_round_trip, delete_via_dispatch_updates_followup_net_info_query, delete_component_dispatch_updates_component_list, delete_component_dispatch_updates_followup_components_query, set_design_rule_dispatch_persists_rule_in_memory, set_design_rule_dispatch_updates_followup_design_rules_query, set_value_dispatch_updates_component_value, set_value_dispatch_updates_followup_components_query, set_reference_dispatch_updates_component_reference, set_reference_dispatch_updates_followup_components_query, move_component_dispatch_updates_component_position, move_component_dispatch_updates_followup_unrouted_query, rotate_component_dispatch_updates_component_rotation, rotate_component_dispatch_updates_followup_components_query, assign_part_dispatch_updates_component_value, assign_part_dispatch_updates_followup_net_info_query, assign_part_dispatch_preserves_logical_nets_across_known_part_remap, set_package_dispatch_updates_component_package, set_package_dispatch_updates_followup_net_info_query, set_package_with_part_dispatch_preserves_logical_nets_for_explicit_candidate, set_package_dispatch_preserves_logical_nets_across_known_part_remap, set_net_class_dispatch_updates_net_class, set_net_class_dispatch_updates_followup_net_info_query (outputs: {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
        save_test,
        roundtrip_test,
        delete_followup_test,
        via_roundtrip_test,
        via_followup_test,
        component_test,
        component_followup_test,
        rule_test,
        rule_followup_test,
        value_test,
        value_followup_test,
        reference_test,
        reference_followup_test,
        move_test,
        move_derived_test,
        rotate_test,
        rotate_followup_test,
        assign_part_test,
        assign_part_followup_test,
        assign_part_remap_test,
        set_package_test,
        set_package_followup_test,
        set_package_with_part_test,
        set_package_remap_test,
        set_net_class_test,
        set_net_class_followup_test
    ))
}

fn check_mcp_surface(cli: &Cli) -> SurfaceCheck {
    match mcp_surface_result(cli) {
        Ok(evidence) => SurfaceCheck {
            surface: "mcp_write_surface".to_string(),
            status: Status::Passed,
            evidence,
        },
        Err(err) => SurfaceCheck {
            surface: "mcp_write_surface".to_string(),
            status: Status::Failed,
            evidence: err.to_string(),
        },
    }
}

fn mcp_surface_result(cli: &Cli) -> Result<String> {
    let script = r#"
import pathlib
import sys
import unittest

repo = pathlib.Path(sys.argv[1])
top = repo / "mcp-server"
wanted = {
    "test_tools_call_dispatches_save",
    "test_tools_call_dispatches_delete_track",
    "test_tools_call_delete_track_changes_followup_check_report",
    "test_tools_call_dispatches_delete_component",
    "test_tools_call_delete_component_changes_followup_components_response",
    "test_tools_call_dispatches_delete_via",
    "test_tools_call_delete_via_changes_followup_net_info_response",
    "test_tools_call_dispatches_move_component",
    "test_tools_call_move_component_changes_followup_unrouted_response",
    "test_tools_call_dispatches_rotate_component",
    "test_tools_call_rotate_component_changes_followup_components_response",
    "test_tools_call_dispatches_set_design_rule",
    "test_tools_call_set_design_rule_changes_followup_design_rules_response",
    "test_tools_call_dispatches_set_value",
    "test_tools_call_set_value_changes_followup_components_response",
    "test_tools_call_dispatches_set_reference",
    "test_tools_call_set_reference_changes_followup_components_response",
    "test_tools_call_dispatches_assign_part",
    "test_tools_call_assign_part_changes_followup_net_info_response",
    "test_tools_call_assign_part_preserves_logical_nets_across_known_part_remap_response",
    "test_tools_call_dispatches_set_package",
    "test_tools_call_dispatches_set_package_with_part",
    "test_tools_call_set_package_changes_followup_net_info_response",
    "test_tools_call_set_package_preserves_logical_nets_across_known_part_remap_response",
    "test_tools_call_set_package_with_part_preserves_logical_nets_for_explicit_candidate",
    "test_tools_call_dispatches_set_net_class",
    "test_tools_call_set_net_class_changes_followup_net_info_response",
    "test_tools_call_dispatches_undo_and_redo",
}

def iter_tests(suite):
    for test in suite:
        if isinstance(test, unittest.TestSuite):
            yield from iter_tests(test)
        else:
            yield test

discovered = unittest.defaultTestLoader.discover(
    start_dir=top,
    pattern="test_*.py",
    top_level_dir=top,
)
selected = unittest.TestSuite()
found = set()
for test in iter_tests(discovered):
    name = test.id().rsplit(".", 1)[-1]
    if name in wanted:
        selected.addTest(test)
        found.add(name)

missing = sorted(wanted - found)
if missing:
    print("missing MCP parity tests:", ", ".join(missing))
    sys.exit(1)

result = unittest.TextTestRunner(verbosity=0).run(selected)
if not result.wasSuccessful():
    sys.exit(1)
print("selected MCP write-surface dispatch tests passed")
"#;

    let output = run_command_checked(
        Command::new("python3")
            .arg("-c")
            .arg(script)
            .arg(&cli.repo_root)
            .current_dir(&cli.repo_root),
        "mcp write-surface parity probe",
    )?;

    Ok(output)
}

fn check_cli_surface(cli: &Cli) -> SurfaceCheck {
    match (
        cli_surface_result(cli),
        cli_via_surface_result(cli),
        cli_component_surface_result(cli),
        cli_move_surface_result(cli),
        cli_rotate_surface_result(cli),
        cli_rule_surface_result(cli),
        cli_value_surface_result(cli),
        cli_reference_surface_result(cli),
        cli_assign_part_surface_result(cli),
        cli_assign_part_remap_surface_result(cli),
        cli_set_package_surface_result(cli),
        cli_set_package_remap_surface_result(cli),
        cli_set_package_with_part_surface_result(cli),
        cli_set_net_class_surface_result(cli),
    ) {
        (
            Ok(track_evidence),
            Ok(via_evidence),
            Ok(component_evidence),
            Ok(move_evidence),
            Ok(rotate_evidence),
            Ok(rule_evidence),
            Ok(value_evidence),
            Ok(reference_evidence),
            Ok(assign_part_evidence),
            Ok(assign_part_remap_evidence),
            Ok(set_package_evidence),
            Ok(set_package_remap_evidence),
            Ok(set_package_with_part_evidence),
            Ok(net_class_evidence),
        ) => SurfaceCheck {
            surface: "cli_modify_surface".to_string(),
            status: Status::Passed,
            evidence: format!(
                "{track_evidence}; {via_evidence}; {component_evidence}; {move_evidence}; {rotate_evidence}; {rule_evidence}; {value_evidence}; {reference_evidence}; {assign_part_evidence}; {assign_part_remap_evidence}; {set_package_evidence}; {set_package_remap_evidence}; {set_package_with_part_evidence}; {net_class_evidence}"
            ),
        },
        (Err(err), _, _, _, _, _, _, _, _, _, _, _, _, _)
        | (_, Err(err), _, _, _, _, _, _, _, _, _, _, _, _)
        | (_, _, Err(err), _, _, _, _, _, _, _, _, _, _, _)
        | (_, _, _, Err(err), _, _, _, _, _, _, _, _, _, _)
        | (_, _, _, _, Err(err), _, _, _, _, _, _, _, _, _)
        | (_, _, _, _, _, Err(err), _, _, _, _, _, _, _, _)
        | (_, _, _, _, _, _, Err(err), _, _, _, _, _, _, _)
        | (_, _, _, _, _, _, _, Err(err), _, _, _, _, _, _)
        | (_, _, _, _, _, _, _, _, Err(err), _, _, _, _, _)
        | (_, _, _, _, _, _, _, _, _, Err(err), _, _, _, _)
        | (_, _, _, _, _, _, _, _, _, _, Err(err), _, _, _)
        | (_, _, _, _, _, _, _, _, _, _, _, Err(err), _, _)
        | (_, _, _, _, _, _, _, _, _, _, _, _, Err(err), _)
        | (_, _, _, _, _, _, _, _, _, _, _, _, _, Err(err)) => SurfaceCheck {
            surface: "cli_modify_surface".to_string(),
            status: Status::Failed,
            evidence: err.to_string(),
        },
    }
}

fn cli_surface_result(cli: &Cli) -> Result<String> {
    let roundtrip_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--delete-track")
        .arg(cli.track_uuid.to_string())
        .arg("--undo")
        .arg("1")
        .arg("--redo")
        .arg("1")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI modify parity probe")?;
    if !roundtrip_output.status.success() {
        bail!(
            "CLI roundtrip parity probe failed with status {:?}: {}",
            roundtrip_output.status.code(),
            String::from_utf8_lossy(&roundtrip_output.stderr).trim()
        );
    }
    let roundtrip: CliModifyReport = serde_json::from_slice(&roundtrip_output.stdout)
        .context("failed to parse CLI roundtrip JSON output")?;

    let expected_actions = vec![
        format!("delete_track {}", cli.track_uuid),
        "undo".to_string(),
        "redo".to_string(),
    ];
    if roundtrip.actions != expected_actions {
        bail!(
            "CLI roundtrip actions mismatch: expected {:?}, got {:?}",
            expected_actions,
            roundtrip.actions
        );
    }
    let expected_last_description = format!("redo delete_track {}", cli.track_uuid);
    if roundtrip
        .last_result
        .as_ref()
        .map(|result| result.description.as_str())
        != Some(expected_last_description.as_str())
    {
        bail!("CLI roundtrip last_result description mismatch");
    }

    let target = unique_temp_path("cli-surface-save", "kicad_pcb");
    let save_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--delete-track")
        .arg(cli.track_uuid.to_string())
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI save parity probe")?;
    if !save_output.status.success() {
        bail!(
            "CLI save parity probe failed with status {:?}: {}",
            save_output.status.code(),
            String::from_utf8_lossy(&save_output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&save_output.stdout)
        .context("failed to parse CLI save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    let reloaded_after_save = reloaded.get_net_info()?;
    if reloaded_after_save != after_delete_state(cli)? {
        bail!("CLI save did not persist the current deleted board state");
    }

    let check_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "check",
        ])
        .arg(saved_path)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI delete-track follow-up check")?;
    if !check_output.status.success() {
        bail!(
            "CLI delete-track follow-up check failed with status {:?}: {}",
            check_output.status.code(),
            String::from_utf8_lossy(&check_output.stderr).trim()
        );
    }
    let check_payload: Value = serde_json::from_slice(&check_output.stdout)
        .context("failed to parse CLI delete-track follow-up check JSON")?;
    let diagnostics = check_payload["diagnostics"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI delete-track follow-up check missing diagnostics"))?;
    if !diagnostics
        .iter()
        .any(|diagnostic| diagnostic["kind"] == "net_without_copper")
    {
        bail!("CLI delete-track follow-up check missing net_without_copper");
    }

    Ok(format!(
        "roundtrip_last={}, saved={}, delete_then_save_persisted=true, delete_track_followup_check_changed=true",
        expected_last_description, saved_path
    ))
}

fn cli_via_surface_result(cli: &Cli) -> Result<String> {
    let via_fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let via_uuid =
        Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").expect("uuid should parse");
    let target = unique_temp_path("cli-surface-via-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&via_fixture)
        .arg("--delete-via")
        .arg(via_uuid.to_string())
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI via save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI via save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI via save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI via save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    let expected = after_delete_via_state(&via_fixture, via_uuid)?;
    if reloaded.get_net_info()? != expected {
        bail!("CLI via save did not persist the current deleted board state");
    }
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI delete-via follow-up net query")?;
    if !query_output.status.success() {
        bail!(
            "CLI delete-via follow-up net query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI delete-via follow-up net JSON")?;
    let nets = payload["nets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI delete-via follow-up net query missing nets"))?;
    let gnd = nets
        .iter()
        .find(|net| net["name"] == "GND")
        .ok_or_else(|| anyhow::anyhow!("CLI delete-via follow-up net query missing GND"))?;
    if gnd["vias"] != 0 {
        bail!("CLI delete-via follow-up net query did not reflect removed via");
    }
    Ok(format!(
        "via_saved={}, delete_via_then_save_persisted=true, delete_via_followup_net_info_changed=true",
        saved_path
    ))
}

fn cli_component_surface_result(cli: &Cli) -> Result<String> {
    let target = unique_temp_path("cli-surface-component-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--delete-component")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI delete-component save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI delete-component save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI delete-component save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI delete-component save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI delete-component follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI delete-component follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI delete-component follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI delete-component follow-up query missing components"))?;
    if components
        .iter()
        .any(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
    {
        bail!("CLI delete-component follow-up query still included deleted component");
    }
    Ok(format!(
        "component_saved={}, delete_component_then_save_persisted=true, delete_component_followup_components_changed=true",
        saved_path
    ))
}

fn cli_rule_surface_result(cli: &Cli) -> Result<String> {
    let fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let target = unique_temp_path("cli-surface-rule-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&fixture)
        .arg("--set-clearance-min-nm")
        .arg("125000")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI rule save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI rule save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI rule save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI rule save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    if reloaded.get_design_rules()?.len() != 1 {
        bail!("CLI rule save did not persist one design rule");
    }
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("design-rules")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI rule follow-up query")?;
    if !query_output.status.success() {
        bail!(
            "CLI rule follow-up query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI rule follow-up query JSON")?;
    let rules = payload["rules"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI rule follow-up query missing rules array"))?;
    if rules.len() != 1 || rules[0]["name"] != "default clearance" {
        bail!("CLI rule follow-up query did not reflect current design-rule state");
    }
    Ok(format!(
        "rule_saved={}, set_design_rule_then_save_persisted=true, rule_followup_query_changed=true",
        saved_path
    ))
}

fn cli_value_surface_result(cli: &Cli) -> Result<String> {
    let target = unique_temp_path("cli-surface-value-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--set-value")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:22k")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-value save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-value save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-value save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-value save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-value follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-value follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-value follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-value follow-up query missing components"))?;
    let r1 = components
        .iter()
        .find(|component| component["reference"] == "R1")
        .ok_or_else(|| anyhow::anyhow!("CLI set-value follow-up query missing R1"))?;
    if r1["value"] != "22k" {
        bail!("CLI set-value follow-up query did not reflect updated component value");
    }
    Ok(format!(
        "value_saved={}, set_value_then_save_persisted=true, set_value_followup_components_changed=true",
        saved_path
    ))
}

fn cli_reference_surface_result(cli: &Cli) -> Result<String> {
    let target = unique_temp_path("cli-surface-reference-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--set-reference")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:R10")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-reference save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-reference save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-reference save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-reference save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-reference follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-reference follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-reference follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-reference follow-up query missing components"))?;
    let target_component = components
        .iter()
        .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .ok_or_else(|| anyhow::anyhow!("CLI set-reference follow-up query missing target component"))?;
    if target_component["reference"] != "R10" {
        bail!("CLI set-reference follow-up query did not reflect updated component reference");
    }
    Ok(format!(
        "reference_saved={}, set_reference_then_save_persisted=true, set_reference_followup_components_changed=true",
        saved_path
    ))
}

fn cli_assign_part_surface_result(cli: &Cli) -> Result<String> {
    let library = cli
        .repo_root
        .join("crates/engine/testdata/import/eagle/simple-opamp.lbr");
    let mut engine = Engine::new()?;
    engine.import_eagle_library(&library)?;
    let part_uuid = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP part missing from pool"))?
        .uuid;

    let target = unique_temp_path("cli-surface-assign-part-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--library")
        .arg(&library)
        .arg("--assign-part")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}",
            part_uuid
        ))
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI assign-part save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI assign-part save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI assign-part save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI assign-part save report missing saved_path"))?;
    let saved_contents = std::fs::read_to_string(saved_path)
        .context("failed to read CLI assign-part saved board")?;
    if !saved_contents.contains("(footprint \"ALT-3\"") {
        bail!("CLI assign-part save did not rewrite expected footprint name");
    }
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI assign-part follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI assign-part follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI assign-part follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI assign-part follow-up query missing components"))?;
    let target_component = components
        .iter()
        .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .ok_or_else(|| anyhow::anyhow!("CLI assign-part follow-up query missing target component"))?;
    if target_component["value"] != "ALTAMP" {
        bail!("CLI assign-part follow-up query did not reflect updated component value");
    }
    let net_query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI assign-part follow-up net query")?;
    if !net_query_output.status.success() {
        bail!(
            "CLI assign-part follow-up net query failed with status {:?}: {}",
            net_query_output.status.code(),
            String::from_utf8_lossy(&net_query_output.stderr).trim()
        );
    }
    let net_payload: Value = serde_json::from_slice(&net_query_output.stdout)
        .context("failed to parse CLI assign-part follow-up net JSON")?;
    let nets = net_payload["nets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI assign-part follow-up net query missing nets"))?;
    let sig = nets
        .iter()
        .find(|net| net["name"] == "SIG")
        .ok_or_else(|| anyhow::anyhow!("CLI assign-part follow-up net query missing SIG"))?;
    if sig["pins"].as_array().map(|pins| pins.len()) != Some(1) {
        bail!("CLI assign-part follow-up net query did not reflect regenerated package connectivity");
    }
    Ok(format!(
        "assign_saved={}, assign_part_then_save_persisted=true, assign_part_rewrote_footprint=true, assign_part_followup_components_changed=true, assign_part_followup_net_info_changed=true",
        saved_path
    ))
}

fn cli_assign_part_remap_surface_result(cli: &Cli) -> Result<String> {
    let library = cli
        .repo_root
        .join("crates/engine/testdata/import/eagle/simple-opamp.lbr");
    let mut engine = Engine::new()?;
    engine.import_eagle_library(&library)?;
    let lmv321_part_uuid = engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("LMV321 part missing from pool"))?
        .uuid;
    let altamp_part_uuid = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP part missing from pool"))?
        .uuid;

    let target = unique_temp_path("cli-surface-assign-part-remap-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--library")
        .arg(&library)
        .arg("--assign-part")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}",
            lmv321_part_uuid
        ))
        .arg("--assign-part")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}",
            altamp_part_uuid
        ))
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI assign-part remap save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI assign-part remap save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI assign-part remap save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI assign-part remap save report missing saved_path"))?;
    let net_query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI assign-part remap follow-up net query")?;
    if !net_query_output.status.success() {
        bail!(
            "CLI assign-part remap follow-up net query failed with status {:?}: {}",
            net_query_output.status.code(),
            String::from_utf8_lossy(&net_query_output.stderr).trim()
        );
    }
    let net_payload: Value = serde_json::from_slice(&net_query_output.stdout)
        .context("failed to parse CLI assign-part remap follow-up net JSON")?;
    let nets = net_payload["nets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI assign-part remap follow-up net query missing nets"))?;
    let sig = nets
        .iter()
        .find(|net| net["name"] == "SIG")
        .ok_or_else(|| anyhow::anyhow!("CLI assign-part remap follow-up net query missing SIG"))?;
    if sig["pins"].as_array().map(|pins| pins.len()) != Some(2) {
        bail!("CLI assign-part remap did not preserve logical net connectivity");
    }
    Ok(format!(
        "assign_remap_saved={}, assign_part_logical_remap_preserved=true",
        saved_path
    ))
}

fn cli_set_package_surface_result(cli: &Cli) -> Result<String> {
    let library = cli
        .repo_root
        .join("crates/engine/testdata/import/eagle/simple-opamp.lbr");
    let mut engine = Engine::new()?;
    engine.import_eagle_library(&library)?;
    let package_uuid = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP package missing from pool"))?
        .package_uuid;

    let target = unique_temp_path("cli-surface-set-package-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--library")
        .arg(&library)
        .arg("--set-package")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}",
            package_uuid
        ))
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-package save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-package save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-package save report missing saved_path"))?;
    let saved_contents = std::fs::read_to_string(saved_path)
        .context("failed to read CLI set-package saved board")?;
    if !saved_contents.contains("(footprint \"ALT-3\"") {
        bail!("CLI set-package save did not rewrite expected footprint name");
    }
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-package follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-package follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-package follow-up query missing components"))?;
    let target_component = components
        .iter()
        .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .ok_or_else(|| anyhow::anyhow!("CLI set-package follow-up query missing target component"))?;
    if target_component["package_uuid"] != package_uuid.to_string() {
        bail!("CLI set-package follow-up query did not reflect updated package assignment");
    }
    let net_query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package follow-up net query")?;
    if !net_query_output.status.success() {
        bail!(
            "CLI set-package follow-up net query failed with status {:?}: {}",
            net_query_output.status.code(),
            String::from_utf8_lossy(&net_query_output.stderr).trim()
        );
    }
    let net_payload: Value = serde_json::from_slice(&net_query_output.stdout)
        .context("failed to parse CLI set-package follow-up net JSON")?;
    let nets = net_payload["nets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-package follow-up net query missing nets"))?;
    let sig = nets
        .iter()
        .find(|net| net["name"] == "SIG")
        .ok_or_else(|| anyhow::anyhow!("CLI set-package follow-up net query missing SIG"))?;
    if sig["pins"].as_array().map(|pins| pins.len()) != Some(1) {
        bail!("CLI set-package follow-up net query did not reflect regenerated package connectivity");
    }
    Ok(format!(
        "package_saved={}, set_package_then_save_persisted=true, set_package_rewrote_footprint=true, set_package_followup_components_changed=true, set_package_followup_net_info_changed=true",
        saved_path
    ))
}

fn cli_set_package_remap_surface_result(cli: &Cli) -> Result<String> {
    let library = cli
        .repo_root
        .join("crates/engine/testdata/import/eagle/simple-opamp.lbr");
    let mut engine = Engine::new()?;
    engine.import_eagle_library(&library)?;
    let lmv321_part_uuid = engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("LMV321 part missing from pool"))?
        .uuid;
    let altamp_package_uuid = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP package missing from pool"))?
        .package_uuid;

    let target = unique_temp_path("cli-surface-set-package-remap-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--library")
        .arg(&library)
        .arg("--assign-part")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}",
            lmv321_part_uuid
        ))
        .arg("--set-package")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}",
            altamp_package_uuid
        ))
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package remap save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-package remap save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-package remap save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-package remap save report missing saved_path"))?;
    let net_query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package remap follow-up net query")?;
    if !net_query_output.status.success() {
        bail!(
            "CLI set-package remap follow-up net query failed with status {:?}: {}",
            net_query_output.status.code(),
            String::from_utf8_lossy(&net_query_output.stderr).trim()
        );
    }
    let net_payload: Value = serde_json::from_slice(&net_query_output.stdout)
        .context("failed to parse CLI set-package remap follow-up net JSON")?;
    let nets = net_payload["nets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-package remap follow-up net query missing nets"))?;
    let sig = nets
        .iter()
        .find(|net| net["name"] == "SIG")
        .ok_or_else(|| anyhow::anyhow!("CLI set-package remap follow-up net query missing SIG"))?;
    if sig["pins"].as_array().map(|pins| pins.len()) != Some(2) {
        bail!("CLI set-package remap did not preserve logical net connectivity");
    }
    Ok(format!(
        "package_remap_saved={}, set_package_logical_remap_preserved=true",
        saved_path
    ))
}

fn cli_set_package_with_part_surface_result(cli: &Cli) -> Result<String> {
    let library = cli
        .repo_root
        .join("crates/engine/testdata/import/eagle/simple-opamp.lbr");
    let mut engine = Engine::new()?;
    engine.import_eagle_library(&library)?;
    let lmv321_part_uuid = engine
        .search_pool("LMV321")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("LMV321 part missing from pool"))?
        .uuid;
    let altamp = engine
        .search_pool("ALTAMP")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("ALTAMP part missing from pool"))?;

    let target = unique_temp_path("cli-surface-set-package-with-part-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--library")
        .arg(&library)
        .arg("--assign-part")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}",
            lmv321_part_uuid
        ))
        .arg("--set-package-with-part")
        .arg(format!(
            "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:{}:{}",
            altamp.package_uuid, altamp.uuid
        ))
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package-with-part save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-package-with-part save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-package-with-part save JSON output")?;
    let saved_path = save.saved_path.as_deref().ok_or_else(|| {
        anyhow::anyhow!("CLI set-package-with-part save report missing saved_path")
    })?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package-with-part follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-package-with-part follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-package-with-part follow-up components JSON")?;
    let components = payload["components"].as_array().ok_or_else(|| {
        anyhow::anyhow!("CLI set-package-with-part follow-up query missing components")
    })?;
    let target_component = components
        .iter()
        .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .ok_or_else(|| anyhow::anyhow!("CLI set-package-with-part query missing target component"))?;
    if target_component["package_uuid"] != altamp.package_uuid.to_string() {
        bail!("CLI set-package-with-part follow-up query did not reflect updated package");
    }
    if target_component["value"] != "ALTAMP" {
        bail!("CLI set-package-with-part follow-up query did not reflect explicit part value");
    }
    let net_query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-package-with-part follow-up net query")?;
    if !net_query_output.status.success() {
        bail!(
            "CLI set-package-with-part follow-up net query failed with status {:?}: {}",
            net_query_output.status.code(),
            String::from_utf8_lossy(&net_query_output.stderr).trim()
        );
    }
    let net_payload: Value = serde_json::from_slice(&net_query_output.stdout)
        .context("failed to parse CLI set-package-with-part follow-up net JSON")?;
    let nets = net_payload["nets"].as_array().ok_or_else(|| {
        anyhow::anyhow!("CLI set-package-with-part follow-up query missing nets")
    })?;
    let sig = nets
        .iter()
        .find(|net| net["name"] == "SIG")
        .ok_or_else(|| anyhow::anyhow!("CLI set-package-with-part follow-up query missing SIG"))?;
    if sig["pins"].as_array().map(|pins| pins.len()) != Some(2) {
        bail!("CLI set-package-with-part did not preserve logical net connectivity");
    }
    Ok(format!(
        "package_with_part_saved={}, set_package_with_part_followup_components_changed=true, set_package_with_part_followup_net_info_changed=true",
        saved_path
    ))
}

fn cli_set_net_class_surface_result(cli: &Cli) -> Result<String> {
    let fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let mut engine = Engine::new()?;
    engine.import(&fixture)?;
    let net_uuid = engine
        .get_net_info()?
        .into_iter()
        .find(|net| net.name == "GND")
        .ok_or_else(|| anyhow::anyhow!("GND net missing from CLI set-net-class fixture"))?
        .uuid;

    let target = unique_temp_path("cli-surface-net-class-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&fixture)
        .arg("--set-net-class")
        .arg(format!(
            "{}:power:125000:250000:300000:600000",
            net_uuid
        ))
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-net-class save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI set-net-class save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI set-net-class save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI set-net-class save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("nets")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI set-net-class follow-up net query")?;
    if !query_output.status.success() {
        bail!(
            "CLI set-net-class follow-up net query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI set-net-class follow-up net JSON")?;
    let nets = payload["nets"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI set-net-class follow-up query missing nets"))?;
    let gnd = nets
        .iter()
        .find(|net| net["uuid"] == net_uuid.to_string())
        .ok_or_else(|| anyhow::anyhow!("CLI set-net-class follow-up query missing target net"))?;
    if gnd["class"] != "power" {
        bail!("CLI set-net-class follow-up query did not reflect updated class");
    }
    Ok(format!(
        "net_class_saved={}, set_net_class_then_save_persisted=true, set_net_class_followup_net_info_changed=true",
        saved_path
    ))
}

fn cli_move_surface_result(cli: &Cli) -> Result<String> {
    let baseline_distance =
        cli_unrouted_distance(&cli.repo_root, &cli.roundtrip_board_fixture_path)?;
    let target = unique_temp_path("cli-surface-move-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--move-component")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:15:12:90")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI move save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI move save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI move save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI move save report missing saved_path"))?;
    let mut reloaded = Engine::new()?;
    reloaded.import(Path::new(saved_path))?;
    let moved = reloaded
        .get_components()?
        .into_iter()
        .find(|component| component.reference == "R1")
        .ok_or_else(|| anyhow::anyhow!("CLI move save missing R1 after reimport"))?;
    if moved.position.x != 15_000_000 || moved.position.y != 12_000_000 || moved.rotation != 90 {
        bail!("CLI move save did not persist expected moved component state");
    }
    let moved_distance = cli_unrouted_distance(&cli.repo_root, Path::new(saved_path))?;
    if moved_distance == baseline_distance {
        bail!("CLI follow-up query did not reflect moved-component derived state");
    }
    Ok(format!(
        "move_saved={}, move_component_then_save_persisted=true, cli_followup_unrouted_changed=true",
        saved_path
    ))
}

fn cli_rotate_surface_result(cli: &Cli) -> Result<String> {
    let target = unique_temp_path("cli-surface-rotate-save", "kicad_pcb");
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "modify",
        ])
        .arg(&cli.roundtrip_board_fixture_path)
        .arg("--rotate-component")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa:180")
        .arg("--save")
        .arg(&target)
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI rotate-component save parity probe")?;
    if !output.status.success() {
        bail!(
            "CLI rotate-component save parity probe failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let save: CliModifyReport = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI rotate-component save JSON output")?;
    let saved_path = save
        .saved_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("CLI rotate-component save report missing saved_path"))?;
    let query_output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(saved_path)
        .arg("components")
        .current_dir(&cli.repo_root)
        .output()
        .context("failed to run CLI rotate-component follow-up components query")?;
    if !query_output.status.success() {
        bail!(
            "CLI rotate-component follow-up components query failed with status {:?}: {}",
            query_output.status.code(),
            String::from_utf8_lossy(&query_output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&query_output.stdout)
        .context("failed to parse CLI rotate-component follow-up components JSON")?;
    let components = payload["components"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("CLI rotate-component follow-up query missing components"))?;
    let target_component = components
        .iter()
        .find(|component| component["uuid"] == "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .ok_or_else(|| anyhow::anyhow!("CLI rotate-component follow-up query missing target component"))?;
    if target_component["rotation"] != 180 {
        bail!("CLI rotate-component follow-up query did not reflect updated rotation");
    }
    Ok(format!(
        "rotate_saved={}, rotate_component_then_save_persisted=true, rotate_component_followup_components_changed=true",
        saved_path
    ))
}

fn after_delete_state(cli: &Cli) -> Result<Vec<eda_engine::board::BoardNetInfo>> {
    let mut engine = Engine::new()?;
    engine.import(&cli.roundtrip_board_fixture_path)?;
    engine.delete_track(&cli.track_uuid)?;
    Ok(engine.get_net_info()?)
}

fn after_delete_via_state(
    fixture: &Path,
    via_uuid: Uuid,
) -> Result<Vec<eda_engine::board::BoardNetInfo>> {
    let mut engine = Engine::new()?;
    engine.import(fixture)?;
    engine.delete_via(&via_uuid)?;
    Ok(engine.get_net_info()?)
}

fn run_command_checked(command: &mut Command, label: &str) -> Result<String> {
    let output = command
        .output()
        .with_context(|| format!("failed to execute {label}"))?;
    if !output.status.success() {
        bail!(
            "{label} failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn cli_unrouted_distance(repo_root: &Path, board_path: &Path) -> Result<i64> {
    let output = Command::new("cargo")
        .args([
            "run", "-q", "-p", "eda-cli", "--", "--format", "json", "query",
        ])
        .arg(board_path)
        .arg("unrouted")
        .current_dir(repo_root)
        .output()
        .context("failed to run CLI unrouted query")?;
    if !output.status.success() {
        bail!(
            "CLI unrouted query failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let payload: Value = serde_json::from_slice(&output.stdout)
        .context("failed to parse CLI unrouted JSON output")?;
    payload["airwires"][0]["distance_nm"]
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("CLI unrouted JSON missing first airwire distance"))
}

fn unique_temp_path(prefix: &str, extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be valid")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{unique}.{extension}",
        std::process::id()
    ))
}

fn print_human(report: &Report) {
    println!("m3 write-surface parity preflight:");
    println!("  overall: {:?}", report.overall_status);
    for check in &report.checks {
        println!("  - {}: {:?}", check.surface, check.status);
        println!("    {}", check.evidence);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_temp_path_includes_prefix_and_extension() {
        let path = unique_temp_path("parity-test", "json");
        let text = path.display().to_string();
        assert!(text.contains("parity-test"));
        assert!(text.ends_with(".json"));
    }
}
