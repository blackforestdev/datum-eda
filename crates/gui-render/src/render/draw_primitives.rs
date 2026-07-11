fn width_to_px(width_nm: i64) -> f32 {
    ((width_nm as f32) / 120_000.0).clamp(0.9, 3.6)
}

fn overlay_route_width_px(
    width_nm: Option<i64>,
    selected: bool,
    projection: Option<&Projection>,
) -> f32 {
    // If we have real width and a projection, use camera-aware sizing.
    // Preserve true proportional width down to a sub-pixel legibility floor so
    // distinct physical widths remain visually distinct at wide zoom.
    if let (Some(w), Some(proj)) = (width_nm, projection) {
        let projected = proj.world_length_to_px(w);
        let floor = if selected { 2.5 } else { 2.0 };
        return projected.max(floor).clamp(2.0, 32.0);
    }
    let base = width_nm.map(width_to_px).unwrap_or(2.4);
    let scaled = if selected { base * 3.2 } else { base * 2.0 };
    scaled.clamp(
        if selected { 4.5 } else { 3.2 },
        if selected { 10.0 } else { 7.0 },
    )
}

fn push_overlay(
    out: &mut Vec<Quad>,
    overlay: &ProposalOverlayPrimitive,
    projection: &Projection,
    color: [f32; 3],
    selected: bool,
    editor_move_preview: bool,
) -> Vec<RectPx> {
    if editor_move_preview {
        return push_overlay_move_preview(out, overlay, projection, color, selected);
    }
    let layer_color = proposal_layer_color(overlay.layer_id.as_deref());
    let outer_color = if selected {
        PROPOSAL_OUTER
    } else {
        mix_color(color, layer_color, 0.45)
    };
    let underlay_color = if selected {
        PROPOSAL_UNDERLAY
    } else {
        mix_color(PROPOSAL_UNDERLAY, layer_color, 0.18)
    };
    let fill_color = if selected { PROPOSAL_FOCUS } else { color };
    match overlay.primitive_kind.as_str() {
        "anchor_marker" => {
            let outer_size = if selected { 17.0 } else { 12.0 };
            let ring_size = if selected { 10.0 } else { 7.0 };
            let core_size = if selected { 4.2 } else { 3.0 };
            let mut rects = push_points(
                out,
                &overlay.path,
                projection,
                if selected {
                    PROPOSAL_UNDERLAY
                } else {
                    [0.30, 0.22, 0.12]
                },
                outer_size,
            );
            rects.extend(push_points(
                out,
                &overlay.path,
                projection,
                if selected {
                    PROPOSAL_FOCUS
                } else {
                    PROPOSAL_ANCHOR_RING
                },
                ring_size,
            ));
            rects.extend(push_points(
                out,
                &overlay.path,
                projection,
                PROPOSAL_ANCHOR_CORE,
                core_size,
            ));
            rects
        }
        "via" => {
            let Some(center) = overlay.path.first().copied() else {
                return Vec::new();
            };
            let diameter_nm = overlay
                .diameter_nm
                .or(overlay.width_nm)
                .unwrap_or(600_000)
                .max(1);
            let drill_nm = overlay.drill_nm.unwrap_or(diameter_nm / 2).max(1);
            let radius = (diameter_nm as f32 * 0.5).round() as i64;
            let drill_radius = (drill_nm as f32 * 0.5).round() as i64;
            let outer_rect = datum_gui_protocol::RectNm {
                min_x: center.x - radius,
                min_y: center.y - radius,
                max_x: center.x + radius,
                max_y: center.y + radius,
            };
            push_world_ellipse_nm(out, outer_rect, outer_color, 96);
            let ring_inset = (diameter_nm as f32 * 0.14).round().max(1.0);
            push_world_ellipse_nm(
                out,
                world_inset_rect(outer_rect, ring_inset),
                fill_color,
                96,
            );
            push_world_ellipse_nm(
                out,
                datum_gui_protocol::RectNm {
                    min_x: center.x - drill_radius,
                    min_y: center.y - drill_radius,
                    max_x: center.x + drill_radius,
                    max_y: center.y + drill_radius,
                },
                underlay_color,
                96,
            );
            vec![project_rect(outer_rect, projection)]
        }
        _ => {
            let route_width = overlay_route_width_px(overlay.width_nm, selected, Some(projection));
            let underlay_width = if selected {
                route_width + 5.2
            } else {
                route_width + 1.8
            };
            let outer_width = if selected {
                route_width + 2.2
            } else {
                route_width + 0.55
            };
            let inner_width = if selected {
                route_width + 0.45
            } else {
                route_width.max(1.5)
            };
            let mut hit_rects = push_polyline_segments(
                out,
                &overlay.path,
                projection,
                underlay_color,
                underlay_width,
            );
            if selected || overlay.path.len() == 2 {
                hit_rects.extend(push_polyline_endcaps(
                    out,
                    &overlay.path,
                    projection,
                    outer_color,
                    outer_width,
                    (route_width * 2.7).clamp(10.0, 18.0),
                ));
            }
            if let (Some(first), Some(last)) = (overlay.path.first(), overlay.path.last()) {
                let endpoint_radius = if selected {
                    (route_width + 1.8).clamp(4.8, 8.0)
                } else {
                    (route_width + 1.0).clamp(3.8, 5.8)
                };
                hit_rects.extend(push_points(
                    out,
                    &[*first, *last],
                    projection,
                    outer_color,
                    endpoint_radius,
                ));
                hit_rects.extend(push_points(
                    out,
                    &[*first, *last],
                    projection,
                    fill_color,
                    (endpoint_radius * 0.42).clamp(1.8, 3.2),
                ));
            }
            hit_rects.extend(push_polyline_segments(
                out,
                &overlay.path,
                projection,
                outer_color,
                outer_width,
            ));
            hit_rects.extend(push_polyline_segments(
                out,
                &overlay.path,
                projection,
                if selected { PROPOSAL_FOCUS } else { color },
                inner_width,
            ));
            if selected {
                hit_rects.extend(push_polyline_endcaps(
                    out,
                    &overlay.path,
                    projection,
                    PROPOSAL_FOCUS,
                    route_width + 0.8,
                    (route_width * 2.0).clamp(8.0, 14.0),
                ));
            }
            let corner_fill = if selected {
                (inner_width - 0.25).max(1.2)
            } else {
                (inner_width - 0.35).max(1.0)
            };
            if overlay.path.len() > 2 {
                hit_rects.extend(push_points(
                    out,
                    &overlay.path[1..overlay.path.len() - 1],
                    projection,
                    fill_color,
                    corner_fill,
                ));
            }
            hit_rects
        }
    }
}

