use uuid::Uuid;

use crate::board::{Board, RoutePreflightAnchor, Via};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_via_selection::candidate_vias_for_net;
use super::route_segment_blockage::{RouteSegmentBlockageAnalysis, analyze_route_segment};

pub const ROUTE_PATH_CANDIDATE_TWO_VIA_SELECTION_RULE: &str = "select the first unblocked matching authored via pair in ascending (via_a_uuid, via_b_uuid) order whose layer sequence connects the requested anchor layers through one intermediate copper layer";

#[derive(Debug, Clone)]
pub(super) struct RoutePathCandidateTwoViaMatch {
    pub via_a: Via,
    pub via_b: Via,
    pub intermediate_layer: LayerId,
    pub source_segment: RouteSegmentBlockageAnalysis,
    pub middle_segment: RouteSegmentBlockageAnalysis,
    pub target_segment: RouteSegmentBlockageAnalysis,
}

pub(super) fn candidate_two_via_matches(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> (Vec<Via>, Vec<RoutePathCandidateTwoViaMatch>) {
    let candidate_vias = candidate_vias_for_net(board, net_uuid);
    let matching_pairs = candidate_vias
        .iter()
        .enumerate()
        .flat_map(|(via_a_index, via_a)| {
            candidate_vias
                .iter()
                .enumerate()
                .filter(move |(via_b_index, _)| *via_b_index != via_a_index)
                .filter_map(move |(_, via_b)| {
                    matching_two_via_pair(board, net_uuid, from_anchor, to_anchor, via_a, via_b)
                })
        })
        .collect::<Vec<_>>();

    (candidate_vias, matching_pairs)
}

fn matching_two_via_pair(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
    via_a: &Via,
    via_b: &Via,
) -> Option<RoutePathCandidateTwoViaMatch> {
    let intermediate_layer =
        intermediate_layer_for_pair(via_a, via_b, from_anchor.layer, to_anchor.layer)?;

    Some(RoutePathCandidateTwoViaMatch {
        via_a: via_a.clone(),
        via_b: via_b.clone(),
        intermediate_layer,
        source_segment: analyze_route_segment(
            board,
            net_uuid,
            from_anchor.layer,
            from_anchor.position,
            via_a.position,
            &format!(
                "source-to-via-a segment via {} on layer {}",
                via_a.uuid, from_anchor.layer
            ),
        ),
        middle_segment: analyze_route_segment(
            board,
            net_uuid,
            intermediate_layer,
            via_a.position,
            via_b.position,
            &format!(
                "via-a-to-via-b segment via {} to via {} on layer {}",
                via_a.uuid, via_b.uuid, intermediate_layer
            ),
        ),
        target_segment: analyze_route_segment(
            board,
            net_uuid,
            to_anchor.layer,
            via_b.position,
            to_anchor.position,
            &format!(
                "via-b-to-target segment via {} on layer {}",
                via_b.uuid, to_anchor.layer
            ),
        ),
    })
}

fn intermediate_layer_for_pair(
    via_a: &Via,
    via_b: &Via,
    from_layer: LayerId,
    to_layer: LayerId,
) -> Option<LayerId> {
    let via_a_other_layer = other_boundary_layer(via_a, from_layer)?;
    let via_b_other_layer = other_boundary_layer(via_b, to_layer)?;
    if via_a_other_layer == via_b_other_layer {
        Some(via_a_other_layer)
    } else {
        None
    }
}

fn other_boundary_layer(via: &Via, boundary_layer: LayerId) -> Option<LayerId> {
    if via.from_layer == boundary_layer {
        Some(via.to_layer)
    } else if via.to_layer == boundary_layer {
        Some(via.from_layer)
    } else {
        None
    }
}

pub(super) fn selected_matching_two_via(
    matches: &[RoutePathCandidateTwoViaMatch],
) -> Option<&RoutePathCandidateTwoViaMatch> {
    matches.iter().find(|entry| {
        entry.source_segment.blockages.is_empty()
            && entry.middle_segment.blockages.is_empty()
            && entry.target_segment.blockages.is_empty()
    })
}

pub(super) fn two_via_path_points(
    entry: &RoutePathCandidateTwoViaMatch,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> [[Point; 2]; 3] {
    [
        [from_anchor.position, entry.via_a.position],
        [entry.via_a.position, entry.via_b.position],
        [entry.via_b.position, to_anchor.position],
    ]
}
