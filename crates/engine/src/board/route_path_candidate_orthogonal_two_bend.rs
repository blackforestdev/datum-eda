use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_two_bend_selection::{
    OrthogonalTwoBendOrientation, ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND_SELECTION_RULE,
    candidate_orthogonal_two_bend_paths, selected_orthogonal_two_bend_path,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalTwoBendOrientationView {
    HorizontalDetour,
    VerticalDetour,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalTwoBendPath {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub orientation: RoutePathCandidateOrthogonalTwoBendOrientationView,
    pub detour_coordinate: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalTwoBendSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_path_count: usize,
    pub blocked_path_count: usize,
    pub available_path_count: usize,
    pub path_point_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalTwoBendReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalTwoBendSummary,
    pub path: Option<RoutePathCandidateOrthogonalTwoBendPath>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_two_bend(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalTwoBendReport, RoutePathCandidateError> {
        if from_anchor_pad_uuid == to_anchor_pad_uuid {
            return Err(RoutePathCandidateError::DuplicateAnchorPair {
                pad_uuid: from_anchor_pad_uuid,
            });
        }

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
        let candidate_paths = candidate_orthogonal_two_bend_paths(
            self,
            net_uuid,
            from_anchor,
            to_anchor,
            &candidate_copper_layers,
        );
        let blocked_path_count = candidate_paths.iter().filter(|path| path.blocked).count();
        let available_path_count = candidate_paths.len().saturating_sub(blocked_path_count);
        let selected = selected_orthogonal_two_bend_path(&candidate_paths);
        let path = selected.map(|entry| RoutePathCandidateOrthogonalTwoBendPath {
            layer: entry.layer,
            points: entry.points.clone(),
            orientation: render_orientation(entry.orientation),
            detour_coordinate: entry.detour_coordinate,
        });
        let path_point_count = path.as_ref().map(|entry| entry.points.len()).unwrap_or(0);
        let status = if path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateOrthogonalTwoBendReport {
            contract: "m5_route_path_candidate_orthogonal_two_bend_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND_SELECTION_RULE.to_string(),
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers,
            summary: RoutePathCandidateOrthogonalTwoBendSummary {
                candidate_copper_layer_count: preflight
                    .candidate_copper_layers
                    .iter()
                    .filter(|layer| layer.id == from_anchor.layer && layer.id == to_anchor.layer)
                    .count(),
                candidate_path_count: candidate_paths.len(),
                blocked_path_count,
                available_path_count,
                path_point_count,
            },
            path,
        })
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
