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
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| matches_run(&entry.key, run, extent))
        {
            self.entries[index].last_used_frame = self.frame;
            return (index, false);
        }
        let key = text_buffer_key(run, width, height);
        let mut buffer = Buffer::new(font_system, Metrics::new(run.size, run.size * 1.22));
        let (buffer_width, buffer_height) = text_buffer_extent(run, width, height);
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
mod tests {
    use super::*;

    fn run() -> TextRun {
        TextRun {
            text: "Cache label".into(),
            rich_spans: vec![],
            x: 0.0,
            y: 0.0,
            size: 12.0,
            color: TEXT_PRIMARY,
            face: TextFace::Ui,
            clip_bounds: None,
        }
    }

    #[test]
    fn borrowed_lookup_preserves_the_complete_existing_key() {
        let mut original = run();
        original.rich_spans.push(TextRunSpan {
            text: "span".into(),
            color: TEXT_PRIMARY,
            bold: false,
            italic: false,
        });
        let key = text_buffer_key(&original, 1280, 800);
        let mut variants = vec![original.clone()];
        for change in 0..10 {
            let mut changed = original.clone();
            match change {
                0 => {
                    changed.x += 20.0;
                    changed.y += 30.0;
                    changed.color = TEXT_SECONDARY;
                }
                1 => changed.text.push('!'),
                2 => changed.size += 1.0,
                3 => changed.face = TextFace::Terminal,
                4 => changed.rich_spans[0].text.push('!'),
                5 => changed.rich_spans[0].bold = true,
                6 => changed.rich_spans[0].italic = true,
                7 => changed.rich_spans[0].color = TEXT_SECONDARY,
                8 => changed.rich_spans.clear(),
                _ => {
                    changed.clip_bounds = Some(RectPx {
                        x: 0.0,
                        y: 0.0,
                        width: 40.0,
                        height: 20.0,
                    })
                }
            }
            variants.push(changed);
        }
        for candidate in variants {
            for (width, height) in [(1280, 800), (25, 10)] {
                assert_eq!(
                    matches_run(
                        &key,
                        &candidate,
                        text_buffer_extent(&candidate, width, height)
                    ),
                    key == text_buffer_key(&candidate, width, height)
                );
            }
        }
    }

    #[test]
    fn real_shaped_cache_reuses_placement_changes_and_retires_old_workspace_rows() {
        let mut fonts = FontSystem::new();
        load_datum_fonts(&mut fonts);
        let mut cache = TextBufferCache::default();
        let original = run();
        cache.begin_frame(Profile::Workspace);
        let (indices, cold) = cache.indices(&mut fonts, std::slice::from_ref(&original), 1280, 800);
        assert_eq!(cold.misses, 1);
        assert!(
            cache.entries()[indices[0]]
                .buffer
                .layout_runs()
                .next()
                .is_some()
        );
        let key_storage = cache.entries()[indices[0]].key.text.as_ptr();
        let mut moved = original.clone();
        moved.x += 20.0;
        moved.color = TEXT_SECONDARY;
        cache.begin_frame(Profile::Workspace);
        let (warm_indices, warm) = cache.indices(&mut fonts, &[moved], 1280, 800);
        assert_eq!(warm.hits, 1);
        assert_eq!(warm.misses, 0);
        assert_eq!(indices, warm_indices);
        assert_eq!(key_storage, cache.entries()[indices[0]].key.text.as_ptr());
        let mut changed = original.clone();
        changed.text.push('!');
        cache.begin_frame(Profile::Workspace);
        assert_eq!(cache.indices(&mut fonts, &[changed], 1280, 800).1.misses, 1);
        assert_eq!(cache.entries().len(), 2);
        cache.begin_frame(Profile::Workspace);
        assert_eq!(cache.entries().len(), 1);
        assert_ne!(cache.entries()[0].key.text, original.text);
        cache.begin_frame(Profile::Overlay);
        cache.indices(&mut fonts, std::slice::from_ref(&original), 1280, 800);
        cache.trim_overlay();
        cache.begin_frame(Profile::Overlay);
        assert_eq!(
            cache.indices(&mut fonts, &[original], 1280, 800).1.misses,
            0
        );
    }

    #[derive(Debug)]
    struct SimulatedBuffer {
        key: String,
        last_used_frame: u64,
    }

    #[test]
    fn dialog_cache_retains_scroll_history_with_count_and_text_budgets() {
        let mut entries = Vec::new();
        for frame in 0..1000 {
            entries.push(SimulatedBuffer {
                key: format!("row-{frame}"),
                last_used_frame: frame,
            });
            retain_overlay_buffers(
                &mut entries,
                |entry| entry.last_used_frame,
                |entry| entry.key.len(),
            );
            assert!(entries.len() <= MAX_OVERLAY_BUFFERS);
        }
        assert!(entries.iter().any(|entry| entry.key == "row-990"));
        assert!(entries.iter().any(|entry| entry.key == "row-999"));
        entries.extend((0..100).map(|n| SimulatedBuffer {
            key: "x".repeat(1024),
            last_used_frame: 1000 + n,
        }));
        retain_overlay_buffers(
            &mut entries,
            |entry| entry.last_used_frame,
            |entry| entry.key.len(),
        );
        assert!(
            entries.iter().map(|entry| entry.key.len()).sum::<usize>() <= MAX_OVERLAY_TEXT_BYTES
        );
        entries.push(SimulatedBuffer {
            key: "x".repeat(MAX_OVERLAY_TEXT_BYTES + 1),
            last_used_frame: 2000,
        });
        retain_overlay_buffers(
            &mut entries,
            |entry| entry.last_used_frame,
            |entry| entry.key.len(),
        );
        assert!(
            entries
                .iter()
                .all(|entry| entry.key.len() <= MAX_OVERLAY_TEXT_BYTES)
        );
    }

    #[test]
    fn animated_agent_text_cache_retains_only_two_visible_generations() {
        const VISIBLE_RUNS: usize = 64;
        let mut cache: Vec<SimulatedBuffer> = Vec::new();
        let mut maximum_resident = 0usize;
        for unique in 0..100_000_usize {
            let frame = (unique / VISIBLE_RUNS + 1) as u64;
            if unique % VISIBLE_RUNS == 0 {
                retain_recent_text_buffers(&mut cache, frame, |entry| entry.last_used_frame);
            }
            let key = format!("agent-frame-{frame}-run-{}", unique % VISIBLE_RUNS);
            cache.push(SimulatedBuffer {
                key,
                last_used_frame: frame,
            });
            assert!(
                cache
                    .iter()
                    .any(|entry| entry.key == cache.last().unwrap().key)
            );
            maximum_resident = maximum_resident.max(cache.len());
        }
        assert_eq!(maximum_resident, VISIBLE_RUNS * 2);
        assert!(cache.len() <= VISIBLE_RUNS * 2);
        let last_frame = 100_000_usize.div_ceil(VISIBLE_RUNS) as u64;
        assert!(
            cache
                .iter()
                .all(|entry| entry.last_used_frame >= last_frame - 1)
        );
    }
}
