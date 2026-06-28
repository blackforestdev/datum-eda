use std::path::Path;

use uuid::Uuid;

use super::{
    DesignModel, EngineError, Operation, OperationBatch, Proposal, SourceShardKind,
    TransactionRecord,
    journal::{StagedShardWrite, stage_new_shard_write},
    proposal_validation::validate_proposal_payload_schema_version,
};

pub(super) fn maybe_stage_proposal_operation(
    project_root: &Path,
    batch: &OperationBatch,
    operation: &Operation,
    staged: &mut Vec<StagedShardWrite>,
) -> Result<(), EngineError> {
    match operation {
        Operation::CreateProposalMetadata {
            relative_path,
            proposal,
            ..
        }
        | Operation::SetProposalMetadata {
            relative_path,
            proposal,
            ..
        } => staged.push(stage_new_shard_write(
            project_root,
            batch,
            SourceShardKind::ProposalMetadata,
            relative_path,
            proposal,
        )?),
        Operation::DeleteProposalMetadata { relative_path, .. } => {
            staged.push(delete_proposal_shard(project_root, relative_path));
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn apply_proposal_shard_operation(
    shard_kind: &SourceShardKind,
    value: &mut serde_json::Value,
    operation: &Operation,
) -> Result<bool, EngineError> {
    if shard_kind != &SourceShardKind::ProposalMetadata {
        return Ok(false);
    }
    match operation {
        Operation::CreateProposalMetadata { proposal, .. }
        | Operation::SetProposalMetadata { proposal, .. } => {
            *value = proposal.clone();
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn inverse_proposal_operation(
    operation: &Operation,
    inverse_operations: &mut Vec<Operation>,
) {
    match operation {
        Operation::CreateProposalMetadata {
            proposal_id,
            relative_path,
            proposal,
        } => inverse_operations.push(Operation::DeleteProposalMetadata {
            proposal_id: *proposal_id,
            relative_path: relative_path.clone(),
            proposal: proposal.clone(),
        }),
        Operation::SetProposalMetadata {
            proposal_id,
            relative_path,
            previous_proposal,
            proposal,
        } => inverse_operations.push(Operation::SetProposalMetadata {
            proposal_id: *proposal_id,
            relative_path: relative_path.clone(),
            previous_proposal: proposal.clone(),
            proposal: previous_proposal.clone(),
        }),
        Operation::DeleteProposalMetadata {
            proposal_id,
            relative_path,
            proposal,
        } => inverse_operations.push(Operation::CreateProposalMetadata {
            proposal_id: *proposal_id,
            relative_path: relative_path.clone(),
            proposal: proposal.clone(),
        }),
        _ => {}
    }
}

pub(super) fn proposal_from_value(value: &serde_json::Value) -> Result<Proposal, EngineError> {
    let proposal = serde_json::from_value(value.clone())
        .map_err(|error| EngineError::Validation(format!("invalid proposal metadata: {error}")))?;
    validate_proposal_payload_schema_version(&proposal)?;
    Ok(proposal)
}

pub(super) fn proposal_relative_path(proposal_id: Uuid) -> String {
    format!(".datum/proposals/{proposal_id}.json")
}

pub(super) fn proposal_operation_write(
    operation: &Operation,
) -> Option<(Uuid, String, Option<&serde_json::Value>)> {
    match operation {
        Operation::CreateProposalMetadata {
            proposal_id,
            relative_path,
            proposal,
        }
        | Operation::SetProposalMetadata {
            proposal_id,
            relative_path,
            proposal,
            ..
        } => Some((*proposal_id, relative_path.clone(), Some(proposal))),
        Operation::DeleteProposalMetadata {
            proposal_id,
            relative_path,
            ..
        } => Some((*proposal_id, relative_path.clone(), None)),
        _ => None,
    }
}

pub(super) fn reconstruct_proposal_metadata_value(
    relative_path: &str,
    journal: &[TransactionRecord],
) -> Result<serde_json::Value, EngineError> {
    let mut value = None;
    for transaction in journal {
        for operation in &transaction.operations {
            if let Some((_, operation_path, next_value)) = proposal_operation_write(operation) {
                if operation_path == relative_path {
                    value = next_value.cloned();
                }
            }
        }
    }
    value.ok_or_else(|| {
        EngineError::Validation(format!(
            "cannot reconstruct missing proposal metadata shard `{relative_path}`"
        ))
    })
}

pub(super) fn apply_proposal_journal_to_map(
    journal: &[TransactionRecord],
    proposals: &mut std::collections::BTreeMap<Uuid, Proposal>,
) -> Result<(), EngineError> {
    for transaction in journal {
        for operation in &transaction.operations {
            match operation {
                Operation::CreateProposalMetadata {
                    proposal_id,
                    proposal,
                    ..
                }
                | Operation::SetProposalMetadata {
                    proposal_id,
                    proposal,
                    ..
                } => {
                    proposals.insert(*proposal_id, proposal_from_value(proposal)?);
                }
                Operation::DeleteProposalMetadata { proposal_id, .. } => {
                    proposals.remove(proposal_id);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub(super) fn apply_proposal_model_operation(
    model: &mut DesignModel,
    operation: &Operation,
) -> Result<bool, EngineError> {
    match operation {
        Operation::CreateProposalMetadata {
            proposal_id,
            proposal,
            ..
        } => {
            let proposal = proposal_from_value(proposal)?;
            if model.proposals.get(proposal_id) == Some(&proposal) {
                return Ok(true);
            }
            if model.proposals.contains_key(proposal_id) {
                return Err(EngineError::Validation(format!(
                    "proposal {proposal_id} already exists"
                )));
            }
            model.proposals.insert(*proposal_id, proposal);
            Ok(true)
        }
        Operation::SetProposalMetadata {
            proposal_id,
            proposal,
            ..
        } => {
            model
                .proposals
                .insert(*proposal_id, proposal_from_value(proposal)?);
            Ok(true)
        }
        Operation::DeleteProposalMetadata { proposal_id, .. } => {
            model.proposals.remove(proposal_id);
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn delete_proposal_shard(project_root: &Path, relative_path: &str) -> StagedShardWrite {
    StagedShardWrite {
        destination: project_root.join(relative_path),
        staged: None,
        kind: SourceShardKind::ProposalMetadata,
        relative_path: relative_path.to_string(),
        content_hash: String::new(),
        schema_version: None,
        delete: true,
    }
}
