use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError,
    RoutePathCandidateOrthogonalGraphPathCost, RoutePathCandidateOrthogonalGraphTwoViaReport,
    RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_selection::{
    orthogonal_graph_path_cost, search_orthogonal_graph_layer,
};
use super::route_path_candidate_two_via_selection::candidate_two_via_matches;

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
        let path_candidate = self.route_path_candidate_orthogonal_graph_two_via(
            net_uuid,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
        )?;
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

        let (_candidate_vias, matching_pairs) =
            candidate_two_via_matches(self, net_uuid, from_anchor, to_anchor);
        let pair_searches = matching_pairs
            .iter()
            .map(|entry| {
                let source_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    from_anchor.layer,
                    from_anchor.position,
                    entry.via_a.position,
                );
                let middle_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    entry.intermediate_layer,
                    entry.via_a.position,
                    entry.via_b.position,
                );
                let target_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    to_anchor.layer,
                    entry.via_b.position,
                    to_anchor.position,
                );
                (entry, source_search, middle_search, target_search)
            })
            .collect::<Vec<_>>();

        let selected_pair = pair_searches
            .iter()
            .find_map(|(entry, source_search, middle_search, target_search)| {
                Some(RoutePathCandidateOrthogonalGraphTwoViaExplainSelectedPair {
                    via_a_uuid: entry.via_a.uuid,
                    via_a_position: entry.via_a.position,
                    via_b_uuid: entry.via_b.uuid,
                    via_b_position: entry.via_b.position,
                    intermediate_layer: entry.intermediate_layer,
                    source_segment: RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                        layer: from_anchor.layer,
                        points: source_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(source_search.path.as_ref()?),
                    },
                    middle_segment: RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                        layer: entry.intermediate_layer,
                        points: middle_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(middle_search.path.as_ref()?),
                    },
                    target_segment: RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                        layer: to_anchor.layer,
                        points: target_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(target_search.path.as_ref()?),
                    },
                    selection_reason: format!(
                        "selected because it is the first matching authored via pair whose source-layer, intermediate-layer, and target-layer orthogonal graph searches all yield deterministic paths between layers {} and {}; each layer-side path is the lowest-cost graph path under the accepted ranking rule",
                        from_anchor.layer, to_anchor.layer
                    ),
                })
            });

        let blocked_matching_pairs = pair_searches
            .iter()
            .filter(|(_, source_search, middle_search, target_search)| {
                source_search.path.is_none()
                    || middle_search.path.is_none()
                    || target_search.path.is_none()
            })
            .map(|(entry, source_search, middle_search, target_search)| {
                RoutePathCandidateOrthogonalGraphTwoViaExplainBlockedPair {
                    via_a_uuid: entry.via_a.uuid,
                    via_a_position: entry.via_a.position,
                    via_b_uuid: entry.via_b.uuid,
                    via_b_position: entry.via_b.position,
                    intermediate_layer: entry.intermediate_layer,
                    source_segment: source_search.path.clone().map(|points| {
                        let cost = orthogonal_graph_path_cost(&points);
                        RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                            layer: from_anchor.layer,
                            points,
                            cost,
                        }
                    }),
                    middle_segment: middle_search.path.clone().map(|points| {
                        let cost = orthogonal_graph_path_cost(&points);
                        RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                            layer: entry.intermediate_layer,
                            points,
                            cost,
                        }
                    }),
                    target_segment: target_search.path.clone().map(|points| {
                        let cost = orthogonal_graph_path_cost(&points);
                        RoutePathCandidateOrthogonalGraphTwoViaExplainSegment {
                            layer: to_anchor.layer,
                            points,
                            cost,
                        }
                    }),
                    source_blockages: source_search
                        .blocked_edges
                        .iter()
                        .flat_map(|edge| edge.blockages.clone())
                        .collect(),
                    middle_blockages: middle_search
                        .blocked_edges
                        .iter()
                        .flat_map(|edge| edge.blockages.clone())
                        .collect(),
                    target_blockages: target_search
                        .blocked_edges
                        .iter()
                        .flat_map(|edge| edge.blockages.clone())
                        .collect(),
                }
            })
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateOrthogonalGraphTwoViaExplainReport {
            contract: "m5_route_path_candidate_orthogonal_graph_two_via_explain_v1".to_string(),
            persisted_native_board_state_only: true,
            status: path_candidate.status.clone(),
            explanation_kind: explanation_kind(&path_candidate),
            net_uuid: path_candidate.net_uuid,
            net_name: path_candidate.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            selection_rule: path_candidate.selection_rule,
            component_selection_rules: path_candidate.component_selection_rules,
            candidate_copper_layers: path_candidate.candidate_copper_layers,
            summary: RoutePathCandidateOrthogonalGraphTwoViaExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
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
    report: &RoutePathCandidateOrthogonalGraphTwoViaReport,
) -> RoutePathCandidateOrthogonalGraphTwoViaExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateOrthogonalGraphTwoViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_via_pair_count == 0 =>
        {
            RoutePathCandidateOrthogonalGraphTwoViaExplainKind::NoMatchingAuthoredViaPair
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateOrthogonalGraphTwoViaExplainKind::AllMatchingViaPairsBlocked
        }
    }
}
