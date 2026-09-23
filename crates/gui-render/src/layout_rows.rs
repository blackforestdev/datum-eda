//! Datum's streaming WordOrGlyph row boundaries over borrowed public shaping.
//! The cursor retains no paragraph-sized layout or scratch allocation.
use glyphon::ShapeLine;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Position {
    pub span: usize,
    pub word: usize,
    pub glyph: usize,
}

pub(super) struct Row {
    pub start: Position,
    pub end: Position,
    pub width: f32,
    pub skipped: Vec<Position>,
}

pub(super) struct Rows<'a> {
    shape: &'a ShapeLine,
    size: f32,
    width: f32,
    cursor: Position,
    emitted: bool,
    empty_before_word: bool,
    skipped: Vec<Position>,
}

impl<'a> Rows<'a> {
    pub fn new(shape: &'a ShapeLine, size: f32, width: u32) -> Self {
        Self {
            shape,
            size,
            width: width as f32,
            cursor: Position::default(),
            emitted: false,
            empty_before_word: false,
            skipped: Vec::new(),
        }
    }

    pub fn layout(&self, row: &Row) -> glyphon::LayoutLine {
        output::materialize(self.shape, row, self.size, self.width as u32)
    }

    pub fn word_index(&self, pos: Position) -> usize {
        let span = &self.shape.spans[pos.span];
        if span.level.is_rtl() == self.shape.rtl {
            pos.word
        } else {
            span.words.len() - 1 - pos.word
        }
    }

    fn finish(&mut self, start: Position, end: Position, width: f32) -> Row {
        self.emitted = true;
        Row {
            start,
            end,
            width,
            skipped: std::mem::take(&mut self.skipped),
        }
    }
}

impl Iterator for Rows<'_> {
    type Item = Row;

    fn next(&mut self) -> Option<Row> {
        if self.empty_before_word {
            self.empty_before_word = false;
            return Some(self.finish(self.cursor, self.cursor, 0.0));
        }
        let mut start = self.cursor;
        let mut committed = 0.0;
        let mut span_width = 0.0;
        let mut trailing_blank = None;
        loop {
            let Some(span) = self.shape.spans.get(self.cursor.span) else {
                return if start != self.cursor || !self.emitted {
                    Some(self.finish(start, self.cursor, committed + span_width))
                } else {
                    None
                };
            };
            if self.cursor.word == span.words.len() {
                committed += span_width;
                span_width = 0.0;
                trailing_blank = None;
                self.cursor = Position {
                    span: self.cursor.span + 1,
                    ..Position::default()
                };
                continue;
            }
            let word = &span.words[self.word_index(self.cursor)];
            let word_width = word.width(self.size);
            if self.cursor.glyph == 0
                && (committed + (span_width + word_width) <= self.width
                    || (word.blank && committed + span_width <= self.width))
            {
                trailing_blank = word.blank.then_some((self.cursor, span_width));
                span_width += word_width;
                self.cursor.word += 1;
                continue;
            }
            if word_width > self.width || self.cursor.glyph != 0 {
                // A word wider than the viewport begins on a fresh row when
                // this span already contributed content, then advances by glyph.
                if self.cursor.glyph == 0 && span_width > 0.0 {
                    self.empty_before_word = span.level.is_rtl() != self.shape.rtl
                        && word
                            .glyphs
                            .last()
                            .is_some_and(|g| g.width(self.size) > self.width);
                    return Some(self.finish(start, self.cursor, committed + span_width));
                }
                let reverse = span.level.is_rtl() != self.shape.rtl;
                while self.cursor.glyph < word.glyphs.len() {
                    let n = self.cursor.glyph;
                    let glyph = &word.glyphs[if reverse {
                        word.glyphs.len() - 1 - n
                    } else {
                        n
                    }];
                    let advance = glyph.width(self.size);
                    if committed + (span_width + advance) > self.width && start != self.cursor {
                        return Some(self.finish(start, self.cursor, committed + span_width));
                    }
                    // A single glyph can exceed the width; consume it to ensure
                    // forward progress even at zero-width viewports.
                    span_width += advance;
                    self.cursor.glyph += 1;
                }
                self.cursor.word += 1;
                self.cursor.glyph = 0;
                trailing_blank = None;
                continue;
            }
            if span_width > 0.0 {
                let (end, width) = trailing_blank.unwrap_or((self.cursor, span_width));
                if word.blank {
                    self.cursor.word += 1;
                }
                return Some(self.finish(start, end, committed + width));
            }
            // Preserve cross-span wrapping: a fresh span does not introduce an
            // extra break until it has contributed its own nonzero range.
            if word.blank {
                self.skipped.push(self.cursor);
                self.cursor.word += 1;
                if committed == 0.0 {
                    start = self.cursor;
                }
            } else {
                span_width = word_width;
                self.cursor.word += 1;
            }
        }
    }
}

#[path = "layout_row_output.rs"]
mod output;
#[cfg(test)]
#[path = "layout_rows_tests.rs"]
mod tests;
