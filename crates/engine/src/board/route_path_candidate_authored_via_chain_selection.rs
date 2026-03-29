use std::cmp::Ordering;

use uuid::Uuid;

use crate::board::{Board, RoutePreflightAnchor, Via};
use crate::ir::geometry::{LayerId, Point};

use super::route_path_candidate_via_selection::candidate_vias_for_net;
use super::route_segment_blockage::{RouteSegmentBlockageAnalysis, analyze_route_segment};

pub const ROUTE_PATH_CANDIDATE_AUTHORED_VIA_CHAIN_SELECTION_RULE: &str = "enumerate authored via chains whose layer sequence connects the requested anchor layers and whose segment geometry is explainable entirely from existing corridor/span facts plus the reused authored vias, order candidate chains by (via_count, via_uuid_sequence) ascending, and select the first unblocked matching chain under that explicit order";

#[derive(Debug, Clone)]
pub(super) struct RoutePathCandidateAuthoredViaChainMatch {
    pub vias: Vec<Via>,
    pub segment_layers: Vec<LayerId>,
    pub segment_analyses: Vec<RouteSegmentBlockageAnalysis>,
}

pub(super) fn candidate_authored_via_chain_matches(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> (Vec<Via>, Vec<RoutePathCandidateAuthoredViaChainMatch>) {
    let candidate_vias = candidate_vias_for_net(board, net_uuid);
    let mut matching_chains = Vec::new();
    let mut used_indexes = Vec::new();
    let mut current_chain = Vec::new();
    let mut current_layers = vec![from_anchor.layer];

    collect_matching_authored_via_chains(
        board,
        net_uuid,
        &candidate_vias,
        from_anchor,
        to_anchor,
        from_anchor.layer,
        &mut used_indexes,
        &mut current_chain,
        &mut current_layers,
        &mut matching_chains,
    );

    matching_chains.sort_by(compare_chain_match);

    (candidate_vias, matching_chains)
}

fn collect_matching_authored_via_chains(
    board: &Board,
    net_uuid: Uuid,
    candidate_vias: &[Via],
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
    current_layer: LayerId,
    used_indexes: &mut Vec<usize>,
    current_chain: &mut Vec<Via>,
    current_layers: &mut Vec<LayerId>,
    matching_chains: &mut Vec<RoutePathCandidateAuthoredViaChainMatch>,
) {
    for (index, via) in candidate_vias.iter().enumerate() {
        if used_indexes.contains(&index) {
            continue;
        }
        let Some(next_layer) = other_boundary_layer(via, current_layer) else {
            continue;
        };
        if next_layer == current_layer {
            continue;
        }

        used_indexes.push(index);
        current_chain.push(via.clone());
        current_layers.push(next_layer);

        if next_layer == to_anchor.layer {
            matching_chains.push(build_chain_match(
                board,
                net_uuid,
                from_anchor,
                to_anchor,
                current_chain,
                current_layers,
            ));
        } else {
            collect_matching_authored_via_chains(
                board,
                net_uuid,
                candidate_vias,
                from_anchor,
                to_anchor,
                next_layer,
                used_indexes,
                current_chain,
                current_layers,
                matching_chains,
            );
        }

        current_chain.pop();
        current_layers.pop();
        used_indexes.pop();
    }
}

fn build_chain_match(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
    vias: &[Via],
    segment_layers: &[LayerId],
) -> RoutePathCandidateAuthoredViaChainMatch {
    let mut segment_analyses = Vec::with_capacity(vias.len() + 1);

    for segment_index in 0..=vias.len() {
        let (from, to, subject) = if segment_index == 0 {
            (
                from_anchor.position,
                vias[0].position,
                format!(
                    "source-to-via segment via {} on layer {}",
                    vias[0].uuid, segment_layers[0]
                ),
            )
        } else if segment_index == vias.len() {
            (
                vias[segment_index - 1].position,
                to_anchor.position,
                format!(
                    "via-to-target segment via {} on layer {}",
                    vias[segment_index - 1].uuid,
                    segment_layers[segment_index]
                ),
            )
        } else {
            (
                vias[segment_index - 1].position,
                vias[segment_index].position,
                format!(
                    "via-to-via segment via {} to via {} on layer {}",
                    vias[segment_index - 1].uuid,
                    vias[segment_index].uuid,
                    segment_layers[segment_index]
                ),
            )
        };

        segment_analyses.push(analyze_route_segment(
            board,
            net_uuid,
            segment_layers[segment_index],
            from,
            to,
            &subject,
        ));
    }

    RoutePathCandidateAuthoredViaChainMatch {
        vias: vias.to_vec(),
        segment_layers: segment_layers.to_vec(),
        segment_analyses,
    }
}

fn compare_chain_match(
    left: &RoutePathCandidateAuthoredViaChainMatch,
    right: &RoutePathCandidateAuthoredViaChainMatch,
) -> Ordering {
    left.vias
        .len()
        .cmp(&right.vias.len())
        .then_with(|| compare_via_uuid_sequence(&left.vias, &right.vias))
}

fn compare_via_uuid_sequence(left: &[Via], right: &[Via]) -> Ordering {
    for (left_via, right_via) in left.iter().zip(right.iter()) {
        let ordering = left_via.uuid.cmp(&right_via.uuid);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.len().cmp(&right.len())
}

pub(super) fn selected_matching_authored_via_chain(
    matches: &[RoutePathCandidateAuthoredViaChainMatch],
) -> Option<&RoutePathCandidateAuthoredViaChainMatch> {
    matches
        .iter()
        .find(|entry| entry.segment_analyses.iter().all(|segment| segment.blockages.is_empty()))
}

pub(super) fn authored_via_chain_path_points(
    entry: &RoutePathCandidateAuthoredViaChainMatch,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> Vec<[Point; 2]> {
    let mut points = Vec::with_capacity(entry.vias.len() + 1);
    let mut current = from_anchor.position;
    for via in &entry.vias {
        points.push([current, via.position]);
        current = via.position;
    }
    points.push([current, to_anchor.position]);
    points
}

fn other_boundary_layer(via: &Via, current_layer: LayerId) -> Option<LayerId> {
    if via.from_layer == current_layer {
        Some(via.to_layer)
    } else if via.to_layer == current_layer {
        Some(via.from_layer)
    } else {
        None
    }
}
