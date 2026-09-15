//! History rows own their wrapped text height and never share a clipping box.
//! Measurement uses the renderer's fonts, shaping and metrics, not character
//! counts. Scroll offsets remain record-based consumer state.

use super::{HistoryRow, RectPx, TextFace};
use crate::{Buffer, Metrics, Shaping, measure_font_system, text_attrs};

fn text_height(text: &str, width: f32, size: f32) -> f32 {
    let mut fonts = measure_font_system()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let line_height = size * 1.22;
    let mut buffer = Buffer::new(&mut fonts, Metrics::new(size, line_height));
    // Match text_buffer_extent's integer width and ensure all wrapped lines
    // participate in measurement, including explicit newlines and long words.
    buffer.set_size(&mut fonts, Some(width.ceil().max(1.0)), None);
    buffer.set_text(
        &mut fonts,
        text,
        &text_attrs(TextFace::Mono),
        Shaping::Basic,
        None,
    );
    buffer.shape_until_scroll(&mut fonts, false);
    buffer.layout_runs().count().max(1) as f32 * line_height
}

pub(super) fn visible_rows(
    rows: &[HistoryRow],
    bounds: RectPx,
    scale: f32,
) -> Vec<(usize, RectPx)> {
    let padding = 3.0 * scale;
    let width = (bounds.width - 20.0 * scale).max(1.0);
    let available = (bounds.height - padding * 2.0).max(0.0);
    if available == 0.0 {
        return Vec::new();
    }
    let gap = 5.0 * scale;
    let mut selected = Vec::new();
    let mut used = 0.0;
    for (index, row) in rows.iter().enumerate().rev() {
        let height = text_height(&row.text, width, 11.5 * scale).ceil();
        let spacing = if selected.is_empty() { 0.0 } else { gap };
        if !selected.is_empty() && used + spacing + height > available {
            break;
        }
        // An oversized record remains visible and cannot paint over its next
        // sibling. Ordinary records are included only when they fit in full.
        let height = height.min(available);
        selected.push((index, height));
        used += spacing + height;
    }
    let mut y = bounds.y + padding;
    selected
        .into_iter()
        .rev()
        .map(|(index, height)| {
            let clip = RectPx {
                x: bounds.x + 10.0 * scale,
                y,
                width,
                height,
            };
            y += height + gap;
            (index, clip)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<HistoryRow> {
        [
            "Layer Visibility menu action is unavailable in this build.",
            "Documents menu action is unavailable in this build.",
            "About Datum is unavailable in this build.",
        ]
        .into_iter()
        .map(|message| HistoryRow {
            text: format!("20:00:00  {message}  gui"),
            color: [1.0; 3],
        })
        .collect()
    }

    #[test]
    fn wrapped_refusals_have_disjoint_complete_clips_at_narrow_and_scaled_sizes() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            for width in [180.0, 330.0, 518.0] {
                let bounds = RectPx {
                    x: 20.0,
                    y: 40.0,
                    width: width * scale,
                    height: 143.0 * scale,
                };
                let rows = rows();
                // Every record can be reached by the existing record-scroll
                // control, even when only one wrapped record fits at a time.
                for end in 1..=rows.len() {
                    let visible = visible_rows(&rows[..end], bounds, scale);
                    assert_eq!(visible.last().unwrap().0, end - 1);
                    for (index, clip) in &visible {
                        assert!(
                            clip.height
                                >= text_height(&rows[*index].text, clip.width, 11.5 * scale)
                        );
                        assert!(clip.y + clip.height <= bounds.y + bounds.height);
                        assert!(clip.x + clip.width <= bounds.x + bounds.width);
                    }
                    for pair in visible.windows(2) {
                        assert!(pair[0].1.y + pair[0].1.height < pair[1].1.y);
                    }
                }
            }
        }
    }

    #[test]
    fn wrapping_counts_newlines_and_long_words_and_bounds_oversized_records() {
        assert!(text_height("first\nsecond", 200.0, 11.5) > text_height("first", 200.0, 11.5));
        assert!(text_height(&"X".repeat(100), 100.0, 11.5) > text_height("X", 100.0, 11.5));
        let rows = vec![HistoryRow {
            text: "long message ".repeat(100),
            color: [1.0; 3],
        }];
        let bounds = RectPx {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
        };
        let visible = visible_rows(&rows, bounds, 1.0);
        assert_eq!(visible.len(), 1);
        assert!(visible[0].1.y + visible[0].1.height <= bounds.height);
        assert!(
            visible_rows(
                &rows,
                RectPx {
                    height: 0.0,
                    ..bounds
                },
                1.0
            )
            .is_empty()
        );
    }
}
