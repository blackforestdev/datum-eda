use uuid::Uuid;

use crate::board::{Board, RoutePreflightAnchor, Via};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_via_selection::candidate_vias_for_net;
use super::route_segment_blockage::{RouteSegmentBlockageAnalysis, analyze_route_segment};

pub const ROUTE_PATH_CANDIDATE_THREE_VIA_SELECTION_RULE: &str = "select the first unblocked matching authored via triple in ascending (via_a_uuid, via_b_uuid, via_c_uuid) order whose layer sequence connects the requested anchor layers through two intermediate copper layers";

#[derive(Debug, Clone)]
pub(super) struct RoutePathCandidateThreeViaMatch {
    pub via_a: Via,
    pub via_b: Via,
    pub via_c: Via,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub source_segment: RouteSegmentBlockageAnalysis,
    pub first_middle_segment: RouteSegmentBlockageAnalysis,
    pub second_middle_segment: RouteSegmentBlockageAnalysis,
    pub target_segment: RouteSegmentBlockageAnalysis,
}

pub(super) fn candidate_three_via_matches(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> (Vec<Via>, Vec<RoutePathCandidateThreeViaMatch>) {
    let candidate_vias = candidate_vias_for_net(board, net_uuid);
    let mut matching_triples = Vec::new();
    for (via_a_index, via_a) in candidate_vias.iter().enumerate() {
        for (via_b_index, via_b) in candidate_vias.iter().enumerate() {
            if via_b_index == via_a_index {
                continue;
            }
            for (via_c_index, via_c) in candidate_vias.iter().enumerate() {
                if via_c_index == via_a_index || via_c_index == via_b_index {
                    continue;
                }
                if let Some(entry) = matching_three_via_triple(
                    board,
                    net_uuid,
                    from_anchor,
                    to_anchor,
                    via_a,
                    via_b,
                    via_c,
                ) {
                    matching_triples.push(entry);
                }
            }
        }
    }

    (candidate_vias, matching_triples)
}

fn matching_three_via_triple(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
    via_a: &Via,
    via_b: &Via,
    via_c: &Via,
) -> Option<RoutePathCandidateThreeViaMatch> {
    let (first_intermediate_layer, second_intermediate_layer) =
        intermediate_layers_for_triple(via_a, via_b, via_c, from_anchor.layer, to_anchor.layer)?;

    Some(RoutePathCandidateThreeViaMatch {
        via_a: via_a.clone(),
        via_b: via_b.clone(),
        via_c: via_c.clone(),
        first_intermediate_layer,
        second_intermediate_layer,
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
        first_middle_segment: analyze_route_segment(
            board,
            net_uuid,
            first_intermediate_layer,
            via_a.position,
            via_b.position,
            &format!(
                "via-a-to-via-b segment via {} to via {} on layer {}",
                via_a.uuid, via_b.uuid, first_intermediate_layer
            ),
        ),
        second_middle_segment: analyze_route_segment(
            board,
            net_uuid,
            second_intermediate_layer,
            via_b.position,
            via_c.position,
            &format!(
                "via-b-to-via-c segment via {} to via {} on layer {}",
                via_b.uuid, via_c.uuid, second_intermediate_layer
            ),
        ),
        target_segment: analyze_route_segment(
            board,
            net_uuid,
            to_anchor.layer,
            via_c.position,
            to_anchor.position,
            &format!(
                "via-c-to-target segment via {} on layer {}",
                via_c.uuid, to_anchor.layer
            ),
        ),
    })
}

fn intermediate_layers_for_triple(
    via_a: &Via,
    via_b: &Via,
    via_c: &Via,
    from_layer: LayerId,
    to_layer: LayerId,
) -> Option<(LayerId, LayerId)> {
    let first_intermediate_layer = other_boundary_layer(via_a, from_layer)?;
    let second_intermediate_layer = other_boundary_layer_for_middle_via(via_b, first_intermediate_layer)?;
    if second_intermediate_layer == first_intermediate_layer {
        return None;
    }
    let via_c_other_layer = other_boundary_layer(via_c, to_layer)?;
    if via_c_other_layer == second_intermediate_layer {
        Some((first_intermediate_layer, second_intermediate_layer))
    } else {
        None
    }
}

fn other_boundary_layer_for_middle_via(via: &Via, connected_layer: LayerId) -> Option<LayerId> {
    if via.from_layer == connected_layer && via.to_layer != connected_layer {
        Some(via.to_layer)
    } else if via.to_layer == connected_layer && via.from_layer != connected_layer {
        Some(via.from_layer)
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

pub(super) fn selected_matching_three_via(
    matches: &[RoutePathCandidateThreeViaMatch],
) -> Option<&RoutePathCandidateThreeViaMatch> {
    matches.iter().find(|entry| {
        entry.source_segment.blockages.is_empty()
            && entry.first_middle_segment.blockages.is_empty()
            && entry.second_middle_segment.blockages.is_empty()
            && entry.target_segment.blockages.is_empty()
    })
}

pub(super) fn three_via_path_points(
    entry: &RoutePathCandidateThreeViaMatch,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> [[Point; 2]; 4] {
    [
        [from_anchor.position, entry.via_a.position],
        [entry.via_a.position, entry.via_b.position],
        [entry.via_b.position, entry.via_c.position],
        [entry.via_c.position, to_anchor.position],
    ]
}
