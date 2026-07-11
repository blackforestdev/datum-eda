#[allow(dead_code)]
fn push_pad_primitive(
    out: &mut Vec<Quad>,
    pad: &datum_gui_protocol::PadPrimitive,
    projection: &Projection,
    _layer_id: &str,
    outer_color: [f32; 3],
    drill_nm: Option<i64>,
    dimmed: bool,
) -> RectPx {
    let outer_color = dim_authored_color(outer_color, dimmed);
    let px = project_rect(pad.bounds, projection);
    let is_ellipse = matches!(pad.shape_kind.as_str(), "circle" | "oval");
    let copper_outline = projected_pad_outline(pad, projection, 0.0);
    push_convex_polygon_fill(out, &copper_outline, outer_color);
    if is_ellipse && drill_nm.is_none() {
        let inner = inset_rect(
            px,
            px.width * 0.22,
            px.height * 0.22,
            px.width * 0.22,
            px.height * 0.22,
        );
        if inner.width > 1.0 && inner.height > 1.0 {
            push_projected_ellipse(
                out,
                inner,
                dim_authored_color([0.79, 0.49, 0.26], dimmed),
                24,
            );
        }
    }
    if let Some(drill_nm) = drill_nm.filter(|value| *value > 0) {
        let drill_px =
            world_length_to_px(drill_nm, projection).clamp(4.0, px.width.min(px.height) - 2.0);
        let hole = RectPx {
            x: px.x + (px.width - drill_px) * 0.5,
            y: px.y + (px.height - drill_px) * 0.5,
            width: drill_px,
            height: drill_px,
        };
        push_projected_ellipse(
            out,
            hole,
            dim_structural_color([0.10, 0.11, 0.12], dimmed),
            22,
        );
        let hole_border = inset_rect(hole, 0.8, 0.8, 0.8, 0.8);
        if hole_border.width > 1.0 && hole_border.height > 1.0 {
            push_projected_ellipse(
                out,
                hole_border,
                dim_structural_color([0.62, 0.66, 0.70], dimmed),
                22,
            );
            let hole_inner = inset_rect(hole_border, 1.0, 1.0, 1.0, 1.0);
            if hole_inner.width > 1.0 && hole_inner.height > 1.0 {
                push_projected_ellipse(
                    out,
                    hole_inner,
                    dim_structural_color([0.10, 0.11, 0.12], dimmed),
                    22,
                );
            }
        }
    }
    px
}

fn push_pad_primitive_world(
    out: &mut Vec<Quad>,
    pad: &datum_gui_protocol::PadPrimitive,
    layer_id: &str,
    outer_color: [f32; 3],
    drill_nm: Option<i64>,
    dimmed: bool,
    reference_projection: &Projection,
) {
    let outer_color = dim_authored_color(outer_color, dimmed);
    let _ = layer_id;
    let copper_outline = world_pad_outline(pad, 0.0, reference_projection);
    push_world_polygon_fill(out, &copper_outline, outer_color);
    if let Some(drill_nm) = drill_nm.filter(|value| *value > 0) {
        let half = drill_nm as f32 * 0.5;
        let center_x = (pad.bounds.min_x + pad.bounds.max_x) as f32 * 0.5;
        let center_y = (pad.bounds.min_y + pad.bounds.max_y) as f32 * 0.5;
        let hole = datum_gui_protocol::RectNm {
            min_x: (center_x - half).round() as i64,
            min_y: (center_y - half).round() as i64,
            max_x: (center_x + half).round() as i64,
            max_y: (center_y + half).round() as i64,
        };
        push_world_ellipse_nm(
            out,
            hole,
            dim_structural_color([0.10, 0.11, 0.12], dimmed),
            128,
        );
    }
}

