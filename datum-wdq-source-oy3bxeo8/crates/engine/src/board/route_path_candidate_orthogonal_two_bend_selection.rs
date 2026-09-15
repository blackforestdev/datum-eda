use crate::board::{
    Board, RouteCorridorObstacleGeometry, RouteCorridorObstacleKind, RouteCorridorSpanBlockage,
    RoutePreflightAnchor, StackupLayer,
};
use crate::ir::geometry::{LayerId, Point};
use uuid::Uuid;

use super::route_segment_blockage::analyze_route_segment;

pub(super) const ROUTE_PATH_CANDIDATE_ORTHOGONAL_TWO_BEND_SELECTION_RULE: &str = "select the first unblocked same-layer orthogonal two-bend path after sorting candidates by candidate copper layer order, then orientation family (horizontal_detour before vertical_detour), then detour coordinate ascending; detour coordinates are taken only from persisted board-outline and authored-obstacle coordinates already present on the candidate layer, and all three constituent segments must be unblocked under existing authored obstacle checks";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum OrthogonalTwoBendOrientation {
    HorizontalDetour,
    VerticalDetour,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OrthogonalTwoBendCandidate {
    pub layer: LayerId,
    pub orientation: OrthogonalTwoBendOrientation,
    pub detour_coordinate: i64,
    pub points: Vec<Point>,
    pub blocked: bool,
    pub blockages: Vec<RouteCorridorSpanBlockage>,
}

pub(super) fn candidate_orthogonal_two_bend_paths(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
    candidate_copper_layers: &[StackupLayer],
) -> Vec<OrthogonalTwoBendCandidate> {
    if from_anchor.layer != to_anchor.layer || from_anchor.position == to_anchor.position {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for layer in candidate_copper_layers {
        for detour_y in
            candidate_detour_ys(board, layer.id, from_anchor.position, to_anchor.position)
        {
            let points = vec![
                from_anchor.position,
                Point::new(from_anchor.position.x, detour_y),
                Point::new(to_anchor.position.x, detour_y),
                to_anchor.position,
            ];
            if has_degenerate_segment(&points) {
                continue;
            }
            let blockages = analyze_segments(
                board,
                net_uuid,
                layer.id,
                &points,
                OrthogonalTwoBendOrientation::HorizontalDetour,
                detour_y,
            );
            candidates.push(OrthogonalTwoBendCandidate {
                layer: layer.id,
                orientation: OrthogonalTwoBendOrientation::HorizontalDetour,
                detour_coordinate: detour_y,
                points,
                blocked: !blockages.is_empty(),
                blockages,
            });
        }
        for detour_x in
            candidate_detour_xs(board, layer.id, from_anchor.position, to_anchor.position)
        {
            let points = vec![
                from_anchor.position,
                Point::new(detour_x, from_anchor.position.y),
                Point::new(detour_x, to_anchor.position.y),
                to_anchor.position,
            ];
            if has_degenerate_segment(&points) {
                continue;
            }
            let blockages = analyze_segments(
                board,
                net_uuid,
                layer.id,
                &points,
                OrthogonalTwoBendOrientation::VerticalDetour,
                detour_x,
            );
            candidates.push(OrthogonalTwoBendCandidate {
                layer: layer.id,
                orientation: OrthogonalTwoBendOrientation::VerticalDetour,
                detour_coordinate: detour_x,
                points,
                blocked: !blockages.is_empty(),
                blockages,
            });
        }
    }

    candidates.sort_by(|a, b| {
        a.layer
            .cmp(&b.layer)
            .then_with(|| a.orientation.cmp(&b.orientation))
            .then_with(|| a.detour_coordinate.cmp(&b.detour_coordinate))
            .then_with(|| compare_points(&a.points, &b.points))
    });
    candidates
}

pub(super) fn selected_orthogonal_two_bend_path(
    candidates: &[OrthogonalTwoBendCandidate],
) -> Option<&OrthogonalTwoBendCandidate> {
    candidates.iter().find(|candidate| !candidate.blocked)
}

fn candidate_detour_xs(board: &Board, layer: LayerId, from: Point, to: Point) -> Vec<i64> {
    candidate_detour_coordinates(board, layer, from, to, true)
}

fn candidate_detour_ys(board: &Board, layer: LayerId, from: Point, to: Point) -> Vec<i64> {
    candidate_detour_coordinates(board, layer, from, to, false)
}

fn candidate_detour_coordinates(
    board: &Board,
    layer: LayerId,
    from: Point,
    to: Point,
    x_axis: bool,
) -> Vec<i64> {
    let mut coordinates = board
        .outline
        .vertices
        .iter()
        .map(|point| if x_axis { point.x } else { point.y })
        .collect::<Vec<_>>();
    coordinates.extend(
        authored_obstacle_geometry(board, layer)
            .into_iter()
            .flat_map(|obstacle| obstacle_coordinates(&obstacle, x_axis)),
    );
    coordinates.sort();
    coordinates.dedup();
    coordinates.retain(|coordinate| {
        let source = if x_axis { from.x } else { from.y };
        let target = if x_axis { to.x } else { to.y };
        *coordinate != source && *coordinate != target
    });
    coordinates
}

fn authored_obstacle_geometry(board: &Board, layer: LayerId) -> Vec<RouteCorridorObstacleGeometry> {
    let mut obstacles = Vec::new();
    for keepout in board
        .keepouts
        .iter()
        .filter(|keepout| keepout.layers.contains(&layer))
    {
        obstacles.push(RouteCorridorObstacleGeometry {
            kind: RouteCorridorObstacleKind::Keepout,
            object_uuid: Some(keepout.uuid),
            layer: Some(layer),
            net_uuid: None,
            net_name: None,
            polygon: Some(keepout.polygon.clone()),
            from: None,
            to: None,
            position: None,
            diameter_nm: None,
            reason: "keepout".to_string(),
        });
    }
    for track in board.tracks.values().filter(|track| track.layer == layer) {
        obstacles.push(RouteCorridorObstacleGeometry {
            kind: RouteCorridorObstacleKind::ForeignTrack,
            object_uuid: Some(track.uuid),
            layer: Some(layer),
            net_uuid: Some(track.net),
            net_name: board.nets.get(&track.net).map(|net| net.name.clone()),
            polygon: None,
            from: Some(track.from),
            to: Some(track.to),
            position: None,
            diameter_nm: Some(track.width),
            reason: "track".to_string(),
        });
    }
    for via in board
        .vias
        .values()
        .filter(|via| layer >= via.from_layer && layer <= via.to_layer)
    {
        obstacles.push(RouteCorridorObstacleGeometry {
            kind: RouteCorridorObstacleKind::ForeignVia,
            object_uuid: Some(via.uuid),
            layer: Some(layer),
            net_uuid: Some(via.net),
            net_name: board.nets.get(&via.net).map(|net| net.name.clone()),
            polygon: None,
            from: None,
            to: None,
            position: Some(via.position),
            diameter_nm: Some(via.diameter),
            reason: "via".to_string(),
        });
    }
    for zone in board.zones.values().filter(|zone| zone.layer == layer) {
        obstacles.push(RouteCorridorObstacleGeometry {
            kind: RouteCorridorObstacleKind::ForeignZone,
            object_uuid: Some(zone.uuid),
            layer: Some(layer),
            net_uuid: Some(zone.net),
            net_name: board.nets.get(&zone.net).map(|net| net.name.clone()),
            polygon: Some(zone.polygon.clone()),
            from: None,
            to: None,
            position: None,
            diameter_nm: None,
            reason: "zone".to_string(),
        });
    }
    obstacles
}

fn obstacle_coordinates(obstacle: &RouteCorridorObstacleGeometry, x_axis: bool) -> Vec<i64> {
    let mut coordinates = Vec::new();
    if let Some(polygon) = &obstacle.polygon {
        coordinates.extend(
            polygon
                .vertices
                .iter()
                .map(|point| if x_axis { point.x } else { point.y }),
        );
    }
    if let Some(from) = obstacle.from {
        coordinates.push(if x_axis { from.x } else { from.y });
    }
    if let Some(to) = obstacle.to {
        coordinates.push(if x_axis { to.x } else { to.y });
    }
    if let Some(position) = obstacle.position {
        coordinates.push(if x_axis { position.x } else { position.y });
    }
    coordinates
}

fn has_degenerate_segment(points: &[Point]) -> bool {
    points.windows(2).any(|segment| segment[0] == segment[1])
}

fn analyze_segments(
    board: &Board,
    net_uuid: Uuid,
    layer: LayerId,
    points: &[Point],
    orientation: OrthogonalTwoBendOrientation,
    detour_coordinate: i64,
) -> Vec<RouteCorridorSpanBlockage> {
    let mut blockages = Vec::new();
    for (segment_index, segment) in points.windows(2).enumerate() {
        let analysis = analyze_route_segment(
            board,
            net_uuid,
            layer,
            segment[0],
            segment[1],
            &format!(
                "orthogonal two-bend {:?} detour {} segment {} on layer {}",
                orientation, detour_coordinate, segment_index, layer
            ),
        );
        blockages.extend(analysis.blockages);
    }
    blockages.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| a.object_uuid.cmp(&b.object_uuid))
            .then_with(|| a.layer.cmp(&b.layer))
            .then_with(|| a.reason.cmp(&b.reason))
    });
    blockages.dedup();
    blockages
}

fn compare_points(left: &[Point], right: &[Point]) -> std::cmp::Ordering {
    left.len().cmp(&right.len()).then_with(|| {
        left.iter()
            .zip(right.iter())
            .map(|(left_point, right_point)| {
                left_point
                    .x
                    .cmp(&right_point.x)
                    .then_with(|| left_point.y.cmp(&right_point.y))
            })
            .find(|ordering| *ordering != std::cmp::Ordering::Equal)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}
