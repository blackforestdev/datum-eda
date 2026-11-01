//! Unit inspection and shared native Symbol body / pin geometry.
use crate::*;
use datum_gui_protocol::library_browser::symbol_geometry::symbol_pin_stub as endpoints;
use datum_gui_protocol::{LibraryJsonValue as Value, ReviewWorkspaceState};

pub(super) fn render(
    state: &ReviewWorkspaceState,
    kind: &str,
    raw: &Value,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
) {
    if kind == "units" {
        let mut y = rect.y;
        for pin in raw
            .get("pins")
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|p| p.values())
        {
            let name = pin
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("Unnamed pin");
            let direction = pin
                .get("electrical_type")
                .or_else(|| pin.get("direction"))
                .and_then(Value::as_str)
                .unwrap_or("Unspecified");
            draw_text_clipped(
                &format!("{name} · {direction}"),
                rect.x + 8.,
                y,
                12.,
                TEXT_PRIMARY,
                TextFace::Ui,
                rect,
                text,
            );
            y += 20.;
        }
        return;
    }
    let unit_id = raw.get("unit").and_then(Value::as_str);
    let unit = state
        .library
        .entries
        .iter()
        .find(|e| e.kind == "units" && unit_id == Some(e.id.to_string().as_str()))
        .map(|e| &e.raw);
    symbol(raw, unit, rect, quads, text);
}

pub(crate) fn symbol(
    raw: &Value,
    unit: Option<&Value>,
    rect: RectPx,
    quads: &mut Vec<Quad>,
    text: &mut Vec<TextRun>,
) {
    let scene =
        match datum_gui_protocol::library_browser::symbol_geometry::preview::project(raw, unit) {
            Ok(scene) => scene,
            Err(reason) => {
                draw_text_clipped(
                    &format!("Preview unavailable · {reason}"),
                    rect.x + 8.,
                    rect.y + 6.,
                    11.,
                    TEXT_MUTED,
                    TextFace::Ui,
                    rect,
                    text,
                );
                return;
            }
        };
    let starts = (quads.len(), text.len());
    let mut camera = CameraState::fit_to_bounds(&scene.bounds);
    camera.zoom = 0.7;
    let projection = Projection::new(rect, &scene.bounds, camera);
    for graphic in &scene.board_graphics {
        push_polyline_segments(
            quads,
            &graphic.path,
            &projection,
            TEXT_PRIMARY,
            projection
                .world_length_to_px(graphic.width_nm.unwrap_or(1))
                .max(1.),
        );
    }
    // The same glyph geometry preserves native text rotation and shape.
    let mut world = Vec::<Quad>::new();
    if let Some(meshes) =
        crate::glyph_mesh_lookup::MeshIndex::new(&scene.glyph_mesh_assets, &mut world)
    {
        for geometry in &scene.board_text_geometries {
            push_board_text_geometry_world(
                &mut world,
                geometry,
                &meshes,
                TEXT_PRIMARY,
                &projection,
            );
        }
        for mut quad in world {
            for point in &mut quad.points {
                point.0 =
                    projection.offset_x + (point.0 - scene.bounds.min_x as f32) * projection.scale;
                point.1 =
                    projection.offset_y + (point.1 - scene.bounds.min_y as f32) * projection.scale;
            }
            quads.push(quad);
        }
    }
    let pins = unit.and_then(|u| u.get("pins")).and_then(Value::as_object);
    for anchor in raw
        .get("pin_anchors")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some((from, to)) = endpoints(anchor) else {
            continue;
        };
        let name = anchor
            .get("pin")
            .and_then(Value::as_str)
            .and_then(|id| pins.and_then(|pins| pins.get(id)))
            .and_then(|pin| pin.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("Unresolved pin");
        let number = anchor.get("number").and_then(Value::as_str).unwrap_or("");
        let (x, y) = projection.project_point(to);
        draw_text_clipped(
            &format!("{number} {name}"),
            x + 4.,
            y - 6.,
            10.,
            TEXT_PRIMARY,
            TextFace::Ui,
            rect,
            text,
        );
        let (x, y) = projection.project_point(from);
        quads.push(Quad::from_rect(
            RectPx {
                x: x - 2.,
                y: y - 2.,
                width: 4.,
                height: 4.,
            },
            TEXT_ACCENT,
        ));
    }
    crate::hit_clipping::clip_content(quads, text, &mut Vec::new(), starts.0, starts.1, 0, rect);
}
