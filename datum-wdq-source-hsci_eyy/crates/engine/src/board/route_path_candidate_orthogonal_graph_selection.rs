use std::cmp::Ordering;
use std::collections::HashMap;

use crate::board::{
    Board, RouteCorridorObstacleGeometry, RouteCorridorObstacleKind, RouteCorridorSpanBlockage,
    RoutePreflightAnchor, StackupLayer, polygon::point_in_polygon,
};
use crate::ir::geometry::{LayerId, Point};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::route_segment_blockage::analyze_route_segment;

pub(super) const ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SELECTION_RULE: &str = "select the lowest-cost unblocked same-layer orthogonal graph path after building a deterministic graph from intersections of persisted board-outline, anchor, and authored-object x/y coordinates on the candidate layer, connecting clear same-layer orthogonal spans only, then ranking candidate graph paths by bend count ascending, segment count ascending, and point-sequence coordinate ascending";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutePathCandidateOrthogonalGraphPathCost {
    pub bend_count: usize,
    pub segment_count: usize,
    pub point_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum OrthogonalGraphEdgeOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OrthogonalGraphBlockedEdge {
    pub layer: LayerId,
    pub from: Point,
    pub to: Point,
    pub orientation: OrthogonalGraphEdgeOrientation,
    pub blockages: Vec<RouteCorridorSpanBlockage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OrthogonalGraphLayerSearch {
    pub layer: LayerId,
    pub node_count: usize,
    pub edge_count: usize,
    pub blocked_edges: Vec<OrthogonalGraphBlockedEdge>,
    pub path: Option<Vec<Point>>,
}

pub(super) fn candidate_orthogonal_graph_layer_searches(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
    candidate_copper_layers: &[StackupLayer],
) -> Vec<OrthogonalGraphLayerSearch> {
    if from_anchor.layer != to_anchor.layer || from_anchor.position == to_anchor.position {
        return Vec::new();
    }

    candidate_copper_layers
        .iter()
        .map(|layer| {
            search_orthogonal_graph_layer(
                board,
                net_uuid,
                layer.id,
                from_anchor.position,
                to_anchor.position,
            )
        })
        .collect()
}

pub(super) fn selected_orthogonal_graph_path(
    searches: &[OrthogonalGraphLayerSearch],
) -> Option<&OrthogonalGraphLayerSearch> {
    searches.iter().find(|search| search.path.is_some())
}

pub(super) fn search_orthogonal_graph_layer(
    board: &Board,
    net_uuid: Uuid,
    layer: LayerId,
    from: Point,
    to: Point,
) -> OrthogonalGraphLayerSearch {
    let nodes = graph_nodes(board, layer, from, to);
    let mut blocked_edges = Vec::new();
    let mut adjacency: HashMap<Point, Vec<GraphNeighbor>> = HashMap::new();
    let mut edge_count = 0usize;

    for (index, source) in nodes.iter().copied().enumerate() {
        for target in nodes.iter().copied().skip(index + 1) {
            if source.x != target.x && source.y != target.y {
                continue;
            }
            let orientation = if source.y == target.y {
                OrthogonalGraphEdgeOrientation::Horizontal
            } else {
                OrthogonalGraphEdgeOrientation::Vertical
            };
            let analysis = analyze_route_segment(
                board,
                net_uuid,
                layer,
                source,
                target,
                "orthogonal_graph_edge",
            );
            if analysis.blockages.is_empty() {
                adjacency.entry(source).or_default().push(GraphNeighbor {
                    to: target,
                    orientation,
                });
                adjacency.entry(target).or_default().push(GraphNeighbor {
                    to: source,
                    orientation,
                });
                edge_count += 1;
            } else {
                blocked_edges.push(OrthogonalGraphBlockedEdge {
                    layer,
                    from: source,
                    to: target,
                    orientation,
                    blockages: analysis.blockages,
                });
            }
        }
    }

    for neighbors in adjacency.values_mut() {
        neighbors.sort_by(|left, right| compare_neighbors(*left, *right));
    }
    blocked_edges.sort_by(|left, right| {
        left.layer
            .cmp(&right.layer)
            .then_with(|| left.orientation.cmp(&right.orientation))
            .then_with(|| compare_points(&[left.from, left.to], &[right.from, right.to]))
    });

    let path = shortest_path(&adjacency, from, to);
    OrthogonalGraphLayerSearch {
        layer,
        node_count: nodes.len(),
        edge_count,
        blocked_edges,
        path,
    }
}

pub(super) fn orthogonal_graph_path_cost(
    points: &[Point],
) -> RoutePathCandidateOrthogonalGraphPathCost {
    let bend_count = points
        .windows(3)
        .filter(|window| {
            orientation_between(window[0], window[1]) != orientation_between(window[1], window[2])
        })
        .count();
    RoutePathCandidateOrthogonalGraphPathCost {
        bend_count,
        segment_count: points.len().saturating_sub(1),
        point_count: points.len(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GraphNeighbor {
    to: Point,
    orientation: OrthogonalGraphEdgeOrientation,
}

fn graph_nodes(board: &Board, layer: LayerId, from: Point, to: Point) -> Vec<Point> {
    let mut xs = vec![from.x, to.x];
    let mut ys = vec![from.y, to.y];
    xs.extend(board.outline.vertices.iter().map(|point| point.x));
    ys.extend(board.outline.vertices.iter().map(|point| point.y));
    for obstacle in authored_object_geometry(board, layer) {
        xs.extend(obstacle_coordinates(&obstacle, true));
        ys.extend(obstacle_coordinates(&obstacle, false));
    }
    xs.sort();
    xs.dedup();
    ys.sort();
    ys.dedup();

    let mut nodes = xs
        .into_iter()
        .flat_map(|x| ys.iter().copied().map(move |y| Point::new(x, y)))
        .filter(|point| point_in_polygon(*point, &board.outline))
        .collect::<Vec<_>>();
    if !nodes.contains(&from) {
        nodes.push(from);
    }
    if !nodes.contains(&to) {
        nodes.push(to);
    }
    nodes.sort_by(|left, right| compare_points(&[*left], &[*right]));
    nodes.dedup();
    nodes
}

fn authored_object_geometry(board: &Board, layer: LayerId) -> Vec<RouteCorridorObstacleGeometry> {
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

fn shortest_path(
    adjacency: &HashMap<Point, Vec<GraphNeighbor>>,
    from: Point,
    to: Point,
) -> Option<Vec<Point>> {
    let mut frontier = vec![GraphSearchState {
        point: from,
        last_orientation: None,
        bends: 0,
        segments: 0,
        points: vec![from],
    }];
    let mut best_state_costs: HashMap<
        (Point, Option<OrthogonalGraphEdgeOrientation>),
        GraphPathCost,
    > = HashMap::from([(
        (from, None),
        GraphPathCost {
            bends: 0,
            segments: 0,
            points: vec![from],
        },
    )]);
    let mut best_target: Option<GraphPathCost> = None;

    while !frontier.is_empty() {
        frontier.sort_by(compare_graph_search_state);
        let current = frontier.remove(0);
        let current_cost = GraphPathCost {
            bends: current.bends,
            segments: current.segments,
            points: current.points.clone(),
        };
        if best_state_costs
            .get(&(current.point, current.last_orientation))
            .is_some_and(|best| compare_graph_path_cost(best, &current_cost) != Ordering::Equal)
        {
            continue;
        }
        if best_target
            .as_ref()
            .is_some_and(|best| compare_graph_path_cost(&current_cost, best) == Ordering::Greater)
        {
            continue;
        }
        if current.point == to {
            if best_target
                .as_ref()
                .is_none_or(|best| compare_graph_path_cost(&current_cost, best) == Ordering::Less)
            {
                best_target = Some(current_cost);
            }
            continue;
        }
        for neighbor in adjacency.get(&current.point).into_iter().flatten().copied() {
            if current.points.contains(&neighbor.to) {
                continue;
            }
            let next_state = GraphSearchState {
                point: neighbor.to,
                last_orientation: Some(neighbor.orientation),
                bends: current.bends
                    + usize::from(
                        current
                            .last_orientation
                            .is_some_and(|orientation| orientation != neighbor.orientation),
                    ),
                segments: current.segments + 1,
                points: current
                    .points
                    .iter()
                    .copied()
                    .chain(std::iter::once(neighbor.to))
                    .collect(),
            };
            let next_cost = GraphPathCost {
                bends: next_state.bends,
                segments: next_state.segments,
                points: next_state.points.clone(),
            };
            if best_target
                .as_ref()
                .is_some_and(|best| compare_graph_path_cost(&next_cost, best) == Ordering::Greater)
            {
                continue;
            }
            let entry = best_state_costs.entry((next_state.point, next_state.last_orientation));
            let should_replace = match entry {
                std::collections::hash_map::Entry::Occupied(ref occupied) => {
                    compare_graph_path_cost(&next_cost, occupied.get()) == Ordering::Less
                }
                std::collections::hash_map::Entry::Vacant(_) => true,
            };
            if should_replace {
                entry
                    .and_modify(|best| *best = next_cost.clone())
                    .or_insert(next_cost);
                frontier.push(next_state);
            }
        }
    }

    best_target.map(|cost| cost.points)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GraphSearchState {
    point: Point,
    last_orientation: Option<OrthogonalGraphEdgeOrientation>,
    bends: usize,
    segments: usize,
    points: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GraphPathCost {
    bends: usize,
    segments: usize,
    points: Vec<Point>,
}

fn compare_graph_search_state(left: &GraphSearchState, right: &GraphSearchState) -> Ordering {
    compare_graph_path_cost(
        &GraphPathCost {
            bends: left.bends,
            segments: left.segments,
            points: left.points.clone(),
        },
        &GraphPathCost {
            bends: right.bends,
            segments: right.segments,
            points: right.points.clone(),
        },
    )
    .then_with(|| left.point.x.cmp(&right.point.x))
    .then_with(|| left.point.y.cmp(&right.point.y))
    .then_with(|| left.last_orientation.cmp(&right.last_orientation))
}

fn compare_graph_path_cost(left: &GraphPathCost, right: &GraphPathCost) -> Ordering {
    left.bends
        .cmp(&right.bends)
        .then_with(|| left.segments.cmp(&right.segments))
        .then_with(|| compare_points(&left.points, &right.points))
}

fn orientation_between(from: Point, to: Point) -> OrthogonalGraphEdgeOrientation {
    if from.y == to.y {
        OrthogonalGraphEdgeOrientation::Horizontal
    } else {
        OrthogonalGraphEdgeOrientation::Vertical
    }
}

fn compare_neighbors(left: GraphNeighbor, right: GraphNeighbor) -> Ordering {
    left.orientation
        .cmp(&right.orientation)
        .then_with(|| left.to.x.cmp(&right.to.x))
        .then_with(|| left.to.y.cmp(&right.to.y))
}

fn compare_points(left: &[Point], right: &[Point]) -> Ordering {
    for (left_point, right_point) in left.iter().zip(right.iter()) {
        let ordering = left_point
            .x
            .cmp(&right_point.x)
            .then_with(|| left_point.y.cmp(&right_point.y));
        if ordering != std::cmp::Ordering::Equal {
            return ordering;
        }
    }
    left.len().cmp(&right.len())
}
