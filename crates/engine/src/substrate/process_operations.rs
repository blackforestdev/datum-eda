//! Canonical process operation vocabulary and exact source preimages.
use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::{DesignModel, Operation, OperationBatch, SourceShardKind};
use crate::{
    board::Stackup,
    error::EngineError,
    pool::process::{ProcessProfile, ProcessSourceCapability},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProcessOperation {
    SetProfiles {
        project_id: Uuid,
        previous: Option<BTreeMap<Uuid, ProcessProfile>>,
        profiles: Option<BTreeMap<Uuid, ProcessProfile>>,
    },
    AdoptCapability {
        board_id: Uuid,
        preimage: String,
        capability: ProcessSourceCapability,
    },
    SetAssociations {
        board_id: Uuid,
        previous: ProcessSourceCapability,
        capability: ProcessSourceCapability,
    },
    RefuseAdoptionInverse {
        board_id: Uuid,
    },
}

impl ProcessOperation {
    pub fn owner_id(&self) -> Uuid {
        match self {
            Self::SetProfiles { project_id, .. } => *project_id,
            Self::AdoptCapability { board_id, .. }
            | Self::SetAssociations { board_id, .. }
            | Self::RefuseAdoptionInverse { board_id } => *board_id,
        }
    }
}

pub(super) fn apply_project(
    value: &mut Value,
    op: &Operation,
) -> Result<Option<bool>, EngineError> {
    let Operation::Process {
        operation:
            ProcessOperation::SetProfiles {
                project_id,
                previous,
                profiles,
            },
    } = op
    else {
        return Ok(None);
    };
    require_identity(value, *project_id)?;
    let current: Option<BTreeMap<Uuid, ProcessProfile>> = value
        .get("process_profiles")
        .map(|v| serde_json::from_value(v.clone()))
        .transpose()?;
    if current != *previous {
        return Err(refusal("stale project profile preimage"));
    }
    if let Some(profiles) = profiles {
        for (id, profile) in profiles {
            if *id != profile.uuid || profile.name.trim().is_empty() {
                return Err(refusal("profile UUID/key mismatch or empty name"));
            }
            profile.basis.validate()?;
        }
        value["process_profiles"] = serde_json::to_value(profiles)?;
    } else {
        value
            .as_object_mut()
            .ok_or_else(|| refusal("ProjectRoot must be an object"))?
            .remove("process_profiles");
    }
    Ok(Some(true))
}

pub(super) fn apply_board(value: &mut Value, op: &Operation) -> Result<Option<bool>, EngineError> {
    let Operation::Process { operation } = op else {
        return Ok(None);
    };
    let capability = match operation {
        ProcessOperation::SetProfiles { .. } => return Ok(None),
        ProcessOperation::RefuseAdoptionInverse { .. } => {
            return Err(refusal(
                "capability adoption is forward-only; inverse refuses",
            ));
        }
        ProcessOperation::AdoptCapability {
            board_id,
            preimage,
            capability,
        } => {
            require_identity(value, *board_id)?;
            let original: Value = serde_json::from_str(preimage)?;
            if original != *value || value.get("process_source_capability").is_some() {
                return Err(refusal(
                    "stale adoption preimage or capability already declared",
                ));
            }
            if value.get("schema_version").and_then(Value::as_u64) != Some(4) {
                return Err(refusal(
                    "process adoption requires independently adopted historical profile 4",
                ));
            }
            capability
        }
        ProcessOperation::SetAssociations {
            board_id,
            previous,
            capability,
        } => {
            require_identity(value, *board_id)?;
            let current = capability_from_board(value)?
                .ok_or_else(|| refusal("missing adopted process capability"))?;
            if current != *previous {
                return Err(refusal("stale process association preimage"));
            }
            capability
        }
    };
    let stackup: Stackup = serde_json::from_value(value["stackup"].clone())?;
    capability.validate(&stackup)?;
    value["process_source_capability"] = serde_json::to_value(capability)?;
    Ok(Some(true))
}

pub fn capability_from_board(
    value: &Value,
) -> Result<Option<ProcessSourceCapability>, EngineError> {
    value
        .get("process_source_capability")
        .map(|v| serde_json::from_value(v.clone()).map_err(EngineError::from))
        .transpose()
}

pub(super) fn validate_native(board: &Value) -> Result<(), EngineError> {
    if let Some(capability) = capability_from_board(board)? {
        let stackup: Stackup = serde_json::from_value(board["stackup"].clone())?;
        capability.validate(&stackup)?;
    }
    Ok(())
}

pub(super) fn inverse(
    value: &mut Value,
    op: &Operation,
    inverse: &mut Vec<Operation>,
    board: bool,
) -> Result<bool, EngineError> {
    let Operation::Process { operation } = op else {
        return Ok(false);
    };
    let inverse_op = match operation {
        ProcessOperation::SetProfiles {
            project_id,
            previous,
            profiles,
        } if !board => ProcessOperation::SetProfiles {
            project_id: *project_id,
            previous: profiles.clone(),
            profiles: previous.clone(),
        },
        ProcessOperation::AdoptCapability { board_id, .. } if board => {
            ProcessOperation::RefuseAdoptionInverse {
                board_id: *board_id,
            }
        }
        ProcessOperation::SetAssociations {
            board_id,
            previous,
            capability,
        } if board => ProcessOperation::SetAssociations {
            board_id: *board_id,
            previous: capability.clone(),
            capability: previous.clone(),
        },
        ProcessOperation::RefuseAdoptionInverse { .. } if board => {
            return Err(refusal("capability adoption inverse cannot replay"));
        }
        _ => return Ok(false),
    };
    if board {
        apply_board(value, op)?;
    } else {
        apply_project(value, op)?;
    }
    inverse.push(Operation::Process {
        operation: inverse_op,
    });
    Ok(true)
}

pub(super) fn validate_batch(
    model: &DesignModel,
    root: &Path,
    batch: &OperationBatch,
) -> Result<(), EngineError> {
    let adoptions: Vec<_> = batch
        .operations
        .iter()
        .filter_map(|op| match op {
            Operation::Process {
                operation: ProcessOperation::AdoptCapability { preimage, .. },
            } => Some(preimage),
            _ => None,
        })
        .collect();
    if !adoptions.is_empty() {
        if adoptions.len() != 1
            || batch.operations.iter().any(|op| {
                !matches!(
                    op,
                    Operation::GuardObjectRevision { .. }
                        | Operation::Process {
                            operation: ProcessOperation::AdoptCapability { .. }
                        }
                )
            })
        {
            return Err(refusal(
                "capability adoption requires a separate single-adoption transaction",
            ));
        }
        let shard = model
            .source_shards
            .iter()
            .find(|s| s.kind == SourceShardKind::BoardRoot)
            .ok_or_else(|| refusal("missing BoardRoot"))?;
        let bytes = std::fs::read(root.join(&shard.relative_path))?;
        if bytes != adoptions[0].as_bytes() || super::sha256_hex(&bytes) != shard.content_hash {
            return Err(refusal(
                "stale actual byte preimage for capability adoption",
            ));
        }
    }
    // Canonical deletion must account for every current dependent selector.
    for op in &batch.operations {
        if let Operation::Process {
            operation:
                ProcessOperation::SetProfiles {
                    previous: Some(previous),
                    profiles,
                    ..
                },
        } = op
        {
            for id in previous
                .keys()
                .filter(|id| profiles.as_ref().is_none_or(|p| !p.contains_key(id)))
            {
                for shard in model
                    .source_shards
                    .iter()
                    .filter(|s| s.kind == SourceShardKind::Pool)
                {
                    let value = super::journal::capture_shard_value(model, shard)?;
                    if value["process_basis"]["kind"] == "project_profile"
                        && value["process_basis"]["profile_id"] == id.to_string()
                    {
                        return Err(refusal(
                            "cannot delete a referenced process profile; reconcile dependent selectors first",
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn require_identity(value: &Value, id: Uuid) -> Result<(), EngineError> {
    if value.get("uuid").and_then(Value::as_str) != Some(id.to_string().as_str()) {
        return Err(refusal("source owner UUID mismatch"));
    }
    Ok(())
}

pub(super) fn apply_objects(
    objects: &mut BTreeMap<Uuid, super::DomainObject>,
    mut diff: Option<&mut super::CommitDiff>,
    operation: &ProcessOperation,
) -> Result<(), EngineError> {
    if matches!(operation, ProcessOperation::RefuseAdoptionInverse { .. }) {
        return Err(refusal(
            "capability adoption is forward-only; inverse refuses",
        ));
    }
    let owner = objects
        .get(&operation.owner_id())
        .cloned()
        .ok_or_else(|| refusal("missing canonical process source owner"))?;
    if let ProcessOperation::SetProfiles {
        previous, profiles, ..
    } = operation
    {
        if let Some(previous) = previous {
            for id in previous
                .keys()
                .filter(|id| profiles.as_ref().is_none_or(|p| !p.contains_key(id)))
            {
                objects.remove(id);
                if let Some(d) = diff.as_deref_mut() {
                    d.deleted.push(*id);
                }
            }
        }
        if let Some(profiles) = profiles {
            for (id, profile) in profiles {
                if previous.as_ref().and_then(|p| p.get(id)) == Some(profile) {
                    continue;
                }
                let existing = objects.get(id).cloned();
                if let Some(existing) = &existing {
                    if existing.source_shard_id != owner.source_shard_id
                        || previous.as_ref().is_none_or(|p| !p.contains_key(id))
                    {
                        return Err(refusal(
                            "process profile UUID collides with another canonical object",
                        ));
                    }
                }
                let revision = existing
                    .as_ref()
                    .map(|o| o.object_revision.0.checked_add(1))
                    .unwrap_or(Some(0))
                    .ok_or_else(|| refusal("process profile revision overflow"))?;
                objects.insert(
                    *id,
                    super::DomainObject {
                        object_id: *id,
                        object_revision: super::ObjectRevision(revision),
                        source_shard_id: owner.source_shard_id,
                        domain: owner.domain.clone(),
                        kind: id.to_string(),
                    },
                );
                if let Some(d) = diff.as_deref_mut() {
                    if existing.is_some() {
                        d.modified.push(*id);
                    } else {
                        d.created.push(*id);
                    }
                }
            }
        }
    }
    super::operation_application_object_revision::bump_existing_object(
        objects,
        operation.owner_id(),
        diff,
    )
}

fn refusal(reason: &str) -> EngineError {
    EngineError::Validation(format!("native process operation: {reason}"))
}
