use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_orthogonal_graph_selection::{
    OrthogonalGraphLayerSearch, ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SELECTION_RULE,
    RoutePathCandidateOrthogonalGraphPathCost, candidate_orthogonal_graph_layer_searches,
    orthogonal_graph_path_cost, search_orthogonal_graph_layer, selected_orthogonal_graph_path,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphPath {
    pub layer: LayerId,
    pub points: Vec<Point>,
    pub cost: RoutePathCandidateOrthogonalGraphPathCost,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSegmentEvidence {
    pub layer_segment_index: usize,
    pub layer_segment_count: usize,
    pub layer: LayerId,
    pub bend_count: usize,
    pub point_count: usize,
    pub track_action_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphSummary {
    pub candidate_copper_layer_count: usize,
    pub graph_node_count: usize,
    pub graph_edge_count: usize,
    pub blocked_edge_count: usize,
    pub path_point_count: usize,
}

#[derive(Debug, Clone)]
pub(super) struct OrthogonalGraphCandidateSpine {
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub status: RoutePathCandidateStatus,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphSummary,
    pub path: Option<RoutePathCandidateOrthogonalGraphPath>,
    pub segment_evidence: Vec<RoutePathCandidateOrthogonalGraphSegmentEvidence>,
    pub searches: Vec<OrthogonalGraphLayerSearch>,
}

#[derive(Debug, Clone)]
pub(super) struct OrthogonalGraphLayerPathSpine {
    pub search: OrthogonalGraphLayerSearch,
    pub path: Option<RoutePathCandidateOrthogonalGraphPath>,
}

pub(super) fn build_orthogonal_graph_layer_path_spine(
    board: &Board,
    net_uuid: Uuid,
    layer: LayerId,
    from: Point,
    to: Point,
) -> OrthogonalGraphLayerPathSpine {
    let search = search_orthogonal_graph_layer(board, net_uuid, layer, from, to);
    let path = search
        .path
        .as_ref()
        .map(|points| RoutePathCandidateOrthogonalGraphPath {
            layer: search.layer,
            points: points.clone(),
            cost: orthogonal_graph_path_cost(points),
        });
    OrthogonalGraphLayerPathSpine { search, path }
}

pub(super) fn build_orthogonal_graph_candidate_spine(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<OrthogonalGraphCandidateSpine, RoutePathCandidateError> {
    if from_anchor_pad_uuid == to_anchor_pad_uuid {
        return Err(RoutePathCandidateError::DuplicateAnchorPair {
            pad_uuid: from_anchor_pad_uuid,
        });
    }

    let preflight = board
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
        board,
        net_uuid,
        from_anchor,
        to_anchor,
        &candidate_copper_layers,
    );
    let graph_node_count = searches.iter().map(|search| search.node_count).sum();
    let graph_edge_count = searches.iter().map(|search| search.edge_count).sum();
    let blocked_edge_count = searches
        .iter()
        .map(|search| search.blocked_edges.len())
        .sum();
    let selected = selected_orthogonal_graph_path(&searches);
    let path = selected.and_then(|search| {
        search
            .path
            .as_ref()
            .map(|points| RoutePathCandidateOrthogonalGraphPath {
                layer: search.layer,
                points: points.clone(),
                cost: orthogonal_graph_path_cost(points),
            })
    });
    let path_point_count = path.as_ref().map(|path| path.points.len()).unwrap_or(0);
    let segment_evidence = path
        .as_ref()
        .map(|path| {
            vec![RoutePathCandidateOrthogonalGraphSegmentEvidence {
                layer_segment_index: 0,
                layer_segment_count: 1,
                layer: path.layer,
                bend_count: path.cost.bend_count,
                point_count: path.points.len(),
                track_action_count: path.points.len().saturating_sub(1),
            }]
        })
        .unwrap_or_default();
    let status = if path.is_some() {
        RoutePathCandidateStatus::DeterministicPathFound
    } else {
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    };

    Ok(OrthogonalGraphCandidateSpine {
        net_uuid: preflight.net_uuid,
        net_name: preflight.net_name,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        selection_rule: ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SELECTION_RULE.to_string(),
        status,
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
        segment_evidence,
        searches,
    })
}
