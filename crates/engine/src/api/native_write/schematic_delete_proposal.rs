//! Typed native manual commands through the existing Proposal lifecycle.
//! No direct-write verb is registered: the complete set always requires review.
pub use super::schematic_annotation::{AnnotationChange, AnnotationSummary};
pub use super::schematic_assignment::{AssignmentChoice, AssignmentSummary};
pub use super::schematic_child_sheet::{ChildSheetRequest, ChildSheetSummary};
pub use super::schematic_disposition::{DispositionKind, DispositionSummary};
pub use super::schematic_field_batch::{FieldBatchSummary, FieldRowEdit};
pub use super::schematic_gate_placement::{GatePlacementRequest, GatePlacementSummary};
pub use super::schematic_interface_binding::{InterfaceBindingRequest, InterfaceBindingSummary};
pub use super::schematic_project_annotation::ProjectAnnotationSummary;
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
#[path = "schematic_proposal_authority.rs"]
mod authority;

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
    SchematicText,
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
            Self::SchematicText => "texts",
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
    CreateBoardFieldRevision {
        proposal_id: Uuid,
        sheet: Uuid,
        #[serde(default)]
        review: Option<super::schematic_board_transfer::field_revision::BoardFieldRevision>,
    },
    CreateDesignEdit {
        proposal_id: Uuid,
        sheet: Uuid,
        instance_path: Vec<Uuid>,
        edit: super::design_clipboard_edit::DesignEditIntent,
    },
    CreateProjectAnnotation {
        proposal_id: Uuid,
    },
    CreateFieldBatch {
        proposal_id: Uuid,
        sheet: Uuid,
        instance_path: Vec<Uuid>,
        edits: Vec<FieldRowEdit>,
    },
    CreateGatePlacement(GatePlacementRequest),
    CreateChildSheet(ChildSheetRequest),
    CreateInterfaceBinding(InterfaceBindingRequest),
    CreateAnnotation {
        proposal_id: Uuid,
        sheet: Uuid,
        instance_path: Vec<Uuid>,
    },
    CreateAssignment {
        proposal_id: Uuid,
        symbol: Uuid,
        instance_path: Vec<Uuid>,
        part: Uuid,
        gate: Uuid,
        #[serde(default)]
        component: Option<super::schematic_library::part_placement::CompletePartSelection>,
    },
    CreateDisposition {
        proposal_id: Uuid,
        check_run_id: Uuid,
        fingerprint: String,
        disposition: DispositionKind,
        rationale: String,
    },
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
            Self::CreateGatePlacement(request) => request.proposal_id,
            Self::CreateChildSheet(request) => request.proposal_id,
            Self::CreateInterfaceBinding(request) => request.proposal_id,
            Self::CreateBoardFieldRevision { proposal_id, .. }
            | Self::CreateDesignEdit { proposal_id, .. }
            | Self::CreateFieldBatch { proposal_id, .. }
            | Self::CreateAnnotation { proposal_id, .. }
            | Self::CreateProjectAnnotation { proposal_id }
            | Self::CreateAssignment { proposal_id, .. }
            | Self::CreateDisposition { proposal_id, .. }
            | Self::CreateDelete { proposal_id, .. }
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board_field_revision:
        Option<super::schematic_board_transfer::field_revision::BoardFieldRevision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotation: Option<AnnotationSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_annotation: Option<ProjectAnnotationSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<FieldBatchSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate_placement: Option<GatePlacementSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_sheet: Option<ChildSheetSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface_binding: Option<InterfaceBindingSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<DispositionSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignment: Option<AssignmentSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owned_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<Uuid>,
}

/// Invocation-only qualified ownership; no menu-paint source reads are needed.
pub fn bound_symbol_owner(
    model: &DesignModel,
    symbol: Uuid,
    instance_path: &[Uuid],
) -> Result<Option<Uuid>, EngineError> {
    let snapshot = crate::substrate::ElectricalSelectionSnapshot::capture(model)
        .map_err(|e| invalid(&format!("Selected source unavailable: {e:?}")))?;
    let source =
        super::schematic_selected_source::SelectedSchematicSource::from_snapshot(&snapshot)?;
    let sheet = source.sheet("symbols", symbol, instance_path)?;
    let raw = super::schematic_symbol_payload::source_sheet_payload(model, sheet, symbol)?;
    let children = super::schematic_delete::owned_symbol_objects(
        model,
        &source.source.sheets[&sheet],
        &std::collections::BTreeSet::from([symbol]),
        &raw,
    )?;
    Ok(super::schematic_assignment::delete_bindings(
        model,
        &std::collections::BTreeSet::from([symbol]),
        &children,
    )?
    .remove(&symbol)
    .map(|(id, _)| id))
}