fn pad_dimensions_nm(pad: &datum_gui_protocol::PadPrimitive) -> (f32, f32) {
    (
        (pad.bounds.max_x - pad.bounds.min_x).max(1) as f32,
        (pad.bounds.max_y - pad.bounds.min_y).max(1) as f32,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PadProcessLayerKind {
    Mask,
    Paste,
}

fn derived_process_pad(
    pad: &datum_gui_protocol::PadPrimitive,
    process_layer_id: &str,
    kind: PadProcessLayerKind,
    _setup: &datum_gui_protocol::ScenePadExpansionSetup,
) -> datum_gui_protocol::PadPrimitive {
    let (width_nm, height_nm) = pad_dimensions_nm(pad);
    let (expanded_width_nm, expanded_height_nm) = match kind {
        PadProcessLayerKind::Mask => {
            let clearance = pad.solder_mask_margin_nm as f32;
            (
                (width_nm + clearance * 2.0).max(1.0),
                (height_nm + clearance * 2.0).max(1.0),
            )
        }
        PadProcessLayerKind::Paste => {
            let clearance = pad.solder_paste_margin_nm as f32;
            let ratio = pad.solder_paste_margin_ratio_ppm as f32 / 1_000_000.0;
            (
                (width_nm + width_nm * ratio + clearance * 2.0).max(1.0),
                (height_nm + height_nm * ratio + clearance * 2.0).max(1.0),
            )
        }
    };
    let half_w = expanded_width_nm * 0.5;
    let half_h = expanded_height_nm * 0.5;
    let center_x = pad.center.x as f32;
    let center_y = pad.center.y as f32;
    let mut derived = pad.clone();
    derived.layer_id = process_layer_id.to_string();
    derived.bounds = datum_gui_protocol::RectNm {
        min_x: (center_x - half_w).round() as i64,
        min_y: (center_y - half_h).round() as i64,
        max_x: (center_x + half_w).round() as i64,
        max_y: (center_y + half_h).round() as i64,
    };
    // Process apertures are not annular copper objects; render as the opening/aperture shape.
    derived.drill_nm = None;
    derived
}

fn pad_corner_radius_nm(
    pad: &datum_gui_protocol::PadPrimitive,
    width_nm: f32,
    height_nm: f32,
    reference_projection: &Projection,
    inset_nm: f32,
) -> f32 {
    let width_nm = (width_nm - inset_nm * 2.0).max(1.0);
    let height_nm = (height_nm - inset_nm * 2.0).max(1.0);
    match pad.shape_kind.as_str() {
        "circle" => width_nm.min(height_nm) * 0.5,
        "oval" => width_nm.min(height_nm) * 0.5,
        "roundrect" | "round_rect" => {
            let ratio = (pad.roundrect_rratio_ppm as f32 / 1_000_000.0).clamp(0.0, 0.5);
            let radius = width_nm.min(height_nm) * ratio;
            radius.max(world_stroke_nm(1.0, reference_projection))
        }
        _ => 0.0,
    }
}

fn rotate_point_about_center(
    center: (f32, f32),
    local: (f32, f32),
    rotation_degrees: f32,
) -> (f32, f32) {
    let rad = (-rotation_degrees).to_radians();
    let cos = rad.cos();
    let sin = rad.sin();
    (
        center.0 + local.0 * cos - local.1 * sin,
        center.1 + local.0 * sin + local.1 * cos,
    )
}

fn rounded_rect_points(
    center: (f32, f32),
    width: f32,
    height: f32,
    rotation_degrees: f32,
    radius: f32,
) -> Vec<(f32, f32)> {
    let half_w = width * 0.5;
    let half_h = height * 0.5;
    let radius = radius.min(half_w).min(half_h).max(0.0);
    if radius <= 0.5 {
        return [
            (-half_w, -half_h),
            (half_w, -half_h),
            (half_w, half_h),
            (-half_w, half_h),
        ]
        .into_iter()
        .map(|local| rotate_point_about_center(center, local, rotation_degrees))
        .collect();
    }

    let segments_per_corner = 8usize;
    let arc_step = std::f32::consts::FRAC_PI_2 / segments_per_corner as f32;
    let corner_centers = [
        (
            half_w - radius,
            -half_h + radius,
            -std::f32::consts::FRAC_PI_2,
        ),
        (half_w - radius, half_h - radius, 0.0),
        (
            -(half_w - radius),
            half_h - radius,
            std::f32::consts::FRAC_PI_2,
        ),
        (-(half_w - radius), -(half_h - radius), std::f32::consts::PI),
    ];
    let mut points = Vec::with_capacity(corner_centers.len() * (segments_per_corner + 1));
    for (cx, cy, start) in corner_centers {
        for step in 0..=segments_per_corner {
            let angle = start + arc_step * step as f32;
            let local = (cx + radius * angle.cos(), cy + radius * angle.sin());
            points.push(rotate_point_about_center(center, local, rotation_degrees));
        }
    }
    points
}

fn ellipse_points(
    center: (f32, f32),
    width: f32,
    height: f32,
    rotation_degrees: f32,
    segments: usize,
) -> Vec<(f32, f32)> {
    let rx = width * 0.5;
    let ry = height * 0.5;
    let segments = segments.max(24);
    (0..segments)
        .map(|i| {
            let theta = std::f32::consts::TAU * (i as f32) / (segments as f32);
            let local = (rx * theta.cos(), ry * theta.sin());
            rotate_point_about_center(center, local, rotation_degrees)
        })
        .collect()
}

fn world_pad_outline(
    pad: &datum_gui_protocol::PadPrimitive,
    inset_nm: f32,
    reference_projection: &Projection,
) -> Vec<PointNm> {
    let (width_nm, height_nm) = pad_dimensions_nm(pad);
    let center = (pad.center.x as f32, pad.center.y as f32);
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

fn projected_pad_outline(
    pad: &datum_gui_protocol::PadPrimitive,
    projection: &Projection,
    inset_px: f32,
) -> Vec<(f32, f32)> {
    let (width_nm, height_nm) = pad_dimensions_nm(pad);
    let center = projection.project_point(pad.center);
    let width_px =
        (projection.world_length_to_px(width_nm.round() as i64) - inset_px * 2.0).max(1.0);
    let height_px =
        (projection.world_length_to_px(height_nm.round() as i64) - inset_px * 2.0).max(1.0);
    match pad.shape_kind.as_str() {
        "circle" | "oval" => ellipse_points(center, width_px, height_px, pad.rotation_degrees, 48),
        _ => {
            let min_dim_px = width_px.min(height_px);
            let radius_px = match pad.shape_kind.as_str() {
                "roundrect" | "round_rect" => {
                    let ratio = (pad.roundrect_rratio_ppm as f32 / 1_000_000.0).clamp(0.0, 0.5);
                    (min_dim_px * ratio).max(1.0)
                }
                _ => 0.0,
            };
            rounded_rect_points(center, width_px, height_px, pad.rotation_degrees, radius_px)
        }
    }
}

#[allow(dead_code)]
fn component_should_draw_package_body(scene: &BoardReviewSceneV1, component_uuid: &str) -> bool {
    let pads: Vec<_> = scene
        .pads
        .iter()
        .filter(|pad| pad.component_uuid == component_uuid)
        .collect();
    let has_closed_outline = scene.component_graphics.iter().any(|graphic| {
        graphic.component_uuid == component_uuid
            && graphic.render_role == "component_mechanical"
            && graphic.closed
            && graphic.path.len() >= 4
    });
    let compact_inferred_body = compact_component_body_bounds(&pads).is_some();
    !pads.is_empty()
        && !pads.iter().any(|pad| pad.drill_nm.unwrap_or(0) > 0)
        && (has_closed_outline || compact_inferred_body)
}

fn compact_component_body_bounds(
    pads: &[&datum_gui_protocol::PadPrimitive],
) -> Option<datum_gui_protocol::RectNm> {
    inferred_component_body_bounds(pads).filter(|body| {
        let width = body.max_x - body.min_x;
        let height = body.max_y - body.min_y;
        width > 0 && height > 0 && width <= 4_500_000 && height <= 4_500_000
    })
}

fn inferred_component_body_bounds(
    pads: &[&datum_gui_protocol::PadPrimitive],
) -> Option<datum_gui_protocol::RectNm> {
    if pads.is_empty() {
        return None;
    }
    let pad_union = pads.iter().fold(
        datum_gui_protocol::RectNm {
            min_x: i64::MAX,
            min_y: i64::MAX,
            max_x: i64::MIN,
            max_y: i64::MIN,
        },
        |mut acc, pad| {
            acc.min_x = acc.min_x.min(pad.bounds.min_x);
            acc.min_y = acc.min_y.min(pad.bounds.min_y);
            acc.max_x = acc.max_x.max(pad.bounds.max_x);
            acc.max_y = acc.max_y.max(pad.bounds.max_y);
            acc
        },
    );
    let spread_x = (pad_union.max_x - pad_union.min_x) as f32;
    let spread_y = (pad_union.max_y - pad_union.min_y) as f32;
    let body = if spread_x >= spread_y {
        datum_gui_protocol::RectNm {
            min_x: (pad_union.min_x as f32 + spread_x * 0.28).round() as i64,
            min_y: (pad_union.min_y as f32 + spread_y * 0.06).round() as i64,
            max_x: (pad_union.max_x as f32 - spread_x * 0.28).round() as i64,
            max_y: (pad_union.max_y as f32 - spread_y * 0.06).round() as i64,
        }
    } else {
        datum_gui_protocol::RectNm {
            min_x: (pad_union.min_x as f32 + spread_x * 0.08).round() as i64,
            min_y: (pad_union.min_y as f32 + spread_y * 0.28).round() as i64,
            max_x: (pad_union.max_x as f32 - spread_x * 0.08).round() as i64,
            max_y: (pad_union.max_y as f32 - spread_y * 0.28).round() as i64,
        }
    };
    (body.max_x > body.min_x && body.max_y > body.min_y).then_some(body)
}

fn closed_component_body_graphic<'a>(
    scene: &'a BoardReviewSceneV1,
    component_uuid: &str,
) -> Option<&'a ComponentGraphicPrimitive> {
    scene
        .component_graphics
        .iter()
        .filter(|graphic| {
            graphic.component_uuid == component_uuid
                && graphic.render_role == "component_mechanical"
                && graphic.closed
                && graphic.path.len() >= 3
        })
        .max_by_key(|graphic| {
            let min_x = graphic.path.iter().map(|p| p.x).min().unwrap_or(0);
            let max_x = graphic.path.iter().map(|p| p.x).max().unwrap_or(0);
            let min_y = graphic.path.iter().map(|p| p.y).min().unwrap_or(0);
            let max_y = graphic.path.iter().map(|p| p.y).max().unwrap_or(0);
            (max_x - min_x) * (max_y - min_y)
        })
}

fn selected_component_body_graphic_id<'a>(
    scene: &'a BoardReviewSceneV1,
    component_uuid: &str,
) -> Option<&'a str> {
    closed_component_body_graphic(scene, component_uuid).map(|graphic| graphic.graphic_id.as_str())
}

