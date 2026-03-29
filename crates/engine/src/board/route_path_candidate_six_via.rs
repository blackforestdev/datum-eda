use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_six_via_selection::{
    ROUTE_PATH_CANDIDATE_SIX_VIA_SELECTION_RULE, candidate_six_via_matches,
    selected_matching_six_via, six_via_path_points,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateSixViaSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateSixViaPath {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub via_c_uuid: Uuid,
    pub via_c_position: Point,
    pub via_d_uuid: Uuid,
    pub via_d_position: Point,
    pub via_e_uuid: Uuid,
    pub via_e_position: Point,
    pub via_f_uuid: Uuid,
    pub via_f_position: Point,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub third_intermediate_layer: LayerId,
    pub fourth_intermediate_layer: LayerId,
    pub fifth_intermediate_layer: LayerId,
    pub segments: Vec<RoutePathCandidateSixViaSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateSixViaSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub candidate_via_sextuple_count: usize,
    pub matching_via_sextuple_count: usize,
    pub blocked_via_sextuple_count: usize,
    pub available_via_sextuple_count: usize,
    pub path_segment_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateSixViaReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateSixViaSummary,
    pub path: Option<RoutePathCandidateSixViaPath>,
}

impl Board {
    pub fn route_path_candidate_six_via(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateSixViaReport, RoutePathCandidateError> {
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

        let (candidate_vias, matching_sextuples) =
            candidate_six_via_matches(self, net_uuid, from_anchor, to_anchor);
        let blocked_via_sextuple_count = matching_sextuples
            .iter()
            .filter(|entry| {
                !(entry.source_segment.blockages.is_empty()
                    && entry.first_middle_segment.blockages.is_empty()
                    && entry.second_middle_segment.blockages.is_empty()
                    && entry.third_middle_segment.blockages.is_empty()
                    && entry.fourth_middle_segment.blockages.is_empty()
                    && entry.fifth_middle_segment.blockages.is_empty()
                    && entry.target_segment.blockages.is_empty())
            })
            .count();
        let available_via_sextuple_count = matching_sextuples
            .len()
            .saturating_sub(blocked_via_sextuple_count);
        let selected_path = selected_matching_six_via(&matching_sextuples).map(|entry| {
            let points = six_via_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateSixViaPath {
                via_a_uuid: entry.via_a.uuid,
                via_a_position: entry.via_a.position,
                via_b_uuid: entry.via_b.uuid,
                via_b_position: entry.via_b.position,
                via_c_uuid: entry.via_c.uuid,
                via_c_position: entry.via_c.position,
                via_d_uuid: entry.via_d.uuid,
                via_d_position: entry.via_d.position,
                via_e_uuid: entry.via_e.uuid,
                via_e_position: entry.via_e.position,
                via_f_uuid: entry.via_f.uuid,
                via_f_position: entry.via_f.position,
                first_intermediate_layer: entry.first_intermediate_layer,
                second_intermediate_layer: entry.second_intermediate_layer,
                third_intermediate_layer: entry.third_intermediate_layer,
                fourth_intermediate_layer: entry.fourth_intermediate_layer,
                fifth_intermediate_layer: entry.fifth_intermediate_layer,
                segments: vec![
                    RoutePathCandidateSixViaSegment {
                        layer: from_anchor.layer,
                        points: points[0].to_vec(),
                    },
                    RoutePathCandidateSixViaSegment {
                        layer: entry.first_intermediate_layer,
                        points: points[1].to_vec(),
                    },
                    RoutePathCandidateSixViaSegment {
                        layer: entry.second_intermediate_layer,
                        points: points[2].to_vec(),
                    },
                    RoutePathCandidateSixViaSegment {
                        layer: entry.third_intermediate_layer,
                        points: points[3].to_vec(),
                    },
                    RoutePathCandidateSixViaSegment {
                        layer: entry.fourth_intermediate_layer,
                        points: points[4].to_vec(),
                    },
                    RoutePathCandidateSixViaSegment {
                        layer: entry.fifth_intermediate_layer,
                        points: points[5].to_vec(),
                    },
                    RoutePathCandidateSixViaSegment {
                        layer: to_anchor.layer,
                        points: points[6].to_vec(),
                    },
                ],
            }
        });
        let status = if selected_path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateSixViaReport {
            contract: "m5_route_path_candidate_six_via_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: ROUTE_PATH_CANDIDATE_SIX_VIA_SELECTION_RULE.to_string(),
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers: preflight.candidate_copper_layers.clone(),
            summary: RoutePathCandidateSixViaSummary {
                candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
                candidate_via_count: candidate_vias.len(),
                candidate_via_sextuple_count: candidate_vias
                    .len()
                    .saturating_mul(candidate_vias.len().saturating_sub(1))
                    .saturating_mul(candidate_vias.len().saturating_sub(2))
                    .saturating_mul(candidate_vias.len().saturating_sub(3))
                    .saturating_mul(candidate_vias.len().saturating_sub(4))
                    .saturating_mul(candidate_vias.len().saturating_sub(5)),
                matching_via_sextuple_count: matching_sextuples.len(),
                blocked_via_sextuple_count,
                available_via_sextuple_count,
                path_segment_count: selected_path
                    .as_ref()
                    .map(|path| path.segments.len())
                    .unwrap_or(0),
            },
            path: selected_path,
        })
    }
}
