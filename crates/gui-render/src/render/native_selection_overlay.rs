#[path = "native_selection_overlay/diagnostics.rs"]
mod diagnostics;
#[path = "native_selection_overlay/lod.rs"]
mod lod;
#[path = "native_selection_overlay/mask.rs"]
pub(crate) mod mask;
use super::*;

pub(super) fn push_lightweight_selection_overlay(
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
    projection: &Projection,
) {
    let SelectionTarget::AuthoredObject(object_id) = &state.selection else {
        return;
    };
    if let Some(text) = scene
        .board_texts
        .iter()
        .find(|text| &text.object_id == object_id)
    {
        if !authored_visible(state) || !layer_visible(state, &text.layer_id) {
            return;
        }
        let rect = project_rect(board_text_hit_rect(text), projection);
        let halo = RectPx {
            x: rect.x - 4.0,
            y: rect.y - 4.0,
            width: rect.width + 8.0,
            height: rect.height + 8.0,
        };
        push_rect_border(out, halo, selected_silk_color(COMPONENT_SILK), 2.0);
        return;
    }
    if let Some(outline) = scene
        .outline
        .iter()
        .find(|outline| &outline.object_id == object_id)
    {
        if !authored_visible(state) || !layer_visible(state, &outline.layer_id) {
            return;
        }
        push_polyline_segments(
            out,
            &outline.path,
            projection,
            selected_mechanical_color(board_surface_color(BoardSurfaceRole::Edge)),
            3.0,
        );
        return;
    }
    if let Some(graphic) = scene
        .board_graphics
        .iter()
        .find(|graphic| &graphic.object_id == object_id)
    {
        if !authored_visible(state) || !layer_visible(state, &graphic.layer_id) {
            return;
        }
        push_polyline_segments(
            out,
            &graphic.path,
            projection,
            selected_mechanical_color(board_graphic_world_color(
                &graphic.layer_id,
                &scene.layers,
                false,
            )),
            3.0,
        );
    }
}

