use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_dogleg_selection::{
    DoglegCornerOrder, ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG_SELECTION_RULE,
    candidate_orthogonal_doglegs, selected_orthogonal_dogleg,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalDoglegCornerOrderView {
    HorizontalThenVertical,
    VerticalThenHorizontal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalDoglegPath {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub corner: Point,
    pub corner_order: RoutePathCandidateOrthogonalDoglegCornerOrderView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalDoglegSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_corner_count: usize,
    pub blocked_corner_count: usize,
    pub available_corner_count: usize,
    pub path_point_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalDoglegReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalDoglegSummary,
    pub path: Option<RoutePathCandidateOrthogonalDoglegPath>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_dogleg(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalDoglegReport, RoutePathCandidateError> {
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
        let candidate_doglegs = candidate_orthogonal_doglegs(
            self,
            net_uuid,
            from_anchor,
            to_anchor,
            &candidate_copper_layers,
        );
        let blocked_corner_count = candidate_doglegs.iter().filter(|dogleg| dogleg.blocked).count();
        let available_corner_count = candidate_doglegs.len().saturating_sub(blocked_corner_count);
        let selected = selected_orthogonal_dogleg(&candidate_doglegs);
        let path = selected.map(|dogleg| RoutePathCandidateOrthogonalDoglegPath {
            layer: dogleg.layer,
            points: vec![from_anchor.position, dogleg.corner, to_anchor.position],
            corner: dogleg.corner,
            corner_order: match dogleg.corner_order {
                DoglegCornerOrder::HorizontalThenVertical => {
                    RoutePathCandidateOrthogonalDoglegCornerOrderView::HorizontalThenVertical
                }
                DoglegCornerOrder::VerticalThenHorizontal => {
                    RoutePathCandidateOrthogonalDoglegCornerOrderView::VerticalThenHorizontal
                }
            },
        });
        let path_point_count = path.as_ref().map(|entry| entry.points.len()).unwrap_or(0);
        let status = if path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateOrthogonalDoglegReport {
            contract: "m5_route_path_candidate_orthogonal_dogleg_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG_SELECTION_RULE.to_string(),
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers,
            summary: RoutePathCandidateOrthogonalDoglegSummary {
                candidate_copper_layer_count: preflight
                    .candidate_copper_layers
                    .iter()
                    .filter(|layer| layer.id == from_anchor.layer && layer.id == to_anchor.layer)
                    .count(),
                candidate_corner_count: candidate_doglegs.len(),
                blocked_corner_count,
                available_corner_count,
                path_point_count,
            },
            path,
        })
    }
}
