use super::*;

impl Engine {
    pub fn assign_part(&mut self, input: AssignPartInput) -> Result<OperationResult, EngineError> {
        let part = self
            .pool
            .parts
            .get(&input.part_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "part",
                uuid: input.part_uuid,
            })?;
        let target_package =
            self.pool
                .packages
                .get(&part.package)
                .ok_or(EngineError::DanglingReference {
                    source_type: "part",
                    source_uuid: input.part_uuid,
                    target_type: "package",
                    target_uuid: part.package,
                })?;
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "assign_part is currently implemented only for board projects".to_string(),
            )
        })?;

        let before = board
            .packages
            .get(&input.uuid)
            .cloned()
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        let before_pads = component_pads(board, input.uuid);
        let package = board
            .packages
            .get_mut(&input.uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        package.part = input.part_uuid;
        package.package = part.package;
        package.value = part.value.clone();
        let after = package.clone();
        replace_component_pads_for_assign_part(
            board,
            &before,
            &after,
            part,
            target_package,
            &self.pool,
        )?;
        let after_pads = component_pads(board, input.uuid);

        self.undo_stack.push(TransactionRecord::AssignPart {
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
                    uuid: input.uuid,
                }],
                deleted: Vec::new(),
            },
            description: format!("assign_part {}", input.uuid),
        })
    }

    pub fn set_package(&mut self, input: SetPackageInput) -> Result<OperationResult, EngineError> {
        let target_package =
            self.pool
                .packages
                .get(&input.package_uuid)
                .ok_or(EngineError::NotFound {
                    object_type: "package",
                    uuid: input.package_uuid,
                })?;
        let compatible_part_uuid = self
            .design
            .as_ref()
            .and_then(|design| design.board.as_ref())
            .and_then(|board| board.packages.get(&input.uuid))
            .and_then(|component| {
                (component.part != Uuid::nil())
                    .then(|| {
                        resolve_compatible_part_for_package_change(
                            component.part,
                            input.package_uuid,
                            &self.pool,
                        )
                    })
                    .flatten()
            });
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "set_package is currently implemented only for board projects".to_string(),
            )
        })?;

        let before = board
            .packages
            .get(&input.uuid)
            .cloned()
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        let before_pads = component_pads(board, input.uuid);
        let package = board
            .packages
            .get_mut(&input.uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        package.package = input.package_uuid;
        if let Some(part_uuid) = compatible_part_uuid {
            if let Some(part) = self.pool.parts.get(&part_uuid) {
                package.part = part.uuid;
                package.value = part.value.clone();
            }
        } else if package.part != Uuid::nil()
            && self
                .pool
                .parts
                .get(&package.part)
                .is_some_and(|part| part.package != input.package_uuid)
        {
            package.part = Uuid::nil();
        }
        let after = package.clone();
        if let Some(part_uuid) = compatible_part_uuid {
            let target_part =
                self.pool
                    .parts
                    .get(&part_uuid)
                    .ok_or(EngineError::DanglingReference {
                        source_type: "component",
                        source_uuid: input.uuid,
                        target_type: "part",
                        target_uuid: part_uuid,
                    })?;
            replace_component_pads_for_assign_part(
                board,
                &before,
                &after,
                target_part,
                target_package,
                &self.pool,
            )?;
        } else {
            replace_component_pads_from_pool_package(board, &after, target_package)?;
        }
        let after_pads = component_pads(board, input.uuid);

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
                    uuid: input.uuid,
                }],
                deleted: Vec::new(),
            },
            description: format!("set_package {}", input.uuid),
        })
    }

    pub fn set_net_class(
        &mut self,
        input: SetNetClassInput,
    ) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "set_net_class is currently implemented only for board projects".to_string(),
            )
        })?;

        let before_net = board
            .nets
            .get(&input.net_uuid)
            .cloned()
            .ok_or(EngineError::NotFound {
                object_type: "net",
                uuid: input.net_uuid,
            })?;
        let target_class_uuid = if before_net.class != Uuid::nil() {
            before_net.class
        } else {
            deterministic_net_class_uuid(input.net_uuid, &input.class_name)
        };
        let previous_class = board.net_classes.get(&target_class_uuid).cloned();
        let current_class = NetClass {
            uuid: target_class_uuid,
            name: input.class_name,
            clearance: input.clearance,
            track_width: input.track_width,
            via_drill: input.via_drill,
            via_diameter: input.via_diameter,
            diffpair_width: input.diffpair_width,
            diffpair_gap: input.diffpair_gap,
        };
        board
            .net_classes
            .insert(target_class_uuid, current_class.clone());
        let net = board
            .nets
            .get_mut(&input.net_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "net",
                uuid: input.net_uuid,
            })?;
        net.class = target_class_uuid;
        let after_net = net.clone();

        self.undo_stack.push(TransactionRecord::SetNetClass {
            before_net: before_net.clone(),
            after_net: after_net.clone(),
            previous_class: previous_class.clone(),
            current_class: current_class.clone(),
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: if previous_class.is_none() {
                    vec![OperationRef {
                        object_type: "net_class".to_string(),
                        uuid: current_class.uuid,
                    }]
                } else {
                    Vec::new()
                },
                modified: vec![OperationRef {
                    object_type: "net".to_string(),
                    uuid: input.net_uuid,
                }],
                deleted: Vec::new(),
            },
            description: format!("set_net_class {}", input.net_uuid),
        })
    }

    pub fn set_design_rule(
        &mut self,
        input: SetDesignRuleInput,
    ) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "set_design_rule is currently implemented only for board projects".to_string(),
            )
        })?;

        let rule_key = (
            input.name.clone(),
            input.rule_type.clone(),
            input.scope.clone(),
        );
        let existing_index = board.rules.iter().position(|rule| {
            (
                Some(rule.name.clone()),
                rule.rule_type.clone(),
                rule.scope.clone(),
            ) == rule_key
                || (rule.name == default_rule_name(&input.rule_type)
                    && input.name.is_none()
                    && rule.rule_type == input.rule_type
                    && rule.scope == input.scope)
        });

        let rule = Rule {
            uuid: existing_index
                .map(|index| board.rules[index].uuid)
                .unwrap_or_else(uuid::Uuid::new_v4),
            name: input
                .name
                .clone()
                .unwrap_or_else(|| default_rule_name(&input.rule_type)),
            scope: input.scope,
            priority: input.priority,
            enabled: true,
            rule_type: input.rule_type,
            parameters: input.parameters,
        };
        validate_rule(&rule)?;

        let previous = existing_index.map(|index| board.rules[index].clone());
        if let Some(index) = existing_index {
            board.rules[index] = rule.clone();
        } else {
            board.rules.push(rule.clone());
        }
        board.rules.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.uuid.cmp(&b.uuid))
        });

        self.undo_stack.push(TransactionRecord::SetDesignRule {
            previous: previous.clone(),
            current: rule.clone(),
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: if previous.is_none() {
                    vec![OperationRef {
                        object_type: "rule".to_string(),
                        uuid: rule.uuid,
                    }]
                } else {
                    Vec::new()
                },
                modified: if previous.is_some() {
                    vec![OperationRef {
                        object_type: "rule".to_string(),
                        uuid: rule.uuid,
                    }]
                } else {
                    Vec::new()
                },
                deleted: Vec::new(),
            },
            description: format!("set_design_rule {}", rule.uuid),
        })
    }
}