/// Selection lives exclusively in immediate pane-clipped geometry. Retained
/// authored colors and buffers do not depend on Native selection/focus/scopes.
#[allow(clippy::too_many_arguments)] // Three output sinks share one configured projection.
fn push_native_impl(
    selection_strokes: &mut Vec<selection_stroke::Stroke>,
    out: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
    viewport: RectPx,
    camera: CameraState,
    surface: datum_gui_protocol::PaneContent,
    mask_world: bool,
) {
    let SelectionTarget::Native(view) = &state.selection else {
        return;
    };
    if !authored_visible(state) || view.stale {
        return;
    }
    let Some(keys) = view.projection_keys(scene, surface) else {
        return;
    };
    let projection = Projection::new(
        inset_rect(viewport, 10.0, 10.0, 10.0, 10.0),
        &scene.bounds,
        camera,
    );
    let start = out.len();
    let before_text = text_runs.len();
    let layer_app = |layer: &str| resolve_layer_appearance_with_scene(Some(layer), &scene.layers);
    let glyph_assets = glyph_mesh_lookup::MeshIndex::new(&scene.glyph_mesh_assets, out);
    let lod = lod::Plan::capture(
        view,
        scene,
        surface,
        state,
        &projection,
        viewport,
        glyph_assets.as_ref(),
    );
    lod.push_cues(selection_strokes, viewport);
    for graphic in &scene.board_graphics {
        if mask_world {
            continue;
        }
        if !lod.detailed(view, scene, surface, &graphic.object_id)
            || !layer_visible(state, &graphic.layer_id)
        {
            continue;
        }
        let selected = keys.contains(&graphic.object_id);
        if selected {
            let material = board_graphic_world_color(&graphic.layer_id, &scene.layers, false);
            let path_subject = view
                .presentation_owner(scene, surface, &graphic.object_id)
                .is_some_and(|owner| {
                    matches!(
                        owner.class,
                        datum_gui_protocol::AuthoredSelectionClass::Wire
                            | datum_gui_protocol::AuthoredSelectionClass::BusSection
                            | datum_gui_protocol::AuthoredSelectionClass::NoConnect
                    )
                });
            if path_subject && graphic.primitive_kind != "polygon" {
                // Membership is typed and source-bound; the existing appearance
                // adapter alone supplies the authored stroke and physical floor.
                let (width, floor) =
                    semantic_graphic_kind(&graphic.object_id, &graphic.layer_id, graphic.width_nm)
                        .nominal_and_floor();
                selection_stroke::push(
                    selection_strokes,
                    &graphic.path,
                    width,
                    floor,
                    material,
                    &projection,
                    viewport,
                    state.ui.global_preferences.high_contrast_noncolor,
                );
            } else {
                push_polyline_segments(
                    out,
                    &graphic.path,
                    &projection,
                    selected_mechanical_color(material),
                    3.0,
                );
            }
        }
    }

    let selected_components: std::collections::BTreeSet<_> = scene
        .components
        .iter()
        .filter(|c| keys.contains(&c.object_id) && lod.detailed(view, scene, surface, &c.object_id))
        .map(|c| c.component_uuid.as_str())
        .collect();
    for graphic in &scene.component_graphics {
        if mask_world {
            continue;
        }
        if selected_components.contains(graphic.component_uuid.as_str())
            && graphic
                .layer_id
                .as_ref()
                .is_none_or(|l| layer_visible(state, l))
        {
            push_component_graphic_primitive(out, graphic, &projection, true, false, false);
        }
    }
    for text in &scene.component_texts {
        if mask_world && !text.cached_polygons.is_empty() {
            continue;
        }
        if selected_components.contains(text.component_uuid.as_str())
            && text
                .layer_id
                .as_ref()
                .is_none_or(|l| layer_visible(state, l))
        {
            push_component_text_world(
                out,
                text_runs,
                text,
                &scene.layers,
                &projection,
                viewport,
                true,
                false,
                false,
            );
        }
    }
    for track in &scene.tracks {
        if mask_world {
            continue;
        }
        if lod.detailed(view, scene, surface, &track.object_id)
            && keys.contains(&track.object_id)
            && layer_visible(state, &track.layer_id)
        {
            selection_stroke::push(
                selection_strokes,
                &track.path,
                track.width_nm,
                1.0,
                layer_app(&track.layer_id).authored_track,
                &projection,
                viewport,
                state.ui.global_preferences.high_contrast_noncolor,
            );
        }
    }
    for pad in &scene.pads {
        if mask_world {
            continue;
        }
        if lod.detailed(view, scene, surface, &pad.object_id)
            && keys.contains(&pad.object_id)
            && pad_visible_on_any_copper_layer(state, pad)
        {
            let path = projected_pad_outline(pad, &projection, 1.5);
            for segment in path.windows(2).chain(std::iter::once(
                &[
                    path.last().copied().unwrap_or_default(),
                    path.first().copied().unwrap_or_default(),
                ][..],
            )) {
                push_screen_line(
                    out,
                    segment[0],
                    segment[1],
                    selected_copper_color(layer_app(&pad.layer_id).authored_track),
                    3.0,
                );
            }
        }
    }
    for via in &scene.vias {
        if mask_world {
            continue;
        }
        if lod.detailed(view, scene, surface, &via.object_id)
            && keys.contains(&via.object_id)
            && (layer_visible(state, &via.start_layer_id)
                || layer_visible(state, &via.end_layer_id))
        {
            let center = projection.project_point(via.position);
            let radius = via.diameter_nm as f32 * projection.scale * 0.5 + 1.5;
            let path: Vec<_> = (0..=32)
                .map(|n| {
                    let a = n as f32 * std::f32::consts::TAU / 32.0;
                    (center.0 + a.cos() * radius, center.1 + a.sin() * radius)
                })
                .collect();
            for segment in path.windows(2) {
                push_screen_line(
                    out,
                    segment[0],
                    segment[1],
                    selected_copper_color(layer_app(&via.start_layer_id).authored_track),
                    3.0,
                );
            }
        }
    }
    for zone in &scene.zones {
        if mask_world {
            continue;
        }
        if lod.detailed(view, scene, surface, &zone.object_id)
            && keys.contains(&zone.object_id)
            && layer_visible(state, &zone.layer_id)
        {
            if !matches!(view.subject, datum_gui_protocol::SelectionSubject::Run(_)) {
                let mut boundary = zone.polygon.clone();
                if let Some(first) = boundary.first().copied() {
                    boundary.push(first);
                }
                push_polyline_segments(
                    out,
                    &boundary,
                    &projection,
                    selected_copper_color(layer_app(&zone.layer_id).authored_track),
                    3.0,
                );
            }
            for polygon in view.zone_regions.get(&zone.object_id).into_iter().flatten() {
                let mut closed = polygon.clone();
                if let Some(first) = closed.first().copied() {
                    closed.push(first);
                }
                push_polyline_segments(
                    out,
                    &closed,
                    &projection,
                    selected_copper_color(layer_app(&zone.layer_id).authored_track),
                    3.0,
                );
            }
        }
    }
    for outline in &scene.outline {
        if mask_world {
            continue;
        }
        if lod.detailed(view, scene, surface, &outline.object_id)
            && keys.contains(&outline.object_id)
            && layer_visible(state, &outline.layer_id)
        {
            push_polyline_segments(
                out,
                &outline.path,
                &projection,
                selected_mechanical_color(board_surface_color(BoardSurfaceRole::Edge)),
                3.0,
            );
        }
    }
    for geometry in &scene.board_text_geometries {
        if mask_world {
            continue;
        }
        if lod.detailed(view, scene, surface, &geometry.object_id)
            && keys.contains(&geometry.object_id)
            && layer_visible(state, &geometry.layer_id)
        {
            let Some(glyph_assets) = glyph_assets.as_ref() else {
                continue;
            };
            let mut glyphs = Vec::new();
            board_text_geometry::push_board_text_geometry_world(
                &mut glyphs,
                geometry,
                glyph_assets,
                selected_mechanical_color(TEXT_PRIMARY),
                &projection,
            );
            for mut quad in glyphs {
                for point in &mut quad.points {
                    *point = projection.project_point(PointNm {
                        x: point.0.round() as i64,
                        y: point.1.round() as i64,
                    });
                }
                out.push(quad);
            }
        }
    }
    crate::hit_clipping::clip_content(
        out,
        text_runs,
        &mut Vec::new(),
        start,
        before_text,
        0,
        viewport,
    );
}
fn push_screen_line(
    out: &mut Vec<Quad>,
    a: (f32, f32),
    b: (f32, f32),
    color: [f32; 3],
    width: f32,
) {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let length = dx.hypot(dy).max(1.0);
    let nx = -dy / length * width * 0.5;
    let ny = dx / length * width * 0.5;
    out.push(Quad {
        points: [
            (a.0 + nx, a.1 + ny),
            (b.0 + nx, b.1 + ny),
            (b.0 - nx, b.1 - ny),
            (a.0 - nx, a.1 - ny),
        ],
        color,
    });
}

