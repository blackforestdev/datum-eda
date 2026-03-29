use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};

use uuid::Uuid;

use crate::board::{Board, RoutePreflightAnchor, Track, Via};
use crate::ir::geometry::{LayerId, Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GraphAnchor {
    point: Point,
    layer: LayerId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum AuthoredCopperGraphStepKind {
    Track,
    Via,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AuthoredCopperGraphStep {
    pub kind: AuthoredCopperGraphStepKind,
    pub object_uuid: Uuid,
    pub from: Point,
    pub to: Point,
    pub layer: LayerId,
    pub from_layer: Option<LayerId>,
    pub to_layer: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AuthoredCopperGraphPathMatch {
    pub steps: Vec<AuthoredCopperGraphStep>,
}

pub const ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_GRAPH_SELECTION_RULE: &str =
    "select the first existing authored-copper path found by breadth-first traversal over persisted target-net track/via graph edges after sorting edges by (step_kind, object_uuid, destination_anchor), which yields deterministic minimum-step path selection with lexicographic tie-breaks";

pub(super) fn candidate_authored_copper_graph_objects(
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

pub(super) fn selected_authored_copper_graph_path(
    board: &Board,
    net_uuid: Uuid,
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> (Vec<Track>, Vec<Via>, Option<AuthoredCopperGraphPathMatch>) {
    let (tracks, vias) = candidate_authored_copper_graph_objects(board, net_uuid);
    let graph = AuthoredCopperGraph::build(from_anchor, to_anchor, &tracks, &vias);
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
        .map(|steps| AuthoredCopperGraphPathMatch { steps });

    (tracks, vias, path)
}

struct AuthoredCopperGraph {
    node_ids: HashMap<GraphAnchor, usize>,
    adjacency: Vec<Vec<(usize, AuthoredCopperGraphStep)>>,
}

impl AuthoredCopperGraph {
    fn build(
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

        anchors.sort_by(|a, b| {
            a.layer
                .cmp(&b.layer)
                .then_with(|| a.point.x.cmp(&b.point.x))
                .then_with(|| a.point.y.cmp(&b.point.y))
        });
        anchors.dedup();

        let node_ids = anchors
            .iter()
            .enumerate()
            .map(|(index, anchor)| (*anchor, index))
            .collect::<HashMap<_, _>>();
        let mut adjacency = vec![Vec::new(); anchors.len()];

        for track in tracks {
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

            let forward = AuthoredCopperGraphStep {
                kind: AuthoredCopperGraphStepKind::Track,
                object_uuid: track.uuid,
                from: track.from,
                to: track.to,
                layer: track.layer,
                from_layer: None,
                to_layer: None,
            };
            let reverse = AuthoredCopperGraphStep {
                kind: AuthoredCopperGraphStepKind::Track,
                object_uuid: track.uuid,
                from: track.to,
                to: track.from,
                layer: track.layer,
                from_layer: None,
                to_layer: None,
            };
            adjacency[from_id].push((to_id, forward));
            adjacency[to_id].push((from_id, reverse));
        }

        for via in vias {
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

            let forward = AuthoredCopperGraphStep {
                kind: AuthoredCopperGraphStepKind::Via,
                object_uuid: via.uuid,
                from: via.position,
                to: via.position,
                layer: via.from_layer,
                from_layer: Some(via.from_layer),
                to_layer: Some(via.to_layer),
            };
            let reverse = AuthoredCopperGraphStep {
                kind: AuthoredCopperGraphStepKind::Via,
                object_uuid: via.uuid,
                from: via.position,
                to: via.position,
                layer: via.to_layer,
                from_layer: Some(via.to_layer),
                to_layer: Some(via.from_layer),
            };
            adjacency[from_id].push((to_id, forward));
            adjacency[to_id].push((from_id, reverse));
        }

        for (node_index, edges) in adjacency.iter_mut().enumerate() {
            let _ = node_index;
            edges.sort_by(|(left_dest, left_step), (right_dest, right_step)| {
                compare_step(left_step, right_step).then_with(|| {
                    let left_anchor = anchors[*left_dest];
                    let right_anchor = anchors[*right_dest];
                    left_anchor
                        .layer
                        .cmp(&right_anchor.layer)
                        .then_with(|| left_anchor.point.x.cmp(&right_anchor.point.x))
                        .then_with(|| left_anchor.point.y.cmp(&right_anchor.point.y))
                })
            });
        }

        Self {
            node_ids,
            adjacency,
        }
    }

    fn shortest_path(
        &self,
        from_anchor: GraphAnchor,
        to_anchor: GraphAnchor,
    ) -> Option<Vec<AuthoredCopperGraphStep>> {
        let start = *self.node_ids.get(&from_anchor)?;
        let target = *self.node_ids.get(&to_anchor)?;
        let mut queue = VecDeque::new();
        let mut visited: HashMap<usize, Option<(usize, AuthoredCopperGraphStep)>> = HashMap::new();

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

fn compare_step(left: &AuthoredCopperGraphStep, right: &AuthoredCopperGraphStep) -> Ordering {
    left.kind
        .cmp(&right.kind)
        .then_with(|| left.object_uuid.cmp(&right.object_uuid))
        .then_with(|| left.layer.cmp(&right.layer))
        .then_with(|| left.from.x.cmp(&right.from.x))
        .then_with(|| left.from.y.cmp(&right.from.y))
        .then_with(|| left.to.x.cmp(&right.to.x))
        .then_with(|| left.to.y.cmp(&right.to.y))
}
