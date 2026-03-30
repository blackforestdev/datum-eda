use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError,
    RoutePathCandidateOrthogonalTwoBendReport, RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_two_bend_selection::{
    OrthogonalTwoBendOrientation, candidate_orthogonal_two_bend_paths,
    selected_orthogonal_two_bend_path,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalTwoBendExplainKind {
    DeterministicPathFound,
    NoSameLayerTwoBendCandidate,
    AllTwoBendCandidatesBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalTwoBendOrientationView {
    HorizontalDetour,
    VerticalDetour,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalTwoBendExplainSelectedPath {
    pub layer: LayerId,
    pub orientation: RoutePathCandidateOrthogonalTwoBendOrientationView,
    pub detour_coordinate: i64,
    pub points: Vec<Point>,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalTwoBendExplainBlockedPath {
    pub layer: LayerId,
    pub orientation: RoutePathCandidateOrthogonalTwoBendOrientationView,
    pub detour_coordinate: i64,
    pub points: Vec<Point>,
    pub blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalTwoBendExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_path_count: usize,
    pub blocked_path_count: usize,
    pub available_path_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalTwoBendExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateOrthogonalTwoBendExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalTwoBendExplainSummary,
    pub selected_path: Option<RoutePathCandidateOrthogonalTwoBendExplainSelectedPath>,
    pub blocked_paths: Vec<RoutePathCandidateOrthogonalTwoBendExplainBlockedPath>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_two_bend_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalTwoBendExplainReport, RoutePathCandidateError> {
        let path_candidate = self.route_path_candidate_orthogonal_two_bend(
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
        let candidates = candidate_orthogonal_two_bend_paths(
            self,
            net_uuid,
            from_anchor,
            to_anchor,
            &candidate_copper_layers,
        );
        let selected_path = selected_orthogonal_two_bend_path(&candidates).map(|candidate| {
            RoutePathCandidateOrthogonalTwoBendExplainSelectedPath {
                layer: candidate.layer,
                orientation: render_orientation(candidate.orientation),
                detour_coordinate: candidate.detour_coordinate,
                points: candidate.points.clone(),
                selection_reason: format!(
                    "selected because it is the first unblocked same-layer orthogonal two-bend path under the deterministic selection rule on layer {}",
                    candidate.layer
                ),
            }
        });
        let blocked_paths = candidates
            .iter()
            .filter(|candidate| candidate.blocked)
            .map(|candidate| RoutePathCandidateOrthogonalTwoBendExplainBlockedPath {
                layer: candidate.layer,
                orientation: render_orientation(candidate.orientation),
                detour_coordinate: candidate.detour_coordinate,
                points: candidate.points.clone(),
                blockages: candidate.blockages.clone(),
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateOrthogonalTwoBendExplainReport {
            contract: "m5_route_path_candidate_orthogonal_two_bend_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateOrthogonalTwoBendExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_path_count: path_candidate.summary.candidate_path_count,
                blocked_path_count: path_candidate.summary.blocked_path_count,
                available_path_count: path_candidate.summary.available_path_count,
            },
            selected_path,
            blocked_paths,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateOrthogonalTwoBendReport,
) -> RoutePathCandidateOrthogonalTwoBendExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateOrthogonalTwoBendExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.candidate_path_count == 0 =>
        {
            RoutePathCandidateOrthogonalTwoBendExplainKind::NoSameLayerTwoBendCandidate
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateOrthogonalTwoBendExplainKind::AllTwoBendCandidatesBlocked
        }
    }
}

fn render_orientation(
    orientation: OrthogonalTwoBendOrientation,
) -> RoutePathCandidateOrthogonalTwoBendOrientationView {
    match orientation {
        OrthogonalTwoBendOrientation::HorizontalDetour => {
            RoutePathCandidateOrthogonalTwoBendOrientationView::HorizontalDetour
        }
        OrthogonalTwoBendOrientation::VerticalDetour => {
            RoutePathCandidateOrthogonalTwoBendOrientationView::VerticalDetour
        }
    }
}
