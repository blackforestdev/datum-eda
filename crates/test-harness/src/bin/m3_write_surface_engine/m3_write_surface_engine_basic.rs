use super::*;

pub(super) fn engine_track_surface_result(cli: &Cli) -> Result<String> {
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

    let target = m3_write_surface_common::unique_temp_path("engine-surface-save", "kicad_pcb");
    engine.save(&target)?;
    let mut reloaded = Engine::new()?;
    reloaded.import(&target)?;
    let reloaded_after_save = reloaded.get_net_info()?;
    if reloaded_after_save != after_redo {
        bail!("saved engine board did not reimport to the current deleted state");
    }

    Ok(format!(
        "delete={}, undo={}, redo={}, saved={}, reimported_deleted_state=true, delete_followup_check_changed=true",
        delete.description,
        undo.description,
        redo.description,
        target.display(),
    ))
}

pub(super) fn engine_via_surface_result(cli: &Cli) -> Result<String> {
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
    let via_target = m3_write_surface_common::unique_temp_path("engine-surface-via-save", "kicad_pcb");
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

    Ok(format!(
        "delete_via={}, via_saved={}, via_reimported_deleted_state=true, delete_via_followup_net_info_changed=true",
        delete_via.description,
        via_target.display()
    ))
}

pub(super) fn engine_component_surface_result(cli: &Cli) -> Result<String> {
    let mut component_engine = Engine::new()?;
    component_engine.import(&cli.roundtrip_board_fixture_path)?;
    let baseline_component_list = {
        let mut baseline_engine = Engine::new()?;
        baseline_engine.import(&cli.roundtrip_board_fixture_path)?;
        baseline_engine.get_components()?
    };
    let delete_component = component_engine
        .delete_component(&Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap())?;
    let component_target =
        m3_write_surface_common::unique_temp_path("engine-surface-component-save", "kicad_pcb");
    component_engine.save(&component_target)?;
    let mut reloaded_component = Engine::new()?;
    reloaded_component.import(&component_target)?;
    let reloaded_component_list = reloaded_component.get_components()?;
    if reloaded_component_list.iter().any(|component| {
        component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
    }) {
        bail!("saved delete_component board still reimported deleted component");
    }
    if baseline_component_list.len() == reloaded_component_list.len() {
        bail!("delete_component did not change engine follow-up components state");
    }

    Ok(format!(
        "delete_component={}, component_saved={}, component_followup_components_changed=true",
        delete_component.description,
        component_target.display()
    ))
}

pub(super) fn engine_rule_surface_result(cli: &Cli) -> Result<String> {
    let fixture = cli
        .repo_root
        .join("crates/engine/testdata/import/kicad/simple-demo.kicad_pcb");
    let mut rule_engine = Engine::new()?;
    rule_engine.import(&fixture)?;
    let baseline_rules = rule_engine.get_design_rules()?;
    let set_rule = rule_engine.set_design_rule(eda_engine::api::SetDesignRuleInput {
        rule_type: eda_engine::rules::ast::RuleType::ClearanceCopper,
        scope: eda_engine::rules::ast::RuleScope::All,
        parameters: eda_engine::rules::ast::RuleParams::Clearance { min: 125_000 },
        priority: 10,
        name: Some("default clearance".to_string()),
    })?;
    let rule_target = m3_write_surface_common::unique_temp_path("engine-surface-rule-save", "kicad_pcb");
    rule_engine.save(&rule_target)?;
    let mut reloaded_rule = Engine::new()?;
    reloaded_rule.import(&rule_target)?;
    if reloaded_rule.get_design_rules()?.len() != 1 {
        bail!("saved engine rule-mutated board did not reimport one design rule");
    }
    if baseline_rules.len() == reloaded_rule.get_design_rules()?.len() {
        bail!("set_design_rule did not change engine follow-up design-rules state");
    }

    Ok(format!(
        "set_rule={}, rule_saved={}, rule_reimported=true, rule_followup_query_changed=true",
        set_rule.description,
        rule_target.display()
    ))
}

pub(super) fn engine_move_surface_result(cli: &Cli) -> Result<String> {
    let mut move_engine = Engine::new()?;
    move_engine.import(&cli.roundtrip_board_fixture_path)?;
    let moved = move_engine.move_component(eda_engine::api::MoveComponentInput {
        uuid: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
        position: eda_engine::ir::geometry::Point::new(15_000_000, 12_000_000),
        rotation: Some(90),
    })?;
    let move_target = m3_write_surface_common::unique_temp_path("engine-surface-move-save", "kicad_pcb");
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
        == baseline_move_airwires.first().map(|airwire| airwire.distance_nm)
    {
        bail!("move_component did not change engine unrouted derived state");
    }

    Ok(format!(
        "move_component={}, moved_saved={}, move_reimported=true, move_followup_unrouted_changed=true",
        moved.description,
        move_target.display()
    ))
}

pub(super) fn engine_rotate_surface_result(cli: &Cli) -> Result<String> {
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
    let rotate_target = m3_write_surface_common::unique_temp_path("engine-surface-rotate-save", "kicad_pcb");
    rotate_engine.save(&rotate_target)?;
    let mut reloaded_rotate = Engine::new()?;
    reloaded_rotate.import(&rotate_target)?;
    let rotated_component = reloaded_rotate
        .get_components()?
        .into_iter()
        .find(|component| {
            component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
        })
        .ok_or_else(|| anyhow::anyhow!("reloaded rotated component missing target"))?;
    if rotated_component.rotation != 180 {
        bail!("saved rotated component did not reimport expected rotation");
    }
    let baseline_rotated = baseline_rotate_components
        .iter()
        .find(|component| {
            component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
        })
        .ok_or_else(|| anyhow::anyhow!("baseline rotate components missing target"))?;
    if baseline_rotated.rotation == rotated_component.rotation {
        bail!("rotate_component did not change engine follow-up components state");
    }

    Ok(format!(
        "rotate_component={}, rotate_saved={}, rotate_followup_components_changed=true",
        rotate.description,
        rotate_target.display()
    ))
}

pub(super) fn engine_value_surface_result(cli: &Cli) -> Result<String> {
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
    let value_target = m3_write_surface_common::unique_temp_path("engine-surface-value-save", "kicad_pcb");
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

    Ok(format!(
        "set_value={}, value_saved={}, value_followup_components_changed=true",
        set_value.description,
        value_target.display()
    ))
}

pub(super) fn engine_reference_surface_result(cli: &Cli) -> Result<String> {
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
    let reference_target =
        m3_write_surface_common::unique_temp_path("engine-surface-reference-save", "kicad_pcb");
    reference_engine.save(&reference_target)?;
    let mut reloaded_reference = Engine::new()?;
    reloaded_reference.import(&reference_target)?;
    let reloaded_reference_components = reloaded_reference.get_components()?;
    let baseline_reference_r1 = baseline_reference_components
        .iter()
        .find(|component| {
            component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
        })
        .ok_or_else(|| anyhow::anyhow!("baseline reference components missing target component"))?;
    let updated_reference = reloaded_reference_components
        .iter()
        .find(|component| {
            component.uuid == Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap()
        })
        .ok_or_else(|| anyhow::anyhow!("reloaded reference components missing target component"))?;
    if updated_reference.reference != "R10" {
        bail!("saved set_reference component did not reimport expected reference");
    }
    if baseline_reference_r1.reference == updated_reference.reference {
        bail!("set_reference did not change engine follow-up components state");
    }

    Ok(format!(
        "set_reference={}, reference_saved={}, reference_followup_components_changed=true",
        set_reference.description,
        reference_target.display()
    ))
}
