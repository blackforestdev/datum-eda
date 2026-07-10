fn push_points(
    out: &mut Vec<Quad>,
    points: &[PointNm],
    projection: &Projection,
    color: [f32; 3],
    size_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    for point in points {
        rects.push(push_point_square(out, *point, projection, size_px, color));
    }
    rects
}

#[allow(dead_code)]
fn push_projected_round_rect(out: &mut Vec<Quad>, rect: RectPx, color: [f32; 3], radius_px: f32) {
    let radius = radius_px.min(rect.width * 0.5).min(rect.height * 0.5);
    if radius <= 0.75 {
        out.push(Quad::from_rect(rect, color));
        return;
    }
    let center = RectPx {
        x: rect.x + radius,
        y: rect.y,
        width: (rect.width - radius * 2.0).max(0.0),
        height: rect.height,
    };
    if center.width > 0.0 && center.height > 0.0 {
        out.push(Quad::from_rect(center, color));
    }
    let middle = RectPx {
        x: rect.x,
        y: rect.y + radius,
        width: rect.width,
        height: (rect.height - radius * 2.0).max(0.0),
    };
    if middle.width > 0.0 && middle.height > 0.0 {
        out.push(Quad::from_rect(middle, color));
    }
    let diameter = radius * 2.0;
    for (x, y) in [
        (rect.x, rect.y),
        (rect.x + rect.width - diameter, rect.y),
        (rect.x, rect.y + rect.height - diameter),
        (
            rect.x + rect.width - diameter,
            rect.y + rect.height - diameter,
        ),
    ] {
        push_projected_ellipse(
            out,
            RectPx {
                x,
                y,
                width: diameter,
                height: diameter,
            },
            color,
            48,
        );
    }
}

fn push_dashed_polyline_segments(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    projection: &Projection,
    color: [f32; 3],
    thickness_px: f32,
    dash_px: f32,
    gap_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    for segment in path.windows(2) {
        let a = project_point(segment[0], projection);
        let b = project_point(segment[1], projection);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let ux = dx / len;
        let uy = dy / len;
        let step = (dash_px + gap_px).max(1.0);
        let mut start = 0.0;
        while start < len {
            let end = (start + dash_px).min(len);
            if end > start {
                let start_point = (a.0 + ux * start, a.1 + uy * start);
                let end_point = (a.0 + ux * end, a.1 + uy * end);
                let seg_dx = end_point.0 - start_point.0;
                let seg_dy = end_point.1 - start_point.1;
                let seg_len = (seg_dx * seg_dx + seg_dy * seg_dy).sqrt().max(1.0);
                let nx = -seg_dy / seg_len * thickness_px * 0.5;
                let ny = seg_dx / seg_len * thickness_px * 0.5;
                let quad = [
                    (start_point.0 + nx, start_point.1 + ny),
                    (end_point.0 + nx, end_point.1 + ny),
                    (end_point.0 - nx, end_point.1 - ny),
                    (start_point.0 - nx, start_point.1 - ny),
                ];
                rects.push(bounds_from_projected_points(&quad));
                push_projected_quad(out, &quad, color);
            }
            start += step;
        }
    }
    rects
}

fn push_polyline_endcaps(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    projection: &Projection,
    color: [f32; 3],
    thickness_px: f32,
    cap_length_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    if path.len() < 2 {
        return rects;
    }

    let first_a = project_point(path[0], projection);
    let first_b = project_point(path[1], projection);
    if let Some(quad) = projected_cap_quad(first_a, first_b, thickness_px, cap_length_px) {
        rects.push(bounds_from_projected_points(&quad));
        push_projected_quad(out, &quad, color);
    }

    let last_a = project_point(path[path.len() - 1], projection);
    let last_b = project_point(path[path.len() - 2], projection);
    if let Some(quad) = projected_cap_quad(last_a, last_b, thickness_px, cap_length_px) {
        rects.push(bounds_from_projected_points(&quad));
        push_projected_quad(out, &quad, color);
    }

    rects
}

fn push_polyline_segments(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    projection: &Projection,
    color: [f32; 3],
    thickness_px: f32,
) -> Vec<RectPx> {
    let mut rects = Vec::new();
    for segment in path.windows(2) {
        let a = project_point(segment[0], projection);
        let b = project_point(segment[1], projection);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let nx = -dy / len * thickness_px * 0.5;
        let ny = dx / len * thickness_px * 0.5;
        let quad = [
            (a.0 + nx, a.1 + ny),
            (b.0 + nx, b.1 + ny),
            (b.0 - nx, b.1 - ny),
            (a.0 - nx, a.1 - ny),
        ];
        let rect = bounds_from_projected_points(&quad);
        rects.push(rect);
        push_projected_quad(out, &quad, color);
    }
    rects
}

