use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn modify_board(
    path: &Path,
    delete_track: &[Uuid],
    delete_via: &[Uuid],
    delete_component: &[Uuid],
    libraries: &[PathBuf],
    move_component: &[MoveComponentInput],
    rotate_component: &[RotateComponentInput],
    set_value: &[SetValueInput],
    assign_part: &[AssignPartInput],
    set_package: &[SetPackageInput],
    set_package_with_part: &[SetPackageWithPartInput],
    replace_component: &[ReplaceComponentInput],
    set_net_class: &[SetNetClassInput],
    set_reference: &[SetReferenceInput],
    set_clearance_min_nm: Option<i64>,
    undo: usize,
    redo: usize,
    save: Option<&Path>,
    save_original: bool,
    apply_replacement_plan: &[PlannedComponentReplacementInput],
    apply_replacement_policy: &[PolicyDrivenComponentReplacementInput],
    apply_scoped_replacement_policy: &[ScopedComponentReplacementPolicyInput],
) -> Result<ModifyReportView> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("kicad_pcb") {
        bail!(
            "modify is currently only implemented for KiCad .kicad_pcb inputs in the current M3 slice: {}",
            path.display()
        );
    }
    if delete_track.is_empty()
        && delete_via.is_empty()
        && delete_component.is_empty()
        && move_component.is_empty()
        && rotate_component.is_empty()
        && set_value.is_empty()
        && assign_part.is_empty()
        && set_package.is_empty()
        && set_package_with_part.is_empty()
        && replace_component.is_empty()
        && apply_replacement_plan.is_empty()
        && apply_replacement_policy.is_empty()
        && apply_scoped_replacement_policy.is_empty()
        && set_net_class.is_empty()
        && set_reference.is_empty()
        && set_clearance_min_nm.is_none()
        && undo == 0
        && redo == 0
        && save.is_none()
        && !save_original
    {
        bail!("modify requires at least one action in the current M3 slice");
    }
    if save.is_some() && save_original {
        bail!("specify either --save or --save-original, not both");
    }

    let mut engine = Engine::new().context("failed to initialize engine")?;
    for path in libraries {
        if path.extension().and_then(|ext| ext.to_str()) != Some("lbr") {
            bail!(
                "modify --library currently only accepts Eagle .lbr inputs in the current M3 slice: {}",
                path.display()
            );
        }
        engine
            .import_eagle_library(path)
            .with_context(|| format!("failed to import Eagle library {}", path.display()))?;
    }
    engine
        .import(path)
        .with_context(|| format!("failed to import board {}", path.display()))?;

    let mut actions = Vec::new();
    let mut last_result = None;
    for uuid in delete_track {
        let result = engine
            .delete_track(uuid)
            .with_context(|| format!("failed to delete track {uuid}"))?;
        actions.push(format!("delete_track {uuid}"));
        last_result = Some(result);
    }
    for uuid in delete_via {
        let result = engine
            .delete_via(uuid)
            .with_context(|| format!("failed to delete via {uuid}"))?;
        actions.push(format!("delete_via {uuid}"));
        last_result = Some(result);
    }
    for uuid in delete_component {
        let result = engine
            .delete_component(uuid)
            .with_context(|| format!("failed to delete component {uuid}"))?;
        actions.push(format!("delete_component {uuid}"));
        last_result = Some(result);
    }
    for input in move_component {
        let result = engine
            .move_component(input.clone())
            .with_context(|| format!("failed to move component {}", input.uuid))?;
        actions.push(format!(
            "move_component {} {} {} {}",
            input.uuid,
            input.position.x,
            input.position.y,
            input.rotation.unwrap_or_default()
        ));
        last_result = Some(result);
    }
    for input in rotate_component {
        let result = engine
            .rotate_component(input.clone())
            .with_context(|| format!("failed to rotate component {}", input.uuid))?;
        actions.push(format!("rotate_component {} {}", input.uuid, input.rotation));
        last_result = Some(result);
    }
    for input in set_value {
        let result = engine
            .set_value(input.clone())
            .with_context(|| format!("failed to set component value {}", input.uuid))?;
        actions.push(format!("set_value {} {}", input.uuid, input.value));
        last_result = Some(result);
    }
    for input in assign_part {
        let result = engine.assign_part(input.clone()).with_context(|| {
            format!(
                "failed to assign part {} to {}",
                input.part_uuid, input.uuid
            )
        })?;
        actions.push(format!("assign_part {} {}", input.uuid, input.part_uuid));
        last_result = Some(result);
    }
    for input in set_package {
        let result = engine.set_package(input.clone()).with_context(|| {
            format!(
                "failed to set package {} on {}",
                input.package_uuid, input.uuid
            )
        })?;
        actions.push(format!("set_package {} {}", input.uuid, input.package_uuid));
        last_result = Some(result);
    }
    for input in set_package_with_part {
        let result = engine
            .set_package_with_part(input.clone())
            .with_context(|| {
                format!(
                    "failed to set package {} with part {} on {}",
                    input.package_uuid, input.part_uuid, input.uuid
                )
            })?;
        actions.push(format!(
            "set_package_with_part {} {} {}",
            input.uuid, input.package_uuid, input.part_uuid
        ));
        last_result = Some(result);
    }
    if replace_component.len() > 1 {
        let result = engine
            .replace_components(replace_component.to_vec())
            .context("failed to replace components in batch")?;
        for input in replace_component {
            actions.push(format!(
                "replace_component {} {} {}",
                input.uuid, input.package_uuid, input.part_uuid
            ));
        }
        last_result = Some(result);
    } else {
        for input in replace_component {
            let result = engine.replace_component(input.clone()).with_context(|| {
                format!(
                    "failed to replace component {} with package {} part {}",
                    input.uuid, input.package_uuid, input.part_uuid
                )
            })?;
            actions.push(format!(
                "replace_component {} {} {}",
                input.uuid, input.package_uuid, input.part_uuid
            ));
            last_result = Some(result);
        }
    }
    if !apply_replacement_plan.is_empty() {
        let result = engine
            .apply_component_replacement_plan(apply_replacement_plan.to_vec())
            .context("failed to apply component replacement plan")?;
        for input in apply_replacement_plan {
            let selector = match (input.package_uuid, input.part_uuid) {
                (Some(package_uuid), Some(part_uuid)) => {
                    format!("package={package_uuid} part={part_uuid}")
                }
                (Some(package_uuid), None) => format!("package={package_uuid}"),
                (None, Some(part_uuid)) => format!("part={part_uuid}"),
                (None, None) => "unresolved".to_string(),
            };
            actions.push(format!("apply_replacement_plan {} {}", input.uuid, selector));
        }
        last_result = Some(result);
    }
    if !apply_replacement_policy.is_empty() {
        let result = engine
            .apply_component_replacement_policy(apply_replacement_policy.to_vec())
            .context("failed to apply component replacement policy")?;
        for input in apply_replacement_policy {
            let selector = match input.policy {
                ComponentReplacementPolicy::BestCompatiblePackage => "best_compatible_package",
                ComponentReplacementPolicy::BestCompatiblePart => "best_compatible_part",
            };
            actions.push(format!("apply_replacement_policy {} {}", input.uuid, selector));
        }
        last_result = Some(result);
    }
    for input in apply_scoped_replacement_policy {
        let result = engine
            .apply_scoped_component_replacement_policy(input.clone())
            .context("failed to apply scoped component replacement policy")?;
        let selector = match input.policy {
            ComponentReplacementPolicy::BestCompatiblePackage => "best_compatible_package",
            ComponentReplacementPolicy::BestCompatiblePart => "best_compatible_part",
        };
        actions.push(format!("apply_scoped_replacement_policy {selector}"));
        last_result = Some(result);
    }
    for input in set_net_class {
        let result = engine
            .set_net_class(input.clone())
            .with_context(|| format!("failed to set net class on {}", input.net_uuid))?;
        actions.push(format!(
            "set_net_class {} {}",
            input.net_uuid, input.class_name
        ));
        last_result = Some(result);
    }
    for input in set_reference {
        let result = engine
            .set_reference(input.clone())
            .with_context(|| format!("failed to set component reference {}", input.uuid))?;
        actions.push(format!("set_reference {} {}", input.uuid, input.reference));
        last_result = Some(result);
    }
    if let Some(min) = set_clearance_min_nm {
        let result = engine
            .set_design_rule(SetDesignRuleInput {
                rule_type: RuleType::ClearanceCopper,
                scope: RuleScope::All,
                parameters: RuleParams::Clearance { min },
                priority: 10,
                name: Some("default clearance".to_string()),
            })
            .context("failed to set default clearance rule")?;
        actions.push(format!("set_design_rule clearance_copper {min}"));
        last_result = Some(result);
    }
    for _ in 0..undo {
        let result = engine.undo().context("failed to undo board transaction")?;
        actions.push("undo".to_string());
        last_result = Some(result);
    }
    for _ in 0..redo {
        let result = engine.redo().context("failed to redo board transaction")?;
        actions.push("redo".to_string());
        last_result = Some(result);
    }

    let saved_path = if let Some(target) = save {
        engine
            .save(target)
            .with_context(|| format!("failed to save board to {}", target.display()))?;
        actions.push(format!("save {}", target.display()));
        Some(target.display().to_string())
    } else if save_original {
        let target = engine
            .save_to_original()
            .context("failed to save board to original path")?;
        actions.push(format!("save {}", target.display()));
        Some(target.display().to_string())
    } else {
        None
    };

    Ok(ModifyReportView {
        actions,
        last_result,
        saved_path,
    })
}

