use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};

use uuid::Uuid;

use crate::board::{Board, RoutePreflightAnchor, Track, Via};
use crate::ir::geometry::{LayerId, Point};

use super::route_segment_blockage::analyze_route_segment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GraphAnchor {
    point: Point,
    layer: LayerId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum AuthoredCopperGraphObstacleAwareStepKind {
    Track,
    Via,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AuthoredCopperGraphObstacleAwareStep {
    pub kind: AuthoredCopperGraphObstacleAwareStepKind,
    pub object_uuid: Uuid,
    pub from: Point,
    pub to: Point,
    pub layer: LayerId,
    pub from_layer: Option<LayerId>,
    pub to_layer: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AuthoredCopperGraphObstacleAwarePathMatch {
    pub steps: Vec<AuthoredCopperGraphObstacleAwareStep>,
}

pub const ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_OBSTACLE_AWARE_SELECTION_RULE: &str = "select the first existing authored-copper path found by breadth-first traversal over persisted target-net track/via graph edges whose reused geometry is unblocked under current authored obstacle checks after sorting edges by (step_kind, object_uuid, destination_anchor), which yields deterministic minimum-step path selection with lexicographic tie-breaks";

pub(super) fn candidate_authored_copper_graph_obstacle_aware_objects(
    board: &Board,
    net_uuid: Uuid,
) -> (Vec<Track>, Vec<Via>) {
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

    (tracks, vias)
}

pub(super) fn selected_authored_copper_graph_obstacle_aware_path(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> (
    Vec<Track>,
    Vec<Via>,
    usize,
    usize,
    Option<AuthoredCopperGraphObstacleAwarePathMatch>,
) {
    let (tracks, vias) = candidate_authored_copper_graph_obstacle_aware_objects(board, net_uuid);
    let graph = AuthoredCopperGraphObstacleAware::build(
        board,
        net_uuid,
        from_anchor,
        to_anchor,
        &tracks,
        &vias,
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
        .map(|steps| AuthoredCopperGraphObstacleAwarePathMatch { steps });

    (
        tracks,
        vias,
        graph.blocked_track_count,
        graph.blocked_via_count,
        path,
    )
}

struct AuthoredCopperGraphObstacleAware {
    node_ids: HashMap<GraphAnchor, usize>,
    adjacency: Vec<Vec<(usize, AuthoredCopperGraphObstacleAwareStep)>>,
    blocked_track_count: usize,
    blocked_via_count: usize,
}

impl AuthoredCopperGraphObstacleAware {
    fn build(
        board: &Board,
        net_uuid: Uuid,
        from_anchor: &RoutePreflightAnchor,
        to_anchor: &RoutePreflightAnchor,
        tracks: &[Track],
        vias: &[Via],
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
                AuthoredCopperGraphObstacleAwareStep {
                    kind: AuthoredCopperGraphObstacleAwareStepKind::Track,
                    object_uuid: track.uuid,
                    from: track.from,
                    to: track.to,
                    layer: track.layer,
                    from_layer: None,
                    to_layer: None,
                },
            ));
            adjacency[to_id].push((
                from_id,
                AuthoredCopperGraphObstacleAwareStep {
                    kind: AuthoredCopperGraphObstacleAwareStepKind::Track,
                    object_uuid: track.uuid,
                    from: track.to,
                    to: track.from,
                    layer: track.layer,
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
                AuthoredCopperGraphObstacleAwareStep {
                    kind: AuthoredCopperGraphObstacleAwareStepKind::Via,
                    object_uuid: via.uuid,
                    from: via.position,
                    to: via.position,
                    layer: via.from_layer,
                    from_layer: Some(via.from_layer),
                    to_layer: Some(via.to_layer),
                },
            ));
            adjacency[to_id].push((
                from_id,
                AuthoredCopperGraphObstacleAwareStep {
                    kind: AuthoredCopperGraphObstacleAwareStepKind::Via,
                    object_uuid: via.uuid,
                    from: via.position,
                    to: via.position,
                    layer: via.to_layer,
                    from_layer: Some(via.to_layer),
                    to_layer: Some(via.from_layer),
                },
            ));
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
        }
    }

    fn shortest_path(
        &self,
        from_anchor: GraphAnchor,
        to_anchor: GraphAnchor,
    ) -> Option<Vec<AuthoredCopperGraphObstacleAwareStep>> {
        let start = *self.node_ids.get(&from_anchor)?;
        let target = *self.node_ids.get(&to_anchor)?;
        let mut queue = VecDeque::new();
        let mut visited: HashMap<usize, Option<(usize, AuthoredCopperGraphObstacleAwareStep)>> =
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
    left: &AuthoredCopperGraphObstacleAwareStep,
    right: &AuthoredCopperGraphObstacleAwareStep,
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