fn projected_cap_quad(
    start: (f32, f32),
    toward: (f32, f32),
    thickness_px: f32,
    cap_length_px: f32,
) -> Option<[(f32, f32); 4]> {
    let dx = toward.0 - start.0;
    let dy = toward.1 - start.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.01 {
        return None;
    }
    let ux = dx / len;
    let uy = dy / len;
    let end = (
        start.0 + ux * cap_length_px.min(len),
        start.1 + uy * cap_length_px.min(len),
    );
    let nx = -uy * thickness_px * 0.5;
    let ny = ux * thickness_px * 0.5;
    Some([
        (start.0 + nx, start.1 + ny),
        (end.0 + nx, end.1 + ny),
        (end.0 - nx, end.1 - ny),
        (start.0 - nx, start.1 - ny),
    ])
}

fn close_path(points: &[PointNm]) -> Vec<PointNm> {
    let mut out = points.to_vec();
    if let (Some(first), Some(last)) = (out.first().copied(), out.last().copied())
        && first != last
    {
        out.push(first);
    }
    out
}

#[allow(dead_code)]
fn push_world_rect(
    out: &mut Vec<Quad>,
    rect: datum_gui_protocol::RectNm,
    projection: &Projection,
    color: [f32; 3],
) -> RectPx {
    let (x0, y0) = project_point(
        PointNm {
            x: rect.min_x,
            y: rect.min_y,
        },
        projection,
    );
    let (x1, y1) = project_point(
        PointNm {
            x: rect.max_x,
            y: rect.max_y,
        },
        projection,
    );
    let px = RectPx {
        x: x0,
        y: y0,
        width: (x1 - x0).max(1.0),
        height: (y1 - y0).max(1.0),
    };
    out.push(Quad::from_rect(px, color));
    px
}

fn project_rect(rect: datum_gui_protocol::RectNm, projection: &Projection) -> RectPx {
    projection.project_rect(rect)
}

fn push_point_square(
    out: &mut Vec<Quad>,
    point: PointNm,
    projection: &Projection,
    size_px: f32,
    color: [f32; 3],
) -> RectPx {
    let (x, y) = project_point(point, projection);
    let rect = RectPx {
        x: x - size_px * 0.5,
        y: y - size_px * 0.5,
        width: size_px.max(1.0),
        height: size_px.max(1.0),
    };
    out.push(Quad::from_rect(rect, color));
    rect
}

fn project_point(point: PointNm, projection: &Projection) -> (f32, f32) {
    projection.project_point(point)
}

fn world_stroke_nm(thickness_px: f32, projection: &Projection) -> f32 {
    (thickness_px / projection.scale).max(1.0)
}

fn push_world_quad(out: &mut Vec<Quad>, quad: &[(f32, f32); 4], color: [f32; 3]) {
    out.push(Quad {
        points: *quad,
        color,
    });
}

fn push_world_triangle(
    out: &mut Vec<Quad>,
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
    color: [f32; 3],
) {
    out.push(Quad {
        points: [a, b, c, c],
        color,
    });
}

fn push_convex_polygon_fill(out: &mut Vec<Quad>, polygon: &[(f32, f32)], color: [f32; 3]) {
    if polygon.len() < 3 {
        return;
    }
    let origin = polygon[0];
    for edge in polygon[1..].windows(2) {
        push_world_triangle(out, origin, edge[0], edge[1], color);
    }
}

fn push_world_rect_nm(out: &mut Vec<Quad>, rect: datum_gui_protocol::RectNm, color: [f32; 3]) {
    out.push(Quad {
        points: [
            (rect.min_x as f32, rect.min_y as f32),
            (rect.max_x as f32, rect.min_y as f32),
            (rect.max_x as f32, rect.max_y as f32),
            (rect.min_x as f32, rect.max_y as f32),
        ],
        color,
    });
}

