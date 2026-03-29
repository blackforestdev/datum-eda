use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};

use uuid::Uuid;

use crate::board::{
    Board, RoutePreflightAnchor, Track, Via, Zone, polygon::point_in_or_on_polygon,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_segment_blockage::analyze_route_segment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GraphAnchor {
    point: Point,
    layer: LayerId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum AuthoredCopperGraphZoneObstacleAwareStepKind {
    Track,
    Via,
    Zone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AuthoredCopperGraphZoneObstacleAwareStep {
    pub kind: AuthoredCopperGraphZoneObstacleAwareStepKind,
    pub object_uuid: Uuid,
    pub layer: LayerId,
    pub from: Point,
    pub to: Point,
    pub from_layer: Option<LayerId>,
    pub to_layer: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AuthoredCopperGraphZoneObstacleAwarePathMatch {
    pub steps: Vec<AuthoredCopperGraphZoneObstacleAwareStep>,
}

pub const ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_ZONE_OBSTACLE_AWARE_SELECTION_RULE: &str = "select the first existing authored-copper path found by breadth-first traversal over persisted target-net track/via/zone graph edges whose reused geometry is unblocked under current authored obstacle checks after sorting edges by (step_kind, object_uuid, destination_anchor), which yields deterministic minimum-step path selection with lexicographic tie-breaks";

pub(super) fn candidate_authored_copper_graph_zone_obstacle_aware_objects(
    board: &Board,
    net_uuid: Uuid,
) -> (Vec<Track>, Vec<Via>, Vec<Zone>) {
    let mut tracks = board
        .tracks
        .values()
        .filter(|track| track.net == net_uuid)
        .cloned()
        .collect::<Vec<_>>();
    tracks.sort_by(|a, b| a.uuid.cmp(&b.uuid));

    let mut vias = board
        .vias
        .values()
        .filter(|via| via.net == net_uuid)
        .cloned()
        .collect::<Vec<_>>();
    vias.sort_by(|a, b| a.uuid.cmp(&b.uuid));

    let mut zones = board
        .zones
        .values()
        .filter(|zone| zone.net == net_uuid)
        .cloned()
        .collect::<Vec<_>>();
    zones.sort_by(|a, b| a.uuid.cmp(&b.uuid));

    (tracks, vias, zones)
}

pub(super) fn selected_authored_copper_graph_zone_obstacle_aware_path(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> (
    Vec<Track>,
    Vec<Via>,
    Vec<Zone>,
    usize,
    usize,
    usize,
    Option<AuthoredCopperGraphZoneObstacleAwarePathMatch>,
) {
    let (tracks, vias, zones) =
        candidate_authored_copper_graph_zone_obstacle_aware_objects(board, net_uuid);
    let graph = AuthoredCopperGraphZoneObstacleAware::build(
        board,
        net_uuid,
        from_anchor,
        to_anchor,
        &tracks,
        &vias,
        &zones,
    );
    let path = graph
        .shortest_path(
            GraphAnchor {
                point: from_anchor.position,
                layer: from_anchor.layer,
            },
            GraphAnchor {
                point: to_anchor.position,
                layer: to_anchor.layer,
            },
        )
        .map(|steps| AuthoredCopperGraphZoneObstacleAwarePathMatch { steps });

    (
        tracks,
        vias,
        zones,
        graph.blocked_track_count,
        graph.blocked_via_count,
        graph.blocked_zone_connection_count,
        path,
    )
}

struct AuthoredCopperGraphZoneObstacleAware {
    node_ids: HashMap<GraphAnchor, usize>,
    adjacency: Vec<Vec<(usize, AuthoredCopperGraphZoneObstacleAwareStep)>>,
    blocked_track_count: usize,
    blocked_via_count: usize,
    blocked_zone_connection_count: usize,
}

impl AuthoredCopperGraphZoneObstacleAware {
    fn build(
        board: &Board,
        net_uuid: Uuid,
        from_anchor: &RoutePreflightAnchor,
        to_anchor: &RoutePreflightAnchor,
        tracks: &[Track],
        vias: &[Via],
        zones: &[Zone],
    ) -> Self {
        let mut anchors = vec![
            GraphAnchor {
                point: from_anchor.position,
                layer: from_anchor.layer,
            },
            GraphAnchor {
                point: to_anchor.position,
                layer: to_anchor.layer,
            },
        ];

        for track in tracks {
            anchors.push(GraphAnchor {
                point: track.from,
                layer: track.layer,
            });
            anchors.push(GraphAnchor {
                point: track.to,
                layer: track.layer,
            });
        }

        for via in vias {
            anchors.push(GraphAnchor {
                point: via.position,
                layer: via.from_layer,
            });
            anchors.push(GraphAnchor {
                point: via.position,
                layer: via.to_layer,
            });
        }

        anchors.sort_by(|a, b| compare_anchor(*a, *b));
        anchors.dedup();

        let node_ids = anchors
            .iter()
            .enumerate()
            .map(|(index, anchor)| (*anchor, index))
            .collect::<HashMap<_, _>>();
        let mut adjacency = vec![Vec::new(); anchors.len()];
        let mut blocked_track_count = 0;
        let mut blocked_via_count = 0;
        let mut blocked_zone_connection_count = 0;

        for track in tracks {
            let subject = format!(
                "existing track edge {} on layer {}",
                track.uuid, track.layer
            );
            let analysis =
                analyze_route_segment(board, net_uuid, track.layer, track.from, track.to, &subject);
            if !analysis.blockages.is_empty() {
                blocked_track_count += 1;
                continue;
            }

            let from = GraphAnchor {
                point: track.from,
                layer: track.layer,
            };
            let to = GraphAnchor {
                point: track.to,
                layer: track.layer,
            };
            let from_id = node_ids[&from];
            let to_id = node_ids[&to];

            adjacency[from_id].push((
                to_id,
                AuthoredCopperGraphZoneObstacleAwareStep {
                    kind: AuthoredCopperGraphZoneObstacleAwareStepKind::Track,
                    object_uuid: track.uuid,
                    layer: track.layer,
                    from: track.from,
                    to: track.to,
                    from_layer: None,
                    to_layer: None,
                },
            ));
            adjacency[to_id].push((
                from_id,
                AuthoredCopperGraphZoneObstacleAwareStep {
                    kind: AuthoredCopperGraphZoneObstacleAwareStepKind::Track,
                    object_uuid: track.uuid,
                    layer: track.layer,
                    from: track.to,
                    to: track.from,
                    from_layer: None,
                    to_layer: None,
                },
            ));
        }

        for via in vias {
            let lower_subject =
                format!("existing via edge {} on layer {}", via.uuid, via.from_layer);
            let lower_analysis = analyze_route_segment(
                board,
                net_uuid,
                via.from_layer,
                via.position,
                via.position,
                &lower_subject,
            );
            let upper_analysis = if via.to_layer == via.from_layer {
                None
            } else {
                let upper_subject =
                    format!("existing via edge {} on layer {}", via.uuid, via.to_layer);
                Some(analyze_route_segment(
                    board,
                    net_uuid,
                    via.to_layer,
                    via.position,
                    via.position,
                    &upper_subject,
                ))
            };

            if !lower_analysis.blockages.is_empty()
                || upper_analysis
                    .as_ref()
                    .is_some_and(|analysis| !analysis.blockages.is_empty())
            {
                blocked_via_count += 1;
                continue;
            }

            let from = GraphAnchor {
                point: via.position,
                layer: via.from_layer,
            };
            let to = GraphAnchor {
                point: via.position,
                layer: via.to_layer,
            };
            let from_id = node_ids[&from];
            let to_id = node_ids[&to];

            adjacency[from_id].push((
                to_id,
                AuthoredCopperGraphZoneObstacleAwareStep {
                    kind: AuthoredCopperGraphZoneObstacleAwareStepKind::Via,
                    object_uuid: via.uuid,
                    layer: via.from_layer,
                    from: via.position,
                    to: via.position,
                    from_layer: Some(via.from_layer),
                    to_layer: Some(via.to_layer),
                },
            ));
            adjacency[to_id].push((
                from_id,
                AuthoredCopperGraphZoneObstacleAwareStep {
                    kind: AuthoredCopperGraphZoneObstacleAwareStepKind::Via,
                    object_uuid: via.uuid,
                    layer: via.to_layer,
                    from: via.position,
                    to: via.position,
                    from_layer: Some(via.to_layer),
                    to_layer: Some(via.from_layer),
                },
            ));
        }

        for zone in zones {
            let member_ids = anchors
                .iter()
                .enumerate()
                .filter(|(_, anchor)| {
                    anchor.layer == zone.layer
                        && point_in_or_on_polygon(anchor.point, &zone.polygon)
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();

            for left_index in 0..member_ids.len() {
                for right_index in (left_index + 1)..member_ids.len() {
                    let from_id = member_ids[left_index];
                    let to_id = member_ids[right_index];
                    let from_anchor = anchors[from_id];
                    let to_anchor = anchors[to_id];
                    let subject =
                        format!("existing zone edge {} on layer {}", zone.uuid, zone.layer);
                    let analysis = analyze_route_segment(
                        board,
                        net_uuid,
                        zone.layer,
                        from_anchor.point,
                        to_anchor.point,
                        &subject,
                    );
                    if !analysis.blockages.is_empty() {
                        blocked_zone_connection_count += 1;
                        continue;
                    }

                    adjacency[from_id].push((
                        to_id,
                        AuthoredCopperGraphZoneObstacleAwareStep {
                            kind: AuthoredCopperGraphZoneObstacleAwareStepKind::Zone,
                            object_uuid: zone.uuid,
                            layer: zone.layer,
                            from: from_anchor.point,
                            to: to_anchor.point,
                            from_layer: None,
                            to_layer: None,
                        },
                    ));
                    adjacency[to_id].push((
                        from_id,
                        AuthoredCopperGraphZoneObstacleAwareStep {
                            kind: AuthoredCopperGraphZoneObstacleAwareStepKind::Zone,
                            object_uuid: zone.uuid,
                            layer: zone.layer,
                            from: to_anchor.point,
                            to: from_anchor.point,
                            from_layer: None,
                            to_layer: None,
                        },
                    ));
                }
            }
        }

        for edges in &mut adjacency {
            edges.sort_by(|(left_dest, left_step), (right_dest, right_step)| {
                compare_step(left_step, right_step)
                    .then_with(|| compare_anchor(anchors[*left_dest], anchors[*right_dest]))
            });
        }

        Self {
            node_ids,
            adjacency,
            blocked_track_count,
            blocked_via_count,
            blocked_zone_connection_count,
        }
    }

    fn shortest_path(
        &self,
        from_anchor: GraphAnchor,
        to_anchor: GraphAnchor,
    ) -> Option<Vec<AuthoredCopperGraphZoneObstacleAwareStep>> {
        let start = *self.node_ids.get(&from_anchor)?;
        let target = *self.node_ids.get(&to_anchor)?;
        let mut queue = VecDeque::new();
        let mut visited: HashMap<usize, Option<(usize, AuthoredCopperGraphZoneObstacleAwareStep)>> =
            HashMap::new();

        queue.push_back(start);
        visited.insert(start, None);

        while let Some(node) = queue.pop_front() {
            if node == target {
                break;
            }

            for (next_node, step) in &self.adjacency[node] {
                if visited.contains_key(next_node) {
                    continue;
                }
                visited.insert(*next_node, Some((node, step.clone())));
                queue.push_back(*next_node);
            }
        }

        if !visited.contains_key(&target) {
            return None;
        }

        let mut steps = Vec::new();
        let mut cursor = target;
        while let Some(Some((previous, step))) = visited.get(&cursor) {
            steps.push(step.clone());
            cursor = *previous;
        }
        steps.reverse();
        Some(steps)
    }
}

fn compare_step(
    left: &AuthoredCopperGraphZoneObstacleAwareStep,
    right: &AuthoredCopperGraphZoneObstacleAwareStep,
) -> Ordering {
    left.kind
        .cmp(&right.kind)
        .then_with(|| left.object_uuid.cmp(&right.object_uuid))
}

fn compare_anchor(left: GraphAnchor, right: GraphAnchor) -> Ordering {
    left.layer
        .cmp(&right.layer)
        .then_with(|| left.point.x.cmp(&right.point.x))
        .then_with(|| left.point.y.cmp(&right.point.y))
}
