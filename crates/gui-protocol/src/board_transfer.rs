//! Transient review over the shared engine initial-transfer planner.
use eda_engine::api::native_write::schematic_board_transfer::{TransferPlan, TransferRequest};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardTransferReview {
    pub request: TransferRequest,
    pub plan: TransferPlan,
    pub first_row: usize,
    pub field_changes: Result<usize, String>,
    pub connectivity_changes: Result<usize, String>,
    pub refusal: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardTransferAction {
    Resolve(usize),
    Resolution(crate::library_browser::component_resolution::ResolutionAction),
    Surface,
    ChooseMap(usize),
    Previous,
    Next,
    PrepareSources,
    ReviewFields,
    ReviewConnectivity,
    Accept,
    Cancel,
}

use crate::schematic_authoring::{delete_review::DeleteReview, review_layout};
use crate::{AuthoringViewBasis, ReviewWorkspaceState};
use eda_engine::api::native_write::{
    schematic_board_transfer::field_revision::{self, BoardFieldRevision},
    schematic_delete_proposal::NativeProposalAction,
};
use eda_engine::substrate::{DesignModel, SourceShardKind};

/// Frozen presentation context from the same qualified model as the engine diff.
#[derive(Debug, Clone, PartialEq)]
pub struct BoardFieldRevisionEvidence {
    pub summary: BoardFieldRevision,
    pub contexts: Vec<String>,
}
pub const FIELD_ACCEPT_HINT: &str = "Apply changes [Tab] · Dismiss [Esc]";
impl DeleteReview {
    pub fn capture_board_field_revision(state: &ReviewWorkspaceState) -> Result<Self, String> {
        let basis =
            AuthoringViewBasis::capture_unique_root_schematic(state, state.ui.layout.focused)?;
        let model = state
            .library
            .basis
            .as_ref()
            .ok_or("Native model unavailable")?;
        if model.model_revision.0 != basis.revision || model.project.project_id != basis.project {
            return Err("Design changed; reopen the PCB update review".into());
        }
        let summary =
            field_revision::preview(model, basis.source, &[]).map_err(|error| error.to_string())?;
        if summary.changes.is_empty() {
            return Err("PCB references and values already match the schematic".into());
        }
        let mut review = Self::capture_source_basis(state, basis)?;
        review.pending = Some(NativeProposalAction::CreateBoardFieldRevision {
            proposal_id: review.proposal_id,
            sheet: summary.sheet,
            review: Some(summary.clone()),
        });
        let board = model
            .materialized_source_shard_value(SourceShardKind::BoardRoot)
            .map_err(|error| error.to_string())?;
        let mut contexts = Vec::new();
        for change in &summary.changes {
            let package = &board["packages"][change.package.to_string()];
            let part: uuid::Uuid = serde_json::from_value(package["part"].clone())
                .map_err(|error| error.to_string())?;
            let pads: Vec<eda_engine::substrate::component_pad_source::ComponentPadSource> =
                serde_json::from_value(board["component_pads"][change.package.to_string()].clone())
                    .map_err(|error| error.to_string())?;
            let footprints = pads
                .iter()
                .map(|pad| {
                    pad.source_binding
                        .as_ref()
                        .map(|binding| binding.land_pattern.object_id)
                        .ok_or("Pinned footprint source unavailable")
                })
                .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
            if footprints.len() != 1 {
                return Err("Pinned footprint source is ambiguous".into());
            }
            let footprint = *footprints.first().unwrap();
            contexts.push(format!(
                "Part: {} · Footprint: {} — same placed component (read-only)",
                source_label(model, part, "parts"),
                source_label(model, footprint, "footprints")
            ));
        }
        review.board_field_revision = Some(BoardFieldRevisionEvidence { summary, contexts });
        Ok(review)
    }
}
pub(crate) fn source_label(model: &DesignModel, id: uuid::Uuid, kind: &str) -> String {
    let name = model.objects.get(&id).and_then(|object| {
        let shard = model
            .source_shards
            .iter()
            .find(|shard| shard.shard_id == object.source_shard_id)?;
        let value = model
            .materialized_source_shard_value_by_relative_path(&shard.relative_path)
            .ok()?;
        (value["uuid"].as_str() == Some(id.to_string().as_str()))
            .then(|| crate::library_browser::display_identity(kind, &value))
            .flatten()
    });
    name.unwrap_or_else(|| id.to_string())
}
impl BoardFieldRevisionEvidence {
    pub fn layout(
        &self,
        refusal: Option<&str>,
        width: f32,
        height: f32,
        bottom_clearance: f32,
        label_widths: (f32, f32),
        measure_height: impl FnMut(&str, f32) -> Result<f32, String>,
    ) -> Result<review_layout::ProposalReviewLayout, String> {
        let mut rows = Vec::new();
        for (change, context) in self.summary.changes.iter().zip(&self.contexts) {
            rows.push(context.clone());
            rows.push(format!("Component: {}", change.component_instance));
            rows.push(format!(
                "Reference: old {} → new {}{}",
                change.previous_reference,
                change.reference,
                if change.previous_reference == change.reference {
                    " (unchanged)"
                } else {
                    ""
                }
            ));
            rows.push(format!(
                "Value: old {} → new {}{}",
                change.previous_value,
                change.value,
                if change.previous_value == change.value {
                    " (unchanged)"
                } else {
                    ""
                }
            ));
        }
        rows.push("Reference and display Value only. Placement, pads, nets, routes and pinned physical choice stay unchanged.".into());
        if let Some(reason) = refusal {
            rows.push(format!("Unavailable: {reason}"));
        }
        let mut result = review_layout::measure_rows(
            &rows,
            label_widths,
            width,
            height,
            bottom_clearance,
            measure_height,
        )?;
        result.fits &= self.contexts.len() == self.summary.changes.len();
        Ok(result)
    }
}
