//! Retained plain/rich shaping and visible layout owned by Datum.
use std::sync::Arc;
#[path = "text_layout_allocation.rs"]
mod allocation;
use crate::text_buffer_cache::budget::Construction;
pub(crate) use allocation::Admission;
use allocation::Storage;
#[path = "font_owner.rs"]
pub(crate) mod fonts;

use crate::cpu_alloc::heap::capacity_bytes;
use crate::{TextRun, text_attrs};
use glyphon::cosmic_text::{BidiParagraphs, LineIter};
#[path = "layout_scratch.rs"]
pub(crate) mod scratch;
use glyphon::{AttrsList, LayoutLine, LayoutRun, ShapeLine, Style, Weight};
use scratch::LayoutScratch;
#[path = "layout_rows.rs"]
mod streaming_rows;

struct Row {
    paragraph: usize,
    layout: LayoutLine,
    construction: Option<Construction>,
    top: f32,
    baseline: f32,
    height: f32,
}

#[derive(Default)]
pub(crate) struct TextLayout {
    shapes: Storage<Arc<fonts::Shape>>,
    rows: Storage<Row>,
    complete: bool,
    valid: bool,
}

impl TextLayout {
    pub fn needs_input(&self) -> bool {
        !self.complete
    }

    pub fn with_input(
        fonts: &mut impl fonts::Source,
        scratch: &mut LayoutScratch,
        run: &TextRun,
        extent: (u32, u32),
        rich_text: &str,
        admission: &Admission<'_>,
    ) -> anyhow::Result<Self> {
        let mut result = Self::default();
        result.relayout_with_input(fonts, scratch, run, extent, rich_text, admission)?;
        Ok(result)
    }

    pub fn relayout_with_input(
        &mut self,
        fonts: &mut impl fonts::Source,
        scratch: &mut LayoutScratch,
        run: &TextRun,
        extent: (u32, u32),
        rich_text: &str,
        admission: &Admission<'_>,
    ) -> anyhow::Result<()> {
        self.valid = false;
        let result = self.relayout_with_attrs(
            fonts,
            scratch,
            run,
            extent,
            &text_attrs(run.face),
            rich_text,
            admission,
        );
        self.valid = result.is_ok();
        result
    }

    #[cfg(test)]
    pub fn new(
        fonts: &mut impl fonts::Source,
        scratch: &mut LayoutScratch,
        run: &TextRun,
        extent: (u32, u32),
    ) -> Self {
        let text: String = run
            .rich_spans
            .iter()
            .map(|span| span.text.as_str())
            .collect();
        {
            let host = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
            Self::with_input(
                fonts,
                scratch,
                run,
                extent,
                &text,
                &Admission {
                    owner: None,
                    host: &host,
                },
            )
            .unwrap()
        }
    }

    #[cfg(test)]
    pub fn relayout(
        &mut self,
        fonts: &mut impl fonts::Source,
        scratch: &mut LayoutScratch,
        run: &TextRun,
        extent: (u32, u32),
    ) {
        let text: String = run
            .rich_spans
            .iter()
            .map(|span| span.text.as_str())
            .collect();
        let host = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
        self.relayout_with_input(
            fonts,
            scratch,
            run,
            extent,
            &text,
            &Admission {
                owner: None,
                host: &host,
            },
        )
        .unwrap();
    }

