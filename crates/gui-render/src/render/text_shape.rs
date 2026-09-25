//! Plain-text measurement through public shaping/layout ownership.
use glyphon::{Attrs, AttrsList, FontSystem, ShapeLine, Shaping, Wrap};

/// Measure one paragraph at a time. Input text stays borrowed; completed public
/// shape/layout vectors are dropped before advancing to the next paragraph.
/// This avoids retaining opaque BufferLine text/attribute/cache allocations.
pub(crate) fn measure(
    fonts: &mut FontSystem,
    text: &str,
    attrs: &Attrs<'_>,
    size: f32,
    width: Option<f32>,
) -> (f32, usize) {
    let attributes = AttrsList::new(attrs);
    let trailing_empty = text.is_empty() || text.ends_with(['\r', '\n']);
    let paragraphs = glyphon::cosmic_text::LineIter::new(text)
        .map(|(range, _)| &text[range])
        .chain(trailing_empty.then_some(""));
    // Reuse layout working storage across paragraphs within this measurement.
    // Nothing escapes the call or adds another retained thread-local cache.
    let mut scratch = glyphon::cosmic_text::ShapeBuffer::default();
    let mut layout = Vec::new();
    let mut widest = 0.0_f32;
    let mut rows = 0;
    for paragraph in paragraphs {
        let shaped = ShapeLine::new(fonts, paragraph, &attributes, Shaping::Basic, 8);
        shaped.layout_to_buffer(
            &mut scratch,
            size,
            width,
            Wrap::WordOrGlyph,
            None,
            &mut layout,
            None,
        );
        rows += layout.len();
        for line in &layout {
            widest = widest.max(line.w);
        }
        layout.clear();
    }
    (widest, rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TextFace, text_attrs};
    use glyphon::{Buffer, Metrics};

    #[test]
    fn public_shape_measurements_match_render_buffer_layout() {
        let mut fonts = super::super::load_datum_fonts();
        for face in [
            TextFace::Ui,
            TextFace::UiMedium,
            TextFace::UiStrong,
            TextFace::Mono,
            TextFace::Terminal,
        ] {
            let attrs = text_attrs(face);
            for size in [9.0, 13.0, 22.0] {
                for text in [
                    "",
                    "\n",
                    "a\n",
                    "a\r\nb\n\rc\rd",
                    "one\n\ntwo\n",
                    "A long label wraps across several words.",
                    "long_unbroken_label_123456789",
                    "a\tb\tΩ",
                    "Datum · µm العربية 漢字",
                    "אבג 123 English",
                    "e\u{301} — 🛠",
                ] {
                    for width in [None, Some(40.0), Some(100.0)] {
                        let actual = measure(&mut fonts, text, &attrs, size, width);
                        let mut baseline = Buffer::new(&mut fonts, Metrics::new(size, size * 1.22));
                        baseline.set_size(&mut fonts, width, None);
                        baseline.set_text(&mut fonts, text, &attrs, Shaping::Basic, None);
                        baseline.shape_until_scroll(&mut fonts, false);
                        let expected_width = baseline
                            .layout_runs()
                            .map(|run| run.line_w)
                            .fold(0.0_f32, f32::max);
                        let expected_rows = baseline.layout_runs().count();
                        assert_eq!(
                            actual.0.to_bits(),
                            expected_width.to_bits(),
                            "width: {face:?} {size} {width:?} {text:?}"
                        );
                        assert_eq!(
                            actual.1, expected_rows,
                            "rows: {face:?} {size} {width:?} {text:?}"
                        );
                    }
                }
            }
        }
    }
}
