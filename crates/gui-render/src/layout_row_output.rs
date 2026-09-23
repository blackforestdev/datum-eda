//! Materialize one selected row; no subsequent row is visited or allocated.
use super::Row;
use glyphon::{LayoutGlyph, LayoutLine, ShapeLine};

pub(super) fn materialize(shape: &ShapeLine, row: &Row, size: f32, width: u32) -> LayoutLine {
    // Equal embedding levels form indivisible runs. Reorder those runs, leaving
    // glyph order within the shaped span under the shaper's authority.
    let mut runs: Vec<std::ops::Range<usize>> = Vec::new();
    let last = (row.end.span + usize::from(row.end.word != 0 || row.end.glyph != 0))
        .min(shape.spans.len());
    for s in row.start.span..last {
        if let Some(run) = runs.last_mut()
            && shape.spans[run.start].level == shape.spans[s].level
        {
            run.end = s + 1;
        } else {
            runs.push(s..s + 1);
        }
    }
    let low = runs
        .iter()
        .map(|r| shape.spans[r.start].level.number())
        .min()
        .unwrap_or(0)
        | 1;
    let high = runs
        .iter()
        .map(|r| shape.spans[r.start].level.number())
        .max()
        .unwrap_or(0);
    for level in (low..=high).rev() {
        let mut offset = 0;
        while offset < runs.len() {
            if shape.spans[runs[offset].start].level.number() < level {
                offset += 1;
                continue;
            }
            let end = (offset + 1..runs.len())
                .find(|&i| shape.spans[runs[i].start].level.number() < level)
                .unwrap_or(runs.len());
            runs[offset..end].reverse();
            offset = end;
        }
    }
    if shape.rtl {
        runs.reverse();
    }
    let mut result = LayoutLine {
        w: row.width,
        max_ascent: 0.0,
        max_descent: 0.0,
        line_height_opt: None,
        glyphs: Vec::new(),
    };
    let mut x = if shape.rtl { width as f32 } else { 0.0 };
    let mut y = 0.0;
    for span_index in runs.into_iter().flatten() {
        let span = &shape.spans[span_index];
        let reverse = span.level.is_rtl() != shape.rtl;
        let first_word = if span_index == row.start.span {
            row.start.word
        } else {
            0
        };
        let end_word = if span_index == row.end.span {
            row.end.word + usize::from(row.end.glyph != 0)
        } else {
            span.words.len()
        };
        let word_range = if reverse {
            span.words.len() - end_word..span.words.len() - first_word
        } else {
            first_word..end_word
        };
        for word_index in word_range {
            let word = &span.words[word_index];
            let logical_word = if reverse {
                span.words.len() - 1 - word_index
            } else {
                word_index
            };
            if row
                .skipped
                .iter()
                .any(|p| p.span == span_index && p.word == logical_word)
            {
                continue;
            }
            let first_glyph = if span_index == row.start.span && logical_word == row.start.word {
                row.start.glyph
            } else {
                0
            };
            let end_glyph = if span_index == row.end.span && logical_word == row.end.word {
                row.end.glyph
            } else {
                word.glyphs.len()
            };
            let glyph_range = if reverse {
                word.glyphs.len() - end_glyph..word.glyphs.len() - first_glyph
            } else {
                first_glyph..end_glyph
            };
            for glyph in &word.glyphs[glyph_range] {
                let font_size = glyph.metrics_opt.map_or(size, |metrics| metrics.font_size);
                let advance = font_size.mul_add(glyph.x_advance, 0.0);
                if shape.rtl {
                    x -= advance;
                }
                let line_height = glyph.metrics_opt.map(|metrics| metrics.line_height);
                result.glyphs.push(LayoutGlyph {
                    font_size,
                    x,
                    y,
                    w: advance,
                    level: span.level,
                    start: glyph.start,
                    end: glyph.end,
                    font_id: glyph.font_id,
                    font_weight: glyph.font_weight,
                    glyph_id: glyph.glyph_id,
                    x_offset: glyph.x_offset,
                    y_offset: glyph.y_offset,
                    color_opt: glyph.color_opt,
                    metadata: glyph.metadata,
                    cache_key_flags: glyph.cache_key_flags,
                    line_height_opt: line_height,
                });
                if !shape.rtl {
                    x += advance;
                }
                y += font_size * glyph.y_advance;
                result.max_ascent = result.max_ascent.max(font_size * glyph.ascent);
                result.max_descent = result.max_descent.max(font_size * glyph.descent);
                if let Some(height) = line_height {
                    result.line_height_opt =
                        Some(result.line_height_opt.map_or(height, |old| old.max(height)));
                }
            }
        }
    }
    if result.glyphs.is_empty() {
        result.line_height_opt = shape.metrics_opt.map(|m| m.line_height);
    }
    result
}
