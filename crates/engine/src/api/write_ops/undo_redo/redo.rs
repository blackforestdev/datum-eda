use super::super::*;

impl Engine {
    fn apply_redo_transaction(
        &mut self,
        transaction: &TransactionRecord,
    ) -> Result<OperationResult, EngineError> {
        Ok(match transaction {
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
                if let Some(existing) = board.rules.iter_mut().find(|rule| rule.uuid == current.uuid)
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
            TransactionRecord::Batch {
                description,
                records,
            } => {
                let mut merged_diff = OperationDiff::default();
                for record in records {
                    let result = self.apply_redo_transaction(record)?;
                    merge_operation_diff(&mut merged_diff, &result.diff);
                }
                OperationResult {
                    diff: merged_diff,
                    description: format!("redo {description}"),
                }
            }
        })
    }

    pub fn redo(&mut self) -> Result<OperationResult, EngineError> {
        let transaction = self.redo_stack.pop().ok_or(EngineError::NothingToRedo)?;
        let result = self.apply_redo_transaction(&transaction)?;
        self.undo_stack.push(transaction);
        self.undo_depth = self.undo_stack.len();
        self.redo_depth = self.redo_stack.len();
        Ok(result)
    }
}