#[allow(dead_code)]
fn push_world_rect_border_nm(
    out: &mut Vec<Quad>,
    rect: datum_gui_protocol::RectNm,
    color: [f32; 3],
    thickness_nm: f32,
) {
    let t = thickness_nm.max(1.0).round() as i64;
    let top = datum_gui_protocol::RectNm {
        min_x: rect.min_x,
        min_y: rect.min_y,
        max_x: rect.max_x,
        max_y: rect.min_y + t,
    };
    let bottom = datum_gui_protocol::RectNm {
        min_x: rect.min_x,
        min_y: rect.max_y - t,
        max_x: rect.max_x,
        max_y: rect.max_y,
    };
    let left = datum_gui_protocol::RectNm {
        min_x: rect.min_x,
        min_y: rect.min_y,
        max_x: rect.min_x + t,
        max_y: rect.max_y,
    };
    let right = datum_gui_protocol::RectNm {
        min_x: rect.max_x - t,
        min_y: rect.min_y,
        max_x: rect.max_x,
        max_y: rect.max_y,
    };
    for edge in [top, bottom, left, right] {
        if edge.max_x > edge.min_x && edge.max_y > edge.min_y {
            push_world_rect_nm(out, edge, color);
        }
    }
}

fn world_inset_rect(rect: datum_gui_protocol::RectNm, inset_nm: f32) -> datum_gui_protocol::RectNm {
    let inset = inset_nm.max(0.0).round() as i64;
    datum_gui_protocol::RectNm {
        min_x: rect.min_x + inset,
        min_y: rect.min_y + inset,
        max_x: rect.max_x - inset,
        max_y: rect.max_y - inset,
    }
}

fn push_world_ellipse_nm(
    out: &mut Vec<Quad>,
    rect: datum_gui_protocol::RectNm,
    color: [f32; 3],
    segments: usize,
) {
    let width = (rect.max_x - rect.min_x) as f32;
    let height = (rect.max_y - rect.min_y) as f32;
    if width <= 1.0 || height <= 1.0 || segments < 3 {
        return;
    }
    let cx = (rect.min_x + rect.max_x) as f32 * 0.5;
    let cy = (rect.min_y + rect.max_y) as f32 * 0.5;
    let rx = width * 0.5;
    let ry = height * 0.5;
    let step = std::f32::consts::TAU / segments as f32;
    let mut prev = (cx + rx, cy);
    for i in 1..=segments {
        let angle = step * i as f32;
        let next = (cx + rx * angle.cos(), cy + ry * angle.sin());
        push_world_triangle(out, (cx, cy), prev, next, color);
        prev = next;
    }
}

#[allow(dead_code)]
fn push_world_round_rect_nm(
    out: &mut Vec<Quad>,
    rect: datum_gui_protocol::RectNm,
    color: [f32; 3],
    radius_nm: f32,
) {
    let width = (rect.max_x - rect.min_x) as f32;
    let height = (rect.max_y - rect.min_y) as f32;
    let radius = radius_nm.min(width * 0.5).min(height * 0.5);
    if radius <= 1.0 {
        push_world_rect_nm(out, rect, color);
        return;
    }
    push_world_rect_nm(
        out,
        datum_gui_protocol::RectNm {
            min_x: (rect.min_x as f32 + radius).round() as i64,
            min_y: rect.min_y,
            max_x: (rect.max_x as f32 - radius).round() as i64,
            max_y: rect.max_y,
        },
        color,
    );
    push_world_rect_nm(
        out,
        datum_gui_protocol::RectNm {
            min_x: rect.min_x,
            min_y: (rect.min_y as f32 + radius).round() as i64,
            max_x: rect.max_x,
            max_y: (rect.max_y as f32 - radius).round() as i64,
        },
        color,
    );
    let diameter = (radius * 2.0).round() as i64;
    for (x, y) in [
        (rect.min_x, rect.min_y),
        (rect.max_x - diameter, rect.min_y),
        (rect.min_x, rect.max_y - diameter),
        (rect.max_x - diameter, rect.max_y - diameter),
    ] {
        push_world_ellipse_nm(
            out,
            datum_gui_protocol::RectNm {
                min_x: x,
                min_y: y,
                max_x: x + diameter,
                max_y: y + diameter,
            },
            color,
            48,
        );
    }
}

fn push_world_polyline_segments(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    thickness_nm: f32,
    color: [f32; 3],
) {
    for segment in path.windows(2) {
        let a = (segment[0].x as f32, segment[0].y as f32);
        let b = (segment[1].x as f32, segment[1].y as f32);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let nx = -dy / len * thickness_nm * 0.5;
        let ny = dx / len * thickness_nm * 0.5;
        push_world_quad(
            out,
            &[
                (a.0 + nx, a.1 + ny),
                (b.0 + nx, b.1 + ny),
                (b.0 - nx, b.1 - ny),
                (a.0 - nx, a.1 - ny),
            ],
            color,
        );
    }
}

