use uuid::Uuid;

use crate::board::{Board, RoutePreflightAnchor, Via};
use crate::ir::geometry::{LayerId, Point};

use super::route_segment_blockage::{RouteSegmentBlockageAnalysis, analyze_route_segment};

pub const ROUTE_PATH_CANDIDATE_VIA_SELECTION_RULE: &str = "select the first authored target-net via in ascending via UUID order whose boundary layers exactly match the requested anchor layers and whose source-to-via and via-to-target segments are both unblocked";

#[derive(Debug, Clone)]
pub(super) struct RoutePathCandidateViaMatch {
    pub via: Via,
    pub source_segment: RouteSegmentBlockageAnalysis,
    pub target_segment: RouteSegmentBlockageAnalysis,
}

pub(super) fn candidate_vias_for_net(board: &Board, net_uuid: Uuid) -> Vec<Via> {
    let mut candidate_vias = board
        .vias
        .values()
        .filter(|via| via.net == net_uuid)
        .cloned()
        .collect::<Vec<_>>();
    candidate_vias.sort_by(|a, b| a.uuid.cmp(&b.uuid));
    candidate_vias
}

pub(super) fn via_matches_anchor_layers(via: &Via, from_layer: LayerId, to_layer: LayerId) -> bool {
    (via.from_layer == from_layer && via.to_layer == to_layer)
        || (via.from_layer == to_layer && via.to_layer == from_layer)
}

pub(super) fn matching_via_analyses(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
    candidate_vias: &[Via],
) -> Vec<RoutePathCandidateViaMatch> {
    candidate_vias
        .iter()
        .filter(|via| via_matches_anchor_layers(via, from_anchor.layer, to_anchor.layer))
        .map(|via| RoutePathCandidateViaMatch {
            via: via.clone(),
            source_segment: analyze_route_segment(
                board,
                net_uuid,
                from_anchor.layer,
                from_anchor.position,
                via.position,
                &format!(
                    "source-to-via segment via {} on layer {}",
                    via.uuid, from_anchor.layer
                ),
            ),
            target_segment: analyze_route_segment(
                board,
                net_uuid,
                to_anchor.layer,
                via.position,
                to_anchor.position,
                &format!(
                    "via-to-target segment via {} on layer {}",
                    via.uuid, to_anchor.layer
                ),
            ),
        })
        .collect()
}

pub(super) fn selected_matching_via(
    matches: &[RoutePathCandidateViaMatch],
) -> Option<&RoutePathCandidateViaMatch> {
    matches.iter().find(|entry| {
        entry.source_segment.blockages.is_empty() && entry.target_segment.blockages.is_empty()
    })
}

pub(super) fn via_path_points(
    entry: &RoutePathCandidateViaMatch,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> [[Point; 2]; 2] {
    [
        [from_anchor.position, entry.via.position],
        [entry.via.position, to_anchor.position],
    ]
}
