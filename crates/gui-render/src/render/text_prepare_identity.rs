//! Placement and paint identity, separate from shaping/layout ownership.
use super::*;

pub(super) fn text_prepare_signature(
    indices: &[usize],
    runs: &[TextRun],
    width: u32,
    height: u32,
) -> TextPrepareSignature {
    TextPrepareSignature {
        span_colors: runs
            .iter()
            .enumerate()
            .flat_map(|(index, run)| {
                run.rich_spans
                    .iter()
                    .map(move |span| (index, span.color.map(f32::to_bits)))
            })
            .collect(),
        width,
        height,
        runs: indices
            .iter()
            .zip(runs.iter())
            .map(|(index, run)| TextPrepareRunKey {
                buffer_index: *index,
                x_bits: run.x.to_bits(),
                y_bits: run.y.to_bits(),
                color_bits: run.color.map(f32::to_bits),
                clip_bounds: run.clip_bounds.map(|rect| RectBits {
                    x_bits: rect.x.to_bits(),
                    y_bits: rect.y.to_bits(),
                    width_bits: rect.width.to_bits(),
                    height_bits: rect.height.to_bits(),
                }),
            })
            .collect(),
    }
}