fn push_world_polyline_mitered(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    thickness_nm: f32,
    color: [f32; 3],
) {
    let n = path.len();
    if n < 2 {
        return;
    }
    let h = thickness_nm * 0.5;
    let is_closed = path[0].x == path[n - 1].x && path[0].y == path[n - 1].y;
    let unit = |a: PointNm, b: PointNm| -> (f32, f32) {
        let dx = (b.x - a.x) as f32;
        let dy = (b.y - a.y) as f32;
        let l = (dx * dx + dy * dy).sqrt().max(1.0);
        (dx / l, dy / l)
    };
    let perp = |d: (f32, f32)| -> (f32, f32) { (-d.1, d.0) };
    let mut offsets: Vec<(f32, f32)> = Vec::with_capacity(n);
    for i in 0..n {
        let prev_idx = if i == 0 {
            if is_closed { Some(n - 2) } else { None }
        } else {
            Some(i - 1)
        };
        let next_idx = if i + 1 == n {
            if is_closed { Some(1) } else { None }
        } else {
            Some(i + 1)
        };
        let n_in = prev_idx.map(|p| perp(unit(path[p], path[i])));
        let n_out = next_idx.map(|q| perp(unit(path[i], path[q])));
        let o = match (n_in, n_out) {
            (Some(a), Some(b)) => {
                let dot = a.0 * b.0 + a.1 * b.1;
                let denom = (1.0 + dot).max(0.2);
                ((a.0 + b.0) * h / denom, (a.1 + b.1) * h / denom)
            }
            (Some(a), None) => (a.0 * h, a.1 * h),
            (None, Some(b)) => (b.0 * h, b.1 * h),
            _ => (0.0, 0.0),
        };
        offsets.push(o);
    }
    for i in 0..(n - 1) {
        let a = path[i];
        let b = path[i + 1];
        let (ax, ay) = (a.x as f32, a.y as f32);
        let (bx, by) = (b.x as f32, b.y as f32);
        let oa = offsets[i];
        let ob = offsets[i + 1];
        push_world_quad(
            out,
            &[
                (ax + oa.0, ay + oa.1),
                (bx + ob.0, by + ob.1),
                (bx - ob.0, by - ob.1),
                (ax - oa.0, ay - oa.1),
            ],
            color,
        );
    }
}

fn push_world_polyline_segments_capped(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    thickness_nm: f32,
    color: [f32; 3],
) {
    let ext = thickness_nm * 0.5;
    for segment in path.windows(2) {
        let a = (segment[0].x as f32, segment[0].y as f32);
        let b = (segment[1].x as f32, segment[1].y as f32);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let ux = dx / len;
        let uy = dy / len;
        let nx = -uy * thickness_nm * 0.5;
        let ny = ux * thickness_nm * 0.5;
        let a_ext = (a.0 - ux * ext, a.1 - uy * ext);
        let b_ext = (b.0 + ux * ext, b.1 + uy * ext);
        push_world_quad(
            out,
            &[
                (a_ext.0 + nx, a_ext.1 + ny),
                (b_ext.0 + nx, b_ext.1 + ny),
                (b_ext.0 - nx, b_ext.1 - ny),
                (a_ext.0 - nx, a_ext.1 - ny),
            ],
            color,
        );
    }
}

fn push_world_dashed_polyline_segments(
    out: &mut Vec<Quad>,
    path: &[PointNm],
    thickness_nm: f32,
    dash_nm: f32,
    gap_nm: f32,
    color: [f32; 3],
) {
    for segment in path.windows(2) {
        let a = (segment[0].x as f32, segment[0].y as f32);
        let b = (segment[1].x as f32, segment[1].y as f32);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let ux = dx / len;
        let uy = dy / len;
        let step = (dash_nm + gap_nm).max(1.0);
        let mut start = 0.0;
        while start < len {
            let end = (start + dash_nm).min(len);
            let start_point = PointNm {
                x: (a.0 + ux * start).round() as i64,
                y: (a.1 + uy * start).round() as i64,
            };
            let end_point = PointNm {
                x: (a.0 + ux * end).round() as i64,
                y: (a.1 + uy * end).round() as i64,
            };
            push_world_polyline_segments_capped(
                out,
                &[start_point, end_point],
                thickness_nm,
                color,
            );
            start += step;
        }
    }
}

#[allow(dead_code)]
fn push_world_points(out: &mut Vec<Quad>, points: &[PointNm], size_nm: f32, color: [f32; 3]) {
    for point in points {
        let half = size_nm * 0.5;
        push_world_rect_nm(
            out,
            datum_gui_protocol::RectNm {
                min_x: (point.x as f32 - half).round() as i64,
                min_y: (point.y as f32 - half).round() as i64,
                max_x: (point.x as f32 + half).round() as i64,
                max_y: (point.y as f32 + half).round() as i64,
            },
            color,
        );
    }
}

