use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError, RoutePathCandidateFiveViaReport,
    RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_five_via_selection::{
    candidate_five_via_matches, five_via_path_points, selected_matching_five_via,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateFiveViaExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredViaQuintuple,
    AllMatchingViaQuintuplesBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFiveViaExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFiveViaExplainBlockedQuintuple {
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
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub third_intermediate_layer: LayerId,
    pub fourth_intermediate_layer: LayerId,
    pub source_segment: RoutePathCandidateFiveViaExplainSegment,
    pub first_middle_segment: RoutePathCandidateFiveViaExplainSegment,
    pub second_middle_segment: RoutePathCandidateFiveViaExplainSegment,
    pub third_middle_segment: RoutePathCandidateFiveViaExplainSegment,
    pub fourth_middle_segment: RoutePathCandidateFiveViaExplainSegment,
    pub target_segment: RoutePathCandidateFiveViaExplainSegment,
    pub source_blockages: Vec<RouteCorridorSpanBlockage>,
    pub first_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub second_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub third_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub fourth_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub target_blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFiveViaExplainSelectedQuintuple {
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
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub third_intermediate_layer: LayerId,
    pub fourth_intermediate_layer: LayerId,
    pub source_segment: RoutePathCandidateFiveViaExplainSegment,
    pub first_middle_segment: RoutePathCandidateFiveViaExplainSegment,
    pub second_middle_segment: RoutePathCandidateFiveViaExplainSegment,
    pub third_middle_segment: RoutePathCandidateFiveViaExplainSegment,
    pub fourth_middle_segment: RoutePathCandidateFiveViaExplainSegment,
    pub target_segment: RoutePathCandidateFiveViaExplainSegment,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFiveViaExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub candidate_via_quintuple_count: usize,
    pub matching_via_quintuple_count: usize,
    pub blocked_via_quintuple_count: usize,
    pub available_via_quintuple_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateFiveViaExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateFiveViaExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateFiveViaExplainSummary,
    pub selected_quintuple: Option<RoutePathCandidateFiveViaExplainSelectedQuintuple>,
    pub blocked_matching_quintuples: Vec<RoutePathCandidateFiveViaExplainBlockedQuintuple>,
}

impl Board {
    pub fn route_path_candidate_five_via_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateFiveViaExplainReport, RoutePathCandidateError> {
        let path_candidate =
            self.route_path_candidate_five_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)?;
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

        let (_, matching_quintuples) =
            candidate_five_via_matches(self, net_uuid, from_anchor, to_anchor);
        let selected_quintuple = selected_matching_five_via(&matching_quintuples).map(|entry| {
            let points = five_via_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateFiveViaExplainSelectedQuintuple {
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
                first_intermediate_layer: entry.first_intermediate_layer,
                second_intermediate_layer: entry.second_intermediate_layer,
                third_intermediate_layer: entry.third_intermediate_layer,
                fourth_intermediate_layer: entry.fourth_intermediate_layer,
                source_segment: RoutePathCandidateFiveViaExplainSegment {
                    layer: from_anchor.layer,
                    points: points[0].to_vec(),
                },
                first_middle_segment: RoutePathCandidateFiveViaExplainSegment {
                    layer: entry.first_intermediate_layer,
                    points: points[1].to_vec(),
                },
                second_middle_segment: RoutePathCandidateFiveViaExplainSegment {
                    layer: entry.second_intermediate_layer,
                    points: points[2].to_vec(),
                },
                third_middle_segment: RoutePathCandidateFiveViaExplainSegment {
                    layer: entry.third_intermediate_layer,
                    points: points[3].to_vec(),
                },
                fourth_middle_segment: RoutePathCandidateFiveViaExplainSegment {
                    layer: entry.fourth_intermediate_layer,
                    points: points[4].to_vec(),
                },
                target_segment: RoutePathCandidateFiveViaExplainSegment {
                    layer: to_anchor.layer,
                    points: points[5].to_vec(),
                },
                selection_reason: format!(
                    "selected because it is the first unblocked matching authored via quintuple under the deterministic selection rule through intermediate layers {}, {}, {}, then {}",
                    entry.first_intermediate_layer,
                    entry.second_intermediate_layer,
                    entry.third_intermediate_layer,
                    entry.fourth_intermediate_layer
                ),
            }
        });

        let blocked_matching_quintuples = matching_quintuples
            .iter()
            .filter(|entry| {
                !(entry.source_segment.blockages.is_empty()
                    && entry.first_middle_segment.blockages.is_empty()
                    && entry.second_middle_segment.blockages.is_empty()
                    && entry.third_middle_segment.blockages.is_empty()
                    && entry.fourth_middle_segment.blockages.is_empty()
                    && entry.target_segment.blockages.is_empty())
            })
            .map(|entry| {
                let points = five_via_path_points(entry, from_anchor, to_anchor);
                RoutePathCandidateFiveViaExplainBlockedQuintuple {
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
                    first_intermediate_layer: entry.first_intermediate_layer,
                    second_intermediate_layer: entry.second_intermediate_layer,
                    third_intermediate_layer: entry.third_intermediate_layer,
                    fourth_intermediate_layer: entry.fourth_intermediate_layer,
                    source_segment: RoutePathCandidateFiveViaExplainSegment {
                        layer: from_anchor.layer,
                        points: points[0].to_vec(),
                    },
                    first_middle_segment: RoutePathCandidateFiveViaExplainSegment {
                        layer: entry.first_intermediate_layer,
                        points: points[1].to_vec(),
                    },
                    second_middle_segment: RoutePathCandidateFiveViaExplainSegment {
                        layer: entry.second_intermediate_layer,
                        points: points[2].to_vec(),
                    },
                    third_middle_segment: RoutePathCandidateFiveViaExplainSegment {
                        layer: entry.third_intermediate_layer,
                        points: points[3].to_vec(),
                    },
                    fourth_middle_segment: RoutePathCandidateFiveViaExplainSegment {
                        layer: entry.fourth_intermediate_layer,
                        points: points[4].to_vec(),
                    },
                    target_segment: RoutePathCandidateFiveViaExplainSegment {
                        layer: to_anchor.layer,
                        points: points[5].to_vec(),
                    },
                    source_blockages: entry.source_segment.blockages.clone(),
                    first_middle_blockages: entry.first_middle_segment.blockages.clone(),
                    second_middle_blockages: entry.second_middle_segment.blockages.clone(),
                    third_middle_blockages: entry.third_middle_segment.blockages.clone(),
                    fourth_middle_blockages: entry.fourth_middle_segment.blockages.clone(),
                    target_blockages: entry.target_segment.blockages.clone(),
                }
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateFiveViaExplainReport {
            contract: "m5_route_path_candidate_five_via_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateFiveViaExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
                candidate_via_quintuple_count: path_candidate.summary.candidate_via_quintuple_count,
                matching_via_quintuple_count: path_candidate.summary.matching_via_quintuple_count,
                blocked_via_quintuple_count: path_candidate.summary.blocked_via_quintuple_count,
                available_via_quintuple_count: path_candidate.summary.available_via_quintuple_count,
            },
            selected_quintuple,
            blocked_matching_quintuples,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateFiveViaReport,
) -> RoutePathCandidateFiveViaExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateFiveViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_via_quintuple_count == 0 =>
        {
            RoutePathCandidateFiveViaExplainKind::NoMatchingAuthoredViaQuintuple
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateFiveViaExplainKind::AllMatchingViaQuintuplesBlocked
        }
    }
}
