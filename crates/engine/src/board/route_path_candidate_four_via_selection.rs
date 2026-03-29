use uuid::Uuid;

use crate::board::{Board, RoutePreflightAnchor, Via};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_via_selection::candidate_vias_for_net;
use super::route_segment_blockage::{RouteSegmentBlockageAnalysis, analyze_route_segment};

pub const ROUTE_PATH_CANDIDATE_FOUR_VIA_SELECTION_RULE: &str = "select the first unblocked matching authored via quadruple in ascending (via_a_uuid, via_b_uuid, via_c_uuid, via_d_uuid) order whose layer sequence connects the requested anchor layers through three intermediate copper layers";

#[derive(Debug, Clone)]
pub(super) struct RoutePathCandidateFourViaMatch {
    pub via_a: Via,
    pub via_b: Via,
    pub via_c: Via,
    pub via_d: Via,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub third_intermediate_layer: LayerId,
    pub source_segment: RouteSegmentBlockageAnalysis,
    pub first_middle_segment: RouteSegmentBlockageAnalysis,
    pub second_middle_segment: RouteSegmentBlockageAnalysis,
    pub third_middle_segment: RouteSegmentBlockageAnalysis,
    pub target_segment: RouteSegmentBlockageAnalysis,
}

pub(super) fn candidate_four_via_matches(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> (Vec<Via>, Vec<RoutePathCandidateFourViaMatch>) {
    let candidate_vias = candidate_vias_for_net(board, net_uuid);
    let mut matching_quadruples = Vec::new();
    for (via_a_index, via_a) in candidate_vias.iter().enumerate() {
        for (via_b_index, via_b) in candidate_vias.iter().enumerate() {
            if via_b_index == via_a_index {
                continue;
            }
            for (via_c_index, via_c) in candidate_vias.iter().enumerate() {
                if via_c_index == via_a_index || via_c_index == via_b_index {
                    continue;
                }
                for (via_d_index, via_d) in candidate_vias.iter().enumerate() {
                    if via_d_index == via_a_index
                        || via_d_index == via_b_index
                        || via_d_index == via_c_index
                    {
                        continue;
                    }
                    if let Some(entry) = matching_four_via_quadruple(
                        board,
                        net_uuid,
                        from_anchor,
                        to_anchor,
                        via_a,
                        via_b,
                        via_c,
                        via_d,
                    ) {
                        matching_quadruples.push(entry);
                    }
                }
            }
        }
    }

    (candidate_vias, matching_quadruples)
}

fn matching_four_via_quadruple(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
    via_a: &Via,
    via_b: &Via,
    via_c: &Via,
    via_d: &Via,
) -> Option<RoutePathCandidateFourViaMatch> {
    let (first_intermediate_layer, second_intermediate_layer, third_intermediate_layer) =
        intermediate_layers_for_quadruple(
            via_a,
            via_b,
            via_c,
            via_d,
            from_anchor.layer,
            to_anchor.layer,
        )?;

    Some(RoutePathCandidateFourViaMatch {
        via_a: via_a.clone(),
        via_b: via_b.clone(),
        via_c: via_c.clone(),
        via_d: via_d.clone(),
        first_intermediate_layer,
        second_intermediate_layer,
        third_intermediate_layer,
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
        third_middle_segment: analyze_route_segment(
            board,
            net_uuid,
            third_intermediate_layer,
            via_c.position,
            via_d.position,
            &format!(
                "via-c-to-via-d segment via {} to via {} on layer {}",
                via_c.uuid, via_d.uuid, third_intermediate_layer
            ),
        ),
        target_segment: analyze_route_segment(
            board,
            net_uuid,
            to_anchor.layer,
            via_d.position,
            to_anchor.position,
            &format!(
                "via-d-to-target segment via {} on layer {}",
                via_d.uuid, to_anchor.layer
            ),
        ),
    })
}

fn intermediate_layers_for_quadruple(
    via_a: &Via,
    via_b: &Via,
    via_c: &Via,
    via_d: &Via,
    from_layer: LayerId,
    to_layer: LayerId,
) -> Option<(LayerId, LayerId, LayerId)> {
    let first_intermediate_layer = other_boundary_layer(via_a, from_layer)?;
    let second_intermediate_layer =
        other_boundary_layer_for_middle_via(via_b, first_intermediate_layer)?;
    if second_intermediate_layer == first_intermediate_layer {
        return None;
    }
    let third_intermediate_layer =
        other_boundary_layer_for_middle_via(via_c, second_intermediate_layer)?;
    if third_intermediate_layer == second_intermediate_layer
        || third_intermediate_layer == first_intermediate_layer
    {
        return None;
    }
    let via_d_other_layer = other_boundary_layer(via_d, to_layer)?;
    if via_d_other_layer == third_intermediate_layer {
        Some((
            first_intermediate_layer,
            second_intermediate_layer,
            third_intermediate_layer,
        ))
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

pub(super) fn selected_matching_four_via(
    matches: &[RoutePathCandidateFourViaMatch],
) -> Option<&RoutePathCandidateFourViaMatch> {
    matches.iter().find(|entry| {
        entry.source_segment.blockages.is_empty()
            && entry.first_middle_segment.blockages.is_empty()
            && entry.second_middle_segment.blockages.is_empty()
            && entry.third_middle_segment.blockages.is_empty()
            && entry.target_segment.blockages.is_empty()
    })
}

pub(super) fn four_via_path_points(
    entry: &RoutePathCandidateFourViaMatch,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> [[Point; 2]; 5] {
    [
        [from_anchor.position, entry.via_a.position],
        [entry.via_a.position, entry.via_b.position],
        [entry.via_b.position, entry.via_c.position],
        [entry.via_c.position, entry.via_d.position],
        [entry.via_d.position, to_anchor.position],
    ]
}
