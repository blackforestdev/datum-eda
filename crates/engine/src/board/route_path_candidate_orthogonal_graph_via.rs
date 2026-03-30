use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_selection::{
    RoutePathCandidateOrthogonalGraphPathCost, orthogonal_graph_path_cost,
    search_orthogonal_graph_layer,
};
use super::route_path_candidate_via_selection::{
    ROUTE_PATH_CANDIDATE_VIA_SELECTION_RULE, candidate_vias_for_net, via_matches_anchor_layers,
};

const ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA_SELECTION_RULE: &str = "select the first authored target-net via in ascending via UUID order whose boundary layers exactly match the requested anchor layers and whose source-layer and target-layer persisted-coordinate orthogonal graph searches both yield deterministic paths; each side reuses the same deterministic graph-search rule as the same-layer orthogonal graph candidate";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaSegment {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub cost: RoutePathCandidateOrthogonalGraphPathCost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaPath {
    pub via_uuid: Uuid,
    pub via_position: Point,
    pub segments: Vec<RoutePathCandidateOrthogonalGraphViaSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaSummary {
    pub candidate_copper_layer_count: usize,
    pub candidate_via_count: usize,
    pub matching_via_count: usize,
    pub blocked_via_count: usize,
    pub available_via_count: usize,
    pub path_segment_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphViaReport {
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
    pub summary: RoutePathCandidateOrthogonalGraphViaSummary,
    pub path: Option<RoutePathCandidateOrthogonalGraphViaPath>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph_via(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphViaReport, RoutePathCandidateError> {
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

        let candidate_vias = candidate_vias_for_net(self, net_uuid);
        let matching_vias = candidate_vias
            .iter()
            .filter(|via| via_matches_anchor_layers(via, from_anchor.layer, to_anchor.layer))
            .cloned()
            .collect::<Vec<_>>();

        let via_searches = matching_vias
            .iter()
            .map(|via| {
                let source_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    from_anchor.layer,
                    from_anchor.position,
                    via.position,
                );
                let target_search = search_orthogonal_graph_layer(
                    self,
                    net_uuid,
                    to_anchor.layer,
                    via.position,
                    to_anchor.position,
                );
                (via.clone(), source_search, target_search)
            })
            .collect::<Vec<_>>();

        let blocked_via_count = via_searches
            .iter()
            .filter(|(_, source_search, target_search)| {
                source_search.path.is_none() || target_search.path.is_none()
            })
            .count();
        let available_via_count = via_searches.len().saturating_sub(blocked_via_count);
        let path = via_searches.iter().find_map(|(via, source_search, target_search)| {
            Some(RoutePathCandidateOrthogonalGraphViaPath {
                via_uuid: via.uuid,
                via_position: via.position,
                segments: vec![
                    RoutePathCandidateOrthogonalGraphViaSegment {
                        layer: from_anchor.layer,
                        points: source_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(source_search.path.as_ref()?),
                    },
                    RoutePathCandidateOrthogonalGraphViaSegment {
                        layer: to_anchor.layer,
                        points: target_search.path.clone()?,
                        cost: orthogonal_graph_path_cost(target_search.path.as_ref()?),
                    },
                ],
            })
        });
        let status = if path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateOrthogonalGraphViaReport {
            contract: "m5_route_path_candidate_orthogonal_graph_via_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA_SELECTION_RULE.to_string(),
            component_selection_rules: vec![
                ROUTE_PATH_CANDIDATE_VIA_SELECTION_RULE.to_string(),
                super::route_path_candidate_orthogonal_graph_selection::ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SELECTION_RULE.to_string(),
            ],
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers: preflight.candidate_copper_layers.clone(),
            summary: RoutePathCandidateOrthogonalGraphViaSummary {
                candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
                candidate_via_count: candidate_vias.len(),
                matching_via_count: matching_vias.len(),
                blocked_via_count,
                available_via_count,
                path_segment_count: path.as_ref().map(|path| path.segments.len()).unwrap_or(0),
            },
            path,
        })
    }
}
