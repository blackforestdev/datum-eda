use super::*;

pub(super) fn segment_length_nm(from: Point, to: Point) -> i64 {
    let dx = (to.x - from.x) as f64;
    let dy = (to.y - from.y) as f64;
    (dx.hypot(dy).round()) as i64
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PadPoint {
    pub(super) component: String,
    pub(super) pin: String,
    pub(super) position: Point,
    pub(super) layer: LayerId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Anchor {
    point: Point,
    layer: LayerId,
}

pub(super) struct BoardNetGraph {
    parents: Vec<usize>,
}

impl BoardNetGraph {
    pub(super) fn build(board: &Board, net: Uuid, pads: &[PadPoint]) -> Self {
        let mut anchors: Vec<Anchor> = pads
            .iter()
            .map(|pad| Anchor {
                point: pad.position,
                layer: pad.layer,
            })
            .collect();
        let pad_count = anchors.len();

        for track in board.tracks.values().filter(|track| track.net == net) {
            anchors.push(Anchor {
                point: track.from,
                layer: track.layer,
            });
            anchors.push(Anchor {
                point: track.to,
                layer: track.layer,
            });
        }

        let via_start = anchors.len();
        for via in board.vias.values().filter(|via| via.net == net) {
            anchors.push(Anchor {
                point: via.position,
                layer: via.from_layer,
            });
            anchors.push(Anchor {
                point: via.position,
                layer: via.to_layer,
            });
        }

        let mut uf = UnionFind::new(anchors.len());

        let mut index_by_anchor: HashMap<Anchor, Vec<usize>> = HashMap::new();
        for (idx, anchor) in anchors.iter().copied().enumerate() {
            index_by_anchor.entry(anchor).or_default().push(idx);
        }
        for indices in index_by_anchor.values() {
            if let Some((&first, rest)) = indices.split_first() {
                for &idx in rest {
                    uf.union(first, idx);
                }
            }
        }

        let mut cursor = pad_count;
        for _track in board.tracks.values().filter(|track| track.net == net) {
            uf.union(cursor, cursor + 1);
            cursor += 2;
        }

        cursor = via_start;
        for _via in board.vias.values().filter(|via| via.net == net) {
            uf.union(cursor, cursor + 1);
            cursor += 2;
        }

        for zone in board.zones.values().filter(|zone| zone.net == net) {
            let contained: Vec<_> = anchors
                .iter()
                .enumerate()
                .filter(|(_, anchor)| {
                    anchor.layer == zone.layer
                        && polygon::point_in_polygon(anchor.point, &zone.polygon)
                })
                .map(|(idx, _)| idx)
                .collect();
            if let Some((&first, rest)) = contained.split_first() {
                for &idx in rest {
                    uf.union(first, idx);
                }
            }
        }

        Self {
            parents: (0..pad_count).map(|idx| uf.find(idx)).collect(),
        }
    }

    pub(super) fn root_of_pad(&self, pad_idx: usize) -> usize {
        self.parents[pad_idx]
    }
}

struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    fn find(&mut self, idx: usize) -> usize {
        if self.parent[idx] != idx {
            let root = self.find(self.parent[idx]);
            self.parent[idx] = root;
        }
        self.parent[idx]
    }

    fn union(&mut self, a: usize, b: usize) {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a != root_b {
            self.parent[root_b] = root_a;
        }
    }
}

pub(super) fn nearest_pin_pair(from: &[PadPoint], to: &[PadPoint]) -> (usize, usize, i64) {
    let mut best = (0usize, 0usize, i64::MAX);
    for (i, from_pin) in from.iter().enumerate() {
        for (j, to_pin) in to.iter().enumerate() {
            let distance_nm = segment_length_nm(from_pin.position, to_pin.position);
            if distance_nm < best.2 {
                best = (i, j, distance_nm);
            }
        }
    }
    best
}
