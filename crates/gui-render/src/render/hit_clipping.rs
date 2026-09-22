//! Shared visible-viewport clipping for editor hits and native/pane content.
//! Geometry, nested text clips and pointer targets agree on positive visible
//! area, preserving pinned chrome and the original triangle contours.

use super::{HitRegion, Quad, RectPx, TextRun};

pub(super) fn clip_new_hit_regions(
    hit_regions: &mut Vec<HitRegion>,
    first_new_region: usize,
    viewport: RectPx,
) {
    // Visit only the appended suffix; pinned layers can contain many entries.
    let start = first_new_region.min(hit_regions.len());
    hit_regions
        .extract_if(start.., |region| {
            if let Some(rect) = region.rect.intersect(viewport) {
                region.rect = rect;
                false
            } else {
                true
            }
        })
        .for_each(drop);
}

/// Clip one appended content layer without touching pinned chrome before it.
/// Triangle intersections preserve diagonals; all three outputs share bounds.
#[allow(clippy::too_many_arguments)]
pub(crate) fn clip_content(
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
    hits: &mut Vec<HitRegion>,
    quad_start: usize,
    text_start: usize,
    hit_start: usize,
    viewport: RectPx,
) {
    // Keep the already-visible prefix in place. A fully visible layer needs
    // no geometry clipping; split_off(len) then allocates no scratch buffer.
    // Starting at the first crossing preserves painter order for the suffix.
    let quad_start = quads[quad_start..]
        .iter()
        .position(|quad| !quad.points.iter().all(|&(x, y)| viewport.contains(x, y)))
        .map_or(quads.len(), |offset| quad_start + offset);
    let original = quads.split_off(quad_start);
    for quad in original {
        if quad.points.iter().all(|&(x, y)| viewport.contains(x, y)) {
            quads.push(quad);
            continue;
        }
        let [a, b, c, d] = quad.points;
        if a.1 == b.1 && b.0 == c.0 && c.1 == d.1 && d.0 == a.0 {
            // Preserve one-quad encoding for the common axis-aligned control.
            let rect = RectPx {
                x: a.0.min(c.0),
                y: a.1.min(c.1),
                width: (c.0 - a.0).abs(),
                height: (c.1 - a.1).abs(),
            };
            if let Some(visible) = rect.intersect(viewport) {
                quads.push(Quad::from_rect(visible, quad.color));
            }
            continue;
        }
        for indices in [[0, 1, 2], [0, 2, 3]] {
            // A triangle clipped against a rectangle has at most seven vertices.
            let mut polygon = [(0.0, 0.0); 8];
            for (out, index) in indices.into_iter().enumerate() {
                polygon[out] = quad.points[index];
            }
            let mut len = 3;
            for (axis, edge, lower) in [
                (0, viewport.x, true),
                (0, viewport.x + viewport.width, false),
                (1, viewport.y, true),
                (1, viewport.y + viewport.height, false),
            ] {
                if len == 0 {
                    break;
                }
                let mut output = [(0.0, 0.0); 8];
                let mut count = 0;
                let value = |p: (f32, f32)| if axis == 0 { p.0 } else { p.1 };
                let inside = |p| {
                    if lower {
                        value(p) >= edge
                    } else {
                        value(p) <= edge
                    }
                };
                let mut previous = polygon[len - 1];
                for &current in &polygon[..len] {
                    if inside(previous) != inside(current) {
                        let t = (edge - value(previous)) / (value(current) - value(previous));
                        let point = if axis == 0 {
                            (edge, previous.1 + t * (current.1 - previous.1))
                        } else {
                            (previous.0 + t * (current.0 - previous.0), edge)
                        };
                        output[count] = point;
                        count += 1;
                    }
                    if inside(current) {
                        output[count] = current;
                        count += 1;
                    }
                    previous = current;
                }
                polygon = output;
                len = count;
            }
            for i in 1..len.saturating_sub(1) {
                quads.push(Quad {
                    points: [polygon[0], polygon[i], polygon[i + 1], polygon[i + 1]],
                    color: quad.color,
                });
            }
        }
    }
    let start = text_start.min(text.len());
    text.extract_if(start.., |run| {
        if let Some(bounds) = run.clip_bounds.unwrap_or(viewport).intersect(viewport) {
            run.clip_bounds = Some(bounds);
            false
        } else {
            true
        }
    })
    .for_each(drop);
    clip_new_hit_regions(hits, hit_start, viewport);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_edge_clip_preserves_triangle_area_and_pinned_content() {
        let viewport = RectPx {
            x: 0.0,
            y: 0.0,
            width: 4.0,
            height: 4.0,
        };
        let pinned = Quad::from_rect(
            RectPx {
                x: -10.0,
                ..viewport
            },
            [1.0; 3],
        );
        let mut quads = vec![
            pinned,
            Quad {
                points: [(-2.0, -2.0), (8.0, -2.0), (-2.0, 8.0), (-2.0, 8.0)],
                color: [0.5; 3],
            },
        ];
        clip_content(
            &mut quads,
            &mut Vec::new(),
            &mut Vec::new(),
            1,
            0,
            0,
            viewport,
        );
        assert_eq!(quads[0], pinned);
        let mut area = 0.0;
        for quad in &quads[1..] {
            assert!(quad.points.iter().all(|&(x, y)| viewport.contains(x, y)));
            for [a, b, c] in [[0, 1, 2], [0, 2, 3]] {
                let (a, b, c) = (quad.points[a], quad.points[b], quad.points[c]);
                area += ((b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)).abs() * 0.5;
            }
        }
        assert!(
            // Correct intersection removes only the upper-right triangle (area2).
            // Clamping original vertices instead creates a triangle of area8.
            (area - 14.0).abs() < 0.0001,
            "clipped diagonal must preserve intersection area: {area}"
        );
    }

    #[test]
    fn descendant_text_and_hits_intersect_both_axes_without_touching_chrome() {
        let viewport = RectPx {
            x: 10.0,
            y: 10.0,
            width: 20.0,
            height: 20.0,
        };
        let child = RectPx {
            x: 20.0,
            y: 0.0,
            width: 20.0,
            height: 20.0,
        };
        let hidden = RectPx { x: 40.0, ..child };
        let mut text = Vec::new();
        crate::draw_text(
            "label",
            0.0,
            0.0,
            12.0,
            [1.0; 3],
            crate::TextFace::Ui,
            &mut text,
        );
        text[0].clip_bounds = Some(child);
        text.push(text[0].clone());
        text.push(text[0].clone());
        text[2].clip_bounds = Some(hidden);
        let mut hits = vec![
            HitRegion {
                target: crate::HitTarget::NewProjectName,
                rect: child
            };
            3
        ];
        hits[2].rect = hidden;
        clip_content(&mut Vec::new(), &mut text, &mut hits, 0, 1, 1, viewport);
        assert_eq!(text.len(), 2);
        assert_eq!(hits.len(), 2);
        assert_eq!(text[0].clip_bounds, Some(child));
        assert_eq!(hits[0].rect, child);
        assert_eq!(text[1].clip_bounds, child.intersect(viewport));
        assert_eq!(hits[1].rect, child.intersect(viewport).unwrap());
        assert!(!hits[1].rect.contains(35.0, 15.0));
    }

    #[test]
    fn suffix_clipping_preserves_order_after_consecutive_removals() {
        let viewport = RectPx {
            x: 0.0,
            y: 0.0,
            width: 20.0,
            height: 20.0,
        };
        let hidden = RectPx {
            x: 30.0,
            ..viewport
        };
        let mut text = Vec::new();
        let mut hits = Vec::new();
        for (label, bounds) in [
            ("pinned", hidden),
            ("hidden1", hidden),
            ("hidden2", hidden),
            ("visible", viewport),
        ] {
            crate::draw_text_clipped(
                label,
                bounds.x,
                bounds.y,
                12.0,
                [1.0; 3],
                crate::TextFace::Ui,
                bounds,
                &mut text,
            );
            hits.push(HitRegion {
                target: crate::HitTarget::GlobalPreferencesControl(label.into()),
                rect: bounds,
            });
        }
        clip_content(&mut Vec::new(), &mut text, &mut hits, 0, 1, 1, viewport);
        assert_eq!(
            text.iter().map(|run| run.text.as_str()).collect::<Vec<_>>(),
            ["pinned", "visible"]
        );
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].rect, hidden);
        assert_eq!(
            hits[1].target,
            crate::HitTarget::GlobalPreferencesControl("visible".into())
        );
        let saved = (text.clone(), hits.clone());
        clip_content(
            &mut Vec::new(),
            &mut text,
            &mut hits,
            0,
            usize::MAX,
            usize::MAX,
            viewport,
        );
        assert_eq!((text, hits), saved);
    }
}
