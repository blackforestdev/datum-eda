use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError, RoutePathCandidateSixViaReport,
    RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_six_via_selection::{
    candidate_six_via_matches, selected_matching_six_via, six_via_path_points,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateSixViaExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredViaSextuple,
    AllMatchingViaSextuplesBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateSixViaExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateSixViaExplainBlockedSextuple {
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
    pub source_segment: RoutePathCandidateSixViaExplainSegment,
    pub first_middle_segment: RoutePathCandidateSixViaExplainSegment,
    pub second_middle_segment: RoutePathCandidateSixViaExplainSegment,
    pub third_middle_segment: RoutePathCandidateSixViaExplainSegment,
    pub fourth_middle_segment: RoutePathCandidateSixViaExplainSegment,
    pub fifth_middle_segment: RoutePathCandidateSixViaExplainSegment,
    pub target_segment: RoutePathCandidateSixViaExplainSegment,
    pub source_blockages: Vec<RouteCorridorSpanBlockage>,
    pub first_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub second_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub third_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub fourth_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub fifth_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub target_blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateSixViaExplainSelectedSextuple {
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
    pub source_segment: RoutePathCandidateSixViaExplainSegment,
    pub first_middle_segment: RoutePathCandidateSixViaExplainSegment,
    pub second_middle_segment: RoutePathCandidateSixViaExplainSegment,
    pub third_middle_segment: RoutePathCandidateSixViaExplainSegment,
    pub fourth_middle_segment: RoutePathCandidateSixViaExplainSegment,
    pub fifth_middle_segment: RoutePathCandidateSixViaExplainSegment,
    pub target_segment: RoutePathCandidateSixViaExplainSegment,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateSixViaExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub candidate_via_sextuple_count: usize,
    pub matching_via_sextuple_count: usize,
    pub blocked_via_sextuple_count: usize,
    pub available_via_sextuple_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateSixViaExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateSixViaExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateSixViaExplainSummary,
    pub selected_sextuple: Option<RoutePathCandidateSixViaExplainSelectedSextuple>,
    pub blocked_matching_sextuples: Vec<RoutePathCandidateSixViaExplainBlockedSextuple>,
}

impl Board {
    pub fn route_path_candidate_six_via_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateSixViaExplainReport, RoutePathCandidateError> {
        let path_candidate =
            self.route_path_candidate_six_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)?;
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

        let (_, matching_sextuples) =
            candidate_six_via_matches(self, net_uuid, from_anchor, to_anchor);
        let selected_sextuple = selected_matching_six_via(&matching_sextuples).map(|entry| {
            let points = six_via_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateSixViaExplainSelectedSextuple {
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
                source_segment: RoutePathCandidateSixViaExplainSegment {
                    layer: from_anchor.layer,
                    points: points[0].to_vec(),
                },
                first_middle_segment: RoutePathCandidateSixViaExplainSegment {
                    layer: entry.first_intermediate_layer,
                    points: points[1].to_vec(),
                },
                second_middle_segment: RoutePathCandidateSixViaExplainSegment {
                    layer: entry.second_intermediate_layer,
                    points: points[2].to_vec(),
                },
                third_middle_segment: RoutePathCandidateSixViaExplainSegment {
                    layer: entry.third_intermediate_layer,
                    points: points[3].to_vec(),
                },
                fourth_middle_segment: RoutePathCandidateSixViaExplainSegment {
                    layer: entry.fourth_intermediate_layer,
                    points: points[4].to_vec(),
                },
                fifth_middle_segment: RoutePathCandidateSixViaExplainSegment {
                    layer: entry.fifth_intermediate_layer,
                    points: points[5].to_vec(),
                },
                target_segment: RoutePathCandidateSixViaExplainSegment {
                    layer: to_anchor.layer,
                    points: points[6].to_vec(),
                },
                selection_reason: format!(
                    "selected because it is the first unblocked matching authored via sextuple under the deterministic selection rule through intermediate layers {}, {}, {}, {}, then {}",
                    entry.first_intermediate_layer,
                    entry.second_intermediate_layer,
                    entry.third_intermediate_layer,
                    entry.fourth_intermediate_layer,
                    entry.fifth_intermediate_layer
                ),
            }
        });

        let blocked_matching_sextuples = matching_sextuples
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
            .map(|entry| {
                let points = six_via_path_points(entry, from_anchor, to_anchor);
                RoutePathCandidateSixViaExplainBlockedSextuple {
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
                    source_segment: RoutePathCandidateSixViaExplainSegment {
                        layer: from_anchor.layer,
                        points: points[0].to_vec(),
                    },
                    first_middle_segment: RoutePathCandidateSixViaExplainSegment {
                        layer: entry.first_intermediate_layer,
                        points: points[1].to_vec(),
                    },
                    second_middle_segment: RoutePathCandidateSixViaExplainSegment {
                        layer: entry.second_intermediate_layer,
                        points: points[2].to_vec(),
                    },
                    third_middle_segment: RoutePathCandidateSixViaExplainSegment {
                        layer: entry.third_intermediate_layer,
                        points: points[3].to_vec(),
                    },
                    fourth_middle_segment: RoutePathCandidateSixViaExplainSegment {
                        layer: entry.fourth_intermediate_layer,
                        points: points[4].to_vec(),
                    },
                    fifth_middle_segment: RoutePathCandidateSixViaExplainSegment {
                        layer: entry.fifth_intermediate_layer,
                        points: points[5].to_vec(),
                    },
                    target_segment: RoutePathCandidateSixViaExplainSegment {
                        layer: to_anchor.layer,
                        points: points[6].to_vec(),
                    },
                    source_blockages: entry.source_segment.blockages.clone(),
                    first_middle_blockages: entry.first_middle_segment.blockages.clone(),
                    second_middle_blockages: entry.second_middle_segment.blockages.clone(),
                    third_middle_blockages: entry.third_middle_segment.blockages.clone(),
                    fourth_middle_blockages: entry.fourth_middle_segment.blockages.clone(),
                    fifth_middle_blockages: entry.fifth_middle_segment.blockages.clone(),
                    target_blockages: entry.target_segment.blockages.clone(),
                }
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateSixViaExplainReport {
            contract: "m5_route_path_candidate_six_via_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateSixViaExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
                candidate_via_sextuple_count: path_candidate.summary.candidate_via_sextuple_count,
                matching_via_sextuple_count: path_candidate.summary.matching_via_sextuple_count,
                blocked_via_sextuple_count: path_candidate.summary.blocked_via_sextuple_count,
                available_via_sextuple_count: path_candidate.summary.available_via_sextuple_count,
            },
            selected_sextuple,
            blocked_matching_sextuples,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateSixViaReport,
) -> RoutePathCandidateSixViaExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateSixViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_via_sextuple_count == 0 =>
        {
            RoutePathCandidateSixViaExplainKind::NoMatchingAuthoredViaSextuple
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateSixViaExplainKind::AllMatchingViaSextuplesBlocked
        }
    }
}
