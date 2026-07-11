use super::{BoardGraphicPrimitive, RectPx, TextFace, TextRun};

pub(crate) fn trace_render_timing(message: String) {
    if std::env::var_os("DATUM_TRACE_TIMING").is_some() {
        eprintln!("[datum-render] {message}");
    }
}

pub(crate) fn trace_graphic_timing(
    graphic: &BoardGraphicPrimitive,
    started: std::time::Instant,
    quad_count: usize,
) {
    let elapsed_ms = started.elapsed().as_millis();
    if std::env::var_os("DATUM_TRACE_GRAPHICS").is_some() && (elapsed_ms >= 5 || quad_count >= 1024)
    {
        eprintln!(
            "[datum-graphic] {} kind={} layer={} points={} holes={} quads={} {}ms",
            graphic.object_id,
            graphic.primitive_kind,
            graphic.layer_id,
            graphic.path.len(),
            graphic.holes.len(),
            quad_count,
            elapsed_ms
        );
    }
}

pub(crate) fn suffix_id(id: &str) -> &str {
    id.rsplit(':').next().unwrap_or(id)
}

pub(crate) fn draw_text(
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    color: [f32; 3],
    face: TextFace,
    out: &mut Vec<TextRun>,
) {
    out.push(TextRun {
        text: text.to_string(),
        x,
        y,
        size,
        color,
        face,
        clip_bounds: None,
    });
}

// Render helper threads many quad/text-run/hit-region sinks.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_text_clipped(
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    color: [f32; 3],
    face: TextFace,
    clip_bounds: RectPx,
    out: &mut Vec<TextRun>,
) {
    out.push(TextRun {
        text: text.to_string(),
        x,
        y,
        size,
        color,
        face,
        clip_bounds: Some(clip_bounds),
    });
}

pub(crate) fn text_row_height_for_size(size: f32) -> f32 {
    (size * 1.6).ceil().max(size + 4.0)
}

pub(crate) fn key_value_row_height() -> f32 {
    // Key/value rows sit on a ~27px rhythm (Design Book kv min-height) so the
    // two type tiers breathe instead of crowding.
    27.0
}
