use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError,
    RoutePathCandidateOrthogonalGraphPathCost, RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_two_via::RoutePathCandidateOrthogonalGraphTwoViaSummary;
use super::route_path_candidate_orthogonal_graph_two_via_spine::build_orthogonal_graph_two_via_candidate_spine;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalGraphTwoViaExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredViaPair,
    AllMatchingViaPairsBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub cost: RoutePathCandidateOrthogonalGraphPathCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphTwoViaExplainBlockedPair {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub intermediate_layer: LayerId,
    pub source_segment: Option<RoutePathCandidateOrthogonalGraphTwoViaExplainSegment>,
    pub middle_segment: Option<RoutePathCandidateOrthogonalGraphTwoViaExplainSegment>,
    pub target_segment: Option<RoutePathCandidateOrthogonalGraphTwoViaExplainSegment>,
    pub source_blockages: Vec<RouteCorridorSpanBlockage>,
    pub middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub target_blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphTwoViaExplainSelectedPair {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub intermediate_layer: LayerId,
    pub source_segment: RoutePathCandidateOrthogonalGraphTwoViaExplainSegment,
    pub middle_segment: RoutePathCandidateOrthogonalGraphTwoViaExplainSegment,
    pub target_segment: RoutePathCandidateOrthogonalGraphTwoViaExplainSegment,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphTwoViaExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub matching_via_pair_count: usize,
    pub blocked_via_pair_count: usize,
    pub available_via_pair_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphTwoViaExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateOrthogonalGraphTwoViaExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub component_selection_rules: Vec<String>,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphTwoViaExplainSummary,
    pub selected_pair: Option<RoutePathCandidateOrthogonalGraphTwoViaExplainSelectedPair>,
    pub blocked_matching_pairs: Vec<RoutePathCandidateOrthogonalGraphTwoViaExplainBlockedPair>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph_two_via_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphTwoViaExplainReport, RoutePathCandidateError> {
        let spine = build_orthogonal_graph_two_via_candidate_spine(
            self,
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;
        let selected_pair = spine
            .pair_searches
            .iter()
            .find_map(|search| {
                Some(RoutePathCandidateOrthogonalGraphTwoViaExplainSelectedPair {
                    via_a_uuid: search.entry.via_a.uuid,
                    via_a_position: search.entry.via_a.position,
                    via_b_uuid: search.entry.via_b.uuid,
                    via_b_position: search.entry.via_b.position,
                    intermediate_layer: search.entry.intermediate_layer,
                    source_segment: RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                        layer: search.source.search.layer,
                        points: search.source.path.as_ref()?.points.clone(),
                        cost: search.source.path.as_ref()?.cost.clone(),
                    },
                    middle_segment: RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                        layer: search.middle.search.layer,
                        points: search.middle.path.as_ref()?.points.clone(),
                        cost: search.middle.path.as_ref()?.cost.clone(),
                    },
                    target_segment: RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                        layer: search.target.search.layer,
                        points: search.target.path.as_ref()?.points.clone(),
                        cost: search.target.path.as_ref()?.cost.clone(),
                    },
                    selection_reason: format!(
                        "selected because it is the first matching authored via pair whose source-layer, intermediate-layer, and target-layer orthogonal graph searches all yield deterministic paths between layers {} and {}; each layer-side path is the lowest-cost graph path under the accepted ranking rule",
                        search.source.search.layer, search.target.search.layer
                    ),
                })
            });

        let blocked_matching_pairs = spine
            .pair_searches
            .iter()
            .filter(|search| {
                search.source.path.is_none()
                    || search.middle.path.is_none()
                    || search.target.path.is_none()
            })
            .map(
                |search| RoutePathCandidateOrthogonalGraphTwoViaExplainBlockedPair {
                    via_a_uuid: search.entry.via_a.uuid,
                    via_a_position: search.entry.via_a.position,
                    via_b_uuid: search.entry.via_b.uuid,
                    via_b_position: search.entry.via_b.position,
                    intermediate_layer: search.entry.intermediate_layer,
                    source_segment: search.source.path.as_ref().map(|path| {
                        RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                            layer: path.layer,
                            points: path.points.clone(),
                            cost: path.cost.clone(),
                        }
                    }),
                    middle_segment: search.middle.path.as_ref().map(|path| {
                        RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                            layer: path.layer,
                            points: path.points.clone(),
                            cost: path.cost.clone(),
                        }
                    }),
                    target_segment: search.target.path.as_ref().map(|path| {
                        RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                            layer: path.layer,
                            points: path.points.clone(),
                            cost: path.cost.clone(),
                        }
                    }),
                    source_blockages: search
                        .source
                        .search
                        .blocked_edges
                        .iter()
                        .flat_map(|edge| edge.blockages.clone())
                        .collect(),
                    middle_blockages: search
                        .middle
                        .search
                        .blocked_edges
                        .iter()
                        .flat_map(|edge| edge.blockages.clone())
                        .collect(),
                    target_blockages: search
                        .target
                        .search
                        .blocked_edges
                        .iter()
                        .flat_map(|edge| edge.blockages.clone())
                        .collect(),
                },
            )
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateOrthogonalGraphTwoViaExplainReport {
            contract: "m5_route_path_candidate_orthogonal_graph_two_via_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: spine.status.clone(),
            explanation_kind: explanation_kind(spine.status.clone(), &spine.summary),
            net_uuid: spine.net_uuid,
            net_name: spine.net_name,
            from_anchor_pad_uuid: spine.from_anchor_pad_uuid,
            to_anchor_pad_uuid: spine.to_anchor_pad_uuid,
            selection_rule: spine.selection_rule,
            component_selection_rules: spine.component_selection_rules,
            candidate_copper_layers: spine.candidate_copper_layers,
            summary: RoutePathCandidateOrthogonalGraphTwoViaExplainSummary {
                candidate_copper_layer_count: spine.summary.candidate_copper_layer_count,
                candidate_via_count: spine.summary.candidate_via_count,
                matching_via_pair_count: spine.summary.matching_via_pair_count,
                blocked_via_pair_count: spine.summary.blocked_via_pair_count,
                available_via_pair_count: spine.summary.available_via_pair_count,
            },
            selected_pair,
            blocked_matching_pairs,
        })
    }
}

fn explanation_kind(
    status: RoutePathCandidateStatus,
    summary: &RoutePathCandidateOrthogonalGraphTwoViaSummary,
) -> RoutePathCandidateOrthogonalGraphTwoViaExplainKind {
    match status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateOrthogonalGraphTwoViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if summary.matching_via_pair_count == 0 =>
        {
            RoutePathCandidateOrthogonalGraphTwoViaExplainKind::NoMatchingAuthoredViaPair
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateOrthogonalGraphTwoViaExplainKind::AllMatchingViaPairsBlocked
        }
    }
}
