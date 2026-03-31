use uuid::Uuid;

use crate::board::{Board, RoutePathCandidateError, RoutePathCandidateStatus, StackupLayer};

use super::route_path_candidate_orthogonal_graph_spine::{
    OrthogonalGraphLayerPathSpine, RoutePathCandidateOrthogonalGraphPath,
    build_orthogonal_graph_layer_path_spine,
};
use super::route_path_candidate_orthogonal_graph_two_via::{
    RoutePathCandidateOrthogonalGraphTwoViaPath, RoutePathCandidateOrthogonalGraphTwoViaSegment,
    RoutePathCandidateOrthogonalGraphTwoViaSummary,
};
use super::route_path_candidate_two_via_selection::{
    ROUTE_PATH_CANDIDATE_TWO_VIA_SELECTION_RULE, RoutePathCandidateTwoViaMatch,
    candidate_two_via_matches,
};

pub(super) const ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_TWO_VIA_SELECTION_RULE: &str = "select the first matching authored via pair in ascending (via_a_uuid, via_b_uuid) order whose source-layer, intermediate-layer, and target-layer persisted-coordinate orthogonal graph searches all yield deterministic paths; each side reuses the same deterministic graph-search rule as the same-layer orthogonal graph candidate";

#[derive(Debug, Clone)]
pub(super) struct OrthogonalGraphTwoViaCandidateSearch {
    pub entry: RoutePathCandidateTwoViaMatch,
    pub source: OrthogonalGraphLayerPathSpine,
    pub middle: OrthogonalGraphLayerPathSpine,
    pub target: OrthogonalGraphLayerPathSpine,
}

#[derive(Debug, Clone)]
pub(super) struct OrthogonalGraphTwoViaCandidateSpine {
    pub net_uuid: Uuid,
    pub net_name: String,
    pub from_anchor_pad_uuid: Uuid,
    pub to_anchor_pad_uuid: Uuid,
    pub selection_rule: String,
    pub component_selection_rules: Vec<String>,
    pub status: RoutePathCandidateStatus,
    pub candidate_copper_layers: Vec<StackupLayer>,
    pub summary: RoutePathCandidateOrthogonalGraphTwoViaSummary,
    pub path: Option<RoutePathCandidateOrthogonalGraphTwoViaPath>,
    pub pair_searches: Vec<OrthogonalGraphTwoViaCandidateSearch>,
}

pub(super) fn build_orthogonal_graph_two_via_candidate_spine(
    board: &Board,
    net_uuid: Uuid,
    from_anchor_pad_uuid: Uuid,
    to_anchor_pad_uuid: Uuid,
) -> Result<OrthogonalGraphTwoViaCandidateSpine, RoutePathCandidateError> {
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

    let (candidate_vias, matching_pairs) =
        candidate_two_via_matches(board, net_uuid, from_anchor, to_anchor);
    let pair_searches = matching_pairs
        .iter()
        .map(|entry| OrthogonalGraphTwoViaCandidateSearch {
            entry: entry.clone(),
            source: build_orthogonal_graph_layer_path_spine(
                board,
                net_uuid,
                from_anchor.layer,
                from_anchor.position,
                entry.via_a.position,
            ),
            middle: build_orthogonal_graph_layer_path_spine(
                board,
                net_uuid,
                entry.intermediate_layer,
                entry.via_a.position,
                entry.via_b.position,
            ),
            target: build_orthogonal_graph_layer_path_spine(
                board,
                net_uuid,
                to_anchor.layer,
                entry.via_b.position,
                to_anchor.position,
            ),
        })
        .collect::<Vec<_>>();

    let blocked_via_pair_count = pair_searches
        .iter()
        .filter(|search| {
            search.source.path.is_none()
                || search.middle.path.is_none()
                || search.target.path.is_none()
        })
        .count();
    let available_via_pair_count = pair_searches.len().saturating_sub(blocked_via_pair_count);
    let path = pair_searches.iter().find_map(|search| {
        Some(RoutePathCandidateOrthogonalGraphTwoViaPath {
            via_a_uuid: search.entry.via_a.uuid,
            via_a_position: search.entry.via_a.position,
            via_b_uuid: search.entry.via_b.uuid,
            via_b_position: search.entry.via_b.position,
            intermediate_layer: search.entry.intermediate_layer,
            segments: vec![
                segment_from_path(search.source.path.as_ref()?),
                segment_from_path(search.middle.path.as_ref()?),
                segment_from_path(search.target.path.as_ref()?),
            ],
        })
    });
    let status = if path.is_some() {
        RoutePathCandidateStatus::DeterministicPathFound
    } else {
        RoutePathCandidateStatus::NoPathUnderCurrentAuthoredConstraints
    };

    Ok(OrthogonalGraphTwoViaCandidateSpine {
        net_uuid: preflight.net_uuid,
        net_name: preflight.net_name,
        from_anchor_pad_uuid,
        to_anchor_pad_uuid,
        selection_rule: ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_TWO_VIA_SELECTION_RULE.to_string(),
        component_selection_rules: vec![
            ROUTE_PATH_CANDIDATE_TWO_VIA_SELECTION_RULE.to_string(),
            super::route_path_candidate_orthogonal_graph_selection::ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SELECTION_RULE.to_string(),
        ],
        status,
        candidate_copper_layers: preflight.candidate_copper_layers.clone(),
        summary: RoutePathCandidateOrthogonalGraphTwoViaSummary {
            candidate_copper_layer_count: preflight.candidate_copper_layers.len(),
            candidate_via_count: candidate_vias.len(),
            candidate_via_pair_count: candidate_vias
                .len()
                .saturating_mul(candidate_vias.len().saturating_sub(1)),
            matching_via_pair_count: matching_pairs.len(),
            blocked_via_pair_count,
            available_via_pair_count,
            path_segment_count: path.as_ref().map(|path| path.segments.len()).unwrap_or(0),
        },
        path,
        pair_searches,
    })
}

fn segment_from_path(
    path: &RoutePathCandidateOrthogonalGraphPath,
) -> RoutePathCandidateOrthogonalGraphTwoViaSegment {
    RoutePathCandidateOrthogonalGraphTwoViaSegment {
        layer: path.layer,
        points: path.points.clone(),
        cost: path.cost.clone(),
    }
}