fn push_world_polygon_fill(out: &mut Vec<Quad>, polygon: &[PointNm], color: [f32; 3]) {
    push_world_polygon_fill_contours(out, polygon, &[], color);
}

fn push_world_polygon_fill_contours(
    out: &mut Vec<Quad>,
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
        match clean_polygon_ring_nm(outer) {
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

    let mut contours = Vec::with_capacity(1 + holes.len());
    if let Some(cleaned_outer) = clean_polygon_ring_nm(outer) {
        contours.push(cleaned_outer);
    }
    for hole in holes {
        if let Some(cleaned_hole) = clean_polygon_ring_nm(hole) {
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

fn clean_polygon_ring_nm(polygon: &[PointNm]) -> Option<Vec<PointNm>> {
    if polygon.len() < 3 {
        return None;
    }
    let mut cleaned: Vec<PointNm> = Vec::with_capacity(polygon.len());
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

fn push_projected_quad(out: &mut Vec<Quad>, quad: &[(f32, f32); 4], color: [f32; 3]) {
    out.push(Quad {
        points: *quad,
        color,
    });
}

#[allow(dead_code)]
fn push_projected_triangle(
    out: &mut Vec<Quad>,
    a: (f32, f32),
    b: (f32, f32),
    c: (f32, f32),
    color: [f32; 3],
) {
    out.push(Quad {
        points: [a, b, c, c],
        color,
    });
}

fn push_projected_polygon_fill(out: &mut Vec<Quad>, polygon: &[(f32, f32)], color: [f32; 3]) {
    push_projected_polygon_fill_contours(out, polygon, &[], color);
}

fn push_projected_polygon_fill_contours(
    out: &mut Vec<Quad>,
    outer: &[(f32, f32)],
    holes: &[Vec<(f32, f32)>],
    color: [f32; 3],
) {
    let mut contours = Vec::with_capacity(1 + holes.len());
    if let Some(cleaned_outer) = clean_polygon_ring_projected(outer) {
        contours.push(cleaned_outer);
    }
    for hole in holes {
        if let Some(cleaned_hole) = clean_polygon_ring_projected(hole) {
            contours.push(cleaned_hole);
        }
    }
    if contours.is_empty() {
        return;
    }
    push_projected_polygon_fill_scanline_contours(out, &contours, color);
}

fn clean_polygon_ring_projected(polygon: &[(f32, f32)]) -> Option<Vec<(f32, f32)>> {
    if polygon.len() < 3 {
        return None;
    }
    let mut cleaned: Vec<(f32, f32)> = Vec::with_capacity(polygon.len());
    for &point in polygon {
        if cleaned.last().is_some_and(|last| {
            (last.0 - point.0).abs() < 0.001 && (last.1 - point.1).abs() < 0.001
        }) {
            continue;
        }
        cleaned.push(point);
    }
    if cleaned.len() >= 2
        && cleaned.first().is_some_and(|first| {
            cleaned.last().is_some_and(|last| {
                (last.0 - first.0).abs() < 0.001 && (last.1 - first.1).abs() < 0.001
            })
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
    out: &mut Vec<Quad>,
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
    }

    impl ScanlineEdge {
        fn x_at(self, y: f64) -> f64 {
            let t = (y - self.ay) / (self.by - self.ay);
            self.ax + (self.bx - self.ax) * t
        }
    }

    let mut ys: Vec<f64> = contours
        .iter()
        .flat_map(|polygon| polygon.iter().map(|p| p.y as f64))
        .collect();
    ys.sort_by(|a, b| a.total_cmp(b));
    ys.dedup_by(|a, b| (*a - *b).abs() <= EPS);
    if ys.len() < 2 {
        return;
    }

    let mut edges: Vec<ScanlineEdge> = Vec::new();
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
            });
        }
    }
    edges.sort_by(|a, b| {
        a.min_y
            .total_cmp(&b.min_y)
            .then_with(|| a.max_y.total_cmp(&b.max_y))
            .then_with(|| a.ax.total_cmp(&b.ax))
            .then_with(|| a.bx.total_cmp(&b.bx))
    });

    let mut next_edge = 0;
    let mut active_edges: Vec<ScanlineEdge> = Vec::new();
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

        let mut spans: Vec<(f64, f64, f64)> = Vec::with_capacity(active_edges.len());
        for edge in &active_edges {
            if y_mid < edge.min_y || y_mid >= edge.max_y {
                continue;
            }
            spans.push((edge.x_at(y_mid), edge.x_at(y0), edge.x_at(y1)));
        }
        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
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

fn push_projected_polygon_fill_scanline_contours(
    out: &mut Vec<Quad>,
    contours: &[Vec<(f32, f32)>],
    color: [f32; 3],
) {
    const EPS: f32 = 1e-4;
    let mut ys: Vec<f32> = contours
        .iter()
        .flat_map(|polygon| polygon.iter().map(|p| p.1))
        .collect();
    ys.sort_by(|a, b| a.total_cmp(b));
    ys.dedup_by(|a, b| (*a - *b).abs() <= EPS);
    if ys.len() < 2 {
        return;
    }

    for band in ys.windows(2) {
        let y0 = band[0];
        let y1 = band[1];
        if y1 - y0 <= EPS {
            continue;
        }
        let y_mid = (y0 + y1) * 0.5;
        let mut spans: Vec<(f32, f32, f32)> = Vec::new();
        for polygon in contours {
            for i in 0..polygon.len() {
                let a = polygon[i];
                let b = polygon[(i + 1) % polygon.len()];
                if (a.1 - b.1).abs() <= EPS {
                    continue;
                }
                let min_y = a.1.min(b.1);
                let max_y = a.1.max(b.1);
                if y_mid < min_y || y_mid >= max_y {
                    continue;
                }
                let x_at = |y: f32| {
                    let t = (y - a.1) / (b.1 - a.1);
                    a.0 + (b.0 - a.0) * t
                };
                spans.push((x_at(y_mid), x_at(y0), x_at(y1)));
            }
        }
        spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in spans.chunks_exact(2) {
            let left = pair[0];
            let right = pair[1];
            if right.0 - left.0 <= EPS {
                continue;
            }
            push_projected_quad(
                out,
                &[(left.1, y0), (right.1, y0), (right.2, y1), (left.2, y1)],
                color,
            );
        }
    }
}

#[allow(dead_code)]
fn push_projected_ellipse(out: &mut Vec<Quad>, rect: RectPx, color: [f32; 3], segments: usize) {
    if rect.width <= 0.5 || rect.height <= 0.5 || segments < 3 {
        return;
    }
    let cx = rect.x + rect.width * 0.5;
    let cy = rect.y + rect.height * 0.5;
    let rx = rect.width * 0.5;
    let ry = rect.height * 0.5;
    let step = std::f32::consts::TAU / segments as f32;
    let mut prev = (cx + rx, cy);
    for i in 1..=segments {
        let angle = step * i as f32;
        let next = (cx + rx * angle.cos(), cy + ry * angle.sin());
        push_projected_triangle(out, (cx, cy), prev, next, color);
        prev = next;
    }
}

fn bounds_from_projected_points(points: &[(f32, f32); 4]) -> RectPx {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for (x, y) in points {
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    RectPx {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(1.0),
        height: (max_y - min_y).max(1.0),
    }
}

fn inset_rect(rect: RectPx, left: f32, top: f32, right: f32, bottom: f32) -> RectPx {
    RectPx {
        x: rect.x + left,
        y: rect.y + top,
        width: (rect.width - left - right).max(1.0),
        height: (rect.height - top - bottom).max(1.0),
    }
}

fn push_rect_border(out: &mut Vec<Quad>, rect: RectPx, color: [f32; 3], thickness: f32) {
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: thickness,
        },
        color,
    ));
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x,
            y: rect.y + rect.height - thickness,
            width: rect.width,
            height: thickness,
        },
        color,
    ));
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x,
            y: rect.y,
            width: thickness,
            height: rect.height,
        },
        color,
    ));
    out.push(Quad::from_rect(
        RectPx {
            x: rect.x + rect.width - thickness,
            y: rect.y,
            width: thickness,
            height: rect.height,
        },
        color,
    ));
}

