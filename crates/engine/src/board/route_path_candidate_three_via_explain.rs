use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError, RoutePathCandidateStatus,
    RoutePathCandidateThreeViaReport, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_three_via_selection::{
    candidate_three_via_matches, selected_matching_three_via, three_via_path_points,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateThreeViaExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredViaTriple,
    AllMatchingViaTriplesBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateThreeViaExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateThreeViaExplainBlockedTriple {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub via_c_uuid: Uuid,
    pub via_c_position: Point,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub source_segment: RoutePathCandidateThreeViaExplainSegment,
    pub first_middle_segment: RoutePathCandidateThreeViaExplainSegment,
    pub second_middle_segment: RoutePathCandidateThreeViaExplainSegment,
    pub target_segment: RoutePathCandidateThreeViaExplainSegment,
    pub source_blockages: Vec<RouteCorridorSpanBlockage>,
    pub first_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub second_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub target_blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateThreeViaExplainSelectedTriple {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub via_c_uuid: Uuid,
    pub via_c_position: Point,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub source_segment: RoutePathCandidateThreeViaExplainSegment,
    pub first_middle_segment: RoutePathCandidateThreeViaExplainSegment,
    pub second_middle_segment: RoutePathCandidateThreeViaExplainSegment,
    pub target_segment: RoutePathCandidateThreeViaExplainSegment,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateThreeViaExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub candidate_via_triple_count: usize,
    pub matching_via_triple_count: usize,
    pub blocked_via_triple_count: usize,
    pub available_via_triple_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateThreeViaExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateThreeViaExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateThreeViaExplainSummary,
    pub selected_triple: Option<RoutePathCandidateThreeViaExplainSelectedTriple>,
    pub blocked_matching_triples: Vec<RoutePathCandidateThreeViaExplainBlockedTriple>,
}

impl Board {
    pub fn route_path_candidate_three_via_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateThreeViaExplainReport, RoutePathCandidateError> {
        let path_candidate =
            self.route_path_candidate_three_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)?;
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

        let (_, matching_triples) =
            candidate_three_via_matches(self, net_uuid, from_anchor, to_anchor);
        let selected_triple = selected_matching_three_via(&matching_triples).map(|entry| {
            let points = three_via_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateThreeViaExplainSelectedTriple {
                via_a_uuid: entry.via_a.uuid,
                via_a_position: entry.via_a.position,
                via_b_uuid: entry.via_b.uuid,
                via_b_position: entry.via_b.position,
                via_c_uuid: entry.via_c.uuid,
                via_c_position: entry.via_c.position,
                first_intermediate_layer: entry.first_intermediate_layer,
                second_intermediate_layer: entry.second_intermediate_layer,
                source_segment: RoutePathCandidateThreeViaExplainSegment {
                    layer: from_anchor.layer,
                    points: points[0].to_vec(),
                },
                first_middle_segment: RoutePathCandidateThreeViaExplainSegment {
                    layer: entry.first_intermediate_layer,
                    points: points[1].to_vec(),
                },
                second_middle_segment: RoutePathCandidateThreeViaExplainSegment {
                    layer: entry.second_intermediate_layer,
                    points: points[2].to_vec(),
                },
                target_segment: RoutePathCandidateThreeViaExplainSegment {
                    layer: to_anchor.layer,
                    points: points[3].to_vec(),
                },
                selection_reason: format!(
                    "selected because it is the first unblocked matching authored via triple under the deterministic selection rule through intermediate layers {} then {}",
                    entry.first_intermediate_layer, entry.second_intermediate_layer
                ),
            }
        });

        let blocked_matching_triples = matching_triples
            .iter()
            .filter(|entry| {
                !(entry.source_segment.blockages.is_empty()
                    && entry.first_middle_segment.blockages.is_empty()
                    && entry.second_middle_segment.blockages.is_empty()
                    && entry.target_segment.blockages.is_empty())
            })
            .map(|entry| {
                let points = three_via_path_points(entry, from_anchor, to_anchor);
                RoutePathCandidateThreeViaExplainBlockedTriple {
                    via_a_uuid: entry.via_a.uuid,
                    via_a_position: entry.via_a.position,
                    via_b_uuid: entry.via_b.uuid,
                    via_b_position: entry.via_b.position,
                    via_c_uuid: entry.via_c.uuid,
                    via_c_position: entry.via_c.position,
                    first_intermediate_layer: entry.first_intermediate_layer,
                    second_intermediate_layer: entry.second_intermediate_layer,
                    source_segment: RoutePathCandidateThreeViaExplainSegment {
                        layer: from_anchor.layer,
                        points: points[0].to_vec(),
                    },
                    first_middle_segment: RoutePathCandidateThreeViaExplainSegment {
                        layer: entry.first_intermediate_layer,
                        points: points[1].to_vec(),
                    },
                    second_middle_segment: RoutePathCandidateThreeViaExplainSegment {
                        layer: entry.second_intermediate_layer,
                        points: points[2].to_vec(),
                    },
                    target_segment: RoutePathCandidateThreeViaExplainSegment {
                        layer: to_anchor.layer,
                        points: points[3].to_vec(),
                    },
                    source_blockages: entry.source_segment.blockages.clone(),
                    first_middle_blockages: entry.first_middle_segment.blockages.clone(),
                    second_middle_blockages: entry.second_middle_segment.blockages.clone(),
                    target_blockages: entry.target_segment.blockages.clone(),
                }
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateThreeViaExplainReport {
            contract: "m5_route_path_candidate_three_via_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateThreeViaExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
                candidate_via_triple_count: path_candidate.summary.candidate_via_triple_count,
                matching_via_triple_count: path_candidate.summary.matching_via_triple_count,
                blocked_via_triple_count: path_candidate.summary.blocked_via_triple_count,
                available_via_triple_count: path_candidate.summary.available_via_triple_count,
            },
            selected_triple,
            blocked_matching_triples,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateThreeViaReport,
) -> RoutePathCandidateThreeViaExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateThreeViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_via_triple_count == 0 =>
        {
            RoutePathCandidateThreeViaExplainKind::NoMatchingAuthoredViaTriple
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateThreeViaExplainKind::AllMatchingViaTriplesBlocked
        }
    }
}
