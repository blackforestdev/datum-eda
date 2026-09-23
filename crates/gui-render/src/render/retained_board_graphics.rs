//! Retained board graphics and text batching with shared layer command ownership.
use super::*;

pub(super) fn push_retained_board_graphic_batches(
    out: &mut Vec<Quad>,
    strokes: &mut Vec<WorldStrokeInstance>,
    draw_commands: &mut Vec<RetainedDrawCommand>,
    scene: &BoardReviewSceneV1,
    _reference_projection: &Projection,
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
        for gfx in scene
            .board_graphics
            .iter()
            .filter(|gfx| render_stage_for_layer(&gfx.layer_id, sl) == stage)
        {
            let quad_start = out.len();
            let command_stroke_start = strokes.len();
            let active_color =
                board_graphic_world_color(&gfx.layer_id, sl, dim_unrelated_active(state));
            if trace_graphics {
                let graphic_started = std::time::Instant::now();
                let graphic_before = out.len();
                push_board_graphic_semantic_stroke(out, strokes, gfx, active_color);
                trace_graphic_timing(
                    gfx,
                    graphic_started,
                    out.len().saturating_sub(graphic_before),
                );
            } else {
                push_board_graphic_semantic_stroke(out, strokes, gfx, active_color);
            }
            scene_retained_access::finish_retained_draw_commands(
                draw_commands,
                Some(gfx.layer_id.clone()),
                quad_start,
                out.len(),
                command_stroke_start,
                strokes.len(),
            );
        }
        for outline in scene
            .outline
            .iter()
            .filter(|outline| render_stage_for_layer(&outline.layer_id, sl) == stage)
        {
            let command_stroke_start = strokes.len();
            push_world_stroke_path(
                strokes,
                &outline.path,
                board_surface_color(BoardSurfaceRole::Edge),
                EDGE_CUT_NM,
                1.0,
            );
            scene_retained_access::finish_retained_draw_commands(
                draw_commands,
                Some(outline.layer_id.clone()),
                out.len(),
                out.len(),
                command_stroke_start,
                strokes.len(),
            );
        }
    }
}

pub(super) fn push_retained_board_text_geometry_batches(
    out: &mut Vec<Quad>,
    draw_commands: &mut Vec<RetainedDrawCommand>,
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
        for text_geometry in scene
            .board_text_geometries
            .iter()
            .filter(|text| render_stage_for_layer(&text.layer_id, sl) == stage)
        {
            if !layer_visible(state, &text_geometry.layer_id) {
                continue;
            }
            let text_color = board_graphic_world_color(&text_geometry.layer_id, sl, dimmed);
            let quad_start = out.len();
            push_board_text_geometry_world(
                out,
                text_geometry,
                &glyph_mesh_assets,
                text_color,
                reference_projection,
            );
            scene_retained_access::finish_retained_draw_commands(
                draw_commands,
                Some(text_geometry.layer_id.clone()),
                quad_start,
                out.len(),
                0,
                0,
            );
        }
    }
}