fn push_section_divider(out: &mut Vec<Quad>, x: f32, y: f32, width: f32, color: [f32; 3]) {
    out.push(Quad::from_rect(
        RectPx {
            x,
            y,
            width,
            height: 1.0,
        },
        color,
    ));
}

fn push_boolean_row(
    x: f32,
    y: f32,
    width: f32,
    label: &str,
    enabled: bool,
    text_runs: &mut Vec<TextRun>,
) {
    draw_text(label, x, y, 13.0, TEXT_SECONDARY, TextFace::Ui, text_runs);
    // Right-align the ON/OFF value to the row's right edge so it never collides
    // with a long label like "DIM UNRELATED".
    let value = if enabled { "ON" } else { "OFF" };
    let value_w = estimated_text_run_width_px(value, 13.0, TextFace::Ui) - 16.0;
    draw_text(
        value,
        x + width - value_w - design_tokens::spacing::SP_03,
        y,
        13.0,
        if enabled { TEXT_PRIMARY } else { TEXT_MUTED },
        TextFace::Ui,
        text_runs,
    );
}

fn push_key_value(
    x: f32,
    y: f32,
    key: &str,
    value: &str,
    text_runs: &mut Vec<TextRun>,
    value_face: TextFace,
) {
    // Two legible tiers: key in TEXT_SECONDARY, value in TEXT_PRIMARY one step
    // larger, on a ~100px key column (Design Book kv grid).
    draw_text(key, x, y, 13.0, TEXT_SECONDARY, TextFace::Ui, text_runs);
    draw_text(
        value,
        x + 100.0,
        y,
        13.5,
        TEXT_PRIMARY,
        value_face,
        text_runs,
    );
}

