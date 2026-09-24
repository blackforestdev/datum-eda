//! Differential proof against the preceding stable-sort scanline implementation.
use super::*;

#[test]
fn polygon_scratch_preserves_holes_crossings_and_tie_order_and_refuses_cleaning() {
    let cases: &[&[(i64, i64)]] = &[
        &[(0, 0), (80, 0), (80, 70), (40, 25), (0, 70)],
        &[(0, 0), (80, 70), (80, 0), (0, 70), (40, 35)],
        &[(0, 0), (0, 0), (80, 0), (80, 70), (40, 25), (0, 70), (0, 0)],
    ];
    for points in cases {
        let polygon: Vec<_> = points.iter().map(|&(x, y)| PointNm { x, y }).collect();
        for holes in [
            Vec::new(),
            vec![vec![
                PointNm { x: 10, y: 10 },
                PointNm { x: 25, y: 10 },
                PointNm { x: 25, y: 20 },
                PointNm { x: 10, y: 20 },
            ]],
        ] {
            let mut expected = Vec::new();
            let contours: Vec<_> = std::iter::once(&polygon)
                .chain(&holes)
                .filter_map(|p| clean_polygon_ring_nm(&mut Vec::<Quad>::new(), p))
                .collect();
            reference_scanline(&mut expected, &contours, [0.5; 3]);
            let mut actual = Vec::new();
            push_world_polygon_fill_contours(&mut actual, &polygon, &holes, [0.5; 3]);
            assert_eq!(actual, expected);
            let mut refused =
                crate::geometry_output::Admitted::new(|_| anyhow::bail!("scratch refused"));
            push_world_polygon_fill_contours(&mut refused, &polygon, &holes, [0.5; 3]);
            assert!(refused.is_empty());
            assert!(refused.finish().is_err());
        }
    }
}

fn reference_scanline(out: &mut impl Output<Quad>, contours: &[Vec<PointNm>], color: [f32; 3]) {
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
        for pair in spans.as_chunks::<2>().0.iter() {
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
