//! Transient PCB ink over the existing per-pane projection; never copper authority.
use super::*;
pub(super) fn render(
    out: &mut Vec<Quad>,
    state: &ReviewWorkspaceState,
    scene: &BoardReviewSceneV1,
    projection: &Projection,
) {
    let Some(setup) = &state.authoring.board_controls else {
        return;
    };
    let outline = state.authoring.board_outline.as_ref();
    if !setup.owns_pane(state)
        || scene.project_uuid != setup.basis.project.to_string()
        || scene.board_uuid != setup.basis.source.to_string()
        || (outline.is_none()
            && (!setup.current(state) || scene.source_revision != setup.basis.revision))
    {
        return;
    }
    let cursor = state.authoring.gesture.preview;
    let mut points: Vec<datum_gui_protocol::PointNm> =
        if let Some(route) = &state.authoring.board_route {
            route
                .preview_points(cursor)
                .unwrap_or_default()
                .into_iter()
                .map(|p| PointNm { x: p.x, y: p.y })
                .collect()
        } else if let Some(outline) = outline {
            let mut vertices: Vec<_> = outline
                .vertices()
                .iter()
                .map(|p| datum_gui_protocol::PointNm { x: p.x, y: p.y })
                .collect();
            if !outline.pending() && setup.current(state) {
                if let Some(cursor) = cursor {
                    vertices.push(cursor);
                }
            }
            vertices
        } else if let Some(zone) = &state.authoring.board_zone {
            let mut vertices: Vec<_> = zone
                .vertices()
                .iter()
                .map(|p| PointNm { x: p.x, y: p.y })
                .collect();
            if let Some(cursor) = cursor {
                vertices.push(cursor);
            }
            vertices
        } else {
            Vec::new()
        };
    if (state.authoring.board_zone.is_some() || outline.is_some()) && points.len() > 2 {
        points.push(points[0]);
    }
    let color = if outline.is_some() && !setup.current(state) && !setup.pending {
        TEXT_MUTED
    } else {
        TEXT_ACCENT
    };
    push_polyline_segments(out, &points, projection, color, 2.);
}