#[allow(dead_code)]
fn push_inferred_package_body_from_pads(
    out: &mut Vec<Quad>,
    _component: &datum_gui_protocol::ComponentBounds,
    pads: &[&datum_gui_protocol::PadPrimitive],
    projection: &Projection,
    selected: bool,
    related: bool,
    dimmed: bool,
) {
    if pads.is_empty() {
        return;
    }
    let Some(body_nm) = inferred_component_body_bounds(pads) else {
        return;
    };
    let body = project_rect(body_nm, projection);
    if body.width <= 2.0 || body.height <= 2.0 {
        return;
    }
    let fill = dim_structural_color(
        if selected {
            [0.30, 0.32, 0.34]
        } else if related {
            [0.25, 0.27, 0.29]
        } else {
            [0.18, 0.19, 0.21]
        },
        dimmed,
    );
    let accent = dim_structural_color(
        if selected {
            AUTHOR_SELECTED
        } else if related {
            PAD_COPPER_RELATED
        } else {
            [0.56, 0.58, 0.62]
        },
        dimmed,
    );
    out.push(Quad::from_rect(body, fill));
    push_rect_border(out, body, accent, 1.0);
    if pads.len() >= 4 {
        let marker = RectPx {
            x: body.x + 4.0,
            y: body.y + 4.0,
            width: 4.0,
            height: 4.0,
        };
        push_projected_ellipse(
            out,
            marker,
            dim_structural_color(
                if selected || related {
                    PAD_COPPER_RELATED
                } else {
                    [0.96, 0.74, 0.44]
                },
                dimmed,
            ),
            14,
        );
    }
    let body_outline = inset_rect(body, 1.0, 1.0, 1.0, 1.0);
    push_rect_border(
        out,
        body_outline,
        dim_structural_color([0.47, 0.52, 0.57], dimmed),
        1.0,
    );
}

