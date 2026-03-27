use super::*;
use std::collections::BTreeSet;

impl Engine {
    fn apply_explicit_component_replacement(
        &mut self,
        uuid: Uuid,
        package_uuid: Uuid,
        part_uuid: Uuid,
        description: &str,
    ) -> Result<OperationResult, EngineError> {
        let target_part = self
            .pool
            .parts
            .get(&part_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "part",
                uuid: part_uuid,
            })?
            .clone();
        if target_part.package != package_uuid {
            return Err(EngineError::Operation(format!(
                "{description} requires part {} to use package {}",
                part_uuid, package_uuid
            )));
        }
        let target_package = self
            .pool
            .packages
            .get(&package_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "package",
                uuid: package_uuid,
            })?
            .clone();
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(format!(
                "{description} is currently implemented only for board projects"
            ))
        })?;

        let before = board
            .packages
            .get(&uuid)
            .cloned()
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid,
            })?;
        if before.part != Uuid::nil() && self.pool.parts.contains_key(&before.part) {
            let current_part = self
                .pool
                .parts
                .get(&before.part)
                .ok_or(EngineError::NotFound {
                    object_type: "part",
                    uuid: before.part,
                })?;
            let current_signature = part_pin_signature(current_part, &self.pool).ok_or(
                EngineError::DanglingReference {
                    source_type: "part",
                    source_uuid: current_part.uuid,
                    target_type: "entity",
                    target_uuid: current_part.entity,
                },
            )?;
            let target_signature = part_pin_signature(&target_part, &self.pool).ok_or(
                EngineError::DanglingReference {
                    source_type: "part",
                    source_uuid: target_part.uuid,
                    target_type: "entity",
                    target_uuid: target_part.entity,
                },
            )?;
            if current_signature != target_signature {
                return Err(EngineError::Operation(format!(
                    "{description} target part {} is not logically compatible with current component {}; inspect get_component_replacement_plan first",
                    part_uuid, uuid
                )));
            }
        }
        let before_pads = component_pads(board, uuid);
        let package = board.packages.get_mut(&uuid).ok_or(EngineError::NotFound {
            object_type: "component",
            uuid,
        })?;
        package.package = package_uuid;
        package.part = part_uuid;
        package.value = target_part.value.clone();
        let after = package.clone();
        replace_component_pads_for_assign_part(
            board,
            &before,
            &after,
            &target_part,
            &target_package,
            &self.pool,
        )?;
        let after_pads = component_pads(board, uuid);

        self.undo_stack.push(TransactionRecord::SetPackage {
            before: before.clone(),
            after: after.clone(),
            before_pads,
            after_pads,
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: vec![OperationRef {
                    object_type: "component".to_string(),
                    uuid,
                }],
                deleted: Vec::new(),
            },
            description: format!("{description} {uuid}"),
        })
    }

    pub fn set_package_with_part(
        &mut self,
        input: SetPackageWithPartInput,
    ) -> Result<OperationResult, EngineError> {
        self.apply_explicit_component_replacement(
            input.uuid,
            input.package_uuid,
            input.part_uuid,
            "set_package_with_part",
        )
    }

    pub fn replace_component(
        &mut self,
        input: ReplaceComponentInput,
    ) -> Result<OperationResult, EngineError> {
        self.apply_explicit_component_replacement(
            input.uuid,
            input.package_uuid,
            input.part_uuid,
            "replace_component",
        )
    }

    pub fn replace_components(
        &mut self,
        inputs: Vec<ReplaceComponentInput>,
    ) -> Result<OperationResult, EngineError> {
        if inputs.is_empty() {
            return Err(EngineError::Operation(
                "replace_components requires at least one replacement".to_string(),
            ));
        }
        let mut seen = BTreeSet::new();
        for input in &inputs {
            if !seen.insert(input.uuid) {
                return Err(EngineError::Operation(format!(
                    "replace_components cannot target component {} more than once in one transaction",
                    input.uuid
                )));
            }
        }

        let snapshot = self.design.clone();
        let original_undo_len = self.undo_stack.len();
        let original_redo = self.redo_stack.clone();
        let original_undo_depth = self.undo_depth;
        let original_redo_depth = self.redo_depth;
        let mut records = Vec::with_capacity(inputs.len());
        let mut merged_diff = OperationDiff::default();
        for input in inputs {
            match self.apply_explicit_component_replacement(
                input.uuid,
                input.package_uuid,
                input.part_uuid,
                "replace_components",
            ) {
                Ok(result) => {
                    merge_operation_diff(&mut merged_diff, &result.diff);
                    let record = self.undo_stack.pop().ok_or_else(|| {
                        EngineError::Operation(
                            "replace_components could not recover staged transaction".to_string(),
                        )
                    })?;
                    records.push(record);
                }
                Err(err) => {
                    self.design = snapshot;
                    self.undo_stack.truncate(original_undo_len);
                    self.redo_stack = original_redo;
                    self.undo_depth = original_undo_depth;
                    self.redo_depth = original_redo_depth;
                    return Err(err);
                }
            }
        }

        self.undo_stack.push(TransactionRecord::Batch {
            description: format!("replace_components {}", records.len()),
            records,
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: merged_diff,
            description: format!("replace_components {}", seen.len()),
        })
    }

    fn resolve_planned_component_replacement(
        &self,
        input: &PlannedComponentReplacementInput,
    ) -> Result<ReplaceComponentInput, EngineError> {
        match (input.package_uuid, input.part_uuid) {
            (Some(package_uuid), Some(part_uuid)) => Ok(ReplaceComponentInput {
                uuid: input.uuid,
                package_uuid,
                part_uuid,
            }),
            (Some(package_uuid), None) => {
                let plan = self.get_component_replacement_plan(&input.uuid)?;
                let candidate = plan
                    .package_change
                    .candidates
                    .iter()
                    .find(|candidate| candidate.package_uuid == package_uuid)
                    .ok_or_else(|| {
                        EngineError::Operation(format!(
                            "apply_component_replacement_plan could not resolve package {} for component {}; inspect get_component_replacement_plan first",
                            package_uuid, input.uuid
                        ))
                    })?;
                Ok(ReplaceComponentInput {
                    uuid: input.uuid,
                    package_uuid: candidate.package_uuid,
                    part_uuid: candidate.compatible_part_uuid,
                })
            }
            (None, Some(part_uuid)) => {
                let plan = self.get_component_replacement_plan(&input.uuid)?;
                let candidate = plan
                    .part_change
                    .candidates
                    .iter()
                    .find(|candidate| candidate.part_uuid == part_uuid)
                    .ok_or_else(|| {
                        EngineError::Operation(format!(
                            "apply_component_replacement_plan could not resolve part {} for component {}; inspect get_component_replacement_plan first",
                            part_uuid, input.uuid
                        ))
                    })?;
                Ok(ReplaceComponentInput {
                    uuid: input.uuid,
                    package_uuid: candidate.package_uuid,
                    part_uuid: candidate.part_uuid,
                })
            }
            (None, None) => Err(EngineError::Operation(
                "apply_component_replacement_plan requires a package_uuid, a part_uuid, or both"
                    .to_string(),
            )),
        }
    }

    pub fn apply_component_replacement_plan(
        &mut self,
        inputs: Vec<PlannedComponentReplacementInput>,
    ) -> Result<OperationResult, EngineError> {
        if inputs.is_empty() {
            return Err(EngineError::Operation(
                "apply_component_replacement_plan requires at least one replacement".to_string(),
            ));
        }
        let resolved = inputs
            .iter()
            .map(|input| self.resolve_planned_component_replacement(input))
            .collect::<Result<Vec<_>, _>>()?;
        self.replace_components(resolved)
    }

    pub(crate) fn resolve_policy_driven_component_replacement(
        &self,
        input: &PolicyDrivenComponentReplacementInput,
    ) -> Result<ReplaceComponentInput, EngineError> {
        let plan = self.get_component_replacement_plan(&input.uuid)?;
        match input.policy {
            ComponentReplacementPolicy::BestCompatiblePackage => {
                let candidate = plan.package_change.candidates.first().ok_or_else(|| {
                    EngineError::Operation(format!(
                        "apply_component_replacement_policy could not find a compatible package candidate for component {}; inspect get_component_replacement_plan first",
                        input.uuid
                    ))
                })?;
                Ok(ReplaceComponentInput {
                    uuid: input.uuid,
                    package_uuid: candidate.package_uuid,
                    part_uuid: candidate.compatible_part_uuid,
                })
            }
            ComponentReplacementPolicy::BestCompatiblePart => {
                let candidate = plan.part_change.candidates.first().ok_or_else(|| {
                    EngineError::Operation(format!(
                        "apply_component_replacement_policy could not find a compatible part candidate for component {}; inspect get_component_replacement_plan first",
                        input.uuid
                    ))
                })?;
                Ok(ReplaceComponentInput {
                    uuid: input.uuid,
                    package_uuid: candidate.package_uuid,
                    part_uuid: candidate.part_uuid,
                })
            }
        }
    }

    pub fn apply_component_replacement_policy(
        &mut self,
        inputs: Vec<PolicyDrivenComponentReplacementInput>,
    ) -> Result<OperationResult, EngineError> {
        if inputs.is_empty() {
            return Err(EngineError::Operation(
                "apply_component_replacement_policy requires at least one replacement".to_string(),
            ));
        }
        let resolved = inputs
            .iter()
            .map(|input| self.resolve_policy_driven_component_replacement(input))
            .collect::<Result<Vec<_>, _>>()?;
        self.replace_components(resolved)
    }

    pub(crate) fn scoped_replacement_candidates(
        &self,
        scope: &ComponentReplacementScope,
    ) -> Result<Vec<PolicyDrivenComponentReplacementInput>, EngineError> {
        if scope.reference_prefix.is_none()
            && scope.value_equals.is_none()
            && scope.current_package_uuid.is_none()
            && scope.current_part_uuid.is_none()
        {
            return Err(EngineError::Operation(
                "apply_scoped_component_replacement_policy requires at least one scope selector"
                    .to_string(),
            ));
        }
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| {
            EngineError::Operation(
                "apply_scoped_component_replacement_policy is currently implemented only for board projects".to_string(),
            )
        })?;
        let mut packages: Vec<_> = board.packages.values().cloned().collect();
        packages.sort_by(|a, b| {
            a.reference
                .cmp(&b.reference)
                .then_with(|| a.uuid.cmp(&b.uuid))
        });
        Ok(packages
            .into_iter()
            .filter(|package| {
                scope
                    .reference_prefix
                    .as_ref()
                    .is_none_or(|prefix| package.reference.starts_with(prefix))
                    && scope
                        .value_equals
                        .as_ref()
                        .is_none_or(|value| &package.value == value)
                    && scope
                        .current_package_uuid
                        .is_none_or(|package_uuid| package.package == package_uuid)
                    && scope
                        .current_part_uuid
                        .is_none_or(|part_uuid| package.part == part_uuid)
            })
            .map(|package| package.uuid)
            .collect::<Vec<_>>()
            .into_iter()
            .map(|uuid| PolicyDrivenComponentReplacementInput {
                uuid,
                policy: ComponentReplacementPolicy::BestCompatiblePackage,
            })
            .collect())
    }

    pub(crate) fn resolve_scoped_component_replacement_policy(
        &self,
        input: &ScopedComponentReplacementPolicyInput,
    ) -> Result<Vec<ReplaceComponentInput>, EngineError> {
        let mut scoped = self.scoped_replacement_candidates(&input.scope)?;
        if scoped.is_empty() {
            return Err(EngineError::Operation(
                "apply_scoped_component_replacement_policy matched no components".to_string(),
            ));
        }
        for candidate in &mut scoped {
            candidate.policy = input.policy;
        }
        scoped
            .iter()
            .map(|candidate| self.resolve_policy_driven_component_replacement(candidate))
            .collect()
    }

    pub fn apply_scoped_component_replacement_policy(
        &mut self,
        input: ScopedComponentReplacementPolicyInput,
    ) -> Result<OperationResult, EngineError> {
        let resolved = self.resolve_scoped_component_replacement_policy(&input)?;
        self.replace_components(resolved)
    }

    pub fn apply_scoped_component_replacement_plan(
        &mut self,
        plan: ScopedComponentReplacementPlan,
    ) -> Result<OperationResult, EngineError> {
        if plan.replacements.is_empty() {
            return Err(EngineError::Operation(
                "apply_scoped_component_replacement_plan requires at least one replacement"
                    .to_string(),
            ));
        }
        let scoped = self.scoped_replacement_candidates(&plan.scope)?;
        let mut matched_component_uuids: Vec<_> =
            scoped.into_iter().map(|item| item.uuid).collect();
        matched_component_uuids.sort();
        let mut preview_component_uuids: Vec<_> = plan
            .replacements
            .iter()
            .map(|item| item.component_uuid)
            .collect();
        preview_component_uuids.sort();
        if matched_component_uuids != preview_component_uuids {
            return Err(EngineError::Operation(
                "apply_scoped_component_replacement_plan no longer matches the previewed scoped component set"
                    .to_string(),
            ));
        }

        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| {
            EngineError::Operation(
                "apply_scoped_component_replacement_plan is currently implemented only for board projects".to_string(),
            )
        })?;
        let resolved = plan
            .replacements
            .iter()
            .map(|item| {
                let component = board
                    .packages
                    .get(&item.component_uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: item.component_uuid,
                    })?;
                let current_part_uuid = (component.part != Uuid::nil()).then_some(component.part);
                if component.reference != item.current_reference
                    || component.value != item.current_value
                    || current_part_uuid != item.current_part_uuid
                    || component.package != item.current_package_uuid
                {
                    return Err(EngineError::Operation(format!(
                        "apply_scoped_component_replacement_plan preview drifted for component {}; refresh get_scoped_component_replacement_plan first",
                        item.component_uuid
                    )));
                }
                Ok(ReplaceComponentInput {
                    uuid: item.component_uuid,
                    package_uuid: item.target_package_uuid,
                    part_uuid: item.target_part_uuid,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.replace_components(resolved)
    }
}
