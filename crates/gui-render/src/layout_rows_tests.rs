use super::*;
use glyphon::{AttrsList, ShapeLine, Shaping, Wrap};

#[test]
fn streaming_rows_match_installed_layout() {
    let mut fonts = crate::load_datum_fonts();
    let attrs = AttrsList::new(&crate::text_attrs(crate::TextFace::Ui));
    let mut failures = Vec::new();
    for text in [
        "",
        "a",
        "one two three",
        "  leading and trailing  ",
        "Unicode e\u{301} Ω العربية אבג",
        "אבג 123 English العربية",
        "a\tb\tlong_unbroken_word_123456",
        "👩‍🔧 😀",
    ] {
        let shape = ShapeLine::new(&mut fonts, text, &attrs, Shaping::Basic, 8);
        for width in [0, 1, 45, 180] {
            let expected = shape.layout(13.0, Some(width as f32), Wrap::WordOrGlyph, None, None);
            let actual: Vec<_> = Rows::new(&shape, 13.0, width)
                .map(|row| output::materialize(&shape, &row, 13.0, width))
                .collect();
            if format!("{actual:?}") != format!("{expected:?}") {
                let summary = |lines: &[glyphon::LayoutLine]| {
                    lines
                        .iter()
                        .map(|r| (r.w, r.glyphs.iter().map(|g| g.start).collect::<Vec<_>>()))
                        .collect::<Vec<_>>()
                };
                failures.push(format!(
                    "{text:?} width={width}: actual {:?}; expected {:?}",
                    summary(&actual),
                    summary(&expected)
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
