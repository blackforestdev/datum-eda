use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_two_via_selection::{
    ROUTE_PATH_CANDIDATE_TWO_VIA_SELECTION_RULE, candidate_two_via_matches,
    selected_matching_two_via, two_via_path_points,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateTwoViaSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateTwoViaPath {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub intermediate_layer: LayerId,
    pub segments: Vec<RoutePathCandidateTwoViaSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateTwoViaSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub candidate_via_pair_count: usize,
    pub matching_via_pair_count: usize,
    pub blocked_via_pair_count: usize,
    pub available_via_pair_count: usize,
    pub path_segment_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateTwoViaReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateTwoViaSummary,
    pub path: Option<RoutePathCandidateTwoViaPath>,
}

impl Board {
    pub fn route_path_candidate_two_via(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateTwoViaReport, RoutePathCandidateError> {
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

        let (candidate_vias, matching_pairs) =
            candidate_two_via_matches(self, net_uuid, from_anchor, to_anchor);
        let blocked_via_pair_count = matching_pairs
            .iter()
            .filter(|entry| {
                !(entry.source_segment.blockages.is_empty()
                    && entry.middle_segment.blockages.is_empty()
                    && entry.target_segment.blockages.is_empty())
            })
            .count();
        let available_via_pair_count = matching_pairs.len().saturating_sub(blocked_via_pair_count);
        let selected_path = selected_matching_two_via(&matching_pairs).map(|entry| {
            let points = two_via_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateTwoViaPath {
                via_a_uuid: entry.via_a.uuid,
                via_a_position: entry.via_a.position,
                via_b_uuid: entry.via_b.uuid,
                via_b_position: entry.via_b.position,
                intermediate_layer: entry.intermediate_layer,
                segments: vec![
                    RoutePathCandidateTwoViaSegment {
                        layer: from_anchor.layer,
                        points: points[0].to_vec(),
                    },
                    RoutePathCandidateTwoViaSegment {
                        layer: entry.intermediate_layer,
                        points: points[1].to_vec(),
                    },
                    RoutePathCandidateTwoViaSegment {
                        layer: to_anchor.layer,
                        points: points[2].to_vec(),
                    },
                ],
            }
        });
        let status = if selected_path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateTwoViaReport {
            contract: "m5_route_path_candidate_two_via_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: ROUTE_PATH_CANDIDATE_TWO_VIA_SELECTION_RULE.to_string(),
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers: preflight.candidate_copper_layers.clone(),
            summary: RoutePathCandidateTwoViaSummary {
                candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
                candidate_via_count: candidate_vias.len(),
                candidate_via_pair_count: candidate_vias
                    .len()
                    .saturating_mul(candidate_vias.len().saturating_sub(1)),
                matching_via_pair_count: matching_pairs.len(),
                blocked_via_pair_count,
                available_via_pair_count,
                path_segment_count: selected_path
                    .as_ref()
                    .map(|path| path.segments.len())
                    .unwrap_or(0),
            },
            path: selected_path,
        })
    }
}