fn push_overlay_move_preview(
    out: &mut Vec<Quad>,
    overlay: &ProposalOverlayPrimitive,
    projection: &Projection,
    color: [f32; 3],
    selected: bool,
) -> Vec<RectPx> {
    match overlay.primitive_kind.as_str() {
        "anchor_marker" => push_points(
            out,
            &overlay.path,
            projection,
            if selected {
                PROPOSAL_FOCUS
            } else {
                AUTHOR_RELATED
            },
            if selected { 5.0 } else { 4.0 },
        ),
        _ => {
            let guide_color = if selected {
                mix_color(PROPOSAL_FOCUS, PROPOSAL_CENTERLINE, 0.35)
            } else {
                mix_color(color, PROPOSAL_CENTERLINE, 0.55)
            };
            let underlay = if selected {
                mix_color(PROPOSAL_UNDERLAY, guide_color, 0.25)
            } else {
                mix_color(DIAGNOSTIC_UNDERLAY, guide_color, 0.18)
            };
            let base_width = if selected { 1.4 } else { 1.1 };
            let mut rects = push_dashed_polyline_segments(
                out,
                &overlay.path,
                projection,
                underlay,
                base_width + 0.9,
                10.0,
                6.0,
            );
            rects.extend(push_dashed_polyline_segments(
                out,
                &overlay.path,
                projection,
                guide_color,
                base_width,
                10.0,
                6.0,
            ));
            rects.extend(push_points(
                out,
                &overlay.path,
                projection,
                guide_color,
                if selected { 3.4 } else { 3.0 },
            ));
            rects
        }
    }
}

#[allow(dead_code)]
fn push_polygon_fill(
    out: &mut Vec<Quad>,
    polygon: &[PointNm],
    projection: &Projection,
    color: [f32; 3],
) {
    if polygon.len() < 3 {
        return;
    }
    let projected: Vec<(f32, f32)> = polygon
        .iter()
        .map(|point| projection.project_point(*point))
        .collect();
    push_projected_polygon_fill(out, &projected, color);
}