pub(super) fn push_region(
    out: &mut Vec<Quad>,
    text_runs: &mut Vec<TextRun>,
    state: &ReviewWorkspaceState,
    pane: datum_gui_protocol::PaneId,
    scene: &BoardReviewSceneV1,
    viewport: RectPx,
    camera: CameraState,
) {
    let Some(feedback) = state
        .ui
        .selection_region
        .as_ref()
        .filter(|f| f.pane == pane)
    else {
        return;
    };
    let projection = Projection::new(
        inset_rect(viewport, 10.0, 10.0, 10.0, 10.0),
        &scene.bounds,
        camera,
    );
    let start = out.len();
    let before_text = text_runs.len();
    if let Some(examined) = feedback.evaluating {
        draw_text(
            &format!("Evaluating selection — {examined} candidates examined; Escape cancels"),
            viewport.x + 12.0,
            viewport.y + 12.0,
            12.0,
            TEXT_PRIMARY,
            TextFace::Ui,
            text_runs,
        );
    }
    let phase = if state.ui.global_preferences.reduced_motion {
        0.0
    } else {
        feedback.phase as f32
    };
    let dash = if feedback.subtractive { 3.0 } else { 6.0 }; // Both shape and cadence distinguish subtraction without color.
    for i in 0..feedback.path.len() {
        let a = projection.project_point(feedback.path[i]);
        let b = projection.project_point(feedback.path[(i + 1) % feedback.path.len()]);
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let length = dx.hypot(dy);
        if length == 0.0 {
            continue;
        }
        let mut at = -phase;
        while at < length {
            let lo = at.max(0.0);
            let hi = (at + dash).min(length);
            if hi > lo {
                let p = (a.0 + dx * lo / length, a.1 + dy * lo / length);
                let q = (a.0 + dx * hi / length, a.1 + dy * hi / length);
                push_screen_line(
                    out,
                    p,
                    q,
                    if ((at + phase) / (dash * 2.0)) as i64 % 2 == 0 {
                        TEXT_PRIMARY
                    } else {
                        [0.0, 0.0, 0.0]
                    },
                    1.0,
                );
            }
            at += dash * 2.0;
        }
    }
    crate::hit_clipping::clip_content(
        out,
        text_runs,
        &mut Vec::new(),
        start,
        before_text,
        0,
        viewport,
    );
}

