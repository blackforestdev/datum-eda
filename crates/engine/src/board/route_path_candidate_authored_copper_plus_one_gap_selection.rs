use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use uuid::Uuid;

use crate::board::{
    Board, RoutePreflightAnchor, StackupLayer, Track, Via,
};
use crate::ir::geometry::{LayerId, Point};

use super::route_segment_blockage::analyze_route_segment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GraphAnchor {
    point: Point,
    layer: LayerId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum AuthoredCopperPlusOneGapStepKind {
    Track,
    Via,
    Gap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AuthoredCopperPlusOneGapStep {
    pub kind: AuthoredCopperPlusOneGapStepKind,
    pub object_uuid: Option<Uuid>,
    pub from: Point,
    pub to: Point,
    pub layer: LayerId,
    pub from_layer: Option<LayerId>,
    pub to_layer: Option<LayerId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AuthoredCopperPlusOneGapPathMatch {
    pub steps: Vec<AuthoredCopperPlusOneGapStep>,
}

pub const ROUTE_PATH_CANDIDATE_AUTHORED_COPPER_PLUS_ONE_GAP_SELECTION_RULE: &str =
    "select the first exact-one-gap path found by breadth-first traversal over persisted target-net track/via graph edges plus eligible same-layer gap edges after sorting edges by (step_kind, object_uuid_or_gap_geometry, destination_anchor); gap edges are eligible only on candidate copper layers, only when the straight segment is unblocked by persisted authored geometry checks, and only when they do not replace an existing authored edge";

pub(super) fn candidate_authored_copper_plus_one_gap_objects(
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

pub(super) fn selected_authored_copper_plus_one_gap_path(
    board: &Board,
    net_uuid: Uuid,
    candidate_copper_layers: &[StackupLayer],
    from_anchor: &RoutePreflightAnchor,
    to_anchor: &RoutePreflightAnchor,
) -> (
    Vec<Track>,
    Vec<Via>,
    usize,
    Option<AuthoredCopperPlusOneGapPathMatch>,
) {
    let (tracks, vias) = candidate_authored_copper_plus_one_gap_objects(board, net_uuid);
    let graph = AuthoredCopperPlusOneGapGraph::build(
        board,
        net_uuid,
        candidate_copper_layers,
        from_anchor,
        to_anchor,
        &tracks,
        &vias,
    );
    let path = graph
        .shortest_exact_one_gap_path(
            GraphAnchor {
                point: from_anchor.position,
                layer: from_anchor.layer,
            },
            GraphAnchor {
                point: to_anchor.position,
                layer: to_anchor.layer,
            },
        )
        .map(|steps| AuthoredCopperPlusOneGapPathMatch { steps });

    (tracks, vias, graph.unique_gap_count, path)
}

struct AuthoredCopperPlusOneGapGraph {
    node_ids: HashMap<GraphAnchor, usize>,
    adjacency: Vec<Vec<(usize, AuthoredCopperPlusOneGapStep)>>,
    unique_gap_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GapUsageState {
    BeforeGapNoAuthored,
    BeforeGapWithAuthored,
    AfterGapNoAuthored,
    AfterGapWithAuthored,
}

impl AuthoredCopperPlusOneGapGraph {
    fn build(
        board: &Board,
        net_uuid: Uuid,
        candidate_copper_layers: &[StackupLayer],
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
        let mut direct_authored_pairs = HashSet::new();
        let from_graph_anchor = GraphAnchor {
            point: from_anchor.position,
            layer: from_anchor.layer,
        };
        let to_graph_anchor = GraphAnchor {
            point: to_anchor.position,
            layer: to_anchor.layer,
        };

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
            let pair = if from_id < to_id {
                (from_id, to_id)
            } else {
                (to_id, from_id)
            };
            direct_authored_pairs.insert(pair);

            let forward = AuthoredCopperPlusOneGapStep {
                kind: AuthoredCopperPlusOneGapStepKind::Track,
                object_uuid: Some(track.uuid),
                from: track.from,
                to: track.to,
                layer: track.layer,
                from_layer: None,
                to_layer: None,
            };
            let reverse = AuthoredCopperPlusOneGapStep {
                kind: AuthoredCopperPlusOneGapStepKind::Track,
                object_uuid: Some(track.uuid),
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

            let forward = AuthoredCopperPlusOneGapStep {
                kind: AuthoredCopperPlusOneGapStepKind::Via,
                object_uuid: Some(via.uuid),
                from: via.position,
                to: via.position,
                layer: via.from_layer,
                from_layer: Some(via.from_layer),
                to_layer: Some(via.to_layer),
            };
            let reverse = AuthoredCopperPlusOneGapStep {
                kind: AuthoredCopperPlusOneGapStepKind::Via,
                object_uuid: Some(via.uuid),
                from: via.position,
                to: via.position,
                layer: via.to_layer,
                from_layer: Some(via.to_layer),
                to_layer: Some(via.from_layer),
            };
            adjacency[from_id].push((to_id, forward));
            adjacency[to_id].push((from_id, reverse));
        }

        let candidate_layers = candidate_copper_layers
            .iter()
            .map(|layer| layer.id)
            .collect::<HashSet<_>>();
        let mut unique_gap_count = 0usize;

        for left_index in 0..anchors.len() {
            for right_index in (left_index + 1)..anchors.len() {
                let left = anchors[left_index];
                let right = anchors[right_index];
                if left.layer != right.layer {
                    continue;
                }
                if !candidate_layers.contains(&left.layer) {
                    continue;
                }
                if left == from_graph_anchor
                    || left == to_graph_anchor
                    || right == from_graph_anchor
                    || right == to_graph_anchor
                {
                    continue;
                }
                if left.point == right.point {
                    continue;
                }
                if direct_authored_pairs.contains(&(left_index, right_index)) {
                    continue;
                }

                let analysis = analyze_route_segment(
                    board,
                    net_uuid,
                    left.layer,
                    left.point,
                    right.point,
                    &format!(
                        "synthetic gap from ({}, {}) to ({}, {}) on layer {}",
                        left.point.x, left.point.y, right.point.x, right.point.y, left.layer
                    ),
                );
                if !analysis.blockages.is_empty() {
                    continue;
                }

                let forward = AuthoredCopperPlusOneGapStep {
                    kind: AuthoredCopperPlusOneGapStepKind::Gap,
                    object_uuid: None,
                    from: left.point,
                    to: right.point,
                    layer: left.layer,
                    from_layer: None,
                    to_layer: None,
                };
                let reverse = AuthoredCopperPlusOneGapStep {
                    kind: AuthoredCopperPlusOneGapStepKind::Gap,
                    object_uuid: None,
                    from: right.point,
                    to: left.point,
                    layer: right.layer,
                    from_layer: None,
                    to_layer: None,
                };
                adjacency[left_index].push((right_index, forward));
                adjacency[right_index].push((left_index, reverse));
                unique_gap_count += 1;
            }
        }

        for edges in &mut adjacency {
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
            unique_gap_count,
        }
    }

    fn shortest_exact_one_gap_path(
        &self,
        from_anchor: GraphAnchor,
        to_anchor: GraphAnchor,
    ) -> Option<Vec<AuthoredCopperPlusOneGapStep>> {
        let start = *self.node_ids.get(&from_anchor)?;
        let target = *self.node_ids.get(&to_anchor)?;
        let mut queue = VecDeque::new();
        let mut visited: HashMap<
            (usize, GapUsageState),
            Option<((usize, GapUsageState), AuthoredCopperPlusOneGapStep)>,
        > =
            HashMap::new();

        queue.push_back((start, GapUsageState::BeforeGapNoAuthored));
        visited.insert((start, GapUsageState::BeforeGapNoAuthored), None);

        while let Some((node, state)) = queue.pop_front() {
            if node == target && state == GapUsageState::AfterGapWithAuthored {
                break;
            }

            for (next_node, step) in &self.adjacency[node] {
                let next_usage_state = match (state, step.kind) {
                    (GapUsageState::BeforeGapNoAuthored, AuthoredCopperPlusOneGapStepKind::Gap) => {
                        continue;
                    }
                    (GapUsageState::BeforeGapNoAuthored, _) => GapUsageState::BeforeGapWithAuthored,
                    (GapUsageState::BeforeGapWithAuthored, AuthoredCopperPlusOneGapStepKind::Gap) => {
                        GapUsageState::AfterGapNoAuthored
                    }
                    (GapUsageState::BeforeGapWithAuthored, _) => GapUsageState::BeforeGapWithAuthored,
                    (GapUsageState::AfterGapNoAuthored, AuthoredCopperPlusOneGapStepKind::Gap) => {
                        continue;
                    }
                    (GapUsageState::AfterGapNoAuthored, _) => GapUsageState::AfterGapWithAuthored,
                    (GapUsageState::AfterGapWithAuthored, AuthoredCopperPlusOneGapStepKind::Gap) => {
                        continue;
                    }
                    (GapUsageState::AfterGapWithAuthored, _) => GapUsageState::AfterGapWithAuthored,
                };
                let next_state = (*next_node, next_usage_state);
                if visited.contains_key(&next_state) {
                    continue;
                }
                visited.insert(next_state, Some(((node, state), step.clone())));
                queue.push_back(next_state);
            }
        }

        let target_state = (target, GapUsageState::AfterGapWithAuthored);
        if !visited.contains_key(&target_state) {
            return None;
        }

        let mut steps = Vec::new();
        let mut cursor = target_state;
        while let Some(Some((previous, step))) = visited.get(&cursor) {
            steps.push(step.clone());
            cursor = *previous;
        }
        steps.reverse();
        Some(steps)
    }
}

fn compare_step(
    left: &AuthoredCopperPlusOneGapStep,
    right: &AuthoredCopperPlusOneGapStep,
) -> Ordering {
    left.kind
        .cmp(&right.kind)
        .then_with(|| left.object_uuid.cmp(&right.object_uuid))
        .then_with(|| left.layer.cmp(&right.layer))
        .then_with(|| left.from.x.cmp(&right.from.x))
        .then_with(|| left.from.y.cmp(&right.from.y))
        .then_with(|| left.to.x.cmp(&right.to.x))
        .then_with(|| left.to.y.cmp(&right.to.y))
}