#[allow(dead_code)]
fn push_component_primitive(
    out: &mut Vec<Quad>,
    component: &datum_gui_protocol::ComponentBounds,
    projection: &Projection,
    selected: bool,
    related: bool,
    dimmed: bool,
) -> RectPx {
    let body = push_world_rect(
        out,
        component.bounds,
        projection,
        dim_structural_color(
            if selected {
                COMPONENT_BODY_SELECTED
            } else if related {
                COMPONENT_BODY_RELATED
            } else {
                COMPONENT_BODY
            },
            dimmed,
        ),
    );
    let header_h = body.height.clamp(6.0, 12.0);
    let header = RectPx {
        x: body.x + 1.0,
        y: body.y + 1.0,
        width: (body.width - 2.0).max(1.0),
        height: (header_h - 1.0).max(1.0),
    };
    out.push(Quad::from_rect(
        header,
        dim_structural_color(COMPONENT_HEADER, dimmed),
    ));
    let inner = inset_rect(body, 2.0, header_h + 1.0, 2.0, 2.0);
    if inner.width > 2.0 && inner.height > 2.0 {
        out.push(Quad::from_rect(
            inner,
            dim_structural_color([0.30, 0.32, 0.36], dimmed),
        ));
    }
    let pin1 = RectPx {
        x: body.x + 4.0,
        y: body.y + 4.0,
        width: 3.0,
        height: 3.0,
    };
    out.push(Quad::from_rect(
        pin1,
        dim_structural_color(
            if selected || related {
                PAD_COPPER_RELATED
            } else {
                PAD_COPPER
            },
            dimmed,
        ),
    ));
    push_rect_border(
        out,
        body,
        dim_structural_color(
            if selected {
                AUTHOR_SELECTED
            } else if related {
                AUTHOR_RELATED
            } else {
                COMPONENT_OUTLINE
            },
            dimmed,
        ),
        1.0,
    );
    body
}

#[allow(dead_code)]
fn push_component_primitive_world(
    out: &mut Vec<Quad>,
    component: &datum_gui_protocol::ComponentBounds,
    selected: bool,
    related: bool,
    dimmed: bool,
    reference_projection: &Projection,
) {
    let body_color = dim_structural_color(
        if selected {
            COMPONENT_BODY_SELECTED
        } else if related {
            COMPONENT_BODY_RELATED
        } else {
            COMPONENT_BODY
        },
        dimmed,
    );
    push_world_rect_nm(out, component.bounds, body_color);
    let stroke_nm = world_stroke_nm(if selected { 2.5 } else { 1.0 }, reference_projection);
    let header_size_nm = world_stroke_nm(10.0, reference_projection);
    let s = stroke_nm.round() as i64;
    let h = header_size_nm.round() as i64;
    let rotation = component.rotation_degrees.round() as i32;
    let header = match rotation.rem_euclid(360) {
        180 => datum_gui_protocol::RectNm {
            min_x: component.bounds.min_x + s,
            min_y: component.bounds.max_y - h,
            max_x: component.bounds.max_x - s,
            max_y: component.bounds.max_y - s,
        },
        90 => datum_gui_protocol::RectNm {
            min_x: component.bounds.max_x - h,
            min_y: component.bounds.min_y + s,
            max_x: component.bounds.max_x - s,
            max_y: component.bounds.max_y - s,
        },
        270 => datum_gui_protocol::RectNm {
            min_x: component.bounds.min_x + s,
            min_y: component.bounds.min_y + s,
            max_x: component.bounds.min_x + h,
            max_y: component.bounds.max_y - s,
        },
        _ => datum_gui_protocol::RectNm {
            min_x: component.bounds.min_x + s,
            min_y: component.bounds.min_y + s,
            max_x: component.bounds.max_x - s,
            max_y: component.bounds.min_y + h,
        },
    };
    if header.max_x > header.min_x && header.max_y > header.min_y {
        push_world_rect_nm(out, header, dim_structural_color(COMPONENT_HEADER, dimmed));
    }
    push_world_rect_border_nm(
        out,
        component.bounds,
        dim_structural_color(
            if selected {
                AUTHOR_SELECTED
            } else if related {
                AUTHOR_RELATED
            } else {
                COMPONENT_OUTLINE
            },
            dimmed,
        ),
        stroke_nm,
    );
}

