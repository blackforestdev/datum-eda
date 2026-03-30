use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_selection::{
    RoutePathCandidateOrthogonalGraphPathCost, orthogonal_graph_path_cost,
    search_orthogonal_graph_layer,
};
use super::route_path_candidate_six_via_selection::{
    ROUTE_PATH_CANDIDATE_SIX_VIA_SELECTION_RULE, candidate_six_via_matches,
};

const ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SIX_VIA_SELECTION_RULE: &str = "select the first matching authored via sextuple in ascending (via_a_uuid, via_b_uuid, via_c_uuid, via_d_uuid, via_e_uuid, via_f_uuid) order whose source-layer, first-intermediate-layer, second-intermediate-layer, third-intermediate-layer, fourth-intermediate-layer, fifth-intermediate-layer, and target-layer persisted-coordinate orthogonal graph searches all yield deterministic paths; each side reuses the same deterministic graph-search rule as the same-layer orthogonal graph candidate";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSixViaSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub cost: RoutePathCandidateOrthogonalGraphPathCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSixViaPath {
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
    pub segments: Vec<RoutePathCandidateOrthogonalGraphSixViaSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSixViaSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub candidate_via_sextuple_count: usize,
    pub matching_via_sextuple_count: usize,
    pub blocked_via_sextuple_count: usize,
    pub available_via_sextuple_count: usize,
    pub path_segment_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSixViaReport {
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
    pub summary: RoutePathCandidateOrthogonalGraphSixViaSummary,
    pub path: Option<RoutePathCandidateOrthogonalGraphSixViaPath>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph_six_via(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphSixViaReport, RoutePathCandidateError> {
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

        let (candidate_vias, matching_sextuples) =
            candidate_six_via_matches(self, net_uuid, from_anchor, to_anchor);
        let sextuple_searches = matching_sextuples
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
                let third_middle_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    entry.third_intermediate_layer,
                    entry.via_c.position,
                    entry.via_d.position,
                );
                let fourth_middle_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    entry.fourth_intermediate_layer,
                    entry.via_d.position,
                    entry.via_e.position,
                );
                let fifth_middle_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    entry.fifth_intermediate_layer,
                    entry.via_e.position,
                    entry.via_f.position,
                );
                let target_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    to_anchor.layer,
                    entry.via_f.position,
                    to_anchor.position,
                );
                (
                    entry,
                    source_search,
                    first_middle_search,
                    second_middle_search,
                    third_middle_search,
                    fourth_middle_search,
                    fifth_middle_search,
                    target_search,
                )
            })
            .collect::<Vec<_>>();

        let blocked_via_sextuple_count = sextuple_searches
            .iter()
            .filter(|(_, source_search, first_middle_search, second_middle_search, third_middle_search, fourth_middle_search, fifth_middle_search, target_search)| {
                source_search.path.is_none()
                    || first_middle_search.path.is_none()
                    || second_middle_search.path.is_none()
                    || third_middle_search.path.is_none()
                    || fourth_middle_search.path.is_none()
                    || fifth_middle_search.path.is_none()
                    || target_search.path.is_none()
            })
            .count();
        let available_via_sextuple_count =
            sextuple_searches.len().saturating_sub(blocked_via_sextuple_count);
        let path = sextuple_searches.iter().find_map(
            |(
                entry,
                source_search,
                first_middle_search,
                second_middle_search,
                third_middle_search,
                fourth_middle_search,
                fifth_middle_search,
                target_search,
            )| {
                Some(RoutePathCandidateOrthogonalGraphSixViaPath {
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
                    segments: vec![
                        RoutePathCandidateOrthogonalGraphSixViaSegment {
                            layer: from_anchor.layer,
                            points: source_search.path.clone()?,
                            cost: orthogonal_graph_path_cost(source_search.path.as_ref()?),
                        },
                        RoutePathCandidateOrthogonalGraphSixViaSegment {
                            layer: entry.first_intermediate_layer,
                            points: first_middle_search.path.clone()?,
                            cost: orthogonal_graph_path_cost(first_middle_search.path.as_ref()?),
                        },
                        RoutePathCandidateOrthogonalGraphSixViaSegment {
                            layer: entry.second_intermediate_layer,
                            points: second_middle_search.path.clone()?,
                            cost: orthogonal_graph_path_cost(second_middle_search.path.as_ref()?),
                        },
                        RoutePathCandidateOrthogonalGraphSixViaSegment {
                            layer: entry.third_intermediate_layer,
                            points: third_middle_search.path.clone()?,
                            cost: orthogonal_graph_path_cost(third_middle_search.path.as_ref()?),
                        },
                        RoutePathCandidateOrthogonalGraphSixViaSegment {
                            layer: entry.fourth_intermediate_layer,
                            points: fourth_middle_search.path.clone()?,
                            cost: orthogonal_graph_path_cost(fourth_middle_search.path.as_ref()?),
                        },
                        RoutePathCandidateOrthogonalGraphSixViaSegment {
                            layer: entry.fifth_intermediate_layer,
                            points: fifth_middle_search.path.clone()?,
                            cost: orthogonal_graph_path_cost(fifth_middle_search.path.as_ref()?),
                        },
                        RoutePathCandidateOrthogonalGraphSixViaSegment {
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

        Ok(RoutePathCandidateOrthogonalGraphSixViaReport {
            contract: "m5_route_path_candidate_orthogonal_graph_six_via_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule:
                ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SIX_VIA_SELECTION_RULE.to_string(),
            component_selection_rules: vec![
                ROUTE_PATH_CANDIDATE_SIX_VIA_SELECTION_RULE.to_string(),
                super::route_path_candidate_orthogonal_graph_selection::ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SELECTION_RULE.to_string(),
            ],
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers: preflight.candidate_copper_layers.clone(),
            summary: RoutePathCandidateOrthogonalGraphSixViaSummary {
                candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
                candidate_via_count: candidate_vias.len(),
                candidate_via_sextuple_count: permutation_count(candidate_vias.len(), 6),
                matching_via_sextuple_count: matching_sextuples.len(),
                blocked_via_sextuple_count,
                available_via_sextuple_count,
                path_segment_count: path.as_ref().map(|path| path.segments.len()).unwrap_or(0),
            },
            path,
        })
    }
}

fn permutation_count(n: usize, pick: usize) -> usize {
    (0..pick).fold(1usize, |acc, index| acc.saturating_mul(n.saturating_sub(index)))
}