fn push_board_text_property_row(
    x: f32,
    y: f32,
    key: &str,
    value: &str,
    text_runs: &mut Vec<TextRun>,
) {
    draw_text(
        &format!("{key:<8} {value}"),
        x,
        y,
        12.5,
        TEXT_PANEL_VALUE,
        TextFace::Mono,
        text_runs,
    );
}

fn workspace_tool_label(tool: WorkspaceTool) -> &'static str {
    match tool {
        WorkspaceTool::Select => "SELECT",
        WorkspaceTool::DrawBoardTrack => "DRAW TRACK",
        WorkspaceTool::PlaceBoardVia => "PLACE VIA",
        WorkspaceTool::PlaceBoardText => "PLACE TEXT",
        WorkspaceTool::Move => "MOVE",
        WorkspaceTool::Delete => "DELETE",
    }
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    if max_chars <= 3 {
        return text.chars().take(max_chars).collect();
    }
    let keep = max_chars - 3;
    let front = keep / 2;
    let back = keep - front;
    let head: String = text.chars().take(front).collect();
    let tail: String = text
        .chars()
        .rev()
        .take(back)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{head}...{tail}")
}

fn text_buffer_key(run: &TextRun, width: u32, height: u32) -> TextBufferKey {
    let (width_px, height_px) = text_buffer_extent(run, width, height);
    TextBufferKey {
        text: run.text.clone(),
        size_bits: run.size.to_bits(),
        face: run.face,
        width_px,
        height_px,
    }
}

fn text_buffer_extent(run: &TextRun, surface_width: u32, surface_height: u32) -> (u32, u32) {
    let max_width = surface_width.max(1);
    let max_height = surface_height.max(1);
    let width = run.clip_bounds.map_or_else(
        || estimated_text_run_width_px(&run.text, run.size, run.face),
        |bounds| bounds.width.ceil().max(1.0),
    );
    let height = run.clip_bounds.map_or_else(
        || run.size * 1.55 + 6.0,
        |bounds| bounds.height.ceil().max(1.0),
    );
    (
        (width.ceil() as u32).clamp(1, max_width),
        (height.ceil() as u32).clamp(1, max_height),
    )
}

fn estimated_text_run_width_px(text: &str, size: f32, face: TextFace) -> f32 {
    let advance_factor = match face {
        TextFace::Ui => 0.62,
        TextFace::UiMedium => 0.64,
        TextFace::UiStrong => 0.66,
        TextFace::Mono => 0.72,
    };
    let glyphs = text.chars().count().max(1) as f32;
    glyphs * size * advance_factor + 16.0
}

/// Shared, lazily-initialized measuring `FontSystem` loaded with the SAME vendored
/// IBM Plex faces the renderer uses (`load_datum_fonts`), so a measured width here
/// matches what gpu.rs actually shapes. Kept separate from the renderer's own
/// `FontSystem` because measurement happens during scene preparation (no GPU) and
/// must stay deterministic across threads (goldens depend on it).
static MEASURE_FS: std::sync::OnceLock<std::sync::Mutex<FontSystem>> = std::sync::OnceLock::new();

fn measure_font_system() -> &'static std::sync::Mutex<FontSystem> {
    MEASURE_FS.get_or_init(|| {
        let mut font_system = FontSystem::new();
        load_datum_fonts(&mut font_system);
        std::sync::Mutex::new(font_system)
    })
}