#[allow(dead_code)]
fn push_component_graphic_primitive(
    out: &mut Vec<Quad>,
    graphic: &ComponentGraphicPrimitive,
    projection: &Projection,
    selected: bool,
    related: bool,
    dimmed: bool,
) {
    let (base_color, width_scale) = match graphic.render_role.as_str() {
        "component_mechanical" => (
            if selected {
                selected_mechanical_color(COMPONENT_MECHANICAL)
            } else if related {
                COMPONENT_MECHANICAL_RELATED
            } else {
                COMPONENT_MECHANICAL
            },
            1.0,
        ),
        _ => (
            if selected {
                selected_silk_color(component_silk_color(graphic.layer_id.as_deref()))
            } else if related {
                COMPONENT_SILK_RELATED
            } else {
                component_silk_color(graphic.layer_id.as_deref())
            },
            1.15,
        ),
    };
    let color = dim_context_color(base_color, dimmed);
    if graphic.primitive_kind == "polygon" && graphic.path.len() >= 3 {
        let fill_color = match graphic.render_role.as_str() {
            "component_mechanical" => mix_color(color, BOARD_INNER_FIELD, 0.55),
            _ if graphic.width_nm.is_none() => color,
            _ => mix_color(color, BOARD_INNER_FIELD, 0.20),
        };
        push_polygon_fill(out, &graphic.path, projection, fill_color);
        if graphic.width_nm.is_none() && graphic.render_role != "component_mechanical" {
            return;
        }
    }
    let width = graphic.width_nm.map(width_to_px).unwrap_or(1.1) * width_scale;
    let path = if graphic.closed {
        close_path(&graphic.path)
    } else {
        graphic.path.clone()
    };
    if graphic.closed && graphic.render_role == "component_mechanical" {
        push_dashed_polyline_segments(out, &path, projection, color, width.max(0.8), 10.0, 7.0);
        return;
    }
    push_polyline_segments(out, &path, projection, color, width.max(1.0));
}

fn push_component_graphic_primitive_world(
    out: &mut Vec<Quad>,
    graphic: &ComponentGraphicPrimitive,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    selected: bool,
    related: bool,
    dimmed: bool,
    reference_projection: &Projection,
) {
    let (base_color, width_scale) = match graphic.render_role.as_str() {
        "component_mechanical" => (
            if selected {
                selected_mechanical_color(COMPONENT_MECHANICAL)
            } else if related {
                COMPONENT_MECHANICAL_RELATED
            } else {
                COMPONENT_MECHANICAL
            },
            1.0,
        ),
        _ => (
            if selected {
                selected_silk_color(
                    resolve_layer_appearance_with_scene(graphic.layer_id.as_deref(), scene_layers)
                        .silkscreen,
                )
            } else if related {
                COMPONENT_SILK_RELATED
            } else {
                resolve_layer_appearance_with_scene(graphic.layer_id.as_deref(), scene_layers)
                    .silkscreen
            },
            1.15,
        ),
    };
    let color = dim_context_color(base_color, dimmed);
    if graphic.primitive_kind == "polygon" && graphic.path.len() >= 3 {
        let fill_color = match graphic.render_role.as_str() {
            "component_mechanical" => mix_color(color, BOARD_INNER_FIELD, 0.55),
            _ if graphic.width_nm.is_none() => color,
            _ => mix_color(color, BOARD_INNER_FIELD, 0.20),
        };
        push_world_polygon_fill_contours(out, &graphic.path, &graphic.holes, fill_color);
        if graphic.width_nm.is_none() && graphic.render_role != "component_mechanical" {
            return;
        }
    }
    let width_nm = board_graphic_nominal_nm(
        graphic.layer_id.as_deref().unwrap_or("F.SilkS"),
        graphic.width_nm,
    ) as f32
        * width_scale;
    let path = if graphic.closed {
        close_path(&graphic.path)
    } else {
        graphic.path.clone()
    };
    if graphic.closed && graphic.render_role == "component_mechanical" {
        push_world_dashed_polyline_segments(
            out,
            &path,
            width_nm.max(1.0),
            world_stroke_nm(10.0, reference_projection),
            world_stroke_nm(7.0, reference_projection),
            color,
        );
        return;
    }
    let w = width_nm.max(1.0);
    push_world_polyline_segments(out, &path, w, color);
    // Round-cap each vertex so that separate fp_line segments sharing an
    // endpoint don't leave diagonal gaps at 90-degree corners. Each cap is
    // a small filled circle matching the stroke width.
    let half = (w * 0.5) as i64;
    for pt in &path {
        push_world_ellipse_nm(
            out,
            datum_gui_protocol::RectNm {
                min_x: pt.x - half,
                min_y: pt.y - half,
                max_x: pt.x + half,
                max_y: pt.y + half,
            },
            color,
            16,
        );
    }
}

