use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError, RoutePathCandidateStatus,
    RoutePathCandidateTwoViaReport, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_two_via_selection::{
    candidate_two_via_matches, selected_matching_two_via, two_via_path_points,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateTwoViaExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredViaPair,
    AllMatchingViaPairsBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateTwoViaExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateTwoViaExplainBlockedPair {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub intermediate_layer: LayerId,
    pub source_segment: RoutePathCandidateTwoViaExplainSegment,
    pub middle_segment: RoutePathCandidateTwoViaExplainSegment,
    pub target_segment: RoutePathCandidateTwoViaExplainSegment,
    pub source_blockages: Vec<RouteCorridorSpanBlockage>,
    pub middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub target_blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateTwoViaExplainSelectedPair {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub intermediate_layer: LayerId,
    pub source_segment: RoutePathCandidateTwoViaExplainSegment,
    pub middle_segment: RoutePathCandidateTwoViaExplainSegment,
    pub target_segment: RoutePathCandidateTwoViaExplainSegment,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateTwoViaExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub candidate_via_pair_count: usize,
    pub matching_via_pair_count: usize,
    pub blocked_via_pair_count: usize,
    pub available_via_pair_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateTwoViaExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateTwoViaExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateTwoViaExplainSummary,
    pub selected_pair: Option<RoutePathCandidateTwoViaExplainSelectedPair>,
    pub blocked_matching_pairs: Vec<RoutePathCandidateTwoViaExplainBlockedPair>,
}

impl Board {
    pub fn route_path_candidate_two_via_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateTwoViaExplainReport, RoutePathCandidateError> {
        let path_candidate =
            self.route_path_candidate_two_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)?;
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

        let (_, matching_pairs) = candidate_two_via_matches(self, net_uuid, from_anchor, to_anchor);
        let selected_pair = selected_matching_two_via(&matching_pairs).map(|entry| {
            let points = two_via_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateTwoViaExplainSelectedPair {
                via_a_uuid: entry.via_a.uuid,
                via_a_position: entry.via_a.position,
                via_b_uuid: entry.via_b.uuid,
                via_b_position: entry.via_b.position,
                intermediate_layer: entry.intermediate_layer,
                source_segment: RoutePathCandidateTwoViaExplainSegment {
                    layer: from_anchor.layer,
                    points: points[0].to_vec(),
                },
                middle_segment: RoutePathCandidateTwoViaExplainSegment {
                    layer: entry.intermediate_layer,
                    points: points[1].to_vec(),
                },
                target_segment: RoutePathCandidateTwoViaExplainSegment {
                    layer: to_anchor.layer,
                    points: points[2].to_vec(),
                },
                selection_reason: format!(
                    "selected because it is the first unblocked matching authored via pair under the deterministic selection rule through intermediate layer {}",
                    entry.intermediate_layer
                ),
            }
        });

        let blocked_matching_pairs = matching_pairs
            .iter()
            .filter(|entry| {
                !(entry.source_segment.blockages.is_empty()
                    && entry.middle_segment.blockages.is_empty()
                    && entry.target_segment.blockages.is_empty())
            })
            .map(|entry| {
                let points = two_via_path_points(entry, from_anchor, to_anchor);
                RoutePathCandidateTwoViaExplainBlockedPair {
                    via_a_uuid: entry.via_a.uuid,
                    via_a_position: entry.via_a.position,
                    via_b_uuid: entry.via_b.uuid,
                    via_b_position: entry.via_b.position,
                    intermediate_layer: entry.intermediate_layer,
                    source_segment: RoutePathCandidateTwoViaExplainSegment {
                        layer: from_anchor.layer,
                        points: points[0].to_vec(),
                    },
                    middle_segment: RoutePathCandidateTwoViaExplainSegment {
                        layer: entry.intermediate_layer,
                        points: points[1].to_vec(),
                    },
                    target_segment: RoutePathCandidateTwoViaExplainSegment {
                        layer: to_anchor.layer,
                        points: points[2].to_vec(),
                    },
                    source_blockages: entry.source_segment.blockages.clone(),
                    middle_blockages: entry.middle_segment.blockages.clone(),
                    target_blockages: entry.target_segment.blockages.clone(),
                }
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateTwoViaExplainReport {
            contract: "m5_route_path_candidate_two_via_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateTwoViaExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
                candidate_via_pair_count: path_candidate.summary.candidate_via_pair_count,
                matching_via_pair_count: path_candidate.summary.matching_via_pair_count,
                blocked_via_pair_count: path_candidate.summary.blocked_via_pair_count,
                available_via_pair_count: path_candidate.summary.available_via_pair_count,
            },
            selected_pair,
            blocked_matching_pairs,
        })
    }
}

fn explanation_kind(
    report: &RoutePathCandidateTwoViaReport,
) -> RoutePathCandidateTwoViaExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateTwoViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_via_pair_count == 0 =>
        {
            RoutePathCandidateTwoViaExplainKind::NoMatchingAuthoredViaPair
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateTwoViaExplainKind::AllMatchingViaPairsBlocked
        }
    }
}
