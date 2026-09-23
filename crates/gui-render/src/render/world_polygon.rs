//! World polygon fill and admitted scanline scratch.
use super::*;

pub(super) fn push_world_polygon_fill(
    out: &mut impl Output<Quad>,
    polygon: &[PointNm],
    color: [f32; 3],
) {
    push_world_polygon_fill_contours(out, polygon, &[], color);
}

pub(super) fn push_world_polygon_fill_contours(
    out: &mut impl Output<Quad>,
    outer: &[PointNm],
    holes: &[Vec<PointNm>],
    color: [f32; 3],
) {
    if holes.is_empty() {
        if outer.len() == 3 && is_convex_polygon_nm(outer) {
            push_world_triangle(
                out,
                (outer[0].x as f32, outer[0].y as f32),
                (outer[1].x as f32, outer[1].y as f32),
                (outer[2].x as f32, outer[2].y as f32),
                color,
            );
            return;
        }
        if outer.len() == 4 && is_convex_polygon_nm(outer) {
            push_world_quad(
                out,
                &[
                    (outer[0].x as f32, outer[0].y as f32),
                    (outer[1].x as f32, outer[1].y as f32),
                    (outer[2].x as f32, outer[2].y as f32),
                    (outer[3].x as f32, outer[3].y as f32),
                ],
                color,
            );
            return;
        }
        match clean_polygon_ring_nm(out, outer) {
            Some(cleaned) if cleaned.len() == 3 && is_convex_polygon_nm(&cleaned) => {
                push_world_triangle(
                    out,
                    (cleaned[0].x as f32, cleaned[0].y as f32),
                    (cleaned[1].x as f32, cleaned[1].y as f32),
                    (cleaned[2].x as f32, cleaned[2].y as f32),
                    color,
                );
                return;
            }
            Some(cleaned) if cleaned.len() == 4 && is_convex_polygon_nm(&cleaned) => {
                push_world_quad(
                    out,
                    &[
                        (cleaned[0].x as f32, cleaned[0].y as f32),
                        (cleaned[1].x as f32, cleaned[1].y as f32),
                        (cleaned[2].x as f32, cleaned[2].y as f32),
                        (cleaned[3].x as f32, cleaned[3].y as f32),
                    ],
                    color,
                );
                return;
            }
            Some(cleaned) => {
                push_world_polygon_fill_scanline_contours(out, &[cleaned], color);
                return;
            }
            None => return,
        }
    }

    let Some(mut contours) = out.scratch(holes.len().saturating_add(1)) else {
        return;
    };
    if let Some(cleaned_outer) = clean_polygon_ring_nm(out, outer) {
        contours.push(cleaned_outer);
    }
    for hole in holes {
        if let Some(cleaned_hole) = clean_polygon_ring_nm(out, hole) {
            contours.push(cleaned_hole);
        }
    }
    if contours.is_empty() {
        return;
    }
    push_world_polygon_fill_scanline_contours(out, &contours, color);
}

fn is_convex_polygon_nm(polygon: &[PointNm]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let mut sign = 0_i128;
    for index in 0..polygon.len() {
        let a = polygon[index];
        let b = polygon[(index + 1) % polygon.len()];
        let c = polygon[(index + 2) % polygon.len()];
        let abx = (b.x - a.x) as i128;
        let aby = (b.y - a.y) as i128;
        let bcx = (c.x - b.x) as i128;
        let bcy = (c.y - b.y) as i128;
        let cross = abx * bcy - aby * bcx;
        if cross == 0 {
            continue;
        }
        if sign == 0 {
            sign = cross.signum();
        } else if cross.signum() != sign {
            return false;
        }
    }
    sign != 0
}

fn clean_polygon_ring_nm(out: &mut impl Output<Quad>, polygon: &[PointNm]) -> Option<Vec<PointNm>> {
    if polygon.len() < 3 {
        return None;
    }
    let mut cleaned: Vec<PointNm> = out.scratch(polygon.len())?;
    for &point in polygon {
        if cleaned
            .last()
            .is_some_and(|last| last.x == point.x && last.y == point.y)
        {
            continue;
        }
        cleaned.push(point);
    }
    if cleaned.len() >= 2
        && cleaned.first().is_some_and(|first| {
            cleaned
                .last()
                .is_some_and(|last| last.x == first.x && last.y == first.y)
        })
    {
        cleaned.pop();
    }
    if cleaned.len() < 3 {
        return None;
    }
    Some(cleaned)
}

