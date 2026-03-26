use super::super::*;

impl Engine {
    fn apply_undo_transaction(
        &mut self,
        transaction: &TransactionRecord,
    ) -> Result<OperationResult, EngineError> {
        Ok(match transaction {
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
            TransactionRecord::Batch {
                description,
                records,
            } => {
                let mut merged_diff = OperationDiff::default();
                for record in records.iter().rev() {
                    let result = self.apply_undo_transaction(record)?;
                    merge_operation_diff(&mut merged_diff, &result.diff);
                }
                OperationResult {
                    diff: merged_diff,
                    description: format!("undo {description}"),
                }
            }
        })
    }

    pub fn undo(&mut self) -> Result<OperationResult, EngineError> {
        let transaction = self.undo_stack.pop().ok_or(EngineError::NothingToUndo)?;
        let result = self.apply_undo_transaction(&transaction)?;
        self.redo_stack.push(transaction);
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = self.redo_stack.len();
        Ok(result)
    }
}
