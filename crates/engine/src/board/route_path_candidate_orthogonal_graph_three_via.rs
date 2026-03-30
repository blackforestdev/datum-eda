use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_selection::{
    RoutePathCandidateOrthogonalGraphPathCost, orthogonal_graph_path_cost,
    search_orthogonal_graph_layer,
};
use super::route_path_candidate_three_via_selection::{
    ROUTE_PATH_CANDIDATE_THREE_VIA_SELECTION_RULE, candidate_three_via_matches,
};

const ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_THREE_VIA_SELECTION_RULE: &str = "select the first matching authored via triple in ascending (via_a_uuid, via_b_uuid, via_c_uuid) order whose source-layer, first-intermediate-layer, second-intermediate-layer, and target-layer persisted-coordinate orthogonal graph searches all yield deterministic paths; each side reuses the same deterministic graph-search rule as the same-layer orthogonal graph candidate";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphThreeViaSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub cost: RoutePathCandidateOrthogonalGraphPathCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphThreeViaPath {
    pub via_a_uuid: Uuid,
    pub via_a_position: Point,
    pub via_b_uuid: Uuid,
    pub via_b_position: Point,
    pub via_c_uuid: Uuid,
    pub via_c_position: Point,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub segments: Vec<RoutePathCandidateOrthogonalGraphThreeViaSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphThreeViaSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub candidate_via_triple_count: usize,
    pub matching_via_triple_count: usize,
    pub blocked_via_triple_count: usize,
    pub available_via_triple_count: usize,
    pub path_segment_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphThreeViaReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub component_selection_rules: Vec<String>,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphThreeViaSummary,
    pub path: Option<RoutePathCandidateOrthogonalGraphThreeViaPath>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph_three_via(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphThreeViaReport, RoutePathCandidateError> {
        if from_anchor_pad_uuid == to_anchor_pad_uuid {
            return Err(RoutePathCandidateError::DuplicateAnchorPair {
                pad_uuid: from_anchor_pad_uuid,
            });
        }

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

        let (candidate_vias, matching_triples) =
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

        let blocked_via_triple_count = triple_searches
            .iter()
            .filter(
                |(_, source_search, first_middle_search, second_middle_search, target_search)| {
                    source_search.path.is_none()
                        || first_middle_search.path.is_none()
                        || second_middle_search.path.is_none()
                        || target_search.path.is_none()
                },
            )
            .count();
        let available_via_triple_count = triple_searches
            .len()
            .saturating_sub(blocked_via_triple_count);
        let path = triple_searches.iter().find_map(
            |(entry, source_search, first_middle_search, second_middle_search, target_search)| {
                Some(RoutePathCandidateOrthogonalGraphThreeViaPath {
                    via_a_uuid: entry.via_a.uuid,
                    via_a_position: entry.via_a.position,
                    via_b_uuid: entry.via_b.uuid,
                    via_b_position: entry.via_b.position,
                    via_c_uuid: entry.via_c.uuid,
                    via_c_position: entry.via_c.position,
                    first_intermediate_layer: entry.first_intermediate_layer,
                    second_intermediate_layer: entry.second_intermediate_layer,
                    segments: vec![
                        RoutePathCandidateOrthogonalGraphThreeViaSegment {
                            layer: from_anchor.layer,
                            points: source_search.path.clone()?,
                            cost: orthogonal_graph_path_cost(source_search.path.as_ref()?),
                        },
                        RoutePathCandidateOrthogonalGraphThreeViaSegment {
                            layer: entry.first_intermediate_layer,
                            points: first_middle_search.path.clone()?,
                            cost: orthogonal_graph_path_cost(first_middle_search.path.as_ref()?),
                        },
                        RoutePathCandidateOrthogonalGraphThreeViaSegment {
                            layer: entry.second_intermediate_layer,
                            points: second_middle_search.path.clone()?,
                            cost: orthogonal_graph_path_cost(second_middle_search.path.as_ref()?),
                        },
                        RoutePathCandidateOrthogonalGraphThreeViaSegment {
                            layer: to_anchor.layer,
                            points: target_search.path.clone()?,
                            cost: orthogonal_graph_path_cost(target_search.path.as_ref()?),
                        },
                    ],
                })
            },
        );
        let status = if path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateOrthogonalGraphThreeViaReport {
            contract: "m5_route_path_candidate_orthogonal_graph_three_via_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule:
                ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_THREE_VIA_SELECTION_RULE.to_string(),
            component_selection_rules: vec![
                ROUTE_PATH_CANDIDATE_THREE_VIA_SELECTION_RULE.to_string(),
                super::route_path_candidate_orthogonal_graph_selection::ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SELECTION_RULE.to_string(),
            ],
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers: preflight.candidate_copper_layers.clone(),
            summary: RoutePathCandidateOrthogonalGraphThreeViaSummary {
                candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
                candidate_via_count: candidate_vias.len(),
                candidate_via_triple_count: permutation_count(candidate_vias.len(), 3),
                matching_via_triple_count: matching_triples.len(),
                blocked_via_triple_count,
                available_via_triple_count,
                path_segment_count: path.as_ref().map(|path| path.segments.len()).unwrap_or(0),
            },
            path,
        })
    }
}

fn permutation_count(n: usize, pick: usize) -> usize {
    (0..pick).fold(1usize, |acc, index| {
        acc.saturating_mul(n.saturating_sub(index))
    })
}
