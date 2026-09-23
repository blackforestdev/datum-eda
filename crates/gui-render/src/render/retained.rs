#[path = "retained_board_graphics.rs"]
mod retained_board_graphics;
use retained_board_graphics::{
    push_retained_board_graphic_batches, push_retained_board_text_geometry_batches,
    trace_retained_stage,
};

/// Retained authored-board geometry pass.
///
/// Contract (`M7-REN-006`, `docs/gui/M7_RENDER_LAYER_DISCIPLINE_MEMO.md`):
/// layer/material semantics are primary — layer ownership decides visibility,
/// the layer's material decides base appearance, and stage order follows the
/// declared `RenderStage` policy. Primitive class only refines stroke/fill.
///
/// Bounded exceptions (explicit, product-justified; do not grow this list
/// without a memo note):
/// - through-hole pads: drawn in a dedicated post-layer pass because their
///   copper spans multiple layers and must follow the visible-copper rule;
/// - vias: a distinct geometry family (annulus + hole), though their color
///   inherits the visible copper layer's material;
/// - board outline / `board_graphics` Edge overlay: the board-boundary view
///   is a product-level overlay on top of the authored stage walk;
/// - selection/hover/review emphasis: interaction-state styling deliberately
///   overrides material color for the owned object only;
/// - unknown-layer fallback appearance: deliberately divergent so unresolved
///   layer identity stays visible (see `resolve_layer_appearance_with_scene`).
fn push_retained_scene_geometry(
    out: &mut impl Output<Quad>,
    strokes: &mut impl Output<WorldStrokeInstance>,
    draw_commands: &mut impl Output<RetainedDrawCommand>,
    scene: &BoardReviewSceneV1,
    reference_projection: &Projection,
    state: &ReviewWorkspaceState,
) {
    let active_move_component_uuid: Option<String> = None;
    let sl = &scene.layers;
    let preview_ids = proposal_preview_ids(state);
    let Some(mut preview_affected_ids) = out.scratch(preview_ids.clone().count()) else {
        return;
    };
    preview_affected_ids.extend(preview_ids);
    let layer_app = |id: &str| resolve_layer_appearance_with_scene(Some(id), sl);
    // Render copper in physical stack order first; later stages (paste/mask/silk/mechanical/edge)
    // are handled by explicit render-stage grouping below.
    let copper_started = std::time::Instant::now();
    let copper_before = out.len();
    for pass_priority in [0u32, 1, 2] {
        for zone in &scene.zones {
            if copper_pass_priority_for_layer(&zone.layer_id, sl) != Some(pass_priority) {
                continue;
            }
            if !authored_visible(state) || !layer_visible(state, &zone.layer_id) {
                continue;
            }
            let related = zone_matches_active_action(zone, state)
                || source_object_matches_preview(
                    &preview_affected_ids,
                    &zone.object_id,
                    &zone.source_object_uuid,
                );
            let dimmed = dim_unrelated_active(state) && !related;
            if zone.polygon.len() >= 4 {
                let quad_start = out.len();
                let stroke_start = strokes.len();
                let za = layer_app(&zone.layer_id);
                let (fill_color, outline_color) = (za.zone_fill, za.zone_outline);
                push_world_polygon_fill(out, &zone.polygon, dim_authored_color(fill_color, dimmed));
                let zone_weight = AuthoredStrokePrimitive::CopperZoneOutline;
                gpu_strokes::push_world_stroke_loop(
                    strokes,
                    &zone.polygon,
                    dim_authored_color(outline_color, dimmed),
                    zone_weight.nominal_nm(),
                    1.0,
                );
                scene_retained_access::finish_retained_draw_commands(
                    draw_commands,
                    Some(zone.layer_id.as_str()),
                    quad_start,
                    out.len(),
                    stroke_start,
                    strokes.len(),
                );
            }
        }
        for track in &scene.tracks {
            if copper_pass_priority_for_layer(&track.layer_id, sl) != Some(pass_priority) {
                continue;
            }
            if !authored_visible(state) || !layer_visible(state, &track.layer_id) {
                continue;
            }
            let related = track_matches_active_action(track, state)
                || source_object_matches_preview(
                    &preview_affected_ids,
                    &track.object_id,
                    &track.source_object_uuid,
                );
            let selected = matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &track.object_id);
            let color = if selected {
                selected_copper_color(layer_app(&track.layer_id).authored_track)
            } else if related {
                AUTHOR_RELATED
            } else {
                dim_authored_color(
                    layer_app(&track.layer_id).authored_track,
                    dim_unrelated_active(state) && !selected && !related,
                )
            };
            let track_width_nm = AuthoredStrokePrimitive::CopperTrace {
                width_nm: track.width_nm,
            }
            .nominal_nm() as f32;
            let start = strokes.len();
            push_world_stroke_path(strokes, &track.path, color, track_width_nm as i64, 1.0);
            scene_retained_access::finish_retained_draw_commands(
                draw_commands,
                Some(track.layer_id.as_str()),
                out.len(),
                out.len(),
                start,
                strokes.len(),
            );
        }
        for pad in &scene.pads {
            if !authored_visible(state) {
                continue;
            }
            if active_move_component_uuid.as_deref() == Some(pad.component_uuid.as_str()) {
                continue;
            }
            let active = matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &pad.object_id)
                || component_is_selection_active(&pad.component_uuid, scene, state);
            let related = pad_matches_active_action(pad, state)
                || component_is_selection_related(&pad.component_uuid, scene, state)
                || source_object_matches_preview(
                    &preview_affected_ids,
                    &pad.object_id,
                    &pad.source_object_uuid,
                )
                || component_matches_preview(&pad.component_uuid, scene, &preview_affected_ids);
            let hovered = is_hovered(state, &pad.object_id);
            let dimmed = dim_unrelated_active(state) && !active && !related && !hovered;
            for render_layer in pad_copper_layer_ids(pad) {
                if copper_pass_priority_for_layer(render_layer, sl) != Some(pass_priority) {
                    continue;
                }
                if !layer_visible(state, render_layer) {
                    continue;
                }
                let quad_start = out.len();
                push_pad_primitive_world(
                    out,
                    pad,
                    pad.bounds,
                    if active {
                        selected_copper_color(layer_app(render_layer).pad_copper)
                    } else if hovered || related {
                        layer_app(render_layer).pad_related
                    } else {
                        dim_authored_color(layer_app(render_layer).pad_copper, dimmed)
                    },
                    pad.drill_nm,
                    dimmed,
                    reference_projection,
                );
                scene_retained_access::finish_retained_draw_commands(
                    draw_commands,
                    Some(render_layer),
                    quad_start,
                    out.len(),
                    strokes.len(),
                    strokes.len(),
                );
            }
        }
        for via in &scene.vias {
            if !authored_visible(state)
                || !via_visible(state, &via.start_layer_id, &via.end_layer_id)
            {
                continue;
            }
            let display_layer = if layer_visible(state, &via.start_layer_id) {
                via.start_layer_id.as_str()
            } else if layer_visible(state, &via.end_layer_id) {
                via.end_layer_id.as_str()
            } else {
                continue;
            };
            if copper_pass_priority_for_layer(display_layer, sl) != Some(pass_priority) {
                continue;
            }
            let selected = matches!(
                state.selection,
                SelectionTarget::AuthoredObject(ref id) if id == &via.object_id
            );
            let related = via_matches_active_action(via, state)
                || source_object_matches_preview(
                    &preview_affected_ids,
                    &via.object_id,
                    &via.source_object_uuid,
                );
            let dimmed = dim_unrelated_active(state) && !selected && !related;
            let quad_start = out.len();
            push_via_primitive_world(
                out,
                via,
                layer_app(display_layer).pad_copper,
                selected,
                dimmed,
                reference_projection,
            );
            scene_retained_access::finish_retained_draw_commands(
                draw_commands,
                Some(display_layer),
                quad_start,
                out.len(),
                strokes.len(),
                strokes.len(),
            );
        }
    }
    trace_retained_stage("copper", copper_started, copper_before, out.len());
    let mechanical_graphics = scene.component_graphics.iter().filter(|graphic| {
        graphic.render_role == "component_mechanical"
            && active_move_component_uuid.as_deref() != Some(graphic.component_uuid.as_str())
    });
    let process = scene
        .layers
        .iter()
        .enumerate()
        .filter_map(|(index, layer)| {
            let kind = match render_stage_for_layer(&layer.layer_id, sl) {
                RenderStage::BottomPaste | RenderStage::TopPaste => PadProcessLayerKind::Paste,
                RenderStage::BottomMask | RenderStage::TopMask => PadProcessLayerKind::Mask,
                _ => return None,
            };
            Some((index, layer.layer_id.as_str(), kind))
        });
    let Some(mut process_layers) = out.scratch(process.clone().count()) else {
        return;
    };
    process_layers.extend(process);
    process_layers
        .sort_unstable_by_key(|(index, layer, _)| (scene_layer_stack_priority(layer, sl), *index));
    let silkscreen_graphics = scene.component_graphics.iter().filter(|graphic| {
        graphic.render_role == "component_silkscreen"
            && active_move_component_uuid.as_deref() != Some(graphic.component_uuid.as_str())
    });
    let post_started = std::time::Instant::now();
    let post_before = out.len();
    let mut process_pad_elapsed = std::time::Duration::ZERO;
    let mut mechanical_elapsed = std::time::Duration::ZERO;
    let mut silkscreen_elapsed = std::time::Duration::ZERO;
    let board_graphics_elapsed = std::time::Duration::ZERO;
    let mut process_pad_quads = 0usize;
    let mut mechanical_quads = 0usize;
    let mut silkscreen_quads = 0usize;
    let board_graphics_quads = 0usize;
    for stage in POST_COPPER_STAGES {
        let process_before = out.len();
        let process_started = std::time::Instant::now();
        for (_, layer_id, kind) in process_layers
            .iter()
            .filter(|(_, layer_id, _)| render_stage_for_layer(layer_id, sl) == stage)
        {
            if !authored_visible(state) || !layer_visible(state, layer_id) {
                continue;
            }
            for pad in &scene.pads {
                let active = matches!(state.selection, SelectionTarget::AuthoredObject(ref id) if id == &pad.object_id)
                    || component_is_selection_active(&pad.component_uuid, scene, state);
                let related = pad_matches_active_action(pad, state)
                    || source_object_matches_preview(
                        &preview_affected_ids,
                        &pad.object_id,
                        &pad.source_object_uuid,
                    )
                    || component_matches_preview(&pad.component_uuid, scene, &preview_affected_ids);
                let hovered = is_hovered(state, &pad.object_id);
                let dimmed = dim_unrelated_active(state) && !active && !related && !hovered;
                let membership = match kind {
                    PadProcessLayerKind::Mask => &pad.mask_layer_ids,
                    PadProcessLayerKind::Paste => &pad.paste_layer_ids,
                };
                if !membership.iter().any(|member| member == layer_id) {
                    continue;
                }
                let bounds = process_pad_bounds(pad, *kind);
                let quad_start = out.len();
                push_pad_primitive_world(
                    out,
                    pad,
                    bounds,
                    if active {
                        selected_silk_color(mask_or_paste_layer_color(layer_id, sl))
                    } else {
                        dim_process_color(mask_or_paste_layer_color(layer_id, sl), dimmed)
                    },
                    None,
                    false,
                    reference_projection,
                );
                scene_retained_access::finish_retained_draw_commands(
                    draw_commands,
                    Some(layer_id),
                    quad_start,
                    out.len(),
                    strokes.len(),
                    strokes.len(),
                );
            }
        }
        process_pad_elapsed += process_started.elapsed();
        process_pad_quads += out.len().saturating_sub(process_before);
        let mechanical_before = out.len();
        let mechanical_started = std::time::Instant::now();
        for graphic in mechanical_graphics.clone().filter(|graphic| {
            graphic_render_stage(graphic.layer_id.as_deref(), sl, RenderStage::Mechanical) == stage
        }) {
            if !authored_visible(state) {
                continue;
            }
            if let Some(lid) = graphic.layer_id.as_deref()
                && !layer_visible(state, lid)
            {
                continue;
            }
            let selected_body_graphic_id =
                selected_component_body_graphic_id(scene, &graphic.component_uuid);
            if selected_body_graphic_id.is_some_and(|id| id == graphic.graphic_id) {
                continue;
            }
            let related = component_graphic_matches_active_action(graphic, scene, state)
                || component_is_selection_related(&graphic.component_uuid, scene, state)
                || component_matches_preview(&graphic.component_uuid, scene, &preview_affected_ids);
            let selected_component =
                matches!(
                    state.selection,
                    SelectionTarget::AuthoredObject(ref id)
                        if id == &format!("component:{}", graphic.component_uuid)
                ) || component_is_selection_active(&graphic.component_uuid, scene, state);
            let selected = false;
            let quad_start = out.len();
            let stroke_start = strokes.len();
            push_component_graphic_primitive_world(
                out,
                strokes,
                graphic,
                sl,
                selected,
                related || selected_component,
                dim_unrelated_active(state) && !selected_component && !related,
                reference_projection,
            );
            scene_retained_access::finish_retained_draw_commands(
                draw_commands,
                graphic.layer_id.as_deref(),
                quad_start,
                out.len(),
                stroke_start,
                strokes.len(),
            );
        }
        mechanical_elapsed += mechanical_started.elapsed();
        mechanical_quads += out.len().saturating_sub(mechanical_before);
        let silkscreen_before = out.len();
        let silkscreen_started = std::time::Instant::now();
        for graphic in silkscreen_graphics.clone().filter(|graphic| {
            graphic_render_stage(graphic.layer_id.as_deref(), sl, RenderStage::TopSilk) == stage
        }) {
            if !authored_visible(state) {
                continue;
            }
            if let Some(lid) = graphic.layer_id.as_deref()
                && !layer_visible(state, lid)
            {
                continue;
            }
            let related = component_graphic_matches_active_action(graphic, scene, state)
                || component_is_selection_related(&graphic.component_uuid, scene, state)
                || component_matches_preview(&graphic.component_uuid, scene, &preview_affected_ids);
            let selected =
                matches!(
                    state.selection,
                    SelectionTarget::AuthoredObject(ref id)
                        if id == &format!("component:{}", graphic.component_uuid)
                ) || component_is_selection_active(&graphic.component_uuid, scene, state);
            let quad_start = out.len();
            let stroke_start = strokes.len();
            push_component_graphic_primitive_world(
                out,
                strokes,
                graphic,
                sl,
                selected,
                related,
                dim_unrelated_active(state) && !selected && !related,
                reference_projection,
            );
            scene_retained_access::finish_retained_draw_commands(
                draw_commands,
                graphic.layer_id.as_deref(),
                quad_start,
                out.len(),
                stroke_start,
                strokes.len(),
            );
        }
        silkscreen_elapsed += silkscreen_started.elapsed();
        silkscreen_quads += out.len().saturating_sub(silkscreen_before);
    }
    trace_retained_stage("post-copper", post_started, post_before, out.len());
    trace_render_timing(format!(
        "retained detail process_pads={}ms/{}q mechanical={}ms/{}q component_silk={}ms/{}q board_graphics={}ms/{}q",
        process_pad_elapsed.as_millis(),
        process_pad_quads,
        mechanical_elapsed.as_millis(),
        mechanical_quads,
        silkscreen_elapsed.as_millis(),
        silkscreen_quads,
        board_graphics_elapsed.as_millis(),
        board_graphics_quads
    ));
    let active_started = std::time::Instant::now();
    let active_before = out.len();
    if let Some(active_component_uuid) = active_move_component_uuid.as_deref()
        && let Some(component) = scene
            .components
            .iter()
            .find(|component| component.component_uuid == active_component_uuid)
    {
        let selected = true;
        let related = component_overlaps_active_action(component, state)
            || component_is_selection_related(&component.component_uuid, scene, state);
        let dimmed = false;
        let selected_body_graphic_id =
            selected_component_body_graphic_id(scene, &component.component_uuid);
        for graphic in scene
            .component_graphics
            .iter()
            .filter(|graphic| graphic.component_uuid == component.component_uuid)
            .filter(|graphic| graphic.render_role == "component_mechanical")
        {
            if selected_body_graphic_id.is_some_and(|id| id == graphic.graphic_id) {
                continue;
            }
            push_component_graphic_primitive_world(
                out,
                strokes,
                graphic,
                sl,
                false,
                related,
                dimmed,
                reference_projection,
            );
        }
        for pad in scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == component.component_uuid)
        {
            for render_layer in pad_copper_layer_ids(pad) {
                if !layer_visible(state, render_layer) {
                    continue;
                }
                push_pad_primitive_world(
                    out,
                    pad,
                    pad.bounds,
                    selected_copper_color(layer_app(render_layer).pad_copper),
                    pad.drill_nm,
                    dimmed,
                    reference_projection,
                );
            }
        }
        for graphic in scene
            .component_graphics
            .iter()
            .filter(|graphic| graphic.component_uuid == component.component_uuid)
            .filter(|graphic| graphic.render_role == "component_silkscreen")
        {
            push_component_graphic_primitive_world(
                out,
                strokes,
                graphic,
                sl,
                selected,
                related,
                dimmed,
                reference_projection,
            );
        }
    }
    trace_retained_stage("active-component", active_started, active_before, out.len());
    let unrouted_started = std::time::Instant::now();
    let unrouted_before = out.len();
    if unrouted_visible(state) {
        // Local batch buffer whose tuple shape is self-documenting inline.
        #[allow(clippy::type_complexity)]
        let Some(mut unrouted_batches): Option<
            Vec<(&[PointNm], [f32; 3], [f32; 3], f32, f32, f32, f32)>,
        > = out.scratch(scene.unrouted_primitives.len()) else {
            return;
        };
        for unrouted in &scene.unrouted_primitives {
            let related = unrouted_matches_active_action(unrouted, state);
            let dimmed = dim_unrelated_active(state) && !related;
            let net_color = unrouted_base_color(scene, unrouted);
            let base_color = if related {
                mix_color(net_color, UNROUTED_FOCUS, 0.35)
            } else {
                dim_context_color(net_color, dimmed)
            };
            let color = mix_color(base_color, BOARD_INNER_FIELD, 0.18);
            let under_color =
                mix_color(BOARD_OUTER_FIELD, color, if related { 0.28 } else { 0.22 });
            let width_px = if related { 1.55 } else { 1.2 };
            let width_nm = world_stroke_nm(width_px, reference_projection).max(1.0);
            let under_width_nm = world_stroke_nm(
                width_px + if related { 0.9 } else { 0.7 },
                reference_projection,
            )
            .max(width_nm + 1.0);
            let endpoint_radius_nm =
                world_stroke_nm(if related { 1.15 } else { 0.95 }, reference_projection).max(1.0);
            let endpoint_under_radius_nm = (endpoint_radius_nm
                + ((under_width_nm - width_nm) * 0.5))
                .max(endpoint_radius_nm + 0.5);
            unrouted_batches.push((
                unrouted.path.as_slice(),
                color,
                under_color,
                width_nm,
                under_width_nm,
                endpoint_radius_nm,
                endpoint_under_radius_nm,
            ));
        }
        for (
            path,
            _color,
            under_color,
            _width_nm,
            under_width_nm,
            _endpoint_radius_nm,
            _endpoint_under_radius_nm,
        ) in &unrouted_batches
        {
            push_world_polyline_segments_capped(out, path, *under_width_nm, *under_color);
        }
        for (
            path,
            _color,
            under_color,
            _width_nm,
            _under_width_nm,
            _endpoint_radius_nm,
            endpoint_under_radius_nm,
        ) in &unrouted_batches
        {
            for point in path.first().into_iter().chain(path.last()) {
                let under_r = endpoint_under_radius_nm.round() as i64;
                push_world_ellipse_nm(
                    out,
                    datum_gui_protocol::RectNm {
                        min_x: point.x - under_r,
                        min_y: point.y - under_r,
                        max_x: point.x + under_r,
                        max_y: point.y + under_r,
                    },
                    *under_color,
                    24,
                );
            }
        }
        for (
            path,
            color,
            _under_color,
            width_nm,
            _under_width_nm,
            _endpoint_radius_nm,
            _endpoint_under_radius_nm,
        ) in &unrouted_batches
        {
            push_world_polyline_segments_capped(out, path, *width_nm, *color);
        }
        for (
            path,
            color,
            _under_color,
            _width_nm,
            _under_width_nm,
            endpoint_radius_nm,
            _endpoint_under_radius_nm,
        ) in &unrouted_batches
        {
            for point in path.first().into_iter().chain(path.last()) {
                let r = endpoint_radius_nm.round() as i64;
                push_world_ellipse_nm(
                    out,
                    datum_gui_protocol::RectNm {
                        min_x: point.x - r,
                        min_y: point.y - r,
                        max_x: point.x + r,
                        max_y: point.y + r,
                    },
                    *color,
                    24,
                );
            }
        }
    }
    scene_retained_access::finish_retained_draw_commands(
        draw_commands,
        None,
        unrouted_before,
        out.len(),
        strokes.len(),
        strokes.len(),
    );
    trace_retained_stage("unrouted", unrouted_started, unrouted_before, out.len());
    let outline_started = std::time::Instant::now();
    let outline_before = out.len();
    trace_retained_stage("outline", outline_started, outline_before, out.len());
}
