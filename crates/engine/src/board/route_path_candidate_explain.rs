use super::route_path_candidate_selection::{
    matching_corridor_spans, oriented_span_points, selected_matching_span,
};
use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError, RoutePathCandidateReport,
    RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateExplainKind {
    DeterministicPathFound,
    NoMatchingCorridorSpan,
    AllMatchingSpansBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateExplainBlockedSpan {
    pub pair_index: usize,
    pub layer: LayerId,
    pub from: Point,
    pub to: Point,
    pub blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateExplainSelectedSpan {
    pub pair_index: usize,
    pub layer: LayerId,
    pub from: Point,
    pub to: Point,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub matching_span_count: usize,
    pub blocked_span_count: usize,
    pub available_span_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateExplainSummary,
    pub selected_span: Option<RoutePathCandidateExplainSelectedSpan>,
    pub blocked_matching_spans: Vec<RoutePathCandidateExplainBlockedSpan>,
}

impl Board {
    pub fn route_path_candidate_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateExplainReport, RoutePathCandidateError> {
        let path_candidate =
            self.route_path_candidate(net_uuid, from_anchor_pad_uuid, to_anchor_pad_uuid)?;
        let corridor = self
            .route_corridor(net_uuid)
            .ok_or(RoutePathCandidateError::NetNotFound { net_uuid })?;
        let matching_spans = matching_corridor_spans(
            &corridor.corridor_spans,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        );
        let selected_span = selected_matching_span(&matching_spans).map(|span| {
            let points = oriented_span_points(span, from_anchor_pad_uuid, to_anchor_pad_uuid);
            RoutePathCandidateExplainSelectedSpan {
                pair_index: span.pair_index,
                layer: span.layer,
                from: points[0],
                to: points[1],
                selection_reason: format!(
                    "selected because it is the first unblocked matching corridor span under the deterministic selection rule on layer {}",
                    span.layer
                ),
            }
        });
        let blocked_matching_spans = matching_spans
            .iter()
            .filter(|span| span.blocked)
            .map(|span| {
                let points = oriented_span_points(span, from_anchor_pad_uuid, to_anchor_pad_uuid);
                RoutePathCandidateExplainBlockedSpan {
                    pair_index: span.pair_index,
                    layer: span.layer,
                    from: points[0],
                    to: points[1],
                    blockages: span.blockages.clone(),
                }
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateExplainReport {
            contract: "m5_route_path_candidate_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                matching_span_count: path_candidate.summary.matching_span_count,
                blocked_span_count: path_candidate.summary.blocked_span_count,
                available_span_count: path_candidate.summary.available_span_count,
            },
            selected_span,
            blocked_matching_spans,
        })
    }
}

fn explanation_kind(report: &RoutePathCandidateReport) -> RoutePathCandidateExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_span_count == 0 =>
        {
            RoutePathCandidateExplainKind::NoMatchingCorridorSpan
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateExplainKind::AllMatchingSpansBlocked
        }
    }
}