pub(super) fn parse_move_component_arg(value: &str) -> Result<MoveComponentInput> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 3 && parts.len() != 4 {
        bail!("--move-component expects <uuid>:<x_mm>:<y_mm>[:<rotation_deg>]");
    }
    let uuid = Uuid::parse_str(parts[0])?;
    let x_mm = parts[1].parse::<f64>()?;
    let y_mm = parts[2].parse::<f64>()?;
    let rotation = if parts.len() == 4 {
        Some(parts[3].parse::<i32>()?)
    } else {
        None
    };
    Ok(MoveComponentInput {
        uuid,
        position: eda_engine::ir::geometry::Point::new(
            eda_engine::ir::units::mm_to_nm(x_mm),
            eda_engine::ir::units::mm_to_nm(y_mm),
        ),
        rotation,
    })
}

pub(super) fn parse_set_value_arg(value: &str) -> Result<SetValueInput> {
    let (uuid, component_value) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("--set-value expects <uuid>:<value>"))?;
    Ok(SetValueInput {
        uuid: Uuid::parse_str(uuid)?,
        value: component_value.to_string(),
    })
}

pub(super) fn parse_rotate_component_arg(value: &str) -> Result<RotateComponentInput> {
    let (uuid, rotation) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("--rotate-component expects <uuid>:<rotation_deg>"))?;
    Ok(RotateComponentInput {
        uuid: Uuid::parse_str(uuid)?,
        rotation: rotation.parse::<i32>()?,
    })
}

