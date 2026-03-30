use crate::board::{Board, RouteCorridorSpanBlockage, RoutePreflightAnchor, StackupLayer};
use crate::ir::geometry::{LayerId, Point};
use uuid::Uuid;

use super::route_segment_blockage::analyze_route_segment;

pub(super) const ROUTE_PATH_CANDIDATE_ORTHOGONAL_DOGLEG_SELECTION_RULE: &str = "select the first unblocked same-layer orthogonal dogleg after sorting candidates by candidate copper layer order, then corner order (horizontal_then_vertical before vertical_then_horizontal); each candidate uses exactly one canonical Manhattan corner and both segments must be unblocked under existing authored obstacle checks";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum DoglegCornerOrder {
    HorizontalThenVertical,
    VerticalThenHorizontal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OrthogonalDoglegCandidate {
    pub layer: LayerId,
    pub corner: Point,
    pub corner_order: DoglegCornerOrder,
    pub blocked: bool,
    pub blockages: Vec<RouteCorridorSpanBlockage>,
}

pub(super) fn candidate_orthogonal_doglegs(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
    candidate_copper_layers: &[StackupLayer],
) -> Vec<OrthogonalDoglegCandidate> {
    if from_anchor.layer != to_anchor.layer
        || from_anchor.position.x == to_anchor.position.x
        || from_anchor.position.y == to_anchor.position.y
    {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for layer in candidate_copper_layers {
        let corners = [
            (
                DoglegCornerOrder::HorizontalThenVertical,
                Point::new(to_anchor.position.x, from_anchor.position.y),
            ),
            (
                DoglegCornerOrder::VerticalThenHorizontal,
                Point::new(from_anchor.position.x, to_anchor.position.y),
            ),
        ];
        for (corner_order, corner) in corners {
            let first = analyze_route_segment(
                board,
                net_uuid,
                layer.id,
                from_anchor.position,
                corner,
                &format!(
                    "orthogonal dogleg {:?} first segment on layer {}",
                    corner_order, layer.id
                ),
            );
            let second = analyze_route_segment(
                board,
                net_uuid,
                layer.id,
                corner,
                to_anchor.position,
                &format!(
                    "orthogonal dogleg {:?} second segment on layer {}",
                    corner_order, layer.id
                ),
            );
            let mut blockages = first.blockages;
            blockages.extend(second.blockages);
            blockages.sort_by(|a, b| {
                a.kind
                    .cmp(&b.kind)
                    .then_with(|| a.object_uuid.cmp(&b.object_uuid))
                    .then_with(|| a.layer.cmp(&b.layer))
                    .then_with(|| a.reason.cmp(&b.reason))
            });
            blockages.dedup();
            candidates.push(OrthogonalDoglegCandidate {
                layer: layer.id,
                corner,
                corner_order,
                blocked: !blockages.is_empty(),
                blockages,
            });
        }
    }

    candidates.sort_by(|a, b| {
        a.layer
            .cmp(&b.layer)
            .then_with(|| a.corner_order.cmp(&b.corner_order))
            .then_with(|| a.corner.x.cmp(&b.corner.x))
            .then_with(|| a.corner.y.cmp(&b.corner.y))
    });
    candidates
}

pub(super) fn selected_orthogonal_dogleg(
    candidates: &[OrthogonalDoglegCandidate],
) -> Option<&OrthogonalDoglegCandidate> {
    candidates.iter().find(|candidate| !candidate.blocked)
}