fn push_world_polygon_fill_scanline_contours(
    out: &mut impl Output<Quad>,
    contours: &[Vec<PointNm>],
    color: [f32; 3],
) {
    const EPS: f64 = 1e-6;
    #[derive(Clone, Copy)]
    struct ScanlineEdge {
        min_y: f64,
        max_y: f64,
        ax: f64,
        ay: f64,
        bx: f64,
        by: f64,
        order: usize,
    }

    impl ScanlineEdge {
        fn x_at(self, y: f64) -> f64 {
            let t = (y - self.ay) / (self.by - self.ay);
            self.ax + (self.bx - self.ax) * t
        }
    }

    let points = contours
        .iter()
        .fold(0usize, |n, p| n.saturating_add(p.len()));
    let Some(mut ys) = out.scratch::<f64>(points) else {
        return;
    };
    ys.extend(
        contours
            .iter()
            .flat_map(|polygon| polygon.iter().map(|p| p.y as f64)),
    );
    ys.sort_unstable_by(|a, b| a.total_cmp(b));
    ys.dedup_by(|a, b| (*a - *b).abs() <= EPS);
    if ys.len() < 2 {
        return;
    }

    let Some(mut edges) = out.scratch::<ScanlineEdge>(points) else {
        return;
    };
    for polygon in contours {
        for i in 0..polygon.len() {
            let a = polygon[i];
            let b = polygon[(i + 1) % polygon.len()];
            let ay = a.y as f64;
            let by = b.y as f64;
            if (ay - by).abs() <= EPS {
                continue;
            }
            edges.push(ScanlineEdge {
                min_y: ay.min(by),
                max_y: ay.max(by),
                ax: a.x as f64,
                ay,
                bx: b.x as f64,
                by,
                order: edges.len(),
            });
        }
    }
    edges.sort_unstable_by(|a, b| {
        a.min_y
            .total_cmp(&b.min_y)
            .then_with(|| a.max_y.total_cmp(&b.max_y))
            .then_with(|| a.ax.total_cmp(&b.ax))
            .then_with(|| a.bx.total_cmp(&b.bx))
            .then_with(|| a.order.cmp(&b.order))
    });

    let mut next_edge = 0;
    let Some(mut active_edges) = out.scratch::<ScanlineEdge>(edges.len()) else {
        return;
    };
    let Some(mut spans) = out.scratch::<(f64, f64, f64, usize)>(edges.len()) else {
        return;
    };
    for band in ys.windows(2) {
        let y0 = band[0];
        let y1 = band[1];
        if y1 - y0 <= EPS {
            continue;
        }
        let y_mid = (y0 + y1) * 0.5;

        while next_edge < edges.len() && edges[next_edge].min_y <= y_mid {
            active_edges.push(edges[next_edge]);
            next_edge += 1;
        }
        active_edges.retain(|edge| y_mid < edge.max_y);

        spans.clear();
        for (order, edge) in active_edges.iter().enumerate() {
            if y_mid < edge.min_y || y_mid >= edge.max_y {
                continue;
            }
            spans.push((edge.x_at(y_mid), edge.x_at(y0), edge.x_at(y1), order));
        }
        // Original emission order breaks ties exactly as the stable sort did,
        // without allocating sorting scratch for every band.
        spans.sort_unstable_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.3.cmp(&b.3)));
        for pair in spans.chunks_exact(2) {
            let left = pair[0];
            let right = pair[1];
            if right.0 - left.0 <= EPS {
                continue;
            }
            push_world_quad(
                out,
                &[
                    (left.1 as f32, y0 as f32),
                    (right.1 as f32, y0 as f32),
                    (right.2 as f32, y1 as f32),
                    (left.2 as f32, y1 as f32),
                ],
                color,
            );
        }
    }
}

#[cfg(test)]
#[path = "world_polygon_tests.rs"]
mod tests;
