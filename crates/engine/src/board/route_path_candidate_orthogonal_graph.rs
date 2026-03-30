use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_selection::{
    ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SELECTION_RULE,
    candidate_orthogonal_graph_layer_searches, selected_orthogonal_graph_path,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphPath {
    pub layer: LayerId,
    pub points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSummary {
    pub candidate_copper_layer_count: usize,
    pub graph_node_count: usize,
    pub graph_edge_count: usize,
    pub blocked_edge_count: usize,
    pub path_point_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphReport {
    pub contract: String,
    pub persisted_native_board_state_only: bool,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphSummary,
    pub path: Option<RoutePathCandidateOrthogonalGraphPath>,
}

impl Board {
    pub fn route_path_candidate_orthogonal_graph(
        &self,
        net_uuid: Uuid,
        from_anchor_pad_uuid: Uuid,
        to_anchor_pad_uuid: Uuid,
    ) -> Result<RoutePathCandidateOrthogonalGraphReport, RoutePathCandidateError> {
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

        let candidate_copper_layers = preflight
            .candidate_copper_layers
            .iter()
            .filter(|layer| layer.id == from_anchor.layer && layer.id == to_anchor.layer)
            .cloned()
            .collect::<Vec<_>>();
        let searches = candidate_orthogonal_graph_layer_searches(
            self,
            net_uuid,
            from_anchor,
            to_anchor,
            &candidate_copper_layers,
        );
        let graph_node_count = searches.iter().map(|search| search.node_count).sum();
        let graph_edge_count = searches.iter().map(|search| search.edge_count).sum();
        let blocked_edge_count = searches.iter().map(|search| search.blocked_edges.len()).sum();
        let selected = selected_orthogonal_graph_path(&searches);
        let path = selected.and_then(|search| {
            search.path.as_ref().map(|points| RoutePathCandidateOrthogonalGraphPath {
                layer: search.layer,
                points: points.clone(),
            })
        });
        let path_point_count = path.as_ref().map(|path| path.points.len()).unwrap_or(0);
        let status = if path.is_some() {
            RoutePathCandidateStatus::DeterministicPathFound
        } else {
            RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
        };

        Ok(RoutePathCandidateOrthogonalGraphReport {
            contract: "m5_route_path_candidate_orthogonal_graph_v1".to_string(),
            persisted_native_board_state_only: true,
            selection_rule: ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SELECTION_RULE.to_string(),
            status,
            net_uuid: preflight.net_uuid,
            net_name: preflight.net_name,
            from_anchor_pad_uuid,
            to_anchor_pad_uuid,
            candidate_copper_layers,
            summary: RoutePathCandidateOrthogonalGraphSummary {
                candidate_copper_layer_count: preflight
                    .candidate_copper_layers
                    .iter()
                    .filter(|layer| layer.id == from_anchor.layer && layer.id == to_anchor.layer)
                    .count(),
                graph_node_count,
                graph_edge_count,
                blocked_edge_count,
                path_point_count,
            },
            path,
        })
    }
}