#[allow(dead_code)]
fn push_inferred_package_body_from_pads_world(
    out: &mut Vec<Quad>,
    component: &datum_gui_protocol::ComponentBounds,
    pads: &[&datum_gui_protocol::PadPrimitive],
    selected: bool,
    related: bool,
    dimmed: bool,
    reference_projection: &Projection,
) {
    if pads.is_empty() {
        return;
    }
    let Some((center, width, height, rotation_degrees)) =
        inferred_component_body_geometry(pads, component.rotation_degrees)
    else {
        return;
    };
    let body_polygon: Vec<PointNm> =
        rounded_rect_points(center, width, height, rotation_degrees, 0.0)
            .into_iter()
            .map(|(x, y)| PointNm {
                x: x.round() as i64,
                y: y.round() as i64,
            })
            .collect();
    let fill = dim_structural_color(
        if selected {
            [0.30, 0.32, 0.34]
        } else if related {
            [0.25, 0.27, 0.29]
        } else {
            [0.18, 0.19, 0.21]
        },
        dimmed,
    );
    let accent = dim_structural_color(
        if selected {
            AUTHOR_SELECTED
        } else if related {
            PAD_COPPER_RELATED
        } else {
            [0.56, 0.58, 0.62]
        },
        dimmed,
    );
    push_world_polygon_fill(out, &body_polygon, fill);
    let border_stroke = world_stroke_nm(if selected { 2.5 } else { 1.0 }, reference_projection);
    push_world_polyline_segments(out, &close_path(&body_polygon), border_stroke, accent);
    let inset = border_stroke.max(1.0) * 2.0;
    let inner_width = (width - inset * 2.0).max(1.0);
    let inner_height = (height - inset * 2.0).max(1.0);
    if inner_width > 1.0 && inner_height > 1.0 {
        let inner_polygon: Vec<PointNm> =
            rounded_rect_points(center, inner_width, inner_height, rotation_degrees, 0.0)
                .into_iter()
                .map(|(x, y)| PointNm {
                    x: x.round() as i64,
                    y: y.round() as i64,
                })
                .collect();
        push_world_polyline_segments(
            out,
            &close_path(&inner_polygon),
            border_stroke,
            dim_structural_color([0.47, 0.52, 0.57], dimmed),
        );
    }
}

