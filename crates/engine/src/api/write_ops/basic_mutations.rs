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
}
