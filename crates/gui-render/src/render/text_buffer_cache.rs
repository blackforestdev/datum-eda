//! Shared shaped-buffer ownership and bounded workspace/dialog retention.
use super::*;
use glyphon::Style;

fn text_buffer_frame_is_recent(last_used_frame: u64, current_frame: u64) -> bool {
    last_used_frame >= current_frame.saturating_sub(1)
}

fn retain_recent_text_buffers<T>(
    entries: &mut Vec<T>,
    current_frame: u64,
    last_used_frame: impl Fn(&T) -> u64,
) {
    entries.retain(|entry| text_buffer_frame_is_recent(last_used_frame(entry), current_frame));
}

const MAX_OVERLAY_BUFFERS: usize = 128;
const MAX_OVERLAY_TEXT_BYTES: usize = 32 * 1024;

fn retain_overlay_buffers<T>(
    entries: &mut Vec<T>,
    age: impl Fn(&T) -> u64,
    size: impl Fn(&T) -> usize,
) {
    entries.sort_by_key(|entry| std::cmp::Reverse(age(entry)));
    let mut bytes = 0;
    let mut count = 0;
    entries.retain(|entry| {
        let next = bytes + size(entry);
        if count >= MAX_OVERLAY_BUFFERS || next > MAX_OVERLAY_TEXT_BYTES {
            return false;
        }
        bytes = next;
        count += 1;
        true
    });
}

#[derive(Default)]
pub(crate) struct TextBufferCache {
    entries: Vec<CachedTextBuffer>,
    frame: u64,
    revision: u64,
    #[cfg(test)]
    pub(crate) shape_reuses: usize,
}

#[derive(Clone, Copy)]
pub(crate) enum Profile {
    Workspace,
    Overlay,
}

/// Compare the complete existing shaping/layout key without allocating a copy.
/// Position and default color belong to glyph placement, not this buffer key.
fn matches_run(key: &TextBufferKey, run: &TextRun, (width_px, height_px): (u32, u32)) -> bool {
    key.text == run.text
        && key.size_bits == run.size.to_bits()
        && key.face == run.face
        && key.width_px == width_px
        && key.height_px == height_px
        && key.rich_spans.len() == run.rich_spans.len()
        && key
            .rich_spans
            .iter()
            .zip(&run.rich_spans)
            .all(|(key, span)| {
                key.text == span.text
                    && key.color_bits == span.color.map(f32::to_bits)
                    && key.bold == span.bold
                    && key.italic == span.italic
            })
}

impl TextBufferCache {
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn entries(&self) -> &[CachedTextBuffer] {
        &self.entries
    }

    /// Workspace/terminal keeps two generations; dialog history is trimmed
    /// after submission so every current-frame text-area borrow stays valid.
    pub(crate) fn begin_frame(&mut self, profile: Profile) {
        self.frame = self.frame.wrapping_add(1).max(1);
        if matches!(profile, Profile::Workspace) {
            let old_len = self.entries.len();
            retain_recent_text_buffers(&mut self.entries, self.frame, |entry| {
                entry.last_used_frame
            });
            if self.entries.len() != old_len {
                self.revision = self.revision.wrapping_add(1);
            }
        }
    }

    /// Called after glyph preparation/submission, when no text-area borrow is
    /// live. Bound retained buffers and key text; current-frame scratch can grow
    /// only with that frame's visible text. Glyph instances own their GPU data.
    pub(crate) fn trim_overlay(&mut self) {
        // Sorting can change buffer indices even without evicting entries.
        self.revision = self.revision.wrapping_add(1);
        retain_overlay_buffers(
            &mut self.entries,
            |entry| entry.last_used_frame,
            |entry| {
                entry.key.text.len()
                    + entry
                        .key
                        .rich_spans
                        .iter()
                        .map(|span| span.text.len())
                        .sum::<usize>()
            },
        );
    }

    pub(crate) fn indices(
        &mut self,
        font_system: &mut FontSystem,
        text_runs: &[TextRun],
        width: u32,
        height: u32,
    ) -> (Vec<usize>, TextBufferCacheStats) {
        let mut indices = Vec::with_capacity(text_runs.len());
        let mut stats = TextBufferCacheStats::default();
        for run in text_runs {
            let (index, missed) = self.ensure_text_buffer(font_system, run, width, height);
            if missed {
                stats.misses += 1;
            } else {
                stats.hits += 1;
            }
            indices.push(index);
        }
        (indices, stats)
    }

    fn ensure_text_buffer(
        &mut self,
        font_system: &mut FontSystem,
        run: &TextRun,
        width: u32,
        height: u32,
    ) -> (usize, bool) {
        let extent = text_buffer_extent(run, width, height);
        let mut shaped = None;
        for (index, entry) in self.entries.iter().enumerate() {
            let old_extent = (entry.key.width_px, entry.key.height_px);
            if matches_run(&entry.key, run, old_extent) {
                if old_extent == extent {
                    self.entries[index].last_used_frame = self.frame;
                    return (index, false);
                }
                shaped.get_or_insert(index);
            }
        }
        let key = text_buffer_key(run, width, height);
        // Extent belongs to layout. Clone the existing public Buffer cache so
        // already shaped paragraphs survive wrap/clip-size changes. FontSystem,
        // shaping mode, line metrics and font inventory are fixed by this owner;
        // all variable text/face/size/rich-style inputs still have to match.
        let mut buffer = if let Some(index) = shaped {
            #[cfg(test)]
            {
                self.shape_reuses += 1;
            }
            let mut buffer = self.entries[index].buffer.clone();
            buffer.set_size(font_system, Some(extent.0 as f32), Some(extent.1 as f32));
            buffer
        } else {
            let mut buffer = Buffer::new(font_system, Metrics::new(run.size, run.size * 1.22));
            let (buffer_width, buffer_height) = extent;
            buffer.set_size(
                font_system,
                Some(buffer_width as f32),
                Some(buffer_height as f32),
            );
            let attrs = text_attrs(run.face);
            if run.rich_spans.is_empty() {
                buffer.set_text(font_system, &run.text, &attrs, Shaping::Basic, None);
            } else {
                buffer.set_rich_text(
                    font_system,
                    run.rich_spans.iter().map(|span| {
                        let mut span_attrs = attrs.clone().color(text_color(span.color));
                        if span.bold {
                            span_attrs = span_attrs.weight(Weight::BOLD);
                        }
                        if span.italic {
                            span_attrs = span_attrs.style(Style::Italic);
                        }
                        (span.text.as_str(), span_attrs)
                    }),
                    &attrs,
                    Shaping::Basic,
                    None,
                );
            }
            buffer
        };
        buffer.shape_until_scroll(font_system, false);
        self.revision = self.revision.wrapping_add(1);
        self.entries.push(CachedTextBuffer {
            key,
            buffer,
            last_used_frame: self.frame,
        });
        (self.entries.len() - 1, true)
    }
}

#[cfg(test)]
#[path = "text_buffer_cache_tests.rs"]
mod tests;
