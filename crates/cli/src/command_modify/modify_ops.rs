use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn modify_board(
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
    apply_scoped_replacement_plan: &[ScopedComponentReplacementPlan],
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
        && apply_scoped_replacement_plan.is_empty()
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
    for input in apply_scoped_replacement_plan {
        let result = engine
            .apply_scoped_component_replacement_plan(input.clone())
            .context("failed to apply scoped component replacement plan")?;
        let selector = match input.policy {
            ComponentReplacementPolicy::BestCompatiblePackage => "best_compatible_package",
            ComponentReplacementPolicy::BestCompatiblePart => "best_compatible_part",
        };
        actions.push(format!(
            "apply_scoped_replacement_plan {selector} {}",
            input.replacements.len()
        ));
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
