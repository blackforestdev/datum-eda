use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_via_selection::{
    ROUTE_PATH_CANDIDATE_VIA_SELECTION_RULE, candidate_vias_for_net, matching_via_analyses,
    selected_matching_via, via_path_points,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateViaSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateViaPath {
    pub via_uuid: Uuid,
    pub via_position: Point,
    pub segments: Vec<RoutePathCandidateViaSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateViaSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub matching_via_count: usize,
    pub blocked_via_count: usize,
    pub available_via_count: usize,
    pub path_segment_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateViaReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateViaSummary,
    pub path: Option<RoutePathCandidateViaPath>,
}

impl Board {
    pub fn route_path_candidate_via(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateViaReport, RoutePathCandidateError> {
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

        let candidate_vias = candidate_vias_for_net(self, net_uuid);
        let matching_vias =
            matching_via_analyses(self, net_uuid, from_anchor, to_anchor, &candidate_vias);
        let blocked_via_count = matching_vias
            .iter()
            .filter(|entry| {
                !(entry.source_segment.blockages.is_empty() && entry.target_segment.blockages.is_empty())
            })
            .count();
        let available_via_count = matching_vias.len().saturating_sub(blocked_via_count);
        let selected_path = selected_matching_via(&matching_vias).map(|entry| {
            let points = via_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateViaPath {
                via_uuid: entry.via.uuid,
                via_position: entry.via.position,
                segments: vec![
                    RoutePathCandidateViaSegment {
                        layer: from_anchor.layer,
                        points: points[0].to_vec(),
                    },
                    RoutePathCandidateViaSegment {
                        layer: to_anchor.layer,
                        points: points[1].to_vec(),
                    },
                ],
            }
        });
        let status = if selected_path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateViaReport {
            contract: "m5_route_path_candidate_via_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: ROUTE_PATH_CANDIDATE_VIA_SELECTION_RULE.to_string(),
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers: preflight.candidate_copper_layers.clone(),
            summary: RoutePathCandidateViaSummary {
                candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
                candidate_via_count: candidate_vias.len(),
                matching_via_count: matching_vias.len(),
                blocked_via_count,
                available_via_count,
                path_segment_count: selected_path
                    .as_ref()
                    .map(|path| path.segments.len())
                    .unwrap_or(0),
            },
            path: selected_path,
        })
    }
}
