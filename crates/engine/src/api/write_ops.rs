use super::*;

impl Engine {
    pub fn delete_track(&mut self, uuid: &uuid::Uuid) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "delete_track is currently implemented only for board projects".to_string(),
            )
        })?;
        let track = board.tracks.remove(uuid).ok_or(EngineError::NotFound {
            object_type: "track",
            uuid: *uuid,
        })?;

        self.undo_stack.push(TransactionRecord::DeleteTrack {
            track: track.clone(),
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: Vec::new(),
                deleted: vec![OperationRef {
                    object_type: "track".to_string(),
                    uuid: *uuid,
                }],
            },
            description: format!("delete_track {}", uuid),
        })
    }

    pub fn delete_via(&mut self, uuid: &uuid::Uuid) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "delete_via is currently implemented only for board projects".to_string(),
            )
        })?;
        let via = board.vias.remove(uuid).ok_or(EngineError::NotFound {
            object_type: "via",
            uuid: *uuid,
        })?;

        self.undo_stack
            .push(TransactionRecord::DeleteVia { via: via.clone() });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: Vec::new(),
                deleted: vec![OperationRef {
                    object_type: "via".to_string(),
                    uuid: *uuid,
                }],
            },
            description: format!("delete_via {}", uuid),
        })
    }

    pub fn delete_component(&mut self, uuid: &uuid::Uuid) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "delete_component is currently implemented only for board projects".to_string(),
            )
        })?;
        let package = board.packages.remove(uuid).ok_or(EngineError::NotFound {
            object_type: "component",
            uuid: *uuid,
        })?;
        let pad_uuids: Vec<_> = board
            .pads
            .values()
            .filter(|pad| pad.package == *uuid)
            .map(|pad| pad.uuid)
            .collect();
        let mut pads = Vec::with_capacity(pad_uuids.len());
        for pad_uuid in pad_uuids {
            let pad = board.pads.remove(&pad_uuid).ok_or(EngineError::NotFound {
                object_type: "pad",
                uuid: pad_uuid,
            })?;
            pads.push(pad);
        }
        pads.sort_by_key(|pad| pad.uuid);

        self.undo_stack.push(TransactionRecord::DeleteComponent {
            package: package.clone(),
            pads,
        });
        self.redo_stack.clear();
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = 0;

        Ok(OperationResult {
            diff: OperationDiff {
                created: Vec::new(),
                modified: Vec::new(),
                deleted: vec![OperationRef {
                    object_type: "component".to_string(),
                    uuid: *uuid,
                }],
            },
            description: format!("delete_component {}", uuid),
        })
    }

    pub fn move_component(
        &mut self,
        input: MoveComponentInput,
    ) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "move_component is currently implemented only for board projects".to_string(),
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
        let after = crate::board::PlacedPackage {
            position: input.position,
            rotation: input.rotation.unwrap_or(before.rotation),
            ..before.clone()
        };
        let (before_pads, after_pads) = apply_package_transform(board, &before, &after)?;

        self.undo_stack.push(TransactionRecord::MoveComponent {
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
            description: format!("move_component {}", input.uuid),
        })
    }

    pub fn rotate_component(
        &mut self,
        input: RotateComponentInput,
    ) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "rotate_component is currently implemented only for board projects".to_string(),
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
        let after = crate::board::PlacedPackage {
            rotation: input.rotation,
            ..before.clone()
        };
        let (before_pads, after_pads) = apply_package_transform(board, &before, &after)?;

        self.undo_stack.push(TransactionRecord::RotateComponent {
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
            description: format!("rotate_component {}", input.uuid),
        })
    }

    pub fn set_value(&mut self, input: SetValueInput) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "set_value is currently implemented only for board projects".to_string(),
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
        let package = board
            .packages
            .get_mut(&input.uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        package.value = input.value;
        let after = package.clone();

        self.undo_stack.push(TransactionRecord::SetValue {
            before: before.clone(),
            after: after.clone(),
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
            description: format!("set_value {}", input.uuid),
        })
    }

    pub fn set_reference(
        &mut self,
        input: SetReferenceInput,
    ) -> Result<OperationResult, EngineError> {
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "set_reference is currently implemented only for board projects".to_string(),
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
        let package = board
            .packages
            .get_mut(&input.uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        package.reference = input.reference;
        let after = package.clone();

        self.undo_stack.push(TransactionRecord::SetReference {
            before: before.clone(),
            after: after.clone(),
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
            description: format!("set_reference {}", input.uuid),
        })
    }

    pub fn assign_part(&mut self, input: AssignPartInput) -> Result<OperationResult, EngineError> {
        let part = self.pool.parts.get(&input.part_uuid).ok_or(EngineError::NotFound {
            object_type: "part",
            uuid: input.part_uuid,
        })?;
        let target_package = self
            .pool
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
        replace_component_pads_for_assign_part(board, &before, &after, part, target_package, &self.pool)?;
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

    pub fn set_package(
        &mut self,
        input: SetPackageInput,
    ) -> Result<OperationResult, EngineError> {
        let target_package = self
            .pool
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
            let target_part = self.pool.parts.get(&part_uuid).ok_or(EngineError::DanglingReference {
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

    pub fn set_package_with_part(
        &mut self,
        input: SetPackageWithPartInput,
    ) -> Result<OperationResult, EngineError> {
        let target_part = self
            .pool
            .parts
            .get(&input.part_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "part",
                uuid: input.part_uuid,
            })?
            .clone();
        if target_part.package != input.package_uuid {
            return Err(EngineError::Operation(format!(
                "set_package_with_part requires part {} to use package {}",
                input.part_uuid, input.package_uuid
            )));
        }
        let target_package = self
            .pool
            .packages
            .get(&input.package_uuid)
            .ok_or(EngineError::NotFound {
                object_type: "package",
                uuid: input.package_uuid,
            })?
            .clone();
        let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
        let board = design.board.as_mut().ok_or_else(|| {
            EngineError::Operation(
                "set_package_with_part is currently implemented only for board projects"
                    .to_string(),
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
        if before.part != Uuid::nil() && self.pool.parts.contains_key(&before.part) {
            let current_part = self.pool.parts.get(&before.part).ok_or(EngineError::NotFound {
                object_type: "part",
                uuid: before.part,
            })?;
            let current_signature =
                part_pin_signature(current_part, &self.pool).ok_or(EngineError::DanglingReference {
                    source_type: "part",
                    source_uuid: current_part.uuid,
                    target_type: "entity",
                    target_uuid: current_part.entity,
                })?;
            let target_signature =
                part_pin_signature(&target_part, &self.pool).ok_or(EngineError::DanglingReference {
                    source_type: "part",
                    source_uuid: target_part.uuid,
                    target_type: "entity",
                    target_uuid: target_part.entity,
                })?;
            if current_signature != target_signature {
                return Err(EngineError::Operation(format!(
                    "set_package_with_part target part {} is not logically compatible with current component {}; inspect get_package_change_candidates first",
                    input.part_uuid, input.uuid
                )));
            }
        }
        let before_pads = component_pads(board, input.uuid);
        let package = board
            .packages
            .get_mut(&input.uuid)
            .ok_or(EngineError::NotFound {
                object_type: "component",
                uuid: input.uuid,
            })?;
        package.package = input.package_uuid;
        package.part = input.part_uuid;
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
            description: format!("set_package_with_part {}", input.uuid),
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
        board.net_classes.insert(target_class_uuid, current_class.clone());
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

    pub fn undo(&mut self) -> Result<OperationResult, EngineError> {
        let transaction = self.undo_stack.pop().ok_or(EngineError::NothingToUndo)?;
        let result = match &transaction {
            TransactionRecord::DeleteTrack { track } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                board.tracks.insert(track.uuid, track.clone());
                OperationResult {
                    diff: OperationDiff {
                        created: vec![OperationRef {
                            object_type: "track".to_string(),
                            uuid: track.uuid,
                        }],
                        modified: Vec::new(),
                        deleted: Vec::new(),
                    },
                    description: format!("undo delete_track {}", track.uuid),
                }
            }
            TransactionRecord::DeleteVia { via } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                board.vias.insert(via.uuid, via.clone());
                OperationResult {
                    diff: OperationDiff {
                        created: vec![OperationRef {
                            object_type: "via".to_string(),
                            uuid: via.uuid,
                        }],
                        modified: Vec::new(),
                        deleted: Vec::new(),
                    },
                    description: format!("undo delete_via {}", via.uuid),
                }
            }
            TransactionRecord::DeleteComponent { package, pads } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                board.packages.insert(package.uuid, package.clone());
                for pad in pads {
                    board.pads.insert(pad.uuid, pad.clone());
                }
                OperationResult {
                    diff: OperationDiff {
                        created: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: package.uuid,
                        }],
                        modified: Vec::new(),
                        deleted: Vec::new(),
                    },
                    description: format!("undo delete_component {}", package.uuid),
                }
            }
            TransactionRecord::MoveComponent {
                before,
                after,
                before_pads,
                after_pads: _,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                restore_package_transform(board, after.uuid, before.clone(), before_pads)?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo move_component {}", after.uuid),
                }
            }
            TransactionRecord::RotateComponent {
                before,
                after,
                before_pads,
                after_pads: _,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                restore_package_transform(board, after.uuid, before.clone(), before_pads)?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo rotate_component {}", after.uuid),
                }
            }
            TransactionRecord::SetDesignRule { previous, current } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                match previous {
                    Some(previous_rule) => {
                        let rule = board
                            .rules
                            .iter_mut()
                            .find(|rule| rule.uuid == current.uuid)
                            .ok_or(EngineError::NotFound {
                                object_type: "rule",
                                uuid: current.uuid,
                            })?;
                        *rule = previous_rule.clone();
                        OperationResult {
                            diff: OperationDiff {
                                created: Vec::new(),
                                modified: vec![OperationRef {
                                    object_type: "rule".to_string(),
                                    uuid: current.uuid,
                                }],
                                deleted: Vec::new(),
                            },
                            description: format!("undo set_design_rule {}", current.uuid),
                        }
                    }
                    None => {
                        let removed = board
                            .rules
                            .iter()
                            .position(|rule| rule.uuid == current.uuid)
                            .ok_or(EngineError::NotFound {
                                object_type: "rule",
                                uuid: current.uuid,
                            })?;
                        board.rules.remove(removed);
                        OperationResult {
                            diff: OperationDiff {
                                created: Vec::new(),
                                modified: Vec::new(),
                                deleted: vec![OperationRef {
                                    object_type: "rule".to_string(),
                                    uuid: current.uuid,
                                }],
                            },
                            description: format!("undo set_design_rule {}", current.uuid),
                        }
                    }
                }
            }
            TransactionRecord::SetValue { before, after } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = before.clone();
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo set_value {}", after.uuid),
                }
            }
            TransactionRecord::SetReference { before, after } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = before.clone();
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo set_reference {}", after.uuid),
                }
            }
            TransactionRecord::AssignPart {
                before,
                after,
                before_pads,
                after_pads: _,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = before.clone();
                restore_component_pads(board, after.uuid, before_pads);
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo assign_part {}", after.uuid),
                }
            }
            TransactionRecord::SetPackage {
                before,
                after,
                before_pads,
                after_pads: _,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = before.clone();
                restore_component_pads(board, after.uuid, before_pads);
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo set_package {}", after.uuid),
                }
            }
            TransactionRecord::SetNetClass {
                before_net,
                after_net: _,
                previous_class,
                current_class,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "undo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let net = board
                    .nets
                    .get_mut(&before_net.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "net",
                        uuid: before_net.uuid,
                    })?;
                *net = before_net.clone();
                if let Some(previous_class) = previous_class {
                    board
                        .net_classes
                        .insert(previous_class.uuid, previous_class.clone());
                } else if current_class.uuid != Uuid::nil() {
                    board.net_classes.remove(&current_class.uuid);
                }
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "net".to_string(),
                            uuid: before_net.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("undo set_net_class {}", before_net.uuid),
                }
            }
        };
        self.redo_stack.push(transaction);
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = self.redo_stack.len();
        Ok(result)
    }

    pub fn redo(&mut self) -> Result<OperationResult, EngineError> {
        let transaction = self.redo_stack.pop().ok_or(EngineError::NothingToRedo)?;
        let result = match &transaction {
            TransactionRecord::DeleteTrack { track } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let removed = board
                    .tracks
                    .remove(&track.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "track",
                        uuid: track.uuid,
                    })?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: Vec::new(),
                        deleted: vec![OperationRef {
                            object_type: "track".to_string(),
                            uuid: removed.uuid,
                        }],
                    },
                    description: format!("redo delete_track {}", track.uuid),
                }
            }
            TransactionRecord::DeleteVia { via } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let removed = board.vias.remove(&via.uuid).ok_or(EngineError::NotFound {
                    object_type: "via",
                    uuid: via.uuid,
                })?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: Vec::new(),
                        deleted: vec![OperationRef {
                            object_type: "via".to_string(),
                            uuid: removed.uuid,
                        }],
                    },
                    description: format!("redo delete_via {}", via.uuid),
                }
            }
            TransactionRecord::DeleteComponent { package, pads } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let removed = board
                    .packages
                    .remove(&package.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: package.uuid,
                    })?;
                for pad in pads {
                    board.pads.remove(&pad.uuid).ok_or(EngineError::NotFound {
                        object_type: "pad",
                        uuid: pad.uuid,
                    })?;
                }
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: Vec::new(),
                        deleted: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: removed.uuid,
                        }],
                    },
                    description: format!("redo delete_component {}", package.uuid),
                }
            }
            TransactionRecord::MoveComponent {
                before: _,
                after,
                before_pads: _,
                after_pads,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                restore_package_transform(board, after.uuid, after.clone(), after_pads)?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo move_component {}", after.uuid),
                }
            }
            TransactionRecord::RotateComponent {
                before: _,
                after,
                before_pads: _,
                after_pads,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                restore_package_transform(board, after.uuid, after.clone(), after_pads)?;
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo rotate_component {}", after.uuid),
                }
            }
            TransactionRecord::SetDesignRule {
                previous: _,
                current,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                if let Some(existing) = board
                    .rules
                    .iter_mut()
                    .find(|rule| rule.uuid == current.uuid)
                {
                    *existing = current.clone();
                    OperationResult {
                        diff: OperationDiff {
                            created: Vec::new(),
                            modified: vec![OperationRef {
                                object_type: "rule".to_string(),
                                uuid: current.uuid,
                            }],
                            deleted: Vec::new(),
                        },
                        description: format!("redo set_design_rule {}", current.uuid),
                    }
                } else {
                    board.rules.push(current.clone());
                    board.rules.sort_by(|a, b| {
                        a.priority
                            .cmp(&b.priority)
                            .then_with(|| a.name.cmp(&b.name))
                            .then_with(|| a.uuid.cmp(&b.uuid))
                    });
                    OperationResult {
                        diff: OperationDiff {
                            created: vec![OperationRef {
                                object_type: "rule".to_string(),
                                uuid: current.uuid,
                            }],
                            modified: Vec::new(),
                            deleted: Vec::new(),
                        },
                        description: format!("redo set_design_rule {}", current.uuid),
                    }
                }
            }
            TransactionRecord::SetValue { before: _, after } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = after.clone();
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo set_value {}", after.uuid),
                }
            }
            TransactionRecord::SetReference { before: _, after } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = after.clone();
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo set_reference {}", after.uuid),
                }
            }
            TransactionRecord::AssignPart {
                before: _,
                after,
                before_pads: _,
                after_pads,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = after.clone();
                restore_component_pads(board, after.uuid, after_pads);
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo assign_part {}", after.uuid),
                }
            }
            TransactionRecord::SetPackage {
                before: _,
                after,
                before_pads: _,
                after_pads,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                let package = board
                    .packages
                    .get_mut(&after.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "component",
                        uuid: after.uuid,
                    })?;
                *package = after.clone();
                restore_component_pads(board, after.uuid, after_pads);
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "component".to_string(),
                            uuid: after.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo set_package {}", after.uuid),
                }
            }
            TransactionRecord::SetNetClass {
                before_net: _,
                after_net,
                previous_class: _,
                current_class,
            } => {
                let design = self.design.as_mut().ok_or(EngineError::NoProjectOpen)?;
                let board = design.board.as_mut().ok_or_else(|| {
                    EngineError::Operation(
                        "redo is currently implemented only for board transactions".to_string(),
                    )
                })?;
                board
                    .net_classes
                    .insert(current_class.uuid, current_class.clone());
                let net = board
                    .nets
                    .get_mut(&after_net.uuid)
                    .ok_or(EngineError::NotFound {
                        object_type: "net",
                        uuid: after_net.uuid,
                    })?;
                *net = after_net.clone();
                OperationResult {
                    diff: OperationDiff {
                        created: Vec::new(),
                        modified: vec![OperationRef {
                            object_type: "net".to_string(),
                            uuid: after_net.uuid,
                        }],
                        deleted: Vec::new(),
                    },
                    description: format!("redo set_net_class {}", after_net.uuid),
                }
            }
        };
        self.undo_stack.push(transaction);
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = self.redo_stack.len();
        Ok(result)
    }
}
