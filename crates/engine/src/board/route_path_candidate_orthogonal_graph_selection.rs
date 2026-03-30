use std::collections::{HashMap, HashSet, VecDeque};

use crate::board::{
    Board, RouteCorridorObstacleGeometry, RouteCorridorObstacleKind, RouteCorridorSpanBlockage,
    RoutePreflightAnchor, StackupLayer,
    polygon::point_in_polygon,
};
use crate::ir::geometry::{LayerId, Point};
use uuid::Uuid;

use super::route_segment_blockage::analyze_route_segment;

pub(super) const ROUTE_PATH_CANDIDATE_ORTHOGONAL_GRAPH_SELECTION_RULE: &str = "select the first unblocked same-layer orthogonal graph path after building a deterministic graph from intersections of persisted board-outline, anchor, and authored-object x/y coordinates on the candidate layer, connecting clear same-layer orthogonal spans only, then breadth-first searching layer order with neighbor order horizontal before vertical and destination coordinate ascending";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    let mut queue = VecDeque::from([from]);
    let mut visited = HashSet::from([from]);
    let mut previous: HashMap<Point, Point> = HashMap::new();

    while let Some(current) = queue.pop_front() {
        if current == to {
            break;
        }
        for neighbor in adjacency
            .get(&current)
            .into_iter()
            .flatten()
            .copied()
        {
            if visited.insert(neighbor.to) {
                previous.insert(neighbor.to, current);
                queue.push_back(neighbor.to);
            }
        }
    }

    if !visited.contains(&to) {
        return None;
    }

    let mut path = vec![to];
    let mut cursor = to;
    while cursor != from {
        cursor = *previous.get(&cursor)?;
        path.push(cursor);
    }
    path.reverse();
    Some(path)
}

fn compare_neighbors(left: GraphNeighbor, right: GraphNeighbor) -> std::cmp::Ordering {
    left.orientation
        .cmp(&right.orientation)
        .then_with(|| left.to.x.cmp(&right.to.x))
        .then_with(|| left.to.y.cmp(&right.to.y))
}

fn compare_points(left: &[Point], right: &[Point]) -> std::cmp::Ordering {
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