#[allow(dead_code)]
fn push_selected_component_body_from_graphic_world(
    out: &mut Vec<Quad>,
    graphic: &ComponentGraphicPrimitive,
    selected: bool,
    related: bool,
    dimmed: bool,
    reference_projection: &Projection,
) {
    let fill = dim_structural_color(
        if selected {
            [0.30, 0.32, 0.34]
        } else if related {
            [0.25, 0.27, 0.29]
        } else {
            [0.18, 0.19, 0.21]
        },
        dimmed,
    );
    let accent = dim_structural_color(
        if selected {
            AUTHOR_SELECTED
        } else if related {
            PAD_COPPER_RELATED
        } else {
            [0.56, 0.58, 0.62]
        },
        dimmed,
    );
    push_world_convex_polygon_fill(out, &graphic.path, fill);
    let border_stroke = world_stroke_nm(if selected { 2.5 } else { 1.0 }, reference_projection);
    push_world_polyline_segments(out, &close_path(&graphic.path), border_stroke, accent);
}

#[allow(dead_code)]
fn push_world_convex_polygon_fill(out: &mut Vec<Quad>, polygon: &[PointNm], color: [f32; 3]) {
    if polygon.len() < 3 {
        return;
    }
    let center = (
        polygon.iter().map(|p| p.x as f32).sum::<f32>() / polygon.len() as f32,
        polygon.iter().map(|p| p.y as f32).sum::<f32>() / polygon.len() as f32,
    );
    for edge in polygon.windows(2) {
        push_world_triangle(
            out,
            center,
            (edge[0].x as f32, edge[0].y as f32),
            (edge[1].x as f32, edge[1].y as f32),
            color,
        );
    }
    push_world_triangle(
        out,
        center,
        (
            polygon[polygon.len() - 1].x as f32,
            polygon[polygon.len() - 1].y as f32,
        ),
        (polygon[0].x as f32, polygon[0].y as f32),
        color,
    );
}

#[allow(dead_code)]
fn inferred_component_body_geometry(
    pads: &[&datum_gui_protocol::PadPrimitive],
    fallback_rotation_degrees: f32,
) -> Option<((f32, f32), f32, f32, f32)> {
    let body = inferred_component_body_bounds(pads)?;
    let center = (
        ((body.min_x + body.max_x) as f32) * 0.5,
        ((body.min_y + body.max_y) as f32) * 0.5,
    );
    let rotation_degrees = fallback_rotation_degrees;

    let local_points: Vec<(f32, f32)> = pads
        .iter()
        .flat_map(|pad| {
            let corners = [
                (pad.bounds.min_x as f32, pad.bounds.min_y as f32),
                (pad.bounds.max_x as f32, pad.bounds.min_y as f32),
                (pad.bounds.max_x as f32, pad.bounds.max_y as f32),
                (pad.bounds.min_x as f32, pad.bounds.max_y as f32),
            ];
            corners.into_iter().map(move |point| {
                let dx = point.0 - center.0;
                let dy = point.1 - center.1;
                let rad = rotation_degrees.to_radians();
                let cos = rad.cos();
                let sin = rad.sin();
                // Convert world-space points back into the component's local frame.
                // Using the forward rotation here swaps quarter-turn parts.
                (dx * cos + dy * sin, -dx * sin + dy * cos)
            })
        })
        .collect();

    if local_points.is_empty() {
        return None;
    }

    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for (x, y) in local_points {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    let spread_x = max_x - min_x;
    let spread_y = max_y - min_y;
    let (body_min_x, body_max_x, body_min_y, body_max_y) = if spread_x >= spread_y {
        (
            min_x + spread_x * 0.28,
            max_x - spread_x * 0.28,
            min_y + spread_y * 0.06,
            max_y - spread_y * 0.06,
        )
    } else {
        (
            min_x + spread_x * 0.08,
            max_x - spread_x * 0.08,
            min_y + spread_y * 0.28,
            max_y - spread_y * 0.28,
        )
    };
    let width = (body_max_x - body_min_x).max(1.0);
    let height = (body_max_y - body_min_y).max(1.0);
    Some((center, width, height, rotation_degrees))
}

// Render helper threads many quad/text-run/hit-region sinks.
#[allow(clippy::too_many_arguments)]
fn push_component_text_primitive(
    text_runs: &mut Vec<TextRun>,
    text: &ComponentTextPrimitive,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    projection: &Projection,
    clip_bounds: RectPx,
    selected: bool,
    related: bool,
    dimmed: bool,
) {
    let (x, y) = project_point(text.position, projection);
    let color = component_text_color(text, scene_layers, selected, related, dimmed);
    let size = footprint_text_size_px(text.height_nm, projection);
    draw_text_clipped(
        &truncate_text(&text.text.to_uppercase(), 10),
        x - size * 1.2,
        y - size * 0.45,
        size,
        color,
        TextFace::Mono,
        clip_bounds,
        text_runs,
    );
}

fn component_text_color(
    text: &ComponentTextPrimitive,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    selected: bool,
    related: bool,
    dimmed: bool,
) -> [f32; 3] {
    dim_context_color(
        match text.render_role.as_str() {
            "component_mechanical" => {
                if selected {
                    selected_mechanical_color(COMPONENT_MECHANICAL)
                } else if related {
                    COMPONENT_MECHANICAL_RELATED
                } else {
                    COMPONENT_MECHANICAL
                }
            }
            _ => {
                if selected {
                    selected_silk_color(
                        resolve_layer_appearance_with_scene(text.layer_id.as_deref(), scene_layers)
                            .silkscreen,
                    )
                } else if related {
                    COMPONENT_SILK_RELATED
                } else {
                    resolve_layer_appearance_with_scene(text.layer_id.as_deref(), scene_layers)
                        .silkscreen
                }
            }
        },
        dimmed,
    )
}

fn component_has_detail_text(scene: &BoardReviewSceneV1, component_uuid: &str) -> bool {
    scene
        .component_texts
        .iter()
        .any(|text| text.component_uuid == component_uuid)
        || scene.board_texts.iter().any(|text| {
            text.style_class.as_deref().is_some_and(|style_class| {
                imported_board_text_belongs_to_component(style_class, component_uuid)
            })
        })
        || scene.component_graphics.iter().any(|graphic| {
            graphic.component_uuid == component_uuid
                && (graphic.graphic_id.contains(":kicad-text-cache:")
                    || graphic.graphic_id.contains(":prop-cache:")
                    || graphic.graphic_id.contains(":kicad-text-stroke:")
                    || graphic.graphic_id.contains(":prop-stroke:"))
        })
}

fn imported_board_text_belongs_to_component(style_class: &str, component_uuid: &str) -> bool {
    ["imported_kicad_property_text:", "imported_kicad_fp_text:"]
        .iter()
        .any(|prefix| {
            style_class
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with(component_uuid))
        })
}

