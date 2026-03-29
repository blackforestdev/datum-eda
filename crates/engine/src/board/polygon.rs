use crate::ir::geometry::{Point, Polygon};

pub(super) fn point_in_polygon(point: Point, polygon: &Polygon) -> bool {
    let Some(bounds) = polygon.bounding_box() else {
        return false;
    };
    if !bounds.contains(&point) {
        return false;
    }

    let vertices = &polygon.vertices;
    if vertices.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = vertices.len() - 1;
    for i in 0..vertices.len() {
        let xi = vertices[i].x as f64;
        let yi = vertices[i].y as f64;
        let xj = vertices[j].x as f64;
        let yj = vertices[j].y as f64;
        let px = point.x as f64;
        let py = point.y as f64;

        let intersects =
            ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / ((yj - yi).max(1.0)) + xi);
        if intersects {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub(super) fn segment_intersects_polygon(from: Point, to: Point, polygon: &Polygon) -> bool {
    if point_in_polygon(from, polygon) || point_in_polygon(to, polygon) {
        return true;
    }

    polygon_edges(polygon).any(|(a, b)| segments_intersect(from, to, a, b))
}

pub(super) fn polygons_intersect(a: &Polygon, b: &Polygon) -> bool {
    if a.vertices
        .iter()
        .copied()
        .any(|point| point_in_polygon(point, b))
        || b.vertices
            .iter()
            .copied()
            .any(|point| point_in_polygon(point, a))
    {
        return true;
    }

    polygon_edges(a)
        .any(|(a0, a1)| polygon_edges(b).any(|(b0, b1)| segments_intersect(a0, a1, b0, b1)))
}

pub(super) fn segment_intersects_segment(a0: Point, a1: Point, b0: Point, b1: Point) -> bool {
    segments_intersect(a0, a1, b0, b1)
}

pub(super) fn segment_escapes_polygon(from: Point, to: Point, polygon: &Polygon) -> bool {
    if !point_in_or_on_polygon(from, polygon) || !point_in_or_on_polygon(to, polygon) {
        return true;
    }

    let mut crossings = vec![0.0, 1.0];
    for (edge_from, edge_to) in polygon_edges(polygon) {
        crossings.extend(segment_edge_crossing_parameters(
            from, to, edge_from, edge_to,
        ));
    }
    crossings.sort_by(|a, b| a.total_cmp(b));
    crossings.dedup_by(|a, b| (*a - *b).abs() <= 1e-9);

    crossings.windows(2).any(|window| {
        let start = window[0];
        let end = window[1];
        if end - start <= 1e-9 {
            return false;
        }
        let sample = point_at(from, to, (start + end) / 2.0);
        !point_in_or_on_polygon(sample, polygon)
    })
}

pub(super) fn polygon_escapes_polygon(candidate: &Polygon, boundary: &Polygon) -> bool {
    polygon_edges(candidate).any(|(from, to)| segment_escapes_polygon(from, to, boundary))
}

pub(super) fn point_to_segment_distance_nm(point: Point, from: Point, to: Point) -> i64 {
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

fn polygon_edges(polygon: &Polygon) -> impl Iterator<Item = (Point, Point)> + '_ {
    polygon
        .vertices
        .iter()
        .copied()
        .zip(polygon.vertices.iter().copied().cycle().skip(1))
        .take(polygon.vertices.len())
}

fn segments_intersect(a0: Point, a1: Point, b0: Point, b1: Point) -> bool {
    let o1 = orientation(a0, a1, b0);
    let o2 = orientation(a0, a1, b1);
    let o3 = orientation(b0, b1, a0);
    let o4 = orientation(b0, b1, a1);

    if o1 != o2 && o3 != o4 {
        return true;
    }

    (o1 == 0 && point_on_segment(b0, a0, a1))
        || (o2 == 0 && point_on_segment(b1, a0, a1))
        || (o3 == 0 && point_on_segment(a0, b0, b1))
        || (o4 == 0 && point_on_segment(a1, b0, b1))
}

pub(super) fn point_in_or_on_polygon(point: Point, polygon: &Polygon) -> bool {
    point_in_polygon(point, polygon) || point_on_polygon_boundary(point, polygon)
}

fn point_on_polygon_boundary(point: Point, polygon: &Polygon) -> bool {
    polygon_edges(polygon)
        .any(|(from, to)| orientation(from, to, point) == 0 && point_on_segment(point, from, to))
}

fn point_on_segment(point: Point, from: Point, to: Point) -> bool {
    point.x >= from.x.min(to.x)
        && point.x <= from.x.max(to.x)
        && point.y >= from.y.min(to.y)
        && point.y <= from.y.max(to.y)
}

fn orientation(a: Point, b: Point, c: Point) -> i32 {
    let cross =
        (b.y - a.y) as i128 * (c.x - b.x) as i128 - (b.x - a.x) as i128 * (c.y - b.y) as i128;
    if cross == 0 {
        0
    } else if cross > 0 {
        1
    } else {
        2
    }
}

fn segment_edge_crossing_parameters(
    from: Point,
    to: Point,
    edge_from: Point,
    edge_to: Point,
) -> Vec<f64> {
    let p = (from.x as f64, from.y as f64);
    let r = ((to.x - from.x) as f64, (to.y - from.y) as f64);
    let q = (edge_from.x as f64, edge_from.y as f64);
    let s = (
        (edge_to.x - edge_from.x) as f64,
        (edge_to.y - edge_from.y) as f64,
    );
    let r_cross_s = cross(r, s);
    let q_minus_p = (q.0 - p.0, q.1 - p.1);
    let qmp_cross_r = cross(q_minus_p, r);

    if r_cross_s.abs() <= 1e-9 && qmp_cross_r.abs() <= 1e-9 {
        let mut parameters = Vec::new();
        if let Some(t) = parameter_on_segment(from, to, edge_from) {
            parameters.push(t);
        }
        if let Some(t) = parameter_on_segment(from, to, edge_to) {
            parameters.push(t);
        }
        return parameters;
    }

    if r_cross_s.abs() <= 1e-9 {
        return Vec::new();
    }

    let t = cross(q_minus_p, s) / r_cross_s;
    let u = cross(q_minus_p, r) / r_cross_s;
    if (-1e-9..=1.0 + 1e-9).contains(&t) && (-1e-9..=1.0 + 1e-9).contains(&u) {
        vec![t.clamp(0.0, 1.0)]
    } else {
        Vec::new()
    }
}

fn parameter_on_segment(from: Point, to: Point, point: Point) -> Option<f64> {
    if orientation(from, to, point) != 0 || !point_on_segment(point, from, to) {
        return None;
    }

    let dx = (to.x - from.x) as f64;
    let dy = (to.y - from.y) as f64;
    if dx.abs() >= dy.abs() {
        if dx.abs() <= 1e-9 {
            Some(0.0)
        } else {
            Some(((point.x - from.x) as f64 / dx).clamp(0.0, 1.0))
        }
    } else if dy.abs() <= 1e-9 {
        Some(0.0)
    } else {
        Some(((point.y - from.y) as f64 / dy).clamp(0.0, 1.0))
    }
}

fn point_at(from: Point, to: Point, t: f64) -> Point {
    Point::new(
        (from.x as f64 + ((to.x - from.x) as f64 * t)).round() as i64,
        (from.y as f64 + ((to.y - from.y) as f64 * t)).round() as i64,
    )
}

fn cross(a: (f64, f64), b: (f64, f64)) -> f64 {
    a.0 * b.1 - a.1 * b.0
}
