use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError, RoutePathCandidateFourViaReport,
    RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_four_via_selection::{
    candidate_four_via_matches, four_via_path_points, selected_matching_four_via,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateFourViaExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredViaQuadruple,
    AllMatchingViaQuadruplesBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFourViaExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFourViaExplainBlockedQuadruple {
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
    pub source_segment: RoutePathCandidateFourViaExplainSegment,
    pub first_middle_segment: RoutePathCandidateFourViaExplainSegment,
    pub second_middle_segment: RoutePathCandidateFourViaExplainSegment,
    pub third_middle_segment: RoutePathCandidateFourViaExplainSegment,
    pub target_segment: RoutePathCandidateFourViaExplainSegment,
    pub source_blockages: Vec<RouteCorridorSpanBlockage>,
    pub first_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub second_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub third_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub target_blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFourViaExplainSelectedQuadruple {
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
    pub source_segment: RoutePathCandidateFourViaExplainSegment,
    pub first_middle_segment: RoutePathCandidateFourViaExplainSegment,
    pub second_middle_segment: RoutePathCandidateFourViaExplainSegment,
    pub third_middle_segment: RoutePathCandidateFourViaExplainSegment,
    pub target_segment: RoutePathCandidateFourViaExplainSegment,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFourViaExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub candidate_via_quadruple_count: usize,
    pub matching_via_quadruple_count: usize,
    pub blocked_via_quadruple_count: usize,
    pub available_via_quadruple_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFourViaExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateFourViaExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateFourViaExplainSummary,
    pub selected_quadruple: Option<RoutePathCandidateFourViaExplainSelectedQuadruple>,
    pub blocked_matching_quadruples: Vec<RoutePathCandidateFourViaExplainBlockedQuadruple>,
}

impl Board {
    pub fn route_path_candidate_four_via_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateFourViaExplainReport, RoutePathCandidateError> {
        let path_candidate =
            self.route_path_candidate_four_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)?;
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

        let (_, matching_quadruples) =
            candidate_four_via_matches(self, net_uuid, from_anchor, to_anchor);
        let selected_quadruple = selected_matching_four_via(&matching_quadruples).map(|entry| {
            let points = four_via_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateFourViaExplainSelectedQuadruple {
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
                source_segment: RoutePathCandidateFourViaExplainSegment {
                    layer: from_anchor.layer,
                    points: points[0].to_vec(),
                },
                first_middle_segment: RoutePathCandidateFourViaExplainSegment {
                    layer: entry.first_intermediate_layer,
                    points: points[1].to_vec(),
                },
                second_middle_segment: RoutePathCandidateFourViaExplainSegment {
                    layer: entry.second_intermediate_layer,
                    points: points[2].to_vec(),
                },
                third_middle_segment: RoutePathCandidateFourViaExplainSegment {
                    layer: entry.third_intermediate_layer,
                    points: points[3].to_vec(),
                },
                target_segment: RoutePathCandidateFourViaExplainSegment {
                    layer: to_anchor.layer,
                    points: points[4].to_vec(),
                },
                selection_reason: format!(
                    "selected because it is the first unblocked matching authored via quadruple under the deterministic selection rule through intermediate layers {}, {}, then {}",
                    entry.first_intermediate_layer,
                    entry.second_intermediate_layer,
                    entry.third_intermediate_layer
                ),
            }
        });

        let blocked_matching_quadruples = matching_quadruples
            .iter()
            .filter(|entry| {
                !(entry.source_segment.blockages.is_empty()
                    && entry.first_middle_segment.blockages.is_empty()
                    && entry.second_middle_segment.blockages.is_empty()
                    && entry.third_middle_segment.blockages.is_empty()
                    && entry.target_segment.blockages.is_empty())
            })
            .map(|entry| {
                let points = four_via_path_points(entry, from_anchor, to_anchor);
                RoutePathCandidateFourViaExplainBlockedQuadruple {
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
                    source_segment: RoutePathCandidateFourViaExplainSegment {
                        layer: from_anchor.layer,
                        points: points[0].to_vec(),
                    },
                    first_middle_segment: RoutePathCandidateFourViaExplainSegment {
                        layer: entry.first_intermediate_layer,
                        points: points[1].to_vec(),
                    },
                    second_middle_segment: RoutePathCandidateFourViaExplainSegment {
                        layer: entry.second_intermediate_layer,
                        points: points[2].to_vec(),
                    },
                    third_middle_segment: RoutePathCandidateFourViaExplainSegment {
                        layer: entry.third_intermediate_layer,
                        points: points[3].to_vec(),
                    },
                    target_segment: RoutePathCandidateFourViaExplainSegment {
                        layer: to_anchor.layer,
                        points: points[4].to_vec(),
                    },
                    source_blockages: entry.source_segment.blockages.clone(),
                    first_middle_blockages: entry.first_middle_segment.blockages.clone(),
                    second_middle_blockages: entry.second_middle_segment.blockages.clone(),
                    third_middle_blockages: entry.third_middle_segment.blockages.clone(),
                    target_blockages: entry.target_segment.blockages.clone(),
                }
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateFourViaExplainReport {
            contract: "m5_route_path_candidate_four_via_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateFourViaExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
                candidate_via_quadruple_count: path_candidate.summary.candidate_via_quadruple_count,
                matching_via_quadruple_count: path_candidate.summary.matching_via_quadruple_count,
                blocked_via_quadruple_count: path_candidate.summary.blocked_via_quadruple_count,
                available_via_quadruple_count: path_candidate.summary.available_via_quadruple_count,
            },
            selected_quadruple,
            blocked_matching_quadruples,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateFourViaReport,
) -> RoutePathCandidateFourViaExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateFourViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_via_quadruple_count == 0 =>
        {
            RoutePathCandidateFourViaExplainKind::NoMatchingAuthoredViaQuadruple
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateFourViaExplainKind::AllMatchingViaQuadruplesBlocked
        }
    }
}
