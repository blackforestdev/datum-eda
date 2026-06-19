use super::*;
use crate::ir::geometry::LayerId;

pub(super) fn segment_length_nm(from: Point, to: Point) -> i64 {
    let dx = (to.x - from.x) as f64;
    let dy = (to.y - from.y) as f64;
    (dx.hypot(dy).round()) as i64
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PadPoint {
    pub(super) component: String,
    pub(super) pin: String,
    pub(super) uuid: Uuid,
    pub(super) position: Point,
    pub(super) layers: Vec<LayerId>,
    pub(super) shape: PadShape,
    pub(super) diameter: i64,
    pub(super) width: i64,
    pub(super) height: i64,
    pub(super) rotation: i32,
    pub(super) roundrect_rratio_ppm: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Anchor {
    point: Point,
    layer: LayerId,
}

#[derive(Debug, Clone, Copy)]
struct TrackAnchorRange {
    from_idx: usize,
    from: Point,
    to: Point,
    width: i64,
    layer: LayerId,
}

pub(super) struct BoardNetGraph {
    parents: Vec<usize>,
}

impl BoardNetGraph {
    pub(super) fn build(board: &Board, net: Uuid, pads: &[PadPoint]) -> Self {
        let mut anchors: Vec<Anchor> = Vec::new();
        let mut pad_representatives = Vec::with_capacity(pads.len());
        let mut pad_anchor_ranges = Vec::with_capacity(pads.len());
        for pad in pads {
            let start = anchors.len();
            let layers = if pad.layers.is_empty() {
                vec![0]
            } else {
                pad.layers.clone()
            };
            for layer in layers {
                anchors.push(Anchor {
                    point: pad.position,
                    layer,
                });
            }
            let end = anchors.len();
            pad_representatives.push(start);
            pad_anchor_ranges.push((start, end));
        }
        let pad_count = pads.len();
        let pad_anchor_count = anchors.len();

        let mut track_anchor_ranges = Vec::new();
        for track in board.tracks.values().filter(|track| track.net == net) {
            let from_idx = anchors.len();
            anchors.push(Anchor {
                point: track.from,
                layer: track.layer,
            });
            anchors.push(Anchor {
                point: track.to,
                layer: track.layer,
            });
            track_anchor_ranges.push(TrackAnchorRange {
                from_idx,
                from: track.from,
                to: track.to,
                width: track.width,
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

        let zone_source: Vec<_> = board
            .zones
            .values()
            .filter(|zone| zone.net == net)
            .filter_map(|zone| {
                zone.polygon
                    .vertices
                    .first()
                    .copied()
                    .map(|point| (zone, point))
            })
            .collect();
        let zone_anchor_start = anchors.len();
        for (zone, point) in &zone_source {
            anchors.push(Anchor {
                point: *point,
                layer: zone.layer,
            });
        }
        let zone_infos: Vec<_> = zone_source
            .into_iter()
            .enumerate()
            .map(|(i, (zone, _))| (zone, zone_anchor_start + i))
            .collect();

        let mut uf = UnionFind::new(anchors.len());

        for (start, end) in &pad_anchor_ranges {
            for idx in (*start + 1)..*end {
                uf.union(*start, idx);
            }
        }

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

        let mut cursor = pad_anchor_count;
        for _track in board.tracks.values().filter(|track| track.net == net) {
            uf.union(cursor, cursor + 1);
            cursor += 2;
        }

        for track in &track_anchor_ranges {
            for (anchor_idx, anchor) in anchors.iter().enumerate().take(zone_anchor_start) {
                if anchor.layer != track.layer {
                    continue;
                }
                if point_on_track_copper(anchor.point, *track) {
                    uf.union(track.from_idx, anchor_idx);
                }
            }
        }

        cursor = via_start;
        for _via in board.vias.values().filter(|via| via.net == net) {
            uf.union(cursor, cursor + 1);
            cursor += 2;
        }

        for (pad_idx, pad) in pads.iter().enumerate() {
            let representative = pad_representatives[pad_idx];
            for (anchor_idx, anchor) in anchors.iter().enumerate().skip(pad_anchor_count) {
                if !pad.layers.contains(&anchor.layer) {
                    continue;
                }
                if point_in_pad_copper(anchor.point, pad) {
                    uf.union(representative, anchor_idx);
                }
            }
        }

        for (zone, zone_anchor_idx) in zone_infos {
            for (pad_idx, pad) in pads.iter().enumerate() {
                if !pad.layers.contains(&zone.layer) {
                    continue;
                }
                if pad_intersects_zone(pad, zone) {
                    uf.union(zone_anchor_idx, pad_representatives[pad_idx]);
                }
            }
            for (anchor_idx, anchor) in anchors.iter().enumerate().take(zone_anchor_start) {
                if anchor.layer == zone.layer
                    && polygon::point_in_polygon(anchor.point, &zone.polygon)
                {
                    uf.union(zone_anchor_idx, anchor_idx);
                }
            }
        }

        Self {
            parents: (0..pad_count)
                .map(|idx| uf.find(pad_representatives[idx]))
                .collect(),
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

fn point_in_pad_copper(point: Point, pad: &PadPoint) -> bool {
    let local = world_to_pad_local(point, pad.position, pad.rotation);
    match pad.shape {
        PadShape::Circle => {
            let radius = effective_circle_radius_nm(pad);
            local.x * local.x + local.y * local.y <= radius * radius
        }
        PadShape::Rect => {
            let half_w = (pad.width / 2).max(0);
            let half_h = (pad.height / 2).max(0);
            local.x.abs() <= half_w && local.y.abs() <= half_h
        }
        PadShape::Oval => point_in_oval(local, pad.width.max(0), pad.height.max(0)),
        PadShape::RoundRect => point_in_roundrect(
            local,
            pad.width.max(0),
            pad.height.max(0),
            roundrect_radius_nm(pad),
        ),
    }
}

fn pad_intersects_zone(pad: &PadPoint, zone: &Zone) -> bool {
    pad_sample_points_world(pad)
        .into_iter()
        .any(|point| polygon::point_in_polygon(point, &zone.polygon))
}

fn world_to_pad_local(point: Point, origin: Point, rotation_deg: i32) -> Point {
    let dx = point.x - origin.x;
    let dy = point.y - origin.y;
    if rotation_deg == 0 {
        return Point::new(dx, dy);
    }
    let rad = (rotation_deg as f64).to_radians();
    let cos = rad.cos();
    let sin = rad.sin();
    let lx = (dx as f64 * cos - dy as f64 * sin).round() as i64;
    let ly = (dx as f64 * sin + dy as f64 * cos).round() as i64;
    Point::new(lx, ly)
}

fn pad_sample_points_world(pad: &PadPoint) -> Vec<Point> {
    let mut samples = vec![Point::zero()];
    match pad.shape {
        PadShape::Circle => {
            let r = effective_circle_radius_nm(pad);
            samples.extend([
                Point::new(r, 0),
                Point::new(-r, 0),
                Point::new(0, r),
                Point::new(0, -r),
            ]);
        }
        PadShape::Rect | PadShape::RoundRect => {
            let half_w = (pad.width / 2).max(0);
            let half_h = (pad.height / 2).max(0);
            samples.extend([
                Point::new(half_w, 0),
                Point::new(-half_w, 0),
                Point::new(0, half_h),
                Point::new(0, -half_h),
                Point::new(half_w / 2, half_h / 2),
                Point::new(-half_w / 2, half_h / 2),
                Point::new(half_w / 2, -half_h / 2),
                Point::new(-half_w / 2, -half_h / 2),
            ]);
        }
        PadShape::Oval => {
            let half_w = (pad.width / 2).max(0);
            let half_h = (pad.height / 2).max(0);
            samples.extend([
                Point::new(half_w, 0),
                Point::new(-half_w, 0),
                Point::new(0, half_h),
                Point::new(0, -half_h),
            ]);
        }
    }
    samples
        .into_iter()
        .map(|local| pad_local_to_world(local, pad.position, pad.rotation))
        .collect()
}

fn pad_local_to_world(local: Point, origin: Point, rotation_deg: i32) -> Point {
    if rotation_deg == 0 {
        return Point::new(origin.x + local.x, origin.y + local.y);
    }
    let rad = -(rotation_deg as f64).to_radians();
    let cos = rad.cos();
    let sin = rad.sin();
    let rx = (local.x as f64 * cos - local.y as f64 * sin).round() as i64;
    let ry = (local.x as f64 * sin + local.y as f64 * cos).round() as i64;
    Point::new(origin.x + rx, origin.y + ry)
}

fn effective_circle_radius_nm(pad: &PadPoint) -> i64 {
    let by_diameter = pad.diameter / 2;
    let by_size = pad.width.min(pad.height) / 2;
    by_diameter.max(by_size).max(0)
}

fn point_in_oval(local: Point, width: i64, height: i64) -> bool {
    let half_w = width / 2;
    let half_h = height / 2;
    if half_w <= 0 || half_h <= 0 {
        return false;
    }
    if width == height {
        return local.x * local.x + local.y * local.y <= half_w * half_w;
    }
    if width > height {
        let rect_half = half_w - half_h;
        if local.x.abs() <= rect_half && local.y.abs() <= half_h {
            return true;
        }
        let cx = if local.x >= 0 { rect_half } else { -rect_half };
        let dx = local.x - cx;
        dx * dx + local.y * local.y <= half_h * half_h
    } else {
        let rect_half = half_h - half_w;
        if local.y.abs() <= rect_half && local.x.abs() <= half_w {
            return true;
        }
        let cy = if local.y >= 0 { rect_half } else { -rect_half };
        let dy = local.y - cy;
        local.x * local.x + dy * dy <= half_w * half_w
    }
}

fn point_in_roundrect(local: Point, width: i64, height: i64, radius: i64) -> bool {
    let half_w = width / 2;
    let half_h = height / 2;
    if half_w <= 0 || half_h <= 0 {
        return false;
    }
    let r = radius.max(0).min(half_w.min(half_h));
    if r == 0 {
        return local.x.abs() <= half_w && local.y.abs() <= half_h;
    }
    let inner_x = half_w - r;
    let inner_y = half_h - r;
    if local.x.abs() <= inner_x && local.y.abs() <= half_h {
        return true;
    }
    if local.y.abs() <= inner_y && local.x.abs() <= half_w {
        return true;
    }
    let cx = if local.x >= 0 { inner_x } else { -inner_x };
    let cy = if local.y >= 0 { inner_y } else { -inner_y };
    let dx = local.x - cx;
    let dy = local.y - cy;
    dx * dx + dy * dy <= r * r
}

fn roundrect_radius_nm(pad: &PadPoint) -> i64 {
    let min_dim = pad.width.min(pad.height).max(0);
    ((min_dim as i128 * pad.roundrect_rratio_ppm as i128) / 1_000_000i128) as i64
}

fn point_on_track_copper(point: Point, track: TrackAnchorRange) -> bool {
    point_to_segment_distance_nm(point, track.from, track.to) <= (track.width / 2).max(0)
}

fn point_to_segment_distance_nm(point: Point, from: Point, to: Point) -> i64 {
    let px = point.x as f64;
    let py = point.y as f64;
    let x0 = from.x as f64;
    let y0 = from.y as f64;
    let x1 = to.x as f64;
    let y1 = to.y as f64;
    let dx = x1 - x0;
    let dy = y1 - y0;
    if dx == 0.0 && dy == 0.0 {
        return ((px - x0).hypot(py - y0).round()) as i64;
    }
    let t = ((px - x0) * dx + (py - y0) * dy) / (dx * dx + dy * dy);
    let clamped_t = t.clamp(0.0, 1.0);
    let cx = x0 + clamped_t * dx;
    let cy = y0 + clamped_t * dy;
    ((px - cx).hypot(py - cy).round()) as i64
}
