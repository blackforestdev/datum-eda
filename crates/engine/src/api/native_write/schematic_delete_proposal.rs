//! Typed native manual batch deletion through the existing Proposal lifecycle.
//! No direct-write verb is registered: the complete set always requires review.
use super::{context::WriteProvenance, registry::NativeWriteContext, schematic_delete};
use crate::{
    error::EngineError,
    substrate::{
        CommitSource, DesignModel, Operation, Proposal, ProposalCreateRequest, ProposalSource,
        ProposalStatus, apply_accepted_proposal, create_draft_proposal_from_batch,
        preview_proposal_diff_journaled, review_proposal_status,
    },
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeleteClass {
    Symbol,
    Wire,
    Label,
    Port,
    BusSection,
    Junction,
    NoConnect,
}
impl DeleteClass {
    pub(super) fn source_class(self) -> &'static str {
        match self {
            Self::Symbol => "symbols",
            Self::Wire => "wires",
            Self::Label => "labels",
            Self::Port => "ports",
            Self::BusSection => "buses",
            Self::Junction => "junctions",
            Self::NoConnect => "noconnects",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeleteTarget {
    pub class: DeleteClass,
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NativeProposalAction {
    CreateDelete {
        proposal_id: Uuid,
        sheet: Uuid,
        instance_path: Vec<Uuid>,
        targets: Vec<DeleteTarget>,
    },
    Preview {
        proposal_id: Uuid,
    },
    AcceptApply {
        proposal_id: Uuid,
    },
    Reject {
        proposal_id: Uuid,
    },
}
impl NativeProposalAction {
    pub fn proposal_id(&self) -> Uuid {
        match self {
            Self::CreateDelete { proposal_id, .. }
            | Self::Preview { proposal_id }
            | Self::AcceptApply { proposal_id }
            | Self::Reject { proposal_id } => *proposal_id,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProposalRequest {
    pub project_root: PathBuf,
    pub expected_model_revision: String,
    /// Must be present on the wire; null explicitly names an empty journal.
    pub expected_transaction_tip: Option<Uuid>,
    pub action: NativeProposalAction,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletedObject {
    pub id: Uuid,
    pub class: String,
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeProposalResponse {
    pub proposal_id: Uuid,
    pub status: ProposalStatus,
    pub prepared_against: String,
    pub current_model_revision: String,
    pub transaction_tip: Option<Uuid>,
    pub deleted: Vec<DeletedObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<Uuid>,
}

pub fn execute(
    model: &mut DesignModel,
    request: &NativeProposalRequest,
) -> Result<NativeProposalResponse, EngineError> {
    if model.model_revision.0 != request.expected_model_revision
        || crate::revision::transaction_tip(&model.journal) != request.expected_transaction_tip
    {
        return Err(invalid(
            "Selection or Project history changed; prepare the deletion again",
        ));
    }
    let id = request.action.proposal_id();
    let mut selected_count = None;
    let mut transaction_id = None;
    match &request.action {
        NativeProposalAction::CreateDelete {
            sheet,
            instance_path,
            targets,
            ..
        } => {
            let context = NativeWriteContext {
                model,
                project_root: &request.project_root,
            };
            let provenance = WriteProvenance::new(
                "datum-gui",
                CommitSource::Manual,
                "GUI schematic batch Delete",
            );
            let mut prepared =
                schematic_delete::build(&context, provenance, *sheet, instance_path, targets)?;
            prepared.batch.batch_id = id;
            create_draft_proposal_from_batch(
                model,
                &request.project_root,
                ProposalCreateRequest {
                    proposal_id: Some(id),
                    batch: prepared.batch,
                    rationale: format!(
                        "Delete {} selected schematic objects and their owned children",
                        targets.len()
                    ),
                    source: ProposalSource::Manual,
                    checks_run: vec![],
                    finding_fingerprints: vec![],
                },
            )?;
            selected_count = Some(targets.len());
        }
        NativeProposalAction::Preview { .. } => {
            let proposal = delete_proposal(model, id)?;
            if matches!(
                proposal.status,
                ProposalStatus::Draft | ProposalStatus::Accepted
            ) {
                preview_proposal_diff_journaled(model, &request.project_root, id)?;
            }
        }
        NativeProposalAction::AcceptApply { .. } => {
            let status = delete_proposal(model, id)?.status;
            if status == ProposalStatus::Draft {
                review_proposal_status(model, &request.project_root, id, ProposalStatus::Accepted)?;
            } else if status != ProposalStatus::Accepted {
                return Err(invalid(
                    "Only a draft or accepted deletion proposal can be applied",
                ));
            }
            // The existing gateway puts Applied metadata IN the design batch.
            // Nothing is journaled after this: one Undo restores all deletions.
            transaction_id = Some(
                apply_accepted_proposal(model, &request.project_root, id)?
                    .transaction
                    .transaction_id,
            );
        }
        NativeProposalAction::Reject { .. } => {
            delete_proposal(model, id)?;
            review_proposal_status(model, &request.project_root, id, ProposalStatus::Rejected)?;
        }
    }
    let proposal = delete_proposal(model, id)?;
    let deleted: Vec<_> = proposal
        .batch
        .operations
        .iter()
        .filter_map(schematic_delete::deleted_object)
        .collect();
    Ok(NativeProposalResponse {
        proposal_id: id,
        status: proposal.status,
        prepared_against: proposal.prepared_against.0.clone(),
        current_model_revision: model.model_revision.0.clone(),
        transaction_tip: crate::revision::transaction_tip(&model.journal),
        owned_count: selected_count.map(|selected| deleted.len().saturating_sub(selected)),
        selected_count,
        deleted,
        transaction_id: transaction_id.or(proposal.applied_transaction_id),
    })
}

fn delete_proposal(model: &DesignModel, id: Uuid) -> Result<&Proposal, EngineError> {
    let p = model
        .proposals
        .get(&id)
        .ok_or_else(|| invalid("Deletion proposal is unavailable"))?;
    if p.source != ProposalSource::Manual
        || p.batch.provenance.source != CommitSource::Manual
        || !p
            .batch
            .operations
            .iter()
            .any(|o| schematic_delete::deleted_object(o).is_some())
        || p.batch.operations.iter().any(|o| {
            schematic_delete::deleted_object(o).is_none()
                && !matches!(
                    o,
                    Operation::GuardObjectRevision { .. }
                        | Operation::CreateElectricalIdentity { .. }
                        | Operation::SetElectricalIdentity { .. }
                        | Operation::DeleteElectricalIdentity { .. }
                )
        })
    {
        return Err(invalid(
            "This native action requires a manual schematic deletion proposal",
        ));
    }
    Ok(p)
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(reason.into())
}

#[cfg(test)]
#[path = "schematic_delete_proposal_tests.rs"]
mod tests;
