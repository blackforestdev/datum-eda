use super::*;
use crate::text_color;
use glyphon::Shaping;
use glyphon::{Buffer, Metrics};

#[test]
fn owned_layout_matches_buffer_for_plain_rich_and_extent_changes() {
    let mut fonts = crate::load_datum_fonts();
    let mut scratch = LayoutScratch::default();
    for rich in [false, true] {
        for text in [
            "",
            "a\n",
            "one\n\ntwo",
            "a\r\nb\n\rc\rd",
            "Unicode e\u{301} Ω العربية אבג",
            "a\tb\tlong_unbroken_word_123456",
            "👩‍🔧 😀",
        ] {
            let mut run = crate::TextRun {
                text: text.into(),
                x: 0.0,
                y: 0.0,
                size: 13.0,
                color: [1.0; 3],
                face: crate::TextFace::Ui,
                clip_bounds: None,
                layout_size: None,
                rich_spans: Vec::new(),
            };
            if rich {
                run.rich_spans.push(crate::TextRunSpan {
                    text: text.into(),
                    color: [0.3, 0.8, 0.1],
                    bold: true,
                    italic: true,
                });
            }
            let mut owned = TextLayout::default();
            for (width, height) in [(45, 16), (180, 150), (45, 150), (180, 16)] {
                owned.relayout(&mut fonts, &mut scratch, &run, (width, height));
                let mut reference =
                    Buffer::new(&mut fonts, Metrics::new(run.size, run.size * 1.22));
                reference.set_size(&mut fonts, Some(width as f32), Some(height as f32));
                let attrs = text_attrs(run.face);
                if rich {
                    reference.set_rich_text(
                        &mut fonts,
                        [(
                            text,
                            attrs
                                .clone()
                                .weight(Weight::BOLD)
                                .style(Style::Italic)
                                .color(text_color(run.rich_spans[0].color)),
                        )],
                        &attrs,
                        Shaping::Basic,
                        None,
                    );
                } else {
                    reference.set_text(&mut fonts, text, &attrs, Shaping::Basic, None);
                }
                reference.shape_until_scroll(&mut fonts, false);
                let signature = |line: LayoutRun<'_>| {
                    let mut glyphs = line.glyphs.to_vec();
                    // Resolve Datum's paint indices for comparison with the
                    // reference Buffer's baked colors; geometry stays exact.
                    for glyph in &mut glyphs {
                        if let Some(index) = glyph.metadata.checked_sub(1) {
                            glyph.color_opt = Some(text_color(run.rich_spans[index].color));
                            glyph.metadata = 0;
                        }
                    }
                    (
                        line.line_y.to_bits(),
                        line.line_top.to_bits(),
                        line.line_height.to_bits(),
                        line.line_w.to_bits(),
                        format!("{glyphs:?}"),
                    )
                };
                assert_eq!(
                    owned.layout_runs().map(signature).collect::<Vec<_>>(),
                    reference.layout_runs().map(signature).collect::<Vec<_>>(),
                    "rich={rich} {text:?} {width}x{height}"
                );
            }
        }
    }
}

#[test]
fn shape_container_cost_measures_arc_and_allocator_overhead_without_double_counting() {
    let expected = shape_container_bytes();
    let scope = crate::cpu_alloc::Scope::new("shape-container-proof");
    let shape = scope.with(|| {
        Arc::new(fonts::Shape::untracked(ShapeLine {
            rtl: false,
            spans: Vec::new(),
            metrics_opt: None,
        }))
    });
    assert_eq!(
        expected as u64,
        scope.usage().payload_bytes + scope.usage().tracking_bytes
    );
    assert!(expected > std::mem::size_of::<ShapeLine>());
    let mut layout = TextLayout::default();
    layout.shapes.push(shape.clone(), None).unwrap();
    let mut other = TextLayout::default();
    other.shapes.push(shape, None).unwrap();
    let unique: std::collections::BTreeMap<_, _> = layout
        .shape_allocations()
        .chain(other.shape_allocations())
        .collect();
    assert_eq!(unique.len(), 1);
    assert_eq!(unique.values().sum::<usize>(), expected);
    drop(layout);
    assert_eq!(
        scope.usage().payload_bytes + scope.usage().tracking_bytes,
        expected as u64
    );
    drop(other);
    assert_eq!(
        scope.usage().payload_bytes + scope.usage().tracking_bytes,
        0
    );
}

#[test]
fn live_span_paint_matches_baked_reference_across_text_boundaries() {
    let mut fonts = crate::load_datum_fonts();
    let mut scratch = LayoutScratch::default();
    for parts in [
        ["of", "fice"],
        ["e", "\u{301}"],
        ["الع", "ربية"],
        ["a\n", "b\r\nc"],
        ["", "x"],
        ["אבג ", "123 English"],
    ] {
        let run = crate::TextRun {
            text: parts.concat(),
            rich_spans: parts
                .into_iter()
                .enumerate()
                .map(|(index, text)| crate::TextRunSpan {
                    text: text.into(),
                    color: if index == 0 {
                        [1.0, 0.2, 0.1]
                    } else {
                        [0.1, 0.8, 1.0]
                    },
                    bold: false,
                    italic: false,
                })
                .collect(),
            x: 0.0,
            y: 0.0,
            size: 15.0,
            color: [1.0; 3],
            face: crate::TextFace::Ui,
            clip_bounds: None,
            layout_size: None,
        };
        for width in [40, 200] {
            let owned = TextLayout::new(&mut fonts, &mut scratch, &run, (width, 200));
            let attrs = text_attrs(run.face);
            let mut reference = Buffer::new(&mut fonts, Metrics::new(run.size, run.size * 1.22));
            reference.set_size(&mut fonts, Some(width as f32), Some(200.0));
            reference.set_rich_text(
                &mut fonts,
                run.rich_spans.iter().map(|span| {
                    (
                        span.text.as_str(),
                        attrs.clone().color(text_color(span.color)),
                    )
                }),
                &attrs,
                Shaping::Basic,
                None,
            );
            reference.shape_until_scroll(&mut fonts, false);
            let signature = |line: LayoutRun<'_>| {
                let mut glyphs = line.glyphs.to_vec();
                for glyph in &mut glyphs {
                    if let Some(index) = glyph.metadata.checked_sub(1) {
                        glyph.color_opt = Some(text_color(run.rich_spans[index].color));
                        glyph.metadata = 0;
                    }
                }
                (
                    line.line_y.to_bits(),
                    line.line_w.to_bits(),
                    format!("{glyphs:?}"),
                )
            };
            assert_eq!(
                owned.layout_runs().map(signature).collect::<Vec<_>>(),
                reference.layout_runs().map(signature).collect::<Vec<_>>(),
                "{parts:?} width={width}"
            );
        }
    }
}