fn push_board_graphic_primitive_world(
    out: &mut Vec<Quad>,
    graphic: &BoardGraphicPrimitive,
    color: [f32; 3],
    _reference_projection: &Projection,
) {
    if graphic.primitive_kind == "polygon" && graphic.path.len() >= 3 {
        push_world_polygon_fill_contours(out, &graphic.path, &graphic.holes, color);
        if graphic.width_nm.is_none() {
            return;
        }
    }
    let width_nm = board_graphic_nominal_nm(&graphic.layer_id, graphic.width_nm) as f32;
    let path = if graphic.primitive_kind == "polygon" {
        close_path(&graphic.path)
    } else {
        graphic.path.clone()
    };
    push_world_polyline_segments(out, &path, width_nm, color);
    let half = (width_nm * 0.5) as i64;
    for pt in &path {
        push_world_ellipse_nm(
            out,
            datum_gui_protocol::RectNm {
                min_x: pt.x - half,
                min_y: pt.y - half,
                max_x: pt.x + half,
                max_y: pt.y + half,
            },
            color,
            16,
        );
    }
}

fn push_board_text_geometry_world(
    out: &mut Vec<Quad>,
    text_geometry: &BoardTextGeometryPrimitive,
    glyph_mesh_assets: &BTreeMap<GlyphMeshHandlePrimitive, &GlyphMeshAssetPrimitive>,
    color: [f32; 3],
    _reference_projection: &Projection,
) {
    if let Some(transform) = text_geometry.world_transform_nm
        && !text_geometry.glyphs.is_empty() {
            push_board_text_mesh_world(out, text_geometry, glyph_mesh_assets, transform, color);
            return;
        }
    for fill in &text_geometry.fills {
        push_world_polygon_fill_contours(out, &fill.outer, &fill.holes, color);
    }
    for stroke in &text_geometry.strokes {
        push_world_polyline_segments(
            out,
            &[stroke.from, stroke.to],
            stroke.width_nm.max(1) as f32,
            color,
        );
    }
}

fn push_board_text_mesh_world(
    out: &mut Vec<Quad>,
    text_geometry: &BoardTextGeometryPrimitive,
    glyph_mesh_assets: &BTreeMap<GlyphMeshHandlePrimitive, &GlyphMeshAssetPrimitive>,
    transform: Affine2DFixedPrimitive,
    color: [f32; 3],
) {
    for glyph in &text_geometry.glyphs {
        let Some(asset) = glyph_mesh_assets.get(&glyph.glyph_handle) else {
            trace_text_mesh_skip(format!(
                "{} missing glyph mesh asset font={} glyph={} tolerance={} epoch={}",
                text_geometry.object_id,
                glyph.glyph_handle.font_id,
                glyph.glyph_handle.glyph_id,
                glyph.glyph_handle.tolerance_class,
                glyph.glyph_handle.epoch,
            ));
            continue;
        };
        for triangle in asset.indices.chunks_exact(3) {
            let Some(a) = asset.vertices.get(triangle[0] as usize) else {
                trace_text_mesh_skip(format!(
                    "{} glyph={} triangle references missing vertex {}",
                    text_geometry.object_id, glyph.glyph_handle.glyph_id, triangle[0],
                ));
                continue;
            };
            let Some(b) = asset.vertices.get(triangle[1] as usize) else {
                trace_text_mesh_skip(format!(
                    "{} glyph={} triangle references missing vertex {}",
                    text_geometry.object_id, glyph.glyph_handle.glyph_id, triangle[1],
                ));
                continue;
            };
            let Some(c) = asset.vertices.get(triangle[2] as usize) else {
                trace_text_mesh_skip(format!(
                    "{} glyph={} triangle references missing vertex {}",
                    text_geometry.object_id, glyph.glyph_handle.glyph_id, triangle[2],
                ));
                continue;
            };
            let a = transform_text_mesh_point(
                transform,
                glyph.origin_em_nm_x + a.x_em_nm,
                glyph.origin_em_nm_y + a.y_em_nm,
            );
            let b = transform_text_mesh_point(
                transform,
                glyph.origin_em_nm_x + b.x_em_nm,
                glyph.origin_em_nm_y + b.y_em_nm,
            );
            let c = transform_text_mesh_point(
                transform,
                glyph.origin_em_nm_x + c.x_em_nm,
                glyph.origin_em_nm_y + c.y_em_nm,
            );
            push_world_triangle(out, a, b, c, color);
        }
    }
}

