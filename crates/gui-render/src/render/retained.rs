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
    out: &mut Vec<Quad>,
    scene: &BoardReviewSceneV1,
    reference_projection: &Projection,
    state: &ReviewWorkspaceState,
) {
    let active_move_component_uuid: Option<String> = None;
    let sl = &scene.layers;
    let preview_affected_ids = proposal_preview_affected_ids(state);
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
                let za = layer_app(&zone.layer_id);
                let (fill_color, outline_color) = (za.zone_fill, za.zone_outline);
                push_world_polygon_fill(out, &zone.polygon, dim_authored_color(fill_color, dimmed));
                push_world_polyline_mitered(
                    out,
                    &close_path(&zone.polygon),
                    world_stroke_nm(2.0, reference_projection),
                    dim_authored_color(outline_color, dimmed),
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
            let track_width_nm = (track.width_nm as f32).max(world_stroke_nm(
                if selected { 3.0 } else { 2.0 },
                reference_projection,
            ));
            push_world_polyline_segments(out, &track.path, track_width_nm, color);
            let half = (track_width_nm * 0.5).round() as i64;
            for point in &track.path {
                push_world_ellipse_nm(
                    out,
                    datum_gui_protocol::RectNm {
                        min_x: point.x - half,
                        min_y: point.y - half,
                        max_x: point.x + half,
                        max_y: point.y + half,
                    },
                    color,
                    64,
                );
            }
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
                push_pad_primitive_world(
                    out,
                    pad,
                    render_layer,
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
            push_via_primitive_world(
                out,
                via,
                layer_app(display_layer).pad_copper,
                selected,
                dimmed,
                reference_projection,
            );
        }
    }
    trace_retained_stage("copper", copper_started, copper_before, out.len());
    let mechanical_graphics: Vec<_> = scene
        .component_graphics
        .iter()
        .filter(|graphic| {
            graphic.render_role == "component_mechanical"
                && active_move_component_uuid.as_deref() != Some(graphic.component_uuid.as_str())
        })
        .collect();
    let mut process_layers: Vec<_> = scene
        .layers
        .iter()
        .filter_map(|layer| match render_stage_for_layer(&layer.layer_id, sl) {
            RenderStage::BottomPaste | RenderStage::TopPaste => {
                Some((layer.layer_id.clone(), PadProcessLayerKind::Paste))
            }
            RenderStage::BottomMask | RenderStage::TopMask => {
                Some((layer.layer_id.clone(), PadProcessLayerKind::Mask))
            }
            _ => None,
        })
        .collect();
    process_layers.sort_by_key(|(layer_id, _)| scene_layer_stack_priority(layer_id, sl));
    let silkscreen_graphics: Vec<_> = scene
        .component_graphics
        .iter()
        .filter(|graphic| {
            graphic.render_role == "component_silkscreen"
                && active_move_component_uuid.as_deref() != Some(graphic.component_uuid.as_str())
        })
        .collect();
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
        for (layer_id, kind) in process_layers
            .iter()
            .filter(|(layer_id, _)| render_stage_for_layer(layer_id, sl) == stage)
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
                let derived = derived_process_pad(pad, layer_id, *kind, &scene.pad_expansion_setup);
                push_pad_primitive_world(
                    out,
                    &derived,
                    layer_id,
                    if active {
                        selected_silk_color(mask_or_paste_layer_color(layer_id, sl))
                    } else {
                        dim_process_color(mask_or_paste_layer_color(layer_id, sl), dimmed)
                    },
                    None,
                    false,
                    reference_projection,
                );
            }
        }
        process_pad_elapsed += process_started.elapsed();
        process_pad_quads += out.len().saturating_sub(process_before);
        let mechanical_before = out.len();
        let mechanical_started = std::time::Instant::now();
        for graphic in mechanical_graphics.iter().filter(|graphic| {
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
            push_component_graphic_primitive_world(
                out,
                graphic,
                sl,
                selected,
                related || selected_component,
                dim_unrelated_active(state) && !selected_component && !related,
                reference_projection,
            );
        }
        mechanical_elapsed += mechanical_started.elapsed();
        mechanical_quads += out.len().saturating_sub(mechanical_before);
        let silkscreen_before = out.len();
        let silkscreen_started = std::time::Instant::now();
        for graphic in silkscreen_graphics.iter().filter(|graphic| {
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
            push_component_graphic_primitive_world(
                out,
                graphic,
                sl,
                selected,
                related,
                dim_unrelated_active(state) && !selected && !related,
                reference_projection,
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
                    render_layer,
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
        let mut unrouted_batches: Vec<(Vec<PointNm>, [f32; 3], [f32; 3], f32, f32, f32, f32)> =
            Vec::new();
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
                unrouted.path.clone(),
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
    trace_retained_stage("unrouted", unrouted_started, unrouted_before, out.len());
    let outline_started = std::time::Instant::now();
    let outline_before = out.len();
    trace_retained_stage("outline", outline_started, outline_before, out.len());
}

fn push_retained_board_graphic_batches(
    out: &mut Vec<Quad>,
    batches: &mut Vec<RetainedWorldBatch>,
    scene: &BoardReviewSceneV1,
    reference_projection: &Projection,
    state: &ReviewWorkspaceState,
) {
    if !authored_visible(state) {
        return;
    }
    let sl = &scene.layers;
    out.reserve(
        scene
            .board_graphics
            .len()
            .saturating_add(scene.outline.len() * 32),
    );
    let trace_graphics = std::env::var_os("DATUM_TRACE_GRAPHICS").is_some();

    for stage in POST_COPPER_STAGES {
        let mut active_layer: Option<String> = None;
        let mut active_color = [0.0, 0.0, 0.0];
        let mut active_start = out.len();
        for gfx in scene
            .board_graphics
            .iter()
            .filter(|gfx| render_stage_for_layer(&gfx.layer_id, sl) == stage)
        {
            if active_layer.as_deref() != Some(gfx.layer_id.as_str()) {
                finish_retained_quad_batch(batches, active_layer.take(), active_start, out.len());
                active_layer = Some(gfx.layer_id.clone());
                active_color =
                    board_graphic_world_color(&gfx.layer_id, sl, dim_unrelated_active(state));
                active_start = out.len();
            }
            if trace_graphics {
                let graphic_started = std::time::Instant::now();
                let graphic_before = out.len();
                push_board_graphic_primitive_world(out, gfx, active_color, reference_projection);
                trace_graphic_timing(
                    gfx,
                    graphic_started,
                    out.len().saturating_sub(graphic_before),
                );
            } else {
                push_board_graphic_primitive_world(out, gfx, active_color, reference_projection);
            }
        }
        finish_retained_quad_batch(batches, active_layer.take(), active_start, out.len());
        let mut outline_layer: Option<String> = None;
        let mut outline_start = out.len();
        for outline in scene
            .outline
            .iter()
            .filter(|outline| render_stage_for_layer(&outline.layer_id, sl) == stage)
        {
            if outline_layer.as_deref() != Some(outline.layer_id.as_str()) {
                finish_retained_quad_batch(batches, outline_layer.take(), outline_start, out.len());
                outline_layer = Some(outline.layer_id.clone());
                outline_start = out.len();
            }
            push_world_polyline_segments_capped(
                out,
                &outline.path,
                world_stroke_nm(1.6, reference_projection),
                board_surface_color(BoardSurfaceRole::Edge),
            );
        }
        finish_retained_quad_batch(batches, outline_layer.take(), outline_start, out.len());
    }
}

fn push_retained_board_text_geometry_batches(
    out: &mut Vec<Quad>,
    batches: &mut Vec<RetainedWorldBatch>,
    scene: &BoardReviewSceneV1,
    reference_projection: &Projection,
    state: &ReviewWorkspaceState,
) {
    if !authored_visible(state) {
        return;
    }
    let sl = &scene.layers;
    let dimmed = dim_unrelated_active(state);
    let glyph_mesh_assets: BTreeMap<GlyphMeshHandlePrimitive, &GlyphMeshAssetPrimitive> = scene
        .glyph_mesh_assets
        .iter()
        .map(|asset| (asset.handle, asset))
        .collect();
    for stage in POST_COPPER_STAGES {
        let mut active_layer: Option<String> = None;
        let mut active_start = out.len();
        let mut active_color = [0.0, 0.0, 0.0];
        for text_geometry in scene
            .board_text_geometries
            .iter()
            .filter(|text| render_stage_for_layer(&text.layer_id, sl) == stage)
        {
            if !layer_visible(state, &text_geometry.layer_id) {
                continue;
            }
            let text_color = board_graphic_world_color(&text_geometry.layer_id, sl, dimmed);
            if active_layer.as_deref() != Some(text_geometry.layer_id.as_str())
                || active_color != text_color
            {
                finish_retained_quad_batch(batches, active_layer.take(), active_start, out.len());
                active_layer = Some(text_geometry.layer_id.clone());
                active_color = text_color;
                active_start = out.len();
            }
            push_board_text_geometry_world(
                out,
                text_geometry,
                &glyph_mesh_assets,
                active_color,
                reference_projection,
            );
        }
        finish_retained_quad_batch(batches, active_layer.take(), active_start, out.len());
    }
}

fn finish_retained_quad_batch(
    batches: &mut Vec<RetainedWorldBatch>,
    layer_id: Option<String>,
    start_quads: usize,
    end_quads: usize,
) {
    if end_quads <= start_quads {
        return;
    }
    batches.push(RetainedWorldBatch {
        layer_id,
        start: (start_quads * 6) as u32,
        len: ((end_quads - start_quads) * 6) as u32,
    });
}

fn trace_retained_stage(
    name: &str,
    started: std::time::Instant,
    before_quads: usize,
    after_quads: usize,
) {
    trace_render_timing(format!(
        "retained stage {name} {}ms +{}q total={}q",
        started.elapsed().as_millis(),
        after_quads.saturating_sub(before_quads),
        after_quads
    ));
}

fn push_retained_world_hit_regions(
    out: &mut Vec<WorldHitRegion>,
    scene: &BoardReviewSceneV1,
    state: &ReviewWorkspaceState,
) {
    if !authored_visible(state) {
        return;
    }
    for track in &scene.tracks {
        if !layer_visible(state, &track.layer_id) {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(track.object_id.clone()),
            layer_id: Some(track.layer_id.clone()),
            shape: WorldHitShape::Polyline {
                path: track.path.clone(),
                half_width_nm: (track.width_nm as f32 * 0.5).max(150_000.0),
            },
        });
    }
    for via in &scene.vias {
        if !via_visible(state, &via.start_layer_id, &via.end_layer_id) {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(via.object_id.clone()),
            layer_id: None,
            shape: WorldHitShape::Circle {
                center: via.position,
                radius_nm: (via.diameter_nm as f32 * 0.5).max(250_000.0),
            },
        });
    }
    for component in &scene.components {
        if !layer_visible(state, &component.placement_layer) {
            continue;
        }
        let component_pads: Vec<_> = scene
            .pads
            .iter()
            .filter(|pad| pad.component_uuid == component.component_uuid)
            .collect();
        let has_non_edge_graphics = scene.component_graphics.iter().any(|graphic| {
            graphic.component_uuid == component.component_uuid
                && !graphic.layer_id.as_deref().is_some_and(|layer_id| {
                    scene
                        .layers
                        .iter()
                        .find(|layer| layer.layer_id == layer_id)
                        .is_some_and(|layer| layer.name == "Edge.Cuts")
                })
        });
        let has_text = scene
            .component_texts
            .iter()
            .any(|text| text.component_uuid == component.component_uuid);
        if let Some(hit_rect) = compact_component_body_bounds(&component_pads)
            && !has_non_edge_graphics
            && !has_text
        {
            out.push(WorldHitRegion {
                target: HitTarget::AuthoredObject(component.object_id.clone()),
                layer_id: Some(component.placement_layer.clone()),
                shape: WorldHitShape::Rect(hit_rect),
            });
            continue;
        }
        if has_non_edge_graphics || has_text {
            continue;
        }
        let hit_rect = inferred_component_body_bounds(&component_pads).unwrap_or(component.bounds);
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(component.object_id.clone()),
            layer_id: Some(component.placement_layer.clone()),
            shape: WorldHitShape::Rect(hit_rect),
        });
    }
    for pad in &scene.pads {
        let pad_visible = pad_visible_on_any_copper_layer(state, pad);
        if !pad_visible {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(pad.object_id.clone()),
            layer_id: None,
            shape: WorldHitShape::Rect(pad.bounds),
        });
    }
    for zone in &scene.zones {
        if !layer_visible(state, &zone.layer_id) || zone.polygon.len() < 3 {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(zone.object_id.clone()),
            layer_id: Some(zone.layer_id.clone()),
            shape: WorldHitShape::Polygon(zone.polygon.clone()),
        });
    }
    for graphic in &scene.component_graphics {
        let Some(target_id) = component_object_id_for_uuid(scene, &graphic.component_uuid) else {
            continue;
        };
        if let Some(layer_id) = graphic.layer_id.as_deref()
            && !layer_visible(state, layer_id)
        {
            continue;
        }
        if graphic.layer_id.as_deref().is_some_and(|layer_id| {
            scene
                .layers
                .iter()
                .find(|layer| layer.layer_id == layer_id)
                .is_some_and(|layer| layer.name == "Edge.Cuts")
        }) {
            continue;
        }
        let width = graphic.width_nm.unwrap_or(100_000);
        match graphic.primitive_kind.as_str() {
            "polygon" => {
                let (min_x, min_y, max_x, max_y) = graphic.path.iter().fold(
                    (i64::MAX, i64::MAX, i64::MIN, i64::MIN),
                    |(min_x, min_y, max_x, max_y), point| {
                        (
                            min_x.min(point.x),
                            min_y.min(point.y),
                            max_x.max(point.x),
                            max_y.max(point.y),
                        )
                    },
                );
                if min_x <= max_x && min_y <= max_y {
                    out.push(WorldHitRegion {
                        target: HitTarget::AuthoredObject(target_id.to_string()),
                        layer_id: graphic.layer_id.clone(),
                        shape: WorldHitShape::Rect(datum_gui_protocol::RectNm {
                            min_x,
                            min_y,
                            max_x,
                            max_y,
                        }),
                    });
                }
            }
            _ => {
                out.push(WorldHitRegion {
                    target: HitTarget::AuthoredObject(target_id.to_string()),
                    layer_id: graphic.layer_id.clone(),
                    shape: WorldHitShape::Polyline {
                        path: graphic.path.clone(),
                        half_width_nm: (width as f32 * 0.5).max(180_000.0),
                    },
                });
            }
        }
    }
    for text in &scene.board_texts {
        if !layer_visible(state, &text.layer_id) {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(text.object_id.clone()),
            layer_id: Some(text.layer_id.clone()),
            shape: WorldHitShape::Rect(board_text_hit_rect(text)),
        });
    }
    for gfx in &scene.board_graphics {
        if gfx.object_id.starts_with("board-text:") {
            continue;
        }
        if !layer_visible(state, &gfx.layer_id) {
            continue;
        }
        let width = gfx.width_nm.unwrap_or(100_000);
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(gfx.object_id.clone()),
            layer_id: Some(gfx.layer_id.clone()),
            shape: WorldHitShape::Polyline {
                path: gfx.path.clone(),
                half_width_nm: (width as f32 * 0.5).max(150_000.0),
            },
        });
    }
    for outline in &scene.outline {
        if !layer_visible(state, &outline.layer_id) {
            continue;
        }
        out.push(WorldHitRegion {
            target: HitTarget::AuthoredObject(outline.object_id.clone()),
            layer_id: Some(outline.layer_id.clone()),
            shape: WorldHitShape::Polyline {
                path: outline.path.clone(),
                half_width_nm: 300_000.0,
            },
        });
    }
}