pub(super) fn parse_set_reference_arg(value: &str) -> Result<SetReferenceInput> {
    let (uuid, reference) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("--set-reference expects <uuid>:<reference>"))?;
    Ok(SetReferenceInput {
        uuid: Uuid::parse_str(uuid)?,
        reference: reference.to_string(),
    })
}

pub(super) fn parse_assign_part_arg(value: &str) -> Result<AssignPartInput> {
    let (uuid, part_uuid) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("--assign-part expects <uuid>:<part_uuid>"))?;
    Ok(AssignPartInput {
        uuid: Uuid::parse_str(uuid)?,
        part_uuid: Uuid::parse_str(part_uuid)?,
    })
}

pub(super) fn parse_set_package_arg(value: &str) -> Result<SetPackageInput> {
    let (uuid, package_uuid) = value
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("--set-package expects <uuid>:<package_uuid>"))?;
    Ok(SetPackageInput {
        uuid: Uuid::parse_str(uuid)?,
        package_uuid: Uuid::parse_str(package_uuid)?,
    })
}

pub(super) fn parse_set_package_with_part_arg(value: &str) -> Result<SetPackageWithPartInput> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 3 {
        bail!("--set-package-with-part expects <uuid>:<package_uuid>:<part_uuid>");
    }
    Ok(SetPackageWithPartInput {
        uuid: Uuid::parse_str(parts[0])?,
        package_uuid: Uuid::parse_str(parts[1])?,
        part_uuid: Uuid::parse_str(parts[2])?,
    })
}

pub(super) fn parse_replace_component_arg(value: &str) -> Result<ReplaceComponentInput> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 3 {
        bail!("--replace-component expects <uuid>:<package_uuid>:<part_uuid>");
    }
    Ok(ReplaceComponentInput {
        uuid: Uuid::parse_str(parts[0])?,
        package_uuid: Uuid::parse_str(parts[1])?,
        part_uuid: Uuid::parse_str(parts[2])?,
    })
}