// Render helper threads many quad/text-run/hit-region sinks.
#[allow(clippy::too_many_arguments)]
fn push_component_text_world(
    out: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    text: &ComponentTextPrimitive,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    projection: &Projection,
    clip_bounds: RectPx,
    selected: bool,
    related: bool,
    dimmed: bool,
) {
    let color = component_text_color(text, scene_layers, selected, related, dimmed);
    if !text.cached_polygons.is_empty() {
        for polygon in &text.cached_polygons {
            if polygon.len() >= 3 {
                let projected: Vec<(f32, f32)> = polygon
                    .iter()
                    .map(|point| project_point(*point, projection))
                    .collect();
                push_projected_polygon_fill(out, &projected, color);
            }
        }
        return;
    }

    let rotation = text.rotation_degrees.round() as i32;
    if rotation.rem_euclid(180) == 0 {
        push_component_text_primitive(
            text_runs,
            text,
            scene_layers,
            projection,
            clip_bounds,
            selected,
            related,
            dimmed,
        );
        return;
    }

    let normalized = text.text.to_uppercase();
    let board_text = BoardText {
        uuid: Uuid::nil(),
        text: normalized,
        position: Point {
            x: text.position.x,
            y: text.position.y,
        },
        rotation,
        layer: 0 as LayerId,
        render_intent: eda_engine::text::TextRenderIntent::Manufacturing,
        family: eda_engine::text::TextFamilyId::default(),
        family_source: eda_engine::text::TextFamilySource::ImplicitDefault,
        style: eda_engine::text::TextStyleId::default(),
        height_nm: text.height_nm,
        stroke_width_nm: text
            .stroke_width_nm
            .unwrap_or((text.height_nm / 10).clamp(80_000, 250_000)),
        h_align: eda_engine::text::TextHAlign::Left,
        v_align: eda_engine::text::TextVAlign::Bottom,
        mirrored: false,
        keep_upright: false,
        line_spacing_ratio_ppm: 1_000_000,
        italic: false,
        bold: false,
        style_class: None,
    };
    match render_silkscreen_text_strokes(&board_text) {
        Ok(strokes) if !strokes.is_empty() => {
            for stroke in strokes {
                let path = [
                    stroke_text_point_to_board_space(text.position, stroke.from),
                    stroke_text_point_to_board_space(text.position, stroke.to),
                ];
                let thickness_px = projection
                    .world_length_to_px(stroke.width_nm)
                    .clamp(1.0, 6.0);
                push_polyline_segments(out, &path, projection, color, thickness_px);
            }
        }
        _ => push_component_text_primitive(
            text_runs,
            text,
            scene_layers,
            projection,
            clip_bounds,
            selected,
            related,
            dimmed,
        ),
    }
}

