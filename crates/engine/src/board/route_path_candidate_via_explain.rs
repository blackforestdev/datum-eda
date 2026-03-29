use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError, RoutePathCandidateStatus,
    RoutePathCandidateViaReport, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_via_selection::{
    matching_via_analyses, selected_matching_via, via_path_points,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateViaExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredVia,
    AllMatchingViasBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateViaExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateViaExplainBlockedVia {
    pub via_uuid: Uuid,
    pub via_position: Point,
    pub source_segment: RoutePathCandidateViaExplainSegment,
    pub target_segment: RoutePathCandidateViaExplainSegment,
    pub source_blockages: Vec<RouteCorridorSpanBlockage>,
    pub target_blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateViaExplainSelectedVia {
    pub via_uuid: Uuid,
    pub via_position: Point,
    pub source_segment: RoutePathCandidateViaExplainSegment,
    pub target_segment: RoutePathCandidateViaExplainSegment,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateViaExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub matching_via_count: usize,
    pub blocked_via_count: usize,
    pub available_via_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateViaExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateViaExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateViaExplainSummary,
    pub selected_via: Option<RoutePathCandidateViaExplainSelectedVia>,
    pub blocked_matching_vias: Vec<RoutePathCandidateViaExplainBlockedVia>,
}

impl Board {
    pub fn route_path_candidate_via_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateViaExplainReport, RoutePathCandidateError> {
        let path_candidate =
            self.route_path_candidate_via(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)?;
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

        let candidate_vias = super::route_path_candidate_via_selection::candidate_vias_for_net(
            self, net_uuid,
        );
        let matching_vias =
            matching_via_analyses(self, net_uuid, from_anchor, to_anchor, &candidate_vias);

        let selected_via = selected_matching_via(&matching_vias).map(|entry| {
            let points = via_path_points(entry, from_anchor, to_anchor);
            RoutePathCandidateViaExplainSelectedVia {
                via_uuid: entry.via.uuid,
                via_position: entry.via.position,
                source_segment: RoutePathCandidateViaExplainSegment {
                    layer: from_anchor.layer,
                    points: points[0].to_vec(),
                },
                target_segment: RoutePathCandidateViaExplainSegment {
                    layer: to_anchor.layer,
                    points: points[1].to_vec(),
                },
                selection_reason: format!(
                    "selected because it is the first unblocked matching authored via under the deterministic selection rule between layers {} and {}",
                    from_anchor.layer, to_anchor.layer
                ),
            }
        });

        let blocked_matching_vias = matching_vias
            .iter()
            .filter(|entry| {
                !(entry.source_segment.blockages.is_empty() && entry.target_segment.blockages.is_empty())
            })
            .map(|entry| {
                let points = via_path_points(entry, from_anchor, to_anchor);
                RoutePathCandidateViaExplainBlockedVia {
                    via_uuid: entry.via.uuid,
                    via_position: entry.via.position,
                    source_segment: RoutePathCandidateViaExplainSegment {
                        layer: from_anchor.layer,
                        points: points[0].to_vec(),
                    },
                    target_segment: RoutePathCandidateViaExplainSegment {
                        layer: to_anchor.layer,
                        points: points[1].to_vec(),
                    },
                    source_blockages: entry.source_segment.blockages.clone(),
                    target_blockages: entry.target_segment.blockages.clone(),
                }
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateViaExplainReport {
            contract: "m5_route_path_candidate_via_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateViaExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
                matching_via_count: path_candidate.summary.matching_via_count,
                blocked_via_count: path_candidate.summary.blocked_via_count,
                available_via_count: path_candidate.summary.available_via_count,
            },
            selected_via,
            blocked_matching_vias,
        })
    }
}

fn explanation_kind(report: &RoutePathCandidateViaReport) -> RoutePathCandidateViaExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_via_count == 0 =>
        {
            RoutePathCandidateViaExplainKind::NoMatchingAuthoredVia
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateViaExplainKind::AllMatchingViasBlocked
        }
    }
}