impl PreparedScene {
    /// Append once after the shared session applies every pane's warm camera.
    pub(super) fn append_native_selection(
        &mut self,
        state: &ReviewWorkspaceState,
    ) -> anyhow::Result<()> {
        use crate::resource_consumers::{Consumer, Stream};
        self.selection_overlay_split = self.viewport_overlay_vertices.len();
        for pass in &self.surface_passes {
            let (scene, consumer) = match pass.surface {
                SceneSurface::Board => (&state.scene, Consumer::Board),
                SceneSurface::Schematic => {
                    let Some(scene) = state.schematic_scene.as_ref() else {
                        continue;
                    };
                    (scene, Consumer::Schematic)
                }
            };
            let surface = match pass.surface {
                SceneSurface::Board => datum_gui_protocol::PaneContent::Board,
                SceneSurface::Schematic => datum_gui_protocol::PaneContent::Schematic,
            };
            let mut quads = Vec::new();
            if authored_visible(state)
                && let SelectionTarget::Native(view) = &state.selection
                && let Some(mut request) = mask::Request::capture(view, scene, surface)?
            {
                let projection = Projection::new(
                    inset_rect(pass.scene_viewport, 10.0, 10.0, 10.0, 10.0),
                    &scene.bounds,
                    pass.camera,
                );
                let glyphs =
                    glyph_mesh_lookup::MeshIndex::new(&scene.glyph_mesh_assets, &mut quads);
                let lod = lod::Plan::capture(
                    view,
                    scene,
                    surface,
                    state,
                    &projection,
                    pass.scene_viewport,
                    glyphs.as_ref(),
                );
                request.configure(
                    &lod,
                    view,
                    scene,
                    surface,
                    state.ui.global_preferences.high_contrast_noncolor,
                );
                self.selection_masks
                    .push((pass.pane_id, pass.surface, request));
            }
            let before_text = self.text_runs.len();
            let before_strokes = self.selection_strokes.len();
            push_native_impl(
                &mut self.selection_strokes,
                &mut quads,
                &mut self.text_runs,
                scene,
                state,
                pass.scene_viewport,
                pass.camera,
                surface,
                true,
            );
            if self
                .selection_masks
                .iter()
                .any(|(pane, _, _)| *pane == pass.pane_id)
            {
                self.consumers.include(Stream::Selection, consumer.into());
            }
            self.consumers.note(
                Stream::Selection,
                consumer,
                before_strokes,
                self.selection_strokes.len(),
            );
            push_region(
                &mut quads,
                &mut self.text_runs,
                state,
                pass.pane_id,
                scene,
                pass.scene_viewport,
                pass.camera,
            );
            diagnostics::push(
                &mut quads,
                &mut self.hit_regions,
                scene,
                state,
                pass.scene_viewport,
                pass.camera,
                surface,
            );
            TextRun::annotate(
                &mut self.text_runs[before_text..],
                TextOrigin::Viewport(pass.pane_id),
            );
            self.consumers
                .note(Stream::Text, consumer, before_text, self.text_runs.len());
            self.consumers
                .note(Stream::ViewportOverlay, consumer, 0, quads.len());
            self.viewport_overlay_vertices
                .extend(quads_to_vertices(&quads));
        }
        Ok(())
    }
}

pub(super) fn selection_beats_hover(
    state: &ReviewWorkspaceState,
    hover: &datum_gui_protocol::HoverTarget,
) -> bool {
    let SelectionTarget::Native(view) = &state.selection else {
        return false;
    };
    let scene = match hover.surface {
        datum_gui_protocol::PaneContent::Board => Some(&state.scene),
        datum_gui_protocol::PaneContent::Schematic => state.schematic_scene.as_ref(),
        _ => None,
    };
    scene.is_some_and(|s| view.selected_key(s, hover.surface, &hover.object_id))
}

#[cfg(test)]
#[path = "native_selection_overlay/tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "native_selection_overlay/lod_tests.rs"]
mod lod_tests;