fn stroke_text_point_to_board_space(origin: PointNm, point: Point) -> PointNm {
    // The engine silkscreen stroke font is authored in a conventional
    // Cartesian Y-up frame. Datum's board/world render space is Y-down, so
    // reflected text strokes are needed before projection into the viewport.
    PointNm {
        x: point.x,
        y: origin.y * 2 - point.y,
    }
}

fn board_surface_color(role: BoardSurfaceRole) -> [f32; 3] {
    match role {
        BoardSurfaceRole::InnerField => BOARD_INNER_FIELD,
        BoardSurfaceRole::GridMajor => BOARD_GRID_MAJOR,
        BoardSurfaceRole::GridMinor => BOARD_GRID_MINOR,
        BoardSurfaceRole::Edge => BOARD_EDGE,
    }
}

fn mix_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] * (1.0 - t) + b[0] * t,
        a[1] * (1.0 - t) + b[1] * t,
        a[2] * (1.0 - t) + b[2] * t,
    ]
}

fn resolve_layer_family_with_scene(
    layer_id: Option<&str>,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> LayerFamily {
    let Some(id) = layer_id else {
        return LayerFamily::Unknown;
    };
    // Look up the real layer name from the scene
    if let Some(layer) = scene_layers.iter().find(|l| l.layer_id == id) {
        return match layer.name.as_str() {
            "F.Cu" => LayerFamily::TopCopper,
            "B.Cu" => LayerFamily::BottomCopper,
            name if name.ends_with(".Cu") => LayerFamily::InnerCopper,
            _ => LayerFamily::Unknown,
        };
    }
    // Fallback
    match id {
        "L0" | "F.Cu" => LayerFamily::TopCopper,
        "L31" | "B.Cu" => LayerFamily::BottomCopper,
        name if name.ends_with(".Cu") => LayerFamily::InnerCopper,
        _ => LayerFamily::Unknown,
    }
}

/// Build a copper `LayerAppearance` from a single muted Design Book material
/// token. `related`/`proposal` are bounded lightenings of that base and silk
/// comes from the token seam, so no raw hex copper values live in the render
/// path (`docs/gui/VISUAL_LANGUAGE.md` §7; consumes `design_tokens::content`).
fn copper_appearance_from_token(base: [f32; 3], silk: [f32; 3]) -> LayerAppearance {
    let lighten = |t: f32| mix_color(base, [1.0, 1.0, 1.0], t);
    LayerAppearance::from_copper_material(base, lighten(0.42), lighten(0.22), silk)
}

fn resolve_layer_appearance_with_scene(
    layer_id: Option<&str>,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> LayerAppearance {
    match resolve_layer_family_with_scene(layer_id, scene_layers) {
        LayerFamily::TopCopper => copper_appearance_from_token(
            design_tokens::content::COPPER_FRONT,
            design_tokens::content::SILK_TOP,
        ),
        LayerFamily::InnerCopper => copper_appearance_from_token(
            design_tokens::content::COPPER_IN2,
            design_tokens::content::SILK_TOP,
        ),
        LayerFamily::BottomCopper => copper_appearance_from_token(
            design_tokens::content::COPPER_BACK,
            design_tokens::content::SILK_BOTTOM,
        ),
        // Bounded exception: geometry whose layer cannot be resolved to a
        // known copper family keeps deliberately divergent fallback colors so
        // unresolved-layer drift stays visible instead of masquerading as a
        // real material lane.
        LayerFamily::Unknown => LayerAppearance {
            authored_track: AUTHOR_BASE,
            pad_copper: PAD_COPPER,
            pad_related: PAD_COPPER_RELATED,
            zone_fill: [0.26, 0.12, 0.24],
            zone_outline: [0.57, 0.24, 0.53],
            proposal: PROPOSAL_BASE,
            silkscreen: COMPONENT_SILK,
        },
    }
}

fn proposal_layer_color(layer_id: Option<&str>) -> [f32; 3] {
    resolve_layer_appearance(layer_id).proposal
}

fn resolve_layer_appearance(layer_id: Option<&str>) -> LayerAppearance {
    resolve_layer_appearance_with_scene(layer_id, &[])
}

fn layer_swatch_color_with_scene(
    layer_id: Option<&str>,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> [f32; 3] {
    let Some(id) = layer_id else {
        return AUTHOR_BASE;
    };
    let name = scene_layer_name(id, scene_layers).unwrap_or(id);
    match name {
        "F.Cu" => design_tokens::content::COPPER_FRONT,
        "B.Cu" => design_tokens::content::COPPER_BACK,
        "Edge.Cuts" => design_tokens::content::EDGE,
        "Ratsnest" | "Airwires" => design_tokens::content::RATSNEST,
        "F.SilkS" | "F.Silkscreen" => design_tokens::content::SILK_TOP,
        "B.SilkS" | "B.Silkscreen" => design_tokens::content::SILK_BOTTOM,
        name if name.ends_with(".Cu") => design_tokens::content::COPPER_IN2,
        name if name.ends_with(".Mask") => design_tokens::content::MASK,
        name if name.ends_with(".Paste") => design_tokens::content::PASTE,
        _ => resolve_layer_appearance_with_scene(Some(id), scene_layers).authored_track,
    }
}

fn scene_layer_name<'a>(
    layer_id: &str,
    scene_layers: &'a [datum_gui_protocol::SceneLayer],
) -> Option<&'a str> {
    scene_layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .map(|layer| layer.name.as_str())
}

fn render_stage_for_layer(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> RenderStage {
    match scene_layer_name(layer_id, scene_layers).unwrap_or(layer_id) {
        "B.Cu" => RenderStage::BottomCopper,
        name if name.ends_with(".Cu") && name != "F.Cu" => RenderStage::InnerCopper,
        "F.Cu" => RenderStage::TopCopper,
        "B.Paste" => RenderStage::BottomPaste,
        "F.Paste" => RenderStage::TopPaste,
        "B.Mask" => RenderStage::BottomMask,
        "F.Mask" => RenderStage::TopMask,
        "B.SilkS" => RenderStage::BottomSilk,
        "F.SilkS" => RenderStage::TopSilk,
        // Schematic net-role layers (P2.2c) all draw in the top-silk pass so the
        // schematic pane renders in the post-copper walk; within the stage the
        // projection's insertion order is the draw order. Colour is resolved
        // separately by `schematic_layer_world_color`.
        name if name.starts_with("Schematic.") => RenderStage::TopSilk,
        "Edge.Cuts" => RenderStage::Edge,
        name if name.ends_with(".CrtYd") || name.ends_with(".Fab") => RenderStage::Mechanical,
        _ => RenderStage::Other,
    }
}

fn render_stage_priority(stage: RenderStage) -> u32 {
    // The enum declaration order is the single encoding of the declared
    // render-stack policy; priority is its discriminant.
    stage as u32
}

fn scene_layer_stack_priority(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> u32 {
    render_stage_priority(render_stage_for_layer(layer_id, scene_layers))
}

fn graphic_render_stage(
    layer_id: Option<&str>,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    default_stage: RenderStage,
) -> RenderStage {
    layer_id
        .map(|id| render_stage_for_layer(id, scene_layers))
        .unwrap_or(default_stage)
}

fn copper_pass_priority_for_layer(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> Option<u32> {
    match render_stage_for_layer(layer_id, scene_layers) {
        RenderStage::BottomCopper => Some(0),
        RenderStage::InnerCopper => Some(1),
        RenderStage::TopCopper => Some(2),
        _ => None,
    }
}

fn mask_or_paste_layer_color(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
) -> [f32; 3] {
    match scene_layer_name(layer_id, scene_layers) {
        Some("F.Mask") => TOP_MASK_OPENING,
        Some("B.Mask") => BOTTOM_MASK_OPENING,
        Some("F.Paste") => TOP_PASTE_OPENING,
        Some("B.Paste") => BOTTOM_PASTE_OPENING,
        _ => resolve_layer_appearance_with_scene(Some(layer_id), scene_layers).pad_copper,
    }
}

fn footprint_text_size_px(height_nm: i64, projection: &Projection) -> f32 {
    world_length_to_px(height_nm, projection).max(1.0)
}

fn world_length_to_px(length_nm: i64, projection: &Projection) -> f32 {
    projection.world_length_to_px(length_nm)
}

fn component_silk_color(layer_id: Option<&str>) -> [f32; 3] {
    resolve_layer_appearance(layer_id).silkscreen
}

fn detail_tier(projection: &Projection) -> DetailTier {
    let px_per_mm = world_length_to_px(1_000_000, projection);
    if px_per_mm >= 18.0 {
        DetailTier::Fine
    } else if px_per_mm >= 8.0 {
        DetailTier::Normal
    } else {
        DetailTier::Coarse
    }
}

fn floor_multiple(value: i64, pitch: i64) -> i64 {
    value.div_euclid(pitch) * pitch
}

fn ceil_multiple(value: i64, pitch: i64) -> i64 {
    if value.rem_euclid(pitch) == 0 {
        value
    } else {
        value.div_euclid(pitch) * pitch + pitch
    }
}

