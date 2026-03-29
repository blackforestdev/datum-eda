use uuid::Uuid;

use crate::board::{Board, RoutePreflightAnchor, Via};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_via_selection::candidate_vias_for_net;
use super::route_segment_blockage::{RouteSegmentBlockageAnalysis, analyze_route_segment};

pub const ROUTE_PATH_CANDIDATE_SIX_VIA_SELECTION_RULE: &str = "select the first unblocked matching authored via sextuple in ascending (via_a_uuid, via_b_uuid, via_c_uuid, via_d_uuid, via_e_uuid, via_f_uuid) order whose layer sequence connects the requested anchor layers through five intermediate copper layers";

#[derive(Debug, Clone)]
pub(super) struct RoutePathCandidateSixViaMatch {
    pub via_a: Via,
    pub via_b: Via,
    pub via_c: Via,
    pub via_d: Via,
    pub via_e: Via,
    pub via_f: Via,
    pub first_intermediate_layer: LayerId,
    pub second_intermediate_layer: LayerId,
    pub third_intermediate_layer: LayerId,
    pub fourth_intermediate_layer: LayerId,
    pub fifth_intermediate_layer: LayerId,
    pub source_segment: RouteSegmentBlockageAnalysis,
    pub first_middle_segment: RouteSegmentBlockageAnalysis,
    pub second_middle_segment: RouteSegmentBlockageAnalysis,
    pub third_middle_segment: RouteSegmentBlockageAnalysis,
    pub fourth_middle_segment: RouteSegmentBlockageAnalysis,
    pub fifth_middle_segment: RouteSegmentBlockageAnalysis,
    pub target_segment: RouteSegmentBlockageAnalysis,
}

pub(super) fn candidate_six_via_matches(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> (Vec<Via>, Vec<RoutePathCandidateSixViaMatch>) {
    let candidate_vias = candidate_vias_for_net(board, net_uuid);
    let mut matching_sextuples = Vec::new();
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
                    for (via_e_index, via_e) in candidate_vias.iter().enumerate() {
                        if via_e_index == via_a_index
                            || via_e_index == via_b_index
                            || via_e_index == via_c_index
                            || via_e_index == via_d_index
                        {
                            continue;
                        }
                        for (via_f_index, via_f) in candidate_vias.iter().enumerate() {
                            if via_f_index == via_a_index
                                || via_f_index == via_b_index
                                || via_f_index == via_c_index
                                || via_f_index == via_d_index
                                || via_f_index == via_e_index
                            {
                                continue;
                            }
                            if let Some(entry) = matching_six_via_sextuple(
                                board,
                                net_uuid,
                                from_anchor,
                                to_anchor,
                                via_a,
                                via_b,
                                via_c,
                                via_d,
                                via_e,
                                via_f,
                            ) {
                                matching_sextuples.push(entry);
                            }
                        }
                    }
                }
            }
        }
    }

    (candidate_vias, matching_sextuples)
}

fn matching_six_via_sextuple(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
    via_a: &Via,
    via_b: &Via,
    via_c: &Via,
    via_d: &Via,
    via_e: &Via,
    via_f: &Via,
) -> Option<RoutePathCandidateSixViaMatch> {
    let (
        first_intermediate_layer,
        second_intermediate_layer,
        third_intermediate_layer,
        fourth_intermediate_layer,
        fifth_intermediate_layer,
    ) = intermediate_layers_for_sextuple(
        via_a,
        via_b,
        via_c,
        via_d,
        via_e,
        via_f,
        from_anchor.layer,
        to_anchor.layer,
    )?;

    Some(RoutePathCandidateSixViaMatch {
        via_a: via_a.clone(),
        via_b: via_b.clone(),
        via_c: via_c.clone(),
        via_d: via_d.clone(),
        via_e: via_e.clone(),
        via_f: via_f.clone(),
        first_intermediate_layer,
        second_intermediate_layer,
        third_intermediate_layer,
        fourth_intermediate_layer,
        fifth_intermediate_layer,
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
        fourth_middle_segment: analyze_route_segment(
            board,
            net_uuid,
            fourth_intermediate_layer,
            via_d.position,
            via_e.position,
            &format!(
                "via-d-to-via-e segment via {} to via {} on layer {}",
                via_d.uuid, via_e.uuid, fourth_intermediate_layer
            ),
        ),
        fifth_middle_segment: analyze_route_segment(
            board,
            net_uuid,
            fifth_intermediate_layer,
            via_e.position,
            via_f.position,
            &format!(
                "via-e-to-via-f segment via {} to via {} on layer {}",
                via_e.uuid, via_f.uuid, fifth_intermediate_layer
            ),
        ),
        target_segment: analyze_route_segment(
            board,
            net_uuid,
            to_anchor.layer,
            via_f.position,
            to_anchor.position,
            &format!(
                "via-f-to-target segment via {} on layer {}",
                via_f.uuid, to_anchor.layer
            ),
        ),
    })
}

fn intermediate_layers_for_sextuple(
    via_a: &Via,
    via_b: &Via,
    via_c: &Via,
    via_d: &Via,
    via_e: &Via,
    via_f: &Via,
    from_layer: LayerId,
    to_layer: LayerId,
) -> Option<(LayerId, LayerId, LayerId, LayerId, LayerId)> {
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
    let fourth_intermediate_layer =
        other_boundary_layer_for_middle_via(via_d, third_intermediate_layer)?;
    if fourth_intermediate_layer == third_intermediate_layer
        || fourth_intermediate_layer == second_intermediate_layer
        || fourth_intermediate_layer == first_intermediate_layer
    {
        return None;
    }
    let fifth_intermediate_layer =
        other_boundary_layer_for_middle_via(via_e, fourth_intermediate_layer)?;
    if fifth_intermediate_layer == fourth_intermediate_layer
        || fifth_intermediate_layer == third_intermediate_layer
        || fifth_intermediate_layer == second_intermediate_layer
        || fifth_intermediate_layer == first_intermediate_layer
    {
        return None;
    }
    let via_f_other_layer = other_boundary_layer(via_f, to_layer)?;
    if via_f_other_layer == fifth_intermediate_layer {
        Some((
            first_intermediate_layer,
            second_intermediate_layer,
            third_intermediate_layer,
            fourth_intermediate_layer,
            fifth_intermediate_layer,
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

pub(super) fn selected_matching_six_via(
    matches: &[RoutePathCandidateSixViaMatch],
) -> Option<&RoutePathCandidateSixViaMatch> {
    matches.iter().find(|entry| {
        entry.source_segment.blockages.is_empty()
            && entry.first_middle_segment.blockages.is_empty()
            && entry.second_middle_segment.blockages.is_empty()
            && entry.third_middle_segment.blockages.is_empty()
            && entry.fourth_middle_segment.blockages.is_empty()
            && entry.fifth_middle_segment.blockages.is_empty()
            && entry.target_segment.blockages.is_empty()
    })
}

pub(super) fn six_via_path_points(
    entry: &RoutePathCandidateSixViaMatch,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> [[Point; 2]; 7] {
    [
        [from_anchor.position, entry.via_a.position],
        [entry.via_a.position, entry.via_b.position],
        [entry.via_b.position, entry.via_c.position],
        [entry.via_c.position, entry.via_d.position],
        [entry.via_d.position, entry.via_e.position],
        [entry.via_e.position, entry.via_f.position],
        [entry.via_f.position, to_anchor.position],
    ]
}
