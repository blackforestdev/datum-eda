use super::*;

pub(super) fn push_projected_polygon_fill_scanline_contours(
    out: &mut impl Output<Quad>,
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
        for pair in spans.as_chunks::<2>().0.iter() {
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
