use std::collections::{BTreeMap, BTreeSet};

use super::*;

impl Engine {
    pub fn get_package_change_candidates(
        &self,
        component_uuid: &uuid::Uuid,
    ) -> Result<PackageChangeCompatibilityReport, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        let component = board
            .packages
            .get(component_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: *component_uuid,
            })?;
        let current_package_name = self
            .pool
            .packages
            .get(&component.package)
            .map(|package| package.name.clone())
            .unwrap_or_default();

        if component.part == uuid::Uuid::nil() || !self.pool.parts.contains_key(&component.part) {
            return Ok(PackageChangeCompatibilityReport {
                component_uuid: *component_uuid,
                current_part_uuid: None,
                current_package_uuid: component.package,
                current_package_name,
                current_value: component.value.clone(),
                status: PackageChangeCompatibilityStatus::NoKnownPart,
                ambiguous_package_count: 0,
                candidates: Vec::new(),
            });
        }

        let current_part_uuid = component.part;
        let current_part =
            self.pool
                .parts
                .get(&current_part_uuid)
                .ok_or(EngineError::NotFound {
                    object_type: "part",
                    uuid: current_part_uuid,
                })?;
        let current_signature = part_pin_signature(current_part, &self.pool).ok_or_else(|| {
            EngineError::DanglingReference {
                source_type: "part",
                source_uuid: current_part_uuid,
                target_type: "entity",
                target_uuid: current_part.entity,
            }
        })?;

        let mut by_package: BTreeMap<uuid::Uuid, Vec<&crate::pool::Part>> = BTreeMap::new();
        for part in self.pool.parts.values() {
            if part.package == component.package {
                continue;
            }
            if part_pin_signature(part, &self.pool).as_ref() == Some(&current_signature) {
                by_package.entry(part.package).or_default().push(part);
            }
        }

        let mut ambiguous_package_count = 0;
        let mut candidates = Vec::new();
        for (package_uuid, parts) in by_package {
            if parts.len() != 1 {
                ambiguous_package_count += 1;
                continue;
            }
            let part = parts[0];
            let package = self
                .pool
                .packages
                .get(&package_uuid)
                .ok_or(EngineError::NotFound {
                    object_type: "package",
                    uuid: package_uuid,
                })?;
            let mut pin_names: Vec<_> = current_signature.iter().cloned().collect();
            pin_names.sort();
            candidates.push(PackageChangeCandidate {
                package_uuid,
                package_name: package.name.clone(),
                compatible_part_uuid: part.uuid,
                compatible_part_value: part.value.clone(),
                pin_names,
            });
        }
        candidates.sort_by(|a, b| {
            a.package_name
                .cmp(&b.package_name)
                .then_with(|| a.package_uuid.cmp(&b.package_uuid))
        });

        Ok(PackageChangeCompatibilityReport {
            component_uuid: *component_uuid,
            current_part_uuid: Some(current_part_uuid),
            current_package_uuid: component.package,
            current_package_name,
            current_value: component.value.clone(),
            status: if candidates.is_empty() {
                PackageChangeCompatibilityStatus::NoCompatiblePackages
            } else {
                PackageChangeCompatibilityStatus::CandidatesAvailable
            },
            ambiguous_package_count,
            candidates,
        })
    }

    pub fn get_part_change_candidates(
        &self,
        component_uuid: &uuid::Uuid,
    ) -> Result<PartChangeCompatibilityReport, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        let component = board
            .packages
            .get(component_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: *component_uuid,
            })?;
        let current_package_name = self
            .pool
            .packages
            .get(&component.package)
            .map(|package| package.name.clone())
            .unwrap_or_default();

        if component.part == uuid::Uuid::nil() || !self.pool.parts.contains_key(&component.part) {
            return Ok(PartChangeCompatibilityReport {
                component_uuid: *component_uuid,
                current_part_uuid: None,
                current_package_uuid: component.package,
                current_package_name,
                current_value: component.value.clone(),
                status: PartChangeCompatibilityStatus::NoKnownPart,
                candidates: Vec::new(),
            });
        }

        let current_part_uuid = component.part;
        let current_part =
            self.pool
                .parts
                .get(&current_part_uuid)
                .ok_or(EngineError::NotFound {
                    object_type: "part",
                    uuid: current_part_uuid,
                })?;
        let current_signature = part_pin_signature(current_part, &self.pool).ok_or_else(|| {
            EngineError::DanglingReference {
                source_type: "part",
                source_uuid: current_part_uuid,
                target_type: "entity",
                target_uuid: current_part.entity,
            }
        })?;

        let mut candidates = Vec::new();
        for part in self.pool.parts.values() {
            if part.uuid == current_part_uuid {
                continue;
            }
            if part_pin_signature(part, &self.pool).as_ref() != Some(&current_signature) {
                continue;
            }
            let package = self
                .pool
                .packages
                .get(&part.package)
                .ok_or(EngineError::NotFound {
                    object_type: "package",
                    uuid: part.package,
                })?;
            let mut pin_names: Vec<_> = current_signature.iter().cloned().collect();
            pin_names.sort();
            candidates.push(PartChangeCandidate {
                part_uuid: part.uuid,
                package_uuid: part.package,
                package_name: package.name.clone(),
                value: part.value.clone(),
                mpn: part.mpn.clone(),
                manufacturer: part.manufacturer.clone(),
                pin_names,
            });
        }
        candidates.sort_by(|a, b| {
            a.package_name
                .cmp(&b.package_name)
                .then_with(|| a.value.cmp(&b.value))
                .then_with(|| a.mpn.cmp(&b.mpn))
                .then_with(|| a.part_uuid.cmp(&b.part_uuid))
        });

        Ok(PartChangeCompatibilityReport {
            component_uuid: *component_uuid,
            current_part_uuid: Some(current_part_uuid),
            current_package_uuid: component.package,
            current_package_name,
            current_value: component.value.clone(),
            status: if candidates.is_empty() {
                PartChangeCompatibilityStatus::NoCompatibleParts
            } else {
                PartChangeCompatibilityStatus::CandidatesAvailable
            },
            candidates,
        })
    }

    pub fn get_component_replacement_plan(
        &self,
        component_uuid: &uuid::Uuid,
    ) -> Result<ComponentReplacementPlan, EngineError> {
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        let component = board
            .packages
            .get(component_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: *component_uuid,
            })?;
        let current_package_name = self
            .pool
            .packages
            .get(&component.package)
            .map(|package| package.name.clone())
            .unwrap_or_default();

        Ok(ComponentReplacementPlan {
            component_uuid: *component_uuid,
            current_reference: component.reference.clone(),
            current_value: component.value.clone(),
            current_part_uuid: (component.part != uuid::Uuid::nil()).then_some(component.part),
            current_package_uuid: component.package,
            current_package_name,
            package_change: self.get_package_change_candidates(component_uuid)?,
            part_change: self.get_part_change_candidates(component_uuid)?,
        })
    }

    pub fn get_scoped_component_replacement_plan(
        &self,
        input: ScopedComponentReplacementPolicyInput,
    ) -> Result<ScopedComponentReplacementPlan, EngineError> {
        let resolved = self.resolve_scoped_component_replacement_policy(&input)?;
        let design = self.design.as_ref().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_ref().ok_or_else(|| EngineError::NotFound {
            object_type: "board",
            uuid: uuid::Uuid::nil(),
        })?;
        let mut replacements = Vec::with_capacity(resolved.len());
        for replacement in resolved {
            let component = board
                .packages
                .get(&replacement.uuid)
                .ok_or(EngineError::NotFound {
                    object_type: "component",
                    uuid: replacement.uuid,
                })?;
            let target_part =
                self.pool
                    .parts
                    .get(&replacement.part_uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "part",
                        uuid: replacement.part_uuid,
                    })?;
            let target_package =
                self.pool
                    .packages
                    .get(&replacement.package_uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "package",
                        uuid: replacement.package_uuid,
                    })?;
            replacements.push(ScopedComponentReplacementPlanItem {
                component_uuid: replacement.uuid,
                current_reference: component.reference.clone(),
                current_value: component.value.clone(),
                current_part_uuid: (component.part != uuid::Uuid::nil()).then_some(component.part),
                current_package_uuid: component.package,
                target_part_uuid: replacement.part_uuid,
                target_package_uuid: replacement.package_uuid,
                target_value: target_part.value.clone(),
                target_package_name: target_package.name.clone(),
            });
        }

        Ok(ScopedComponentReplacementPlan {
            scope: input.scope,
            policy: input.policy,
            replacements,
        })
    }

    pub fn edit_scoped_component_replacement_plan(
        &self,
        mut plan: ScopedComponentReplacementPlan,
        edit: ScopedComponentReplacementPlanEdit,
    ) -> Result<ScopedComponentReplacementPlan, EngineError> {
        let mut excluded = BTreeSet::new();
        for uuid in &edit.exclude_component_uuids {
            if !excluded.insert(*uuid) {
                return Err(EngineError::Operation(format!(
                    "edit_scoped_component_replacement_plan cannot exclude component {} more than once",
                    uuid
                )));
            }
        }

        let mut seen_overrides = BTreeSet::new();
        for override_item in &edit.overrides {
            if !seen_overrides.insert(override_item.component_uuid) {
                return Err(EngineError::Operation(format!(
                    "edit_scoped_component_replacement_plan cannot override component {} more than once",
                    override_item.component_uuid
                )));
            }
            if excluded.contains(&override_item.component_uuid) {
                return Err(EngineError::Operation(format!(
                    "edit_scoped_component_replacement_plan cannot both exclude and override component {}",
                    override_item.component_uuid
                )));
            }
        }

        plan.replacements
            .retain(|item| !excluded.contains(&item.component_uuid));

        for override_item in edit.overrides {
            let replacement =
                plan.replacements
                    .iter_mut()
                    .find(|item| item.component_uuid == override_item.component_uuid)
                    .ok_or_else(|| {
                        EngineError::Operation(format!(
                            "edit_scoped_component_replacement_plan override targets component {} outside the current scoped plan",
                            override_item.component_uuid
                        ))
                    })?;
            let target_part = self.pool.parts.get(&override_item.target_part_uuid).ok_or(
                EngineError::NotFound {
                    object_type: "part",
                    uuid: override_item.target_part_uuid,
                },
            )?;
            let target_package = self
                .pool
                .packages
                .get(&override_item.target_package_uuid)
                .ok_or(EngineError::NotFound {
                    object_type: "package",
                    uuid: override_item.target_package_uuid,
                })?;
            if target_part.package != override_item.target_package_uuid {
                return Err(EngineError::Operation(format!(
                    "edit_scoped_component_replacement_plan requires part {} to use package {}",
                    override_item.target_part_uuid, override_item.target_package_uuid
                )));
            }

            let component_plan =
                self.get_component_replacement_plan(&override_item.component_uuid)?;
            let package_match = component_plan
                .package_change
                .candidates
                .iter()
                .any(|candidate| {
                    candidate.package_uuid == override_item.target_package_uuid
                        && candidate.compatible_part_uuid == override_item.target_part_uuid
                });
            let part_match = component_plan
                .part_change
                .candidates
                .iter()
                .any(|candidate| {
                    candidate.part_uuid == override_item.target_part_uuid
                        && candidate.package_uuid == override_item.target_package_uuid
                });
            if !package_match && !part_match {
                return Err(EngineError::Operation(format!(
                    "edit_scoped_component_replacement_plan override {} -> ({}, {}) is not compatible; inspect get_component_replacement_plan first",
                    override_item.component_uuid,
                    override_item.target_package_uuid,
                    override_item.target_part_uuid
                )));
            }

            replacement.target_package_uuid = override_item.target_package_uuid;
            replacement.target_part_uuid = override_item.target_part_uuid;
            replacement.target_value = target_part.value.clone();
            replacement.target_package_name = target_package.name.clone();
        }

        Ok(plan)
    }
}
