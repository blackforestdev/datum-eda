// Extracted from m3_sidecar_roundtrip_fidelity.rs (monolith burn-down):
// component-replacement fidelity flows + shared run_replacement_roundtrip.
// Included via include! so it shares the bin's module scope (imports/helpers).

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

    let second_parts_sidecar = part_assignments_sidecar::sidecar_path_for_source(&second_board);
    let second_parts = part_assignments_sidecar::read_sidecar(&second_parts_sidecar)
        .context("failed to decode second replacement part-assignment sidecar")?;
    let second_packages_sidecar =
        package_assignments_sidecar::sidecar_path_for_source(&second_board);
    let second_packages = package_assignments_sidecar::read_sidecar(&second_packages_sidecar)
        .context("failed to decode second replacement package-assignment sidecar")?;

    let second_board_bytes = fs::read(&second_board)?;
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