    // Explicit-font oracle only; production font authority remains TextFace.
    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn with_test_attrs(
        fonts: &mut impl fonts::Source,
        scratch: &mut LayoutScratch,
        run: &TextRun,
        extent: (u32, u32),
        attrs: &glyphon::Attrs<'_>,
    ) -> Self {
        let mut result = Self::default();
        let text: String = run
            .rich_spans
            .iter()
            .map(|span| span.text.as_str())
            .collect();
        let host = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
        result
            .relayout_with_attrs(
                fonts,
                scratch,
                run,
                extent,
                attrs,
                &text,
                &Admission {
                    owner: None,
                    host: &host,
                },
            )
            .unwrap();
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn relayout_with_attrs(
        &mut self,
        fonts: &mut impl fonts::Source,
        scratch: &mut LayoutScratch,
        run: &TextRun,
        extent: (u32, u32),
        attrs: &glyphon::Attrs<'_>,
        rich_text: &str,
        admission: &Admission<'_>,
    ) -> anyhow::Result<()> {
        self.rows.clear();
        let mut top = 0.0;
        let cached = self.shapes.len();
        for index in 0..cached {
            if !self.append_layout(scratch, run, extent, index, &mut top, admission)? {
                return Ok(());
            }
        }
        if self.complete {
            return Ok(());
        }
        let mut process = |text: &str, attributes: AttrsList| -> anyhow::Result<bool> {
            let index = self.shapes.len();
            self.shapes
                .push(Arc::new(fonts.shape(text, &attributes)), admission.owner)?;
            self.append_layout(scratch, run, extent, index, &mut top, admission)
        };
        if run.rich_spans.is_empty() {
            let trailing = run.text.is_empty() || run.text.ends_with(['\r', '\n']);
            for paragraph in LineIter::new(&run.text)
                .map(|(range, _)| &run.text[range])
                .chain(trailing.then_some(""))
                .skip(cached)
            {
                if !process(paragraph, AttrsList::new(attrs))? {
                    return Ok(());
                }
            }
        } else {
            // Borrow the caller's admitted concatenation. Iterate paragraphs and
            // source spans directly instead of building two temporary vectors.
            let mut paragraphs = BidiParagraphs::new(rich_text).peekable();
            let empty = paragraphs.peek().is_none();
            for paragraph in paragraphs.chain(empty.then_some("")).skip(cached) {
                let start = if paragraph.is_empty() {
                    0
                } else {
                    paragraph.as_ptr() as usize - rich_text.as_ptr() as usize
                };
                let end = start + paragraph.len();
                let mut attributes = AttrsList::new(attrs);
                let mut offset = 0;
                for (index, span) in run.rich_spans.iter().enumerate() {
                    let span_end = offset + span.text.len();
                    let left = offset.max(start);
                    let right = span_end.min(end);
                    if left < right {
                        let mut style = attrs.clone().metadata(index + 1);
                        if span.bold {
                            style = style.weight(Weight::BOLD);
                        }
                        if span.italic {
                            style = style.style(Style::Italic);
                        }
                        attributes.add_span(left - start..right - start, &style);
                    }
                    offset = span_end;
                    if offset >= end {
                        break;
                    }
                }
                if !process(paragraph, attributes)? {
                    return Ok(());
                }
            }
        }
        self.complete = true;
        Ok(())
    }

    fn append_layout(
        &mut self,
        scratch: &mut LayoutScratch,
        run: &TextRun,
        extent: (u32, u32),
        index: usize,
        top: &mut f32,
        admission: &Admission<'_>,
    ) -> anyhow::Result<bool> {
        scratch.for_each_row(
            &self.shapes[index],
            run.size,
            extent.0,
            admission,
            |produced| {
                let layout = &produced.layout;
                let height = layout.line_height_opt.unwrap_or(run.size * 1.22);
                let leading = height - (layout.max_ascent + layout.max_descent);
                let baseline = *top + leading / 2.0 + layout.max_ascent;
                if baseline - layout.max_ascent > extent.1 as f32 {
                    return Ok(false);
                }
                self.rows.push(
                    Row {
                        paragraph: index,
                        layout: produced.layout,
                        construction: produced.construction,
                        top: *top,
                        baseline,
                        height,
                    },
                    admission.owner,
                )?;
                *top += height;
                Ok(true)
            },
        )
    }

    /// Fork only shared shaping; the caller computes its own extent's layout.
    pub fn fork_for_relayout(&self, admission: &Admission<'_>) -> anyhow::Result<Self> {
        let mut shapes = Storage::with_capacity(self.shapes.len(), admission.owner)?;
        for shape in self.shapes.iter() {
            shapes.push(shape.clone(), admission.owner)?;
        }
        Ok(Self {
            shapes,
            rows: Storage::default(),
            complete: self.complete,
            valid: false,
        })
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn discard_rows(&mut self) {
        self.valid = false;
        self.rows.clear();
    }

    /// The cache has published these exact capacities under its retained owner.
    pub fn published(&mut self) {
        for row in self.rows.iter_mut() {
            row.construction = None;
        }
        self.rows.published();
        self.shapes.published();
    }

    pub fn layout_runs(&self) -> Runs<'_> {
        Runs {
            layout: self,
            next: 0,
        }
    }

    pub(crate) fn layout_glyph_bytes(&self) -> usize {
        self.rows
            .iter()
            .map(|row| row.layout.glyphs.capacity() * std::mem::size_of::<glyphon::LayoutGlyph>())
            .sum()
    }

    pub(crate) fn layout_glyph_tracking_bytes(&self) -> usize {
        self.rows
            .iter()
            .map(|row| {
                crate::cpu_alloc::heap::tracking_bytes::<glyphon::LayoutGlyph>(
                    row.layout.glyphs.capacity(),
                )
            })
            .sum()
    }

    pub fn layout_storage_bytes(&self) -> usize {
        capacity_bytes::<Arc<fonts::Shape>>(self.shapes.capacity())
            + capacity_bytes::<Row>(self.rows.capacity())
            + self
                .rows
                .iter()
                .map(|row| capacity_bytes::<glyphon::LayoutGlyph>(row.layout.glyphs.capacity()))
                .sum::<usize>()
    }

    /// Unique public shapes plus measured Arc and Datum allocator overhead.
    /// The cache deduplicates these addresses across extent variants; private
    /// scratch and allocator-internal slack remain separate.
    pub fn shape_allocations(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.shapes.iter().map(|shape| {
            let bytes = shape_container_bytes() + shape.payload_bytes();
            (Arc::as_ptr(shape) as usize, bytes)
        })
    }

    #[cfg(test)]
    pub(crate) fn shape_storage(&self) -> *const glyphon::ShapeSpan {
        self.shapes[0].spans.as_ptr()
    }
}

/// Rendering-only view; original text and editing authority remain in TextRun.
#[derive(Clone)]
pub(crate) struct Runs<'a> {
    layout: &'a TextLayout,
    next: usize,
}
impl<'a> Iterator for Runs<'a> {
    type Item = LayoutRun<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let row = self.layout.rows.get(self.next)?;
        self.next += 1;
        Some(LayoutRun {
            line_i: row.paragraph,
            text: "",
            rtl: self.layout.shapes[row.paragraph].rtl,
            glyphs: &row.layout.glyphs,
            line_y: row.baseline,
            line_top: row.top,
            line_height: row.height,
            line_w: row.layout.w,
        })
    }
}

#[cfg(test)]
#[path = "text_layout_tests.rs"]
mod tests;

fn shape_container_bytes() -> usize {
    if !crate::cpu_alloc::installed() {
        // External consumers without Datum's allocator retain public-size reporting.
        return std::mem::size_of::<fonts::Shape>();
    }
    static BYTES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *BYTES.get_or_init(|| {
        crate::cpu_alloc::heap::arc_bytes(fonts::Shape::untracked(ShapeLine {
            rtl: false,
            spans: Vec::new(),
            metrics_opt: None,
        }))
    })
}