pub fn execute(
    model: &mut DesignModel,
    request: &NativeProposalRequest,
) -> Result<NativeProposalResponse, EngineError> {
    if model.model_revision.0 != request.expected_model_revision
        || crate::revision::transaction_tip(&model.journal) != request.expected_transaction_tip
    {
        return Err(invalid(
            "Selection or Project history changed; prepare the proposal again",
        ));
    }
    let id = request.action.proposal_id();
    let mut selected_count = None;
    let mut transaction_id = None;
    match &request.action {
        NativeProposalAction::CreateBoardFieldRevision { sheet, review, .. } => {
            let captured =
                super::schematic_board_transfer::field_revision::preview(model, *sheet, &[])?;
            if review
                .as_ref()
                .is_some_and(|reviewed| *reviewed != captured)
            {
                return Err(invalid(
                    "The reviewed PCB field changes differ from the current source",
                ));
            }
            let write = super::schematic_board_transfer::field_revision::prepare(
                model,
                WriteProvenance::new(
                    "datum-gui",
                    CommitSource::Manual,
                    super::schematic_board_transfer::field_revision::REASON,
                ),
                &captured,
                id,
            )?;
            create_draft_proposal_from_batch(
                model,
                &request.project_root,
                ProposalCreateRequest {
                    proposal_id: Some(id),
                    batch: write.batch,
                    rationale: serde_json::to_string(&captured)?,
                    source: ProposalSource::Manual,
                    checks_run: vec![],
                    finding_fingerprints: vec![],
                },
            )?;
        }
        NativeProposalAction::CreateDesignEdit {
            sheet,
            instance_path,
            edit,
            ..
        } => {
            let context = NativeWriteContext {
                model,
                project_root: &request.project_root,
            };
            let (mut write, rationale) = super::design_clipboard_receipt::prepare(
                &context,
                WriteProvenance::new(
                    "datum-gui",
                    CommitSource::Manual,
                    super::design_clipboard_receipt::REASON,
                ),
                id,
                *sheet,
                instance_path,
                edit,
            )?;
            write.batch.batch_id = id;
            create_draft_proposal_from_batch(
                model,
                &request.project_root,
                ProposalCreateRequest {
                    proposal_id: Some(id),
                    batch: write.batch,
                    rationale,
                    source: ProposalSource::Manual,
                    checks_run: vec![],
                    finding_fingerprints: vec![],
                },
            )?;
        }
        NativeProposalAction::CreateProjectAnnotation { .. } => {
            let mut write = super::schematic_project_annotation::build(model)?;
            write.batch.batch_id = id;
            create_draft_proposal_from_batch(model, &request.project_root, ProposalCreateRequest {
                proposal_id: Some(id), batch: write.batch,
                rationale: "Review complete project reference annotation across every supported root sheet".into(),
                source: ProposalSource::Manual, checks_run: vec![], finding_fingerprints: vec![],
            })?;
        }
        NativeProposalAction::CreateInterfaceBinding(params) => {
            let write = super::schematic_interface_binding::build(model, params)?;
            create_draft_proposal_from_batch(model, &request.project_root, ProposalCreateRequest {
                proposal_id: Some(id), batch: write.batch,
                rationale: "Review one explicit parent Port to child Hierarchical Label membership change".into(),
                source: ProposalSource::Manual, checks_run: vec![], finding_fingerprints: vec![],
            })?;
        }
        NativeProposalAction::CreateChildSheet(params) => {
            super::schematic_child_sheet::check_new_paths(&request.project_root, params)?;
            let write = super::schematic_child_sheet::build(model, params)?;
            create_draft_proposal_from_batch(model,&request.project_root,ProposalCreateRequest {
                proposal_id:Some(id),batch:write.batch,rationale:"Create a new blank child source, definition and placed instance as one operation".into(),
                source:ProposalSource::Manual,checks_run:vec![],finding_fingerprints:vec![],
            })?;
        }
        NativeProposalAction::CreateGatePlacement(params) => {
            let mut write = super::schematic_gate_placement::build(model, params)?;
            write.batch.batch_id = id;
            create_draft_proposal_from_batch(
                model,
                &request.project_root,
                ProposalCreateRequest {
                    proposal_id: Some(id),
                    batch: write.batch,
                    rationale: "Place explicitly selected unused Gate in its existing component"
                        .into(),
                    source: ProposalSource::Manual,
                    checks_run: vec![],
                    finding_fingerprints: vec![],
                },
            )?;
        }
        NativeProposalAction::CreateFieldBatch {
            sheet,
            instance_path,
            edits,
            ..
        } => {
            let mut write =
                super::schematic_field_batch::build(model, *sheet, instance_path, edits)?;
            write.batch.batch_id = id;
            create_draft_proposal_from_batch(
                model,
                &request.project_root,
                ProposalCreateRequest {
                    proposal_id: Some(id),
                    batch: write.batch,
                    rationale: "Review all Reference and Value changes as one atomic batch".into(),
                    source: ProposalSource::Manual,
                    checks_run: vec![],
                    finding_fingerprints: vec![],
                },
            )?;
        }
        NativeProposalAction::CreateAnnotation {
            sheet,
            instance_path,
            ..
        } => {
            let mut write = super::schematic_annotation::build(model, *sheet, instance_path)?;
            write.batch.batch_id = id;
            create_draft_proposal_from_batch(
                model,
                &request.project_root,
                ProposalCreateRequest {
                    proposal_id: Some(id),
                    batch: write.batch,
                    rationale:
                        "Review deterministic reference annotation for the complete active sheet"
                            .into(),
                    source: ProposalSource::Manual,
                    checks_run: vec![],
                    finding_fingerprints: vec![],
                },
            )?;
        }

        NativeProposalAction::CreateAssignment {
            symbol,
            instance_path,
            part,
            gate,
            component,
            ..
        } => {
            let provenance =
                WriteProvenance::new("datum-gui", CommitSource::Manual, "Assign schematic Part");
            let mut write = super::schematic_assignment::build_configured(
                model,
                provenance,
                *symbol,
                instance_path,
                *part,
                *gate,
                component.as_ref(),
            )?;
            write.batch.batch_id = id;
            create_draft_proposal_from_batch(
                model,
                &request.project_root,
                ProposalCreateRequest {
                    proposal_id: Some(id),
                    batch: write.batch,
                    rationale: "Assign compatible Part and Gate to selected Symbol".into(),
                    source: ProposalSource::Manual,
                    checks_run: vec![],
                    finding_fingerprints: vec![],
                },
            )?;
        }
        NativeProposalAction::CreateDisposition {
            check_run_id,
            fingerprint,
            disposition,
            rationale,
            ..
        } => {
            let provenance =
                WriteProvenance::new("datum-gui", CommitSource::Manual, "GUI ERC disposition");
            let mut prepared = super::schematic_disposition::build(
                model,
                provenance,
                *check_run_id,
                fingerprint,
                *disposition,
                rationale,
            )?;
            prepared.batch.batch_id = id;
            create_draft_proposal_from_batch(
                model,
                &request.project_root,
                ProposalCreateRequest {
                    proposal_id: Some(id),
                    batch: prepared.batch,
                    rationale: rationale.clone(),
                    source: ProposalSource::Manual,
                    checks_run: vec![*check_run_id],
                    finding_fingerprints: vec![fingerprint.clone()],
                },
            )?;
        }
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
            let proposal = authority::native_proposal(model, id)?;
            if matches!(
                proposal.status,
                ProposalStatus::Draft | ProposalStatus::Accepted
            ) {
                authority::validate_disposition_current(model, proposal, &request.project_root)?;
                preview_proposal_diff_journaled(model, &request.project_root, id)?;
            }
        }
        NativeProposalAction::AcceptApply { .. } => {
            let proposal = authority::native_proposal(model, id)?;
            if super::schematic_child_sheet::is_child_sheet(proposal) {
                super::schematic_child_sheet::check_destinations(proposal, &request.project_root)?;
            }
            authority::validate_disposition_current(model, proposal, &request.project_root)?;
            let status = proposal.status;
            if status == ProposalStatus::Draft {
                review_proposal_status(model, &request.project_root, id, ProposalStatus::Accepted)?;
            } else if status != ProposalStatus::Accepted {
                return Err(invalid("Only a draft or accepted proposal can be applied"));
            }
            // The existing gateway puts Applied metadata IN the design batch.
            // Nothing is journaled after this: one Undo restores the authored design.
            transaction_id = Some(
                apply_accepted_proposal(model, &request.project_root, id)?
                    .transaction
                    .transaction_id,
            );
        }
        NativeProposalAction::Reject { .. } => {
            authority::native_proposal(model, id)?;
            review_proposal_status(model, &request.project_root, id, ProposalStatus::Rejected)?;
        }
    }
    let proposal = authority::native_proposal(model, id)?;
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
        board_field_revision: if super::schematic_board_transfer::field_revision::is_revision(
            proposal,
        ) {
            let detail =
                super::schematic_board_transfer::field_revision::summary(model, proposal, false);
            if proposal.status == ProposalStatus::Rejected {
                detail.ok()
            } else {
                Some(detail?)
            }
        } else {
            None
        },
        interface_binding: if super::schematic_interface_binding::is_interface_binding(proposal) {
            let detail = super::schematic_interface_binding::summary(model, proposal, false);
            if proposal.status == ProposalStatus::Rejected {
                detail.ok()
            } else {
                Some(detail?)
            }
        } else {
            None
        },
        child_sheet: if super::schematic_child_sheet::is_child_sheet(proposal) {
            let detail = super::schematic_child_sheet::summary(model, proposal, false);
            if proposal.status == ProposalStatus::Rejected {
                detail.ok()
            } else {
                Some(detail?)
            }
        } else {
            None
        },
        gate_placement: if super::schematic_gate_placement::is_gate_placement(proposal) {
            let detail = super::schematic_gate_placement::summary(model, proposal, false);
            if proposal.status == ProposalStatus::Rejected {
                detail.ok()
            } else {
                Some(detail?)
            }
        } else {
            None
        },
        project_annotation: if super::schematic_project_annotation::is_project_annotation(proposal)
        {
            let detail = super::schematic_project_annotation::summary(model, proposal, false);
            if proposal.status == ProposalStatus::Rejected {
                detail.ok()
            } else {
                Some(detail?)
            }
        } else {
            None
        },
        annotation: if super::schematic_annotation::is_annotation(proposal) {
            let detail = super::schematic_annotation::summary(model, proposal, false);
            if proposal.status == ProposalStatus::Rejected {
                detail.ok()
            } else {
                Some(detail?)
            }
        } else {
            None
        },
        fields: if super::schematic_field_batch::is_field_batch(proposal) {
            let bound_current = proposal.status == ProposalStatus::Draft
                && proposal.batch.operations.iter().any(|op| matches!(op,
                    Operation::SetComponentInstance { previous_component_instance, .. }
                        if previous_component_instance["placed_package_refs"].as_array().is_some_and(|refs| !refs.is_empty())));
            let detail = super::schematic_field_batch::summary(model, proposal, bound_current);
            if proposal.status == ProposalStatus::Rejected {
                detail.ok()
            } else {
                Some(detail?)
            }
        } else {
            None
        },
        disposition: if super::schematic_disposition::is_disposition(proposal) {
            Some(super::schematic_disposition::summary(
                model, proposal, false,
            )?)
        } else {
            None
        },
        assignment: if super::schematic_assignment::is_assignment(proposal) {
            let detail = super::schematic_assignment::summary(model, proposal, false);
            if proposal.status == ProposalStatus::Rejected {
                detail.ok()
            } else {
                Some(detail?)
            }
        } else {
            None
        },
        transaction_id: transaction_id.or(proposal.applied_transaction_id),
    })
}

fn invalid(reason: &str) -> EngineError {
    EngineError::Validation(reason.into())
}

#[cfg(test)]
#[path = "schematic_delete_proposal_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "schematic_bound_delete_tests.rs"]
pub(crate) mod bound_tests;
