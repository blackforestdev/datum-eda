//! Native drawing primitives projected through the shared library camera.
use super::*;
use datum_gui_protocol::library_browser::acquisition::FootprintAcquisitionPreview;

pub(super) fn paths(raw: &Value) -> Vec<(Vec<datum_gui_protocol::PointNm>, i64)> {
    FootprintAcquisitionPreview::outline_paths(raw)
}

pub(super) fn render(raw: &Value, projection: &Projection, quads: &mut Vec<Quad>) {
    for (path, width) in paths(raw) {
        push_polyline_segments(
            quads,
            &path,
            projection,
            TEXT_MUTED,
            projection.world_length_to_px(width).max(1.),
        );
    }
}
