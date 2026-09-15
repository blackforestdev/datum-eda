use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer, Via};

use super::route_path_candidate_orthogonal_graph_spine::{
    OrthogonalGraphLayerPathSpine, RoutePathCandidateOrthogonalGraphPath,
    build_orthogonal_graph_layer_path_spine,
};
use super::route_path_candidate_orthogonal_graph_via::{
    RoutePathCandidateOrthogonalGraphViaPath, RoutePathCandidateOrthogonalGraphViaSegment,
    RoutePathCandidateOrthogonalGraphViaSummary,
};
use super::route_path_candidate_via_selection::{
    ROUTE_PATH_CANDIDATE_VIA_SELECTION_RULE, candidate_vias_for_net, via_matches_anchor_layers,
};

pub(super) const ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA_SELECTION_RULE: &str = "select the first authored target-net via in ascending via UUID order whose boundary layers exactly match the requested anchor layers and whose source-layer and target-layer persisted-coordinate orthogonal graph searches both yield deterministic paths; each side reuses the same deterministic graph-search rule as the same-layer orthogonal graph candidate";

#[derive(Debug, Clone)]
pub(super) struct OrthogonalGraphViaCandidateSearch {
    pub via: Via,
    pub source: OrthogonalGraphLayerPathSpine,
    pub target: OrthogonalGraphLayerPathSpine,
}

#[derive(Debug, Clone)]
pub(super) struct OrthogonalGraphViaCandidateSpine {
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub component_selection_rules: Vec<String>,
    pub status: RoutePathCandidateStatus,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphViaSummary,
    pub path: Option<RoutePathCandidateOrthogonalGraphViaPath>,
    pub via_searches: Vec<OrthogonalGraphViaCandidateSearch>,
}

pub(super) fn build_orthogonal_graph_via_candidate_spine(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<OrthogonalGraphViaCandidateSpine, RoutePathCandidateError> {
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

    let candidate_vias = candidate_vias_for_net(board, net_uuid);
    let matching_vias = candidate_vias
        .iter()
        .filter(|via| via_matches_anchor_layers(via, from_anchor.layer, to_anchor.layer))
        .cloned()
        .collect::<Vec<_>>();

    let via_searches = matching_vias
        .iter()
        .map(|via| OrthogonalGraphViaCandidateSearch {
            via: via.clone(),
            source: build_orthogonal_graph_layer_path_spine(
                board,
                net_uuid,
                from_anchor.layer,
                from_anchor.position,
                via.position,
            ),
            target: build_orthogonal_graph_layer_path_spine(
                board,
                net_uuid,
                to_anchor.layer,
                via.position,
                to_anchor.position,
            ),
        })
        .collect::<Vec<_>>();

    let blocked_via_count = via_searches
        .iter()
        .filter(|search| search.source.path.is_none() || search.target.path.is_none())
        .count();
    let available_via_count = via_searches.len().saturating_sub(blocked_via_count);
    let path = via_searches.iter().find_map(|search| {
        Some(RoutePathCandidateOrthogonalGraphViaPath {
            via_uuid: search.via.uuid,
            via_position: search.via.position,
            segments: vec![
                segment_from_path(search.source.path.as_ref()?),
                segment_from_path(search.target.path.as_ref()?),
            ],
        })
    });
    let status = if path.is_some() {
        RoutePathCandidateStatus::DeterministicPathFound
    } else {
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    };

    Ok(OrthogonalGraphViaCandidateSpine {
        net_uuid: preflight.net_uuid,
        net_name: preflight.net_name,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        selection_rule: ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_VIA_SELECTION_RULE.to_string(),
        component_selection_rules: vec![
            ROUTE_PATH_CANDIDATE_VIA_SELECTION_RULE.to_string(),
            super::route_path_candidate_orthogonal_graph_selection::ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SELECTION_RULE.to_string(),
        ],
        status,
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
        via_searches,
    })
}

fn segment_from_path(
    path: &RoutePathCandidateOrthogonalGraphPath,
) -> RoutePathCandidateOrthogonalGraphViaSegment {
    RoutePathCandidateOrthogonalGraphViaSegment {
        layer: path.layer,
        points: path.points.clone(),
        cost: path.cost.clone(),
    }
}