/// Real shaped width of a single text run, in px, using cosmic-text/glyphon with
/// the exact per-`TextFace` `Attrs` and `Metrics` gpu.rs renders with (see
/// `ensure_text_buffer`: `Metrics::new(size, size * 1.22)`, `text_attrs(face)`).
/// Unlike `estimated_text_run_width_px` (a fixed-advance monospace-style estimate
/// with baked padding), this reflects the PROPORTIONAL IBM Plex Sans Condensed UI
/// face, so per-label error is zero and downstream layout gaps stay uniform.
/// Deterministic: same inputs -> same width, so it is golden-stable.
fn measured_text_run_width_px(text: &str, size: f32, face: TextFace) -> f32 {
    let mutex = measure_font_system();
    let mut font_system = mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(size, size * 1.22));
    let attrs = text_attrs(face);
    buffer.set_text(&mut font_system, text, &attrs, Shaping::Basic, None);
    buffer.shape_until_scroll(&mut font_system, false);
    buffer
        .layout_runs()
        .map(|run| run.line_w)
        .fold(0.0_f32, f32::max)
}

fn scale_text_run_sizes(text_runs: &mut [TextRun], scale: f32) {
    for run in text_runs {
        run.size *= scale;
    }
}

/// Load the vendored IBM Plex faces into the glyphon font database so chrome and
/// on-canvas UI text render in the Design Book typeface rather than a system
/// fallback (`docs/gui/DATUM_RENDERING_BOOK.md` §5). Embedded at compile time
/// from the engine's vendored assets so the GUI never depends on the CWD.
fn load_datum_fonts(font_system: &mut FontSystem) {
    let db = font_system.db_mut();
    db.load_font_data(
        include_bytes!(
            "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-Regular.ttf"
        )
        .to_vec(),
    );
    db.load_font_data(
        include_bytes!(
            "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-Medium.ttf"
        )
        .to_vec(),
    );
    db.load_font_data(
        include_bytes!(
            "../../../engine/assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-SemiBold.ttf"
        )
        .to_vec(),
    );
    db.load_font_data(
        include_bytes!("../../../engine/assets/fonts/ibm_plex_mono/IBMPlexMono-Regular.ttf").to_vec(),
    );
    db.load_font_data(
        include_bytes!("../../../engine/assets/fonts/ibm_plex_mono/IBMPlexMono-Medium.ttf").to_vec(),
    );
}

fn text_attrs(face: TextFace) -> Attrs<'static> {
    match face {
        TextFace::Ui => Attrs::new().family(Family::Name("IBM Plex Sans Condensed")),
        TextFace::UiMedium => Attrs::new()
            .family(Family::Name("IBM Plex Sans Condensed"))
            .weight(Weight::MEDIUM),
        TextFace::UiStrong => Attrs::new()
            .family(Family::Name("IBM Plex Sans Condensed"))
            .weight(Weight::SEMIBOLD),
        TextFace::Mono => Attrs::new().family(Family::Name("IBM Plex Mono")),
    }
}

fn text_color(color: [f32; 3]) -> Color {
    Color::rgb(
        (color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn build_text_areas<'a>(
    cache: &'a [CachedTextBuffer],
    indices: &[usize],
    runs: &[TextRun],
) -> Vec<TextArea<'a>> {
    indices
        .iter()
        .zip(runs.iter())
        .map(|(index, run)| TextArea {
            buffer: &cache[*index].buffer,
            left: run.x,
            top: run.y,
            scale: 1.0,
            bounds: run
                .clip_bounds
                .map_or_else(TextBounds::default, |rect| TextBounds {
                    left: rect.x.floor() as i32,
                    top: rect.y.floor() as i32,
                    right: (rect.x + rect.width).ceil() as i32,
                    bottom: (rect.y + rect.height).ceil() as i32,
                }),
            default_color: text_color(run.color),
            custom_glyphs: &[],
        })
        .collect()
}

fn text_prepare_signature(
    indices: &[usize],
    runs: &[TextRun],
    width: u32,
    height: u32,
) -> TextPrepareSignature {
    TextPrepareSignature {
        width,
        height,
        runs: indices
            .iter()
            .zip(runs.iter())
            .map(|(index, run)| TextPrepareRunKey {
                buffer_index: *index,
                x_bits: run.x.to_bits(),
                y_bits: run.y.to_bits(),
                color_bits: run.color.map(f32::to_bits),
                clip_bounds: run.clip_bounds.map(|rect| RectBits {
                    x_bits: rect.x.to_bits(),
                    y_bits: rect.y.to_bits(),
                    width_bits: rect.width.to_bits(),
                    height_bits: rect.height.to_bits(),
                }),
            })
            .collect(),
    }
}

