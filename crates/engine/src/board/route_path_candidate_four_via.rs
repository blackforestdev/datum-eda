use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_four_via_selection::{
    ROUTE_PATH_CANDIDATE_FOUR_VIA_SELECTION_RULE, candidate_four_via_matches, four_via_path_points,
    selected_matching_four_via,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFourViaSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFourViaPath {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub via_c_uuid: Uuid,
    pub via_c_position: Point,
    pub via_d_uuid: Uuid,
    pub via_d_position: Point,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub third_intermediate_layer: LayerId,
    pub segments: Vec<RoutePathCandidateFourViaSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFourViaSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub candidate_via_quadruple_count: usize,
    pub matching_via_quadruple_count: usize,
    pub blocked_via_quadruple_count: usize,
    pub available_via_quadruple_count: usize,
    pub path_segment_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFourViaReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateFourViaSummary,
    pub path: Option<RoutePathCandidateFourViaPath>,
}

impl Board {
    pub fn route_path_candidate_four_via(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateFourViaReport, RoutePathCandidateError> {
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

        let (candidate_vias, matching_quadruples) =
            candidate_four_via_matches(self, net_uuid, from_anchor, to_anchor);
        let blocked_via_quadruple_count = matching_quadruples
            .iter()
            .filter(|entry| {
                !(entry.source_segment.blockages.is_empty()
                    && entry.first_middle_segment.blockages.is_empty()
                    && entry.second_middle_segment.blockages.is_empty()
                    && entry.third_middle_segment.blockages.is_empty()
                    && entry.target_segment.blockages.is_empty())
            })
            .count();
        let available_via_quadruple_count = matching_quadruples
            .len()
            .saturating_sub(blocked_via_quadruple_count);
        let selected_path = selected_matching_four_via(&matching_quadruples).map(|entry| {
            let points = four_via_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateFourViaPath {
                via_a_uuid: entry.via_a.uuid,
                via_a_position: entry.via_a.position,
                via_b_uuid: entry.via_b.uuid,
                via_b_position: entry.via_b.position,
                via_c_uuid: entry.via_c.uuid,
                via_c_position: entry.via_c.position,
                via_d_uuid: entry.via_d.uuid,
                via_d_position: entry.via_d.position,
                first_intermediate_layer: entry.first_intermediate_layer,
                second_intermediate_layer: entry.second_intermediate_layer,
                third_intermediate_layer: entry.third_intermediate_layer,
                segments: vec![
                    RoutePathCandidateFourViaSegment {
                        layer: from_anchor.layer,
                        points: points[0].to_vec(),
                    },
                    RoutePathCandidateFourViaSegment {
                        layer: entry.first_intermediate_layer,
                        points: points[1].to_vec(),
                    },
                    RoutePathCandidateFourViaSegment {
                        layer: entry.second_intermediate_layer,
                        points: points[2].to_vec(),
                    },
                    RoutePathCandidateFourViaSegment {
                        layer: entry.third_intermediate_layer,
                        points: points[3].to_vec(),
                    },
                    RoutePathCandidateFourViaSegment {
                        layer: to_anchor.layer,
                        points: points[4].to_vec(),
                    },
                ],
            }
        });
        let status = if selected_path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateFourViaReport {
            contract: "m5_route_path_candidate_four_via_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: ROUTE_PATH_CANDIDATE_FOUR_VIA_SELECTION_RULE.to_string(),
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers: preflight.candidate_copper_layers.clone(),
            summary: RoutePathCandidateFourViaSummary {
                candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
                candidate_via_count: candidate_vias.len(),
                candidate_via_quadruple_count: candidate_vias
                    .len()
                    .saturating_mul(candidate_vias.len().saturating_sub(1))
                    .saturating_mul(candidate_vias.len().saturating_sub(2))
                    .saturating_mul(candidate_vias.len().saturating_sub(3)),
                matching_via_quadruple_count: matching_quadruples.len(),
                blocked_via_quadruple_count,
                available_via_quadruple_count,
                path_segment_count: selected_path
                    .as_ref()
                    .map(|path| path.segments.len())
                    .unwrap_or(0),
            },
            path: selected_path,
        })
    }
}
