#[path = "board_text_geometry.rs"]
mod board_text_geometry;
use board_text_geometry::push_board_text_geometry_world;

#[path = "overlay_channels.rs"]
mod overlay_channels;
use overlay_channels::{push_overlay_channels, push_review_channels, width_to_px};

#[allow(dead_code)]
fn push_polygon_fill(
    out: &mut impl Output<Quad>,
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

#[cfg(test)]
#[allow(dead_code)]
fn push_component_primitive_world(
    out: &mut impl Output<Quad>,
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
    out: &mut impl Output<Quad>,
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

#[allow(clippy::too_many_arguments)]
fn push_component_graphic_primitive_world(
    out: &mut impl Output<Quad>,
    strokes: &mut impl Output<WorldStrokeInstance>,
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
        let Some(path) = world_primitives::closed_world_path(out, &graphic.path) else {
            return;
        };
        path
    } else {
        std::borrow::Cow::Borrowed(graphic.path.as_slice())
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
    if graphic.render_role == "component_mechanical" {
        push_world_polyline_segments(out, &path, w, color);
    } else {
        push_world_stroke_path(
            strokes,
            &path,
            color,
            graphic.width_nm.unwrap_or(SILK_LINE_NM),
            1.0,
        );
        return;
    }
    // Round-cap each vertex so that separate fp_line segments sharing an
    // endpoint don't leave diagonal gaps at 90-degree corners. Each cap is
    // a small filled circle matching the stroke width.
    let half = (w * 0.5) as i64;
    for pt in path.iter() {
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