fn trace_text_mesh_skip(message: String) {
    if std::env::var_os("DATUM_TRACE_GRAPHICS").is_some() {
        eprintln!("[datum-text-mesh] {message}");
    }
}

fn transform_text_mesh_point(
    transform: Affine2DFixedPrimitive,
    x_em_nm: i64,
    y_em_nm: i64,
) -> (f32, f32) {
    const EM_NM: i128 = 1_000_000;
    let x = (i128::from(transform.m11_ppm) * i128::from(x_em_nm)
        + i128::from(transform.m12_ppm) * i128::from(y_em_nm))
        / EM_NM
        + i128::from(transform.tx_nm);
    let y = (i128::from(transform.m21_ppm) * i128::from(x_em_nm)
        + i128::from(transform.m22_ppm) * i128::from(y_em_nm))
        / EM_NM
        + i128::from(transform.ty_nm);
    (x as f32, y as f32)
}

fn board_graphic_world_color(
    layer_id: &str,
    scene_layers: &[datum_gui_protocol::SceneLayer],
    dimmed: bool,
) -> [f32; 3] {
    let layer_name = scene_layers
        .iter()
        .find(|layer| layer.layer_id == layer_id)
        .map(|layer| layer.name.as_str())
        .unwrap_or("");
    // Schematic colour path (P2.2c). The schematic projection tags each element
    // with a per-net-role `Schematic.*` layer whose prototype token colour is
    // resolved here; board layers never match, so the board colour path below is
    // untouched. This deliberately lifts the "reuse board layers" posture that
    // forced the whole schematic to one silk off-white.
    if let Some(color) = schematic_layer_world_color(layer_name) {
        return dim_context_color(color, dimmed);
    }
    let app = resolve_layer_appearance_with_scene(Some(layer_id), scene_layers);
    let base_color = if layer_name.ends_with(".SilkS") {
        app.silkscreen
    } else {
        app.authored_track
    };
    dim_context_color(base_color, dimmed)
}

/// Maps a schematic net-role layer name to its prototype token colour
/// (`docs/gui/prototypes/schematic-editor.html`). Returns `None` for any
/// non-schematic (board) layer so the board colour path is left byte-identical.
fn schematic_layer_world_color(layer_name: &str) -> Option<[f32; 3]> {
    use crate::design_tokens::{chrome, schematic};
    Some(match layer_name {
        // Nets and junctions read as the green signal path.
        "Schematic.Wire" | "Schematic.Junction" => schematic::WIRE,
        // Symbol body outline, pin lines, and terminal dots are `--sym` grey.
        "Schematic.Symbol" => schematic::SYMBOL,
        // P2.2e typed-object geometry. Buses are the gold bundle path; power
        // flags/stacks are `--pwr` grey; global/hierarchical label tags `--info`.
        "Schematic.Bus" => schematic::BUS,
        "Schematic.Power" => schematic::POWER,
        "Schematic.GlobalLabel" => schematic::GLOBAL_LABEL,
        // RefDes is the brightest annotation (`--tx`).
        "Schematic.RefDes" => schematic::REFDES,
        // Value and pin numbers are the most muted annotation (`--tx3`).
        "Schematic.Value" | "Schematic.PinNumber" => schematic::VALUE,
        // Pin names, no-connect crosses, and generic labels/ports sit at `--tx2`.
        "Schematic.PinName" | "Schematic.NoConnect" | "Schematic.Annotation" => {
            chrome::TEXT_SECONDARY
        }
        _ => return None,
    })
}
