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
