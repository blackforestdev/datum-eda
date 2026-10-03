//! Current generated copper and authored boundaries remain distinct projections.
use super::*;
use crate::presentation_ranges::Role;
#[allow(clippy::too_many_arguments)]
pub(super) fn push(
    out: &mut impl Output<Quad>,
    strokes: &mut impl Output<WorldStrokeInstance>,
    draw_commands: &mut impl Output<RetainedDrawCommand>,
    scene: &BoardReviewSceneV1,
    _reference_projection: &Projection,
    state: &ReviewWorkspaceState,
    pass_priority: u32,
    preview_affected_ids: &[&str],
) {
    for zone in &scene.zones {
        if copper_pass_priority_for_layer(&zone.layer_id, &scene.layers) != Some(pass_priority)
            || !authored_visible(state)
            || !layer_visible(state, &zone.layer_id)
        {
            continue;
        }
        let related = zone_matches_active_action(zone, state)
            || source_object_matches_preview(
                preview_affected_ids,
                &zone.object_id,
                &zone.source_object_uuid,
            );
        let dimmed = dim_unrelated_active(state) && !related;
        let quad_start = out.len();
        let stroke_start = strokes.len();
        let za = resolve_layer_appearance_with_scene(Some(&zone.layer_id), &scene.layers);
        for (index, polygon) in zone.occupied_polygons().enumerate() {
            out.source(Some(&zone.object_id), Role::ZoneIsland(index));
            push_world_polygon_fill(out, polygon, dim_authored_color(za.zone_fill, dimmed));
        }
        let minimum = if zone.fill_projection.is_legacy() {
            4
        } else {
            3
        };
        if zone.polygon.len() >= minimum {
            strokes.source(Some(&zone.object_id), Role::ZoneBoundary);
            gpu_strokes::push_world_stroke_loop(
                strokes,
                &zone.polygon,
                dim_authored_color(za.zone_outline, dimmed),
                AuthoredStrokePrimitive::CopperZoneOutline.nominal_nm(),
                1.0,
            );
        }
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

#[cfg(test)]
#[path = "retained_zones_tests.rs"]
mod tests;
