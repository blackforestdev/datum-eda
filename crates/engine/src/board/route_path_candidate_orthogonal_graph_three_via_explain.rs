use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{
    Board, RouteCorridorSpanBlockage, RoutePathCandidateError,
    RoutePathCandidateOrthogonalGraphPathCost, RoutePathCandidateOrthogonalGraphThreeViaReport,
    RoutePathCandidateStatus, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_selection::{
    orthogonal_graph_path_cost, search_orthogonal_graph_layer,
};
use super::route_path_candidate_three_via_selection::candidate_three_via_matches;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutePathCandidateOrthogonalGraphThreeViaExplainKind {
    DeterministicPathFound,
    NoMatchingAuthoredViaTriple,
    AllMatchingViaTriplesBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphThreeViaExplainSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub cost: RoutePathCandidateOrthogonalGraphPathCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphThreeViaExplainBlockedTriple {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub via_c_uuid: Uuid,
    pub via_c_position: Point,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub source_segment: Option<RoutePathCandidateOrthogonalGraphThreeViaExplainSegment>,
    pub first_middle_segment: Option<RoutePathCandidateOrthogonalGraphThreeViaExplainSegment>,
    pub second_middle_segment: Option<RoutePathCandidateOrthogonalGraphThreeViaExplainSegment>,
    pub target_segment: Option<RoutePathCandidateOrthogonalGraphThreeViaExplainSegment>,
    pub source_blockages: Vec<RouteCorridorSpanBlockage>,
    pub first_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub second_middle_blockages: Vec<RouteCorridorSpanBlockage>,
    pub target_blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphThreeViaExplainSelectedTriple {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub via_c_uuid: Uuid,
    pub via_c_position: Point,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub source_segment: RoutePathCandidateOrthogonalGraphThreeViaExplainSegment,
    pub first_middle_segment: RoutePathCandidateOrthogonalGraphThreeViaExplainSegment,
    pub second_middle_segment: RoutePathCandidateOrthogonalGraphThreeViaExplainSegment,
    pub target_segment: RoutePathCandidateOrthogonalGraphThreeViaExplainSegment,
    pub selection_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphThreeViaExplainSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub matching_via_triple_count: usize,
    pub blocked_via_triple_count: usize,
    pub available_via_triple_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphThreeViaExplainReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub status: RoutePathCandidateStatus,
    pub explanation_kind: RoutePathCandidateOrthogonalGraphThreeViaExplainKind,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub component_selection_rules: Vec<String>,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphThreeViaExplainSummary,
    pub selected_triple: Option<RoutePathCandidateOrthogonalGraphThreeViaExplainSelectedTriple>,
    pub blocked_matching_triples: Vec<RoutePathCandidateOrthogonalGraphThreeViaExplainBlockedTriple>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph_three_via_explain(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphThreeViaExplainReport, RoutePathCandidateError>
    {
        let path_candidate = self.route_path_candidate_orthogonal_graph_three_via(
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

        let (_candidate_vias, matching_triples) =
            candidate_three_via_matches(self, net_uuid, from_anchor, to_anchor);
        let triple_searches = matching_triples
            .iter()
            .map(|entry| {
                let source_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    from_anchor.layer,
                    from_anchor.position,
                    entry.via_a.position,
                );
                let first_middle_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    entry.first_intermediate_layer,
                    entry.via_a.position,
                    entry.via_b.position,
                );
                let second_middle_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    entry.second_intermediate_layer,
                    entry.via_b.position,
                    entry.via_c.position,
                );
                let target_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    to_anchor.layer,
                    entry.via_c.position,
                    to_anchor.position,
                );
                (
                    entry,
                    source_search,
                    first_middle_search,
                    second_middle_search,
                    target_search,
                )
            })
            .collect::<Vec<_>>();

        let selected_triple = triple_searches.iter().find_map(
            |(entry, source_search, first_middle_search, second_middle_search, target_search)| {
                Some(RoutePathCandidateOrthogonalGraphThreeViaExplainSelectedTriple {
                    via_a_uuid: entry.via_a.uuid,
                    via_a_position: entry.via_a.position,
                    via_b_uuid: entry.via_b.uuid,
                    via_b_position: entry.via_b.position,
                    via_c_uuid: entry.via_c.uuid,
                    via_c_position: entry.via_c.position,
                    first_intermediate_layer: entry.first_intermediate_layer,
                    second_intermediate_layer: entry.second_intermediate_layer,
                    source_segment: RoutePathCandidateOrthogonalGraphThreeViaExplainSegment {
                        layer: from_anchor.layer,
                        points: source_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(source_search.path.as_ref()?),
                    },
                    first_middle_segment: RoutePathCandidateOrthogonalGraphThreeViaExplainSegment {
                        layer: entry.first_intermediate_layer,
                        points: first_middle_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(first_middle_search.path.as_ref()?),
                    },
                    second_middle_segment: RoutePathCandidateOrthogonalGraphThreeViaExplainSegment {
                        layer: entry.second_intermediate_layer,
                        points: second_middle_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(second_middle_search.path.as_ref()?),
                    },
                    target_segment: RoutePathCandidateOrthogonalGraphThreeViaExplainSegment {
                        layer: to_anchor.layer,
                        points: target_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(target_search.path.as_ref()?),
                    },
                    selection_reason: format!(
                        "selected because it is the first matching authored via triple whose source-layer, first-intermediate-layer, second-intermediate-layer, and target-layer orthogonal graph searches all yield deterministic paths between layers {} and {}; each layer-side path is the lowest-cost graph path under the accepted ranking rule",
                        from_anchor.layer, to_anchor.layer
                    ),
                })
            },
        );

        let blocked_matching_triples = triple_searches
            .iter()
            .filter(
                |(_, source_search, first_middle_search, second_middle_search, target_search)| {
                    source_search.path.is_none()
                        || first_middle_search.path.is_none()
                        || second_middle_search.path.is_none()
                        || target_search.path.is_none()
                },
            )
            .map(
                |(entry, source_search, first_middle_search, second_middle_search, target_search)| {
                    RoutePathCandidateOrthogonalGraphThreeViaExplainBlockedTriple {
                        via_a_uuid: entry.via_a.uuid,
                        via_a_position: entry.via_a.position,
                        via_b_uuid: entry.via_b.uuid,
                        via_b_position: entry.via_b.position,
                        via_c_uuid: entry.via_c.uuid,
                        via_c_position: entry.via_c.position,
                        first_intermediate_layer: entry.first_intermediate_layer,
                        second_intermediate_layer: entry.second_intermediate_layer,
                        source_segment: source_search.path.clone().map(|points| {
                            let cost = orthogonal_graph_path_cost(&points);
                            RoutePathCandidateOrthogonalGraphThreeViaExplainSegment {
                                layer: from_anchor.layer,
                                points,
                                cost,
                            }
                        }),
                        first_middle_segment: first_middle_search.path.clone().map(|points| {
                            let cost = orthogonal_graph_path_cost(&points);
                            RoutePathCandidateOrthogonalGraphThreeViaExplainSegment {
                                layer: entry.first_intermediate_layer,
                                points,
                                cost,
                            }
                        }),
                        second_middle_segment: second_middle_search.path.clone().map(|points| {
                            let cost = orthogonal_graph_path_cost(&points);
                            RoutePathCandidateOrthogonalGraphThreeViaExplainSegment {
                                layer: entry.second_intermediate_layer,
                                points,
                                cost,
                            }
                        }),
                        target_segment: target_search.path.clone().map(|points| {
                            let cost = orthogonal_graph_path_cost(&points);
                            RoutePathCandidateOrthogonalGraphThreeViaExplainSegment {
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
                        first_middle_blockages: first_middle_search
                            .blocked_edges
                            .iter()
                            .flat_map(|edge| edge.blockages.clone())
                            .collect(),
                        second_middle_blockages: second_middle_search
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
                },
            )
            .collect::<Vec<_>>();

        Ok(RoutePathCandidateOrthogonalGraphThreeViaExplainReport {
            contract: "m5_route_path_candidate_orthogonal_graph_three_via_explain_v1".to_string(),
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
            summary: RoutePathCandidateOrthogonalGraphThreeViaExplainSummary {
                candidate_copper_layer_count: path_candidate.summary.candidate_copper_layer_count,
                candidate_via_count: path_candidate.summary.candidate_via_count,
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
    report: &RoutePathCandidateOrthogonalGraphThreeViaReport,
) -> RoutePathCandidateOrthogonalGraphThreeViaExplainKind {
    match report.status {
        RoutePathCandidateStatus::DeterministicPathFound => {
            RoutePathCandidateOrthogonalGraphThreeViaExplainKind::DeterministicPathFound
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
            if report.summary.matching_via_triple_count == 0 =>
        {
            RoutePathCandidateOrthogonalGraphThreeViaExplainKind::NoMatchingAuthoredViaTriple
        }
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints => {
            RoutePathCandidateOrthogonalGraphThreeViaExplainKind::AllMatchingViaTriplesBlocked
        }
    }
}
