use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError,
    RoutePathCandidateOrthogonalDoglegReport, RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_dogleg_selection::{
    DoglegCornerOrder, candidate_orthogonal_doglegs, selected_orthogonal_dogleg,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalDoglegExplainKind {
    DeterministicPathFound,
    NoSameLayerDoglegCandidate,
    AllDoglegCandidatesBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalDoglegCornerOrderView {
    HorizontalThenVertical,
    VerticalThenHorizontal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalDoglegExplainSelectedDogleg {
    pub layer: LayerId,
    pub corner: Point,
    pub corner_order: RoutePathCandidateOrthogonalDoglegCornerOrderView,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalDoglegExplainBlockedDogleg {
    pub layer: LayerId,
    pub corner: Point,
    pub corner_order: RoutePathCandidateOrthogonalDoglegCornerOrderView,
    pub blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalDoglegExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_corner_count: usize,
    pub blocked_corner_count: usize,
    pub available_corner_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalDoglegExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateOrthogonalDoglegExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalDoglegExplainSummary,
    pub selected_dogleg: Option<RoutePathCandidateOrthogonalDoglegExplainSelectedDogleg>,
    pub blocked_doglegs: Vec<RoutePathCandidateOrthogonalDoglegExplainBlockedDogleg>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_dogleg_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalDoglegExplainReport, RoutePathCandidateError> {
        let path_candidate = self.route_path_candidate_orthogonal_dogleg(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;
        let preflight = self
            .route_preflight(net_uuid)
            .ok_or(RoutePathCandidateError::NetNotFound { net_uuid })?;
        let from_anchor = preflight
            .anchors
            .iter()
            .find(|anchor| anchor.pad_uuid == from_anchor_pad_uuid)
            .ok_or(RoutePathCandidateError::AnchorNotOnNet {
                pad_uuid: from_anchor_pad_uuid,
                net_uuid,
            })?;
        let to_anchor = preflight
            .anchors
            .iter()
            .find(|anchor| anchor.pad_uuid == to_anchor_pad_uuid)
            .ok_or(RoutePathCandidateError::AnchorNotOnNet {
                pad_uuid: to_anchor_pad_uuid,
                net_uuid,
            })?;
        let candidate_copper_layers = preflight
            .candidate_copper_layers
            .iter()
            .filter(|layer| layer.id == from_anchor.layer && layer.id == to_anchor.layer)
            .cloned()
            .collect::<Vec<_>>();
        let candidates = candidate_orthogonal_doglegs(
            self,
            net_uuid,
            from_anchor,
            to_anchor,
            &candidate_copper_layers,
        );
        let selected_dogleg = selected_orthogonal_dogleg(&candidates).map(|candidate| {
            RoutePathCandidateOrthogonalDoglegExplainSelectedDogleg {
                layer: candidate.layer,
                corner: candidate.corner,
                corner_order: render_corner_order(candidate.corner_order),
                selection_reason: format!(
                    "selected because it is the first unblocked same-layer orthogonal dogleg under the deterministic selection rule on layer {}",
                    candidate.layer
                ),
            }
        });
        let blocked_doglegs = candidates
            .iter()
            .filter(|candidate| candidate.blocked)
            .map(
                |candidate| RoutePathCandidateOrthogonalDoglegExplainBlockedDogleg {
                    layer: candidate.layer,
                    corner: candidate.corner,
                    corner_order: render_corner_order(candidate.corner_order),
                    blockages: candidate.blockages.clone(),
                },
            )
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateOrthogonalDoglegExplainReport {
            contract: "m5_route_path_candidate_orthogonal_dogleg_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateOrthogonalDoglegExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_corner_count: path_candidate.summary.candidate_corner_count,
                blocked_corner_count: path_candidate.summary.blocked_corner_count,
                available_corner_count: path_candidate.summary.available_corner_count,
            },
            selected_dogleg,
            blocked_doglegs,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateOrthogonalDoglegReport,
) -> RoutePathCandidateOrthogonalDoglegExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateOrthogonalDoglegExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.candidate_corner_count == 0 =>
        {
            RoutePathCandidateOrthogonalDoglegExplainKind::NoSameLayerDoglegCandidate
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateOrthogonalDoglegExplainKind::AllDoglegCandidatesBlocked
        }
    }
}

fn render_corner_order(
    corner_order: DoglegCornerOrder,
) -> RoutePathCandidateOrthogonalDoglegCornerOrderView {
    match corner_order {
        DoglegCornerOrder::HorizontalThenVertical => {
            RoutePathCandidateOrthogonalDoglegCornerOrderView::HorizontalThenVertical
        }
        DoglegCornerOrder::VerticalThenHorizontal => {
            RoutePathCandidateOrthogonalDoglegCornerOrderView::VerticalThenHorizontal
        }
    }
}
