//! Shared world-space quad and triangle emission.
use super::*;

pub(super) fn push_world_quad(
    out: &mut impl Output<Quad>,
    quad: &[(f32, f32); 4],
    color: [f32; 3],
) {
    out.push(Quad {
        points: *quad,
        color,
    });
}

pub(super) fn push_world_triangle(
    out: &mut impl Output<Quad>,
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

pub(super) fn push_convex_polygon_fill(
    out: &mut impl Output<Quad>,
    polygon: &[(f32, f32)],
    color: [f32; 3],
) {
    if polygon.len() < 3 {
        return;
    }
    let origin = polygon[0];
    for edge in polygon[1..].windows(2) {
        push_world_triangle(out, origin, edge[0], edge[1], color);
    }
}

pub(super) fn push_world_rect_nm(
    out: &mut impl Output<Quad>,
    rect: datum_gui_protocol::RectNm,
    color: [f32; 3],
) {
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

/// Close only when needed, with the retained output's construction admission.
pub(super) fn closed_world_path<'a>(
    out: &mut impl Output<Quad>,
    points: &'a [PointNm],
) -> Option<std::borrow::Cow<'a, [PointNm]>> {
    if let (Some(first), Some(last)) = (points.first(), points.last())
        && first != last
    {
        let mut closed = out.scratch(points.len().saturating_add(1))?;
        closed.extend_from_slice(points);
        closed.push(*first);
        Some(std::borrow::Cow::Owned(closed))
    } else {
        Some(std::borrow::Cow::Borrowed(points))
    }
}

pub(super) fn admit_outline_points(out: &mut impl Output<Quad>, count: usize) -> bool {
    use crate::cpu_alloc::heap::capacity_bytes;
    out.admit_temporary(
        capacity_bytes::<(f32, f32)>(count).saturating_add(capacity_bytes::<PointNm>(count)),
    )
}

pub(super) fn world_pad_outline(
    pad: &datum_gui_protocol::PadPrimitive,
    bounds: datum_gui_protocol::RectNm,
    inset_nm: f32,
    reference_projection: &Projection,
) -> Vec<PointNm> {
    let (width_nm, height_nm) = match pad.center {
        datum_gui_protocol::PadCenter::Exact { .. } => pad_dimensions_nm(pad),
        datum_gui_protocol::PadCenter::Integer(_) => (
            (bounds.max_x - bounds.min_x).max(1) as f32,
            (bounds.max_y - bounds.min_y).max(1) as f32,
        ),
    };
    let center = paint_pad_center(pad.center);
    let width_nm = (width_nm - inset_nm * 2.0).max(1.0);
    let height_nm = (height_nm - inset_nm * 2.0).max(1.0);
    let points = match pad.shape_kind.as_str() {
        "circle" | "oval" => ellipse_points(center, width_nm, height_nm, pad.rotation_degrees, 64),
        _ => {
            let radius_nm =
                pad_corner_radius_nm(pad, width_nm, height_nm, reference_projection, inset_nm);
            rounded_rect_points(center, width_nm, height_nm, pad.rotation_degrees, radius_nm)
        }
    };
    points
        .into_iter()
        .map(|(x, y)| PointNm {
            x: x.round() as i64,
            y: y.round() as i64,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact30_pad_outline_uses_local_aperture_and_preserves_integer_bounds() {
        use datum_gui_protocol::{PadCenter, Quadratic3 as Q, RectNm};
        use eda_engine::board::nominal_geometry::Rational as R;
        let state = datum_gui_protocol::load_fixture_workspace_state();
        let mut pad = state.scene.pads[0].clone();
        pad.shape_kind = "rect".into();
        pad.rotation_degrees = 30.0;
        pad.center = PadCenter::Exact {
            exact: [Q::rational(R::integer(0)); 2],
            local_width_nm: 400_000,
            local_height_nm: 600_000,
            back: false,
        };
        pad.bounds = RectNm {
            min_x: -720_000,
            min_y: -720_000,
            max_x: 720_000,
            max_y: 720_000,
        };
        let viewport = RectPx {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let projection = Projection::new(
            viewport,
            &state.scene.bounds,
            CameraState::fit_to_bounds(&state.scene.bounds),
        );
        let extent = |points: Vec<PointNm>| RectNm {
            min_x: points.iter().map(|p| p.x).min().unwrap(),
            min_y: points.iter().map(|p| p.y).min().unwrap(),
            max_x: points.iter().map(|p| p.x).max().unwrap(),
            max_y: points.iter().map(|p| p.y).max().unwrap(),
        };
        // These are final painting coordinates, not query-geometry tolerances.
        for (inset, x, y) in [(0.0, 323_205, 359_808), (50_000.0, 254_904, 291_506)] {
            assert_eq!(
                extent(world_pad_outline(&pad, pad.bounds, inset, &projection)),
                RectNm {
                    min_x: -x,
                    min_y: -y,
                    max_x: x,
                    max_y: y
                }
            );
        }
        let wider = RectNm {
            min_x: -2_000_000,
            min_y: -3_000_000,
            max_x: 2_000_000,
            max_y: 3_000_000,
        };
        assert_eq!(
            world_pad_outline(&pad, pad.bounds, 0.0, &projection),
            world_pad_outline(&pad, wider, 0.0, &projection),
            "LOD outline must not inflate with conservative search bounds"
        );
        pad.center = PointNm { x: 0, y: 0 }.into();
        pad.rotation_degrees = 0.0;
        assert_eq!(
            extent(world_pad_outline(&pad, wider, 0.0, &projection)),
            wider,
            "integer outline retains caller-supplied bounds"
        );
    }
}