pub(super) fn parse_apply_replacement_plan_arg(
    value: &str,
) -> Result<PlannedComponentReplacementInput> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 3 && parts.len() != 5 {
        bail!(
            "--apply-replacement-plan expects <uuid>:package:<package_uuid> | <uuid>:part:<part_uuid> | <uuid>:package:<package_uuid>:part:<part_uuid>"
        );
    }
    let uuid = Uuid::parse_str(parts[0])?;
    let mut package_uuid = None;
    let mut part_uuid = None;
    let mut index = 1;
    while index + 1 < parts.len() {
        match parts[index] {
            "package" => package_uuid = Some(Uuid::parse_str(parts[index + 1])?),
            "part" => part_uuid = Some(Uuid::parse_str(parts[index + 1])?),
            other => bail!(
                "--apply-replacement-plan selector must be 'package' or 'part', got {other}"
            ),
        }
        index += 2;
    }
    Ok(PlannedComponentReplacementInput {
        uuid,
        package_uuid,
        part_uuid,
    })
}

pub(super) fn parse_apply_replacement_policy_arg(
    value: &str,
) -> Result<PolicyDrivenComponentReplacementInput> {
    let (uuid, selector) = value.split_once(':').ok_or_else(|| {
        anyhow::anyhow!("--apply-replacement-policy expects <uuid>:package|part")
    })?;
    let policy = match selector {
        "package" => ComponentReplacementPolicy::BestCompatiblePackage,
        "part" => ComponentReplacementPolicy::BestCompatiblePart,
        other => bail!("--apply-replacement-policy selector must be 'package' or 'part', got {other}"),
    };
    Ok(PolicyDrivenComponentReplacementInput {
        uuid: Uuid::parse_str(uuid)?,
        policy,
    })
}

pub(super) fn parse_apply_scoped_replacement_policy_arg(
    value: &str,
) -> Result<ScopedComponentReplacementPolicyInput> {
    let mut parts = value.split(':');
    let policy = match parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("--apply-scoped-replacement-policy expects a policy"))?
    {
        "package" => ComponentReplacementPolicy::BestCompatiblePackage,
        "part" => ComponentReplacementPolicy::BestCompatiblePart,
        other => bail!(
            "--apply-scoped-replacement-policy policy must be 'package' or 'part', got {other}"
        ),
    };
    let mut scope = ComponentReplacementScope::default();
    for segment in parts {
        let (key, raw_value) = segment.split_once('=').ok_or_else(|| {
            anyhow::anyhow!(
                "--apply-scoped-replacement-policy scope selectors must use key=value segments"
            )
        })?;
        match key {
            "ref_prefix" => scope.reference_prefix = Some(raw_value.to_string()),
            "value" => scope.value_equals = Some(raw_value.to_string()),
            "package_uuid" => scope.current_package_uuid = Some(Uuid::parse_str(raw_value)?),
            "part_uuid" => scope.current_part_uuid = Some(Uuid::parse_str(raw_value)?),
            other => bail!(
                "--apply-scoped-replacement-policy selector must be one of ref_prefix,value,package_uuid,part_uuid; got {other}"
            ),
        }
    }
    Ok(ScopedComponentReplacementPolicyInput { scope, policy })
}

pub(super) fn parse_set_net_class_arg(value: &str) -> Result<SetNetClassInput> {
    let parts: Vec<_> = value.split(':').collect();
    if parts.len() != 6 && parts.len() != 8 {
        bail!(
            "--set-net-class expects <net_uuid>:<class_name>:<clearance_nm>:<track_width_nm>:<via_drill_nm>:<via_diameter_nm>[:<diffpair_width_nm>:<diffpair_gap_nm>]"
        );
    }
    Ok(SetNetClassInput {
        net_uuid: Uuid::parse_str(parts[0])?,
        class_name: parts[1].to_string(),
        clearance: parts[2].parse::<i64>()?,
        track_width: parts[3].parse::<i64>()?,
        via_drill: parts[4].parse::<i64>()?,
        via_diameter: parts[5].parse::<i64>()?,
        diffpair_width: if parts.len() == 8 {
            parts[6].parse::<i64>()?
        } else {
            0
        },
        diffpair_gap: if parts.len() == 8 {
            parts[7].parse::<i64>()?
        } else {
            0
        },
    })
}
