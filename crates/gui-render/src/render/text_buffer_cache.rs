//! Shared shaped-buffer ownership and bounded workspace/dialog retention.
use super::*;
#[path = "text_cache_admission.rs"]
mod admission;
#[path = "text_cache_budget.rs"]
pub(crate) mod budget;
use crate::cpu_alloc::heap::{capacity_bytes, tracking_bytes};
use crate::text_gpu::Area;
use crate::text_layout::TextLayout;
use crate::text_layout::scratch::LayoutScratch;
use std::hash::{Hash, Hasher};

pub(super) struct CachedTextBuffer {
    key: TextBufferKey,
    buffer: TextLayout,
    last_used_frame: u64,
}

pub(super) fn build_text_areas<'a>(
    cache: &'a [CachedTextBuffer],
    indices: &[usize],
    runs: &[TextRun],
) -> Vec<Area<crate::text_layout::Runs<'a>>> {
    indices
        .iter()
        .zip(runs.iter())
        .map(|(index, run)| Area {
            rows: cache[*index].buffer.layout_runs(),
            left: run.x,
            top: run.y,
            scale: 1.0,
            bounds: run
                .clip_bounds
                .map_or_else(TextBounds::default, |rect| TextBounds {
                    left: rect.x.floor() as i32,
                    top: rect.y.floor() as i32,
                    right: (rect.x + rect.width).ceil() as i32,
                    bottom: (rect.y + rect.height).ceil() as i32,
                }),
            default_color: text_color(run.color),
        })
        .collect()
}

pub(super) fn text_buffer_key(run: &TextRun, width: u32, height: u32) -> TextBufferKey {
    let (width_px, height_px) = text_buffer_extent(run, width, height);
    TextBufferKey {
        text: run.text.clone(),
        rich_spans: run
            .rich_spans
            .iter()
            .map(|span| TextBufferSpanKey {
                text: span.text.clone(),
                color_bits: span.color.map(f32::to_bits),
                bold: span.bold,
                italic: span.italic,
            })
            .collect(),
        size_bits: run.size.to_bits(),
        face: run.face,
        width_px,
        height_px,
    }
}

pub(super) fn text_buffer_extent(
    run: &TextRun,
    surface_width: u32,
    surface_height: u32,
) -> (u32, u32) {
    let max_width = surface_width.max(1);
    let max_height = surface_height.max(1);
    let width = run.layout_size.map(|size| size.0).unwrap_or_else(|| {
        run.clip_bounds.map_or_else(
            || estimated_text_run_width_px(&run.text, run.size, run.face),
            |bounds| bounds.width.ceil().max(1.0),
        )
    });
    let height = run.layout_size.map(|size| size.1).unwrap_or_else(|| {
        run.clip_bounds.map_or_else(
            || run.size * 1.55 + 6.0,
            |bounds| bounds.height.ceil().max(1.0),
        )
    });
    (
        (width.ceil() as u32).clamp(1, max_width),
        (height.ceil() as u32).clamp(1, max_height),
    )
}

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

fn key_text_bytes(key: &TextBufferKey) -> usize {
    key.text.capacity()
        + key
            .rich_spans
            .iter()
            .map(|span| span.text.capacity())
            .sum::<usize>()
}

fn retain_overlay_buffers<T>(
    entries: &mut Vec<T>,
    age: impl Fn(&T) -> u64,
    size: impl Fn(&T) -> usize,
) -> bool {
    let reordered = entries.windows(2).any(|pair| age(&pair[0]) < age(&pair[1]));
    if reordered {
        entries.sort_by_key(|entry| std::cmp::Reverse(age(entry)));
    }
    let old_len = entries.len();
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
    reordered || entries.len() != old_len
}

pub(crate) struct TextBufferCache {
    published_revision: u64,
    published_bytes: usize,
    entries: Vec<CachedTextBuffer>,
    layout_scratch: LayoutScratch,
    layout_output_revision: u64,
    layout_output_bytes: usize,
    layout_output_tracking_bytes: usize,
    // Sorted shaping fingerprint + entry index. This owns no text or shaping
    // payload; exact key comparison remains authoritative within each bucket.
    lookup: Vec<(u64, usize)>,
    frame: u64,
    revision: u64,
    retained_revision: Option<u64>,
    #[cfg(test)]
    pub(crate) shape_reuses: usize,
    #[cfg(test)]
    pub(crate) key_comparisons: usize,
    // Drop the registry owner after all CPU cache storage has been released.
    owner: budget::Owner,
}

impl Default for TextBufferCache {
    fn default() -> Self {
        Self {
            owner: budget::Owner::new(std::mem::size_of::<Self>()),
            published_revision: 0,
            published_bytes: std::mem::size_of::<Self>(),
            entries: Vec::new(),
            layout_scratch: LayoutScratch::default(),
            layout_output_revision: 0,
            layout_output_bytes: 0,
            layout_output_tracking_bytes: 0,
            lookup: Vec::new(),
            frame: 0,
            revision: 0,
            retained_revision: None,
            #[cfg(test)]
            shape_reuses: 0,
            #[cfg(test)]
            key_comparisons: 0,
        }
    }
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

fn shape_fingerprint<'a>(
    text: &str,
    size_bits: u32,
    face: TextFace,
    spans: impl Iterator<Item = (&'a str, [u32; 3], bool, bool)>,
) -> u64 {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    (text, size_bits, face).hash(&mut hash);
    for span in spans {
        span.hash(&mut hash);
    }
    hash.finish()
}

fn run_fingerprint(run: &TextRun) -> u64 {
    shape_fingerprint(
        &run.text,
        run.size.to_bits(),
        run.face,
        run.rich_spans.iter().map(|span| {
            (
                span.text.as_str(),
                span.color.map(f32::to_bits),
                span.bold,
                span.italic,
            )
        }),
    )
}

impl TextBufferCache {
    pub(crate) fn key_usage(&self) -> crate::TextCacheKeyUsage {
        let mut shapes = std::collections::BTreeMap::new();
        let mut layout_bytes = 0;
        for entry in &self.entries {
            layout_bytes += entry.buffer.layout_storage_bytes();
            shapes.extend(entry.buffer.shape_allocations());
        }
        crate::TextCacheKeyUsage {
            owner_id: self.owner.id(),
            shaped_payload_bytes: layout_bytes + shapes.values().sum::<usize>(),
            entries: self.entries.len(),
            key_text_bytes: self
                .entries
                .iter()
                .map(|entry| key_text_bytes(&entry.key))
                .sum(),
            entry_storage_bytes: std::mem::size_of::<Self>()
                + capacity_bytes::<CachedTextBuffer>(self.entries.capacity())
                + capacity_bytes::<(u64, usize)>(self.lookup.capacity())
                + self
                    .entries
                    .iter()
                    .map(|entry| {
                        capacity_bytes::<TextBufferSpanKey>(entry.key.rich_spans.capacity())
                            + tracking_bytes::<u8>(entry.key.text.capacity())
                            + entry
                                .key
                                .rich_spans
                                .iter()
                                .map(|span| tracking_bytes::<u8>(span.text.capacity()))
                                .sum::<usize>()
                    })
                    .sum::<usize>(),
        }
    }

    fn publish_usage(&mut self) {
        if self.published_revision != self.revision {
            self.published_bytes = self.retained_payload_bytes();
            self.owner.publish(self.published_bytes);
            self.published_revision = self.revision;
        }
    }

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
                self.rebuild_lookup();
            }
        }
        self.publish_usage();
    }

    /// Called after glyph preparation/submission, when no text-area borrow is
    /// live. Bound retained buffers and key text; current-frame scratch can grow
    /// only with that frame's visible text. Glyph instances own their GPU data.
    pub(crate) fn trim_overlay(&mut self) {
        let changed = retain_overlay_buffers(
            &mut self.entries,
            |entry| entry.last_used_frame,
            |entry| key_text_bytes(&entry.key),
        );
        if changed {
            // Only actual retirement/reordering invalidates index signatures.
            self.revision = self.revision.wrapping_add(1);
            self.rebuild_lookup();
        }
    }

    /// Bound reusable CPU text after submission has consumed every layout run.
    /// Current preparation and private shaping scratch are separate.
    pub(crate) fn finish_frame(&mut self) {
        if self.retained_revision == Some(self.revision) {
            budget::submitted(self.owner.id());
            return;
        }
        budget::settle(self.owner.id(), |limit| {
            self.trim_payload_to(limit);
            self.published_bytes = self.retained_payload_bytes();
            self.published_bytes
        });
        self.published_revision = self.revision;
        self.retained_revision = Some(self.revision);
    }

    fn retained_payload_bytes(&self) -> usize {
        let usage = self.key_usage();
        usage.key_text_bytes + usage.entry_storage_bytes + usage.shaped_payload_bytes
    }

    fn trim_payload_to(&mut self, limit: usize) {
        if self.retained_payload_bytes() <= limit {
            return;
        }
        // Pressure is uncommon. Newest generations survive first; halving avoids
        // a quadratic full-payload recount for a large obsolete text generation.
        self.entries.sort_by_key(|entry| entry.last_used_frame);
        while !self.entries.is_empty() && self.retained_payload_bytes() > limit {
            let retire = self.entries.len().div_ceil(2);
            self.entries.drain(..retire);
            self.revision = self.revision.wrapping_add(1);
            self.rebuild_lookup();
        }
    }

    fn rebuild_lookup(&mut self) {
        // Retirement must release obsolete entry slots as well as their nested
        // buffers. Shrink only beyond 4x live rows to avoid allocation churn.
        // Moving retained entries preserves indices, keys and shaped payloads;
        // callers have already invalidated glyph signatures before this point.
        if self.entries.capacity() > self.entries.len().saturating_mul(4) {
            self.entries = std::mem::take(&mut self.entries)
                .into_boxed_slice()
                .into_vec();
        }
        self.lookup.clear();
        self.lookup
            .extend(self.entries.iter().enumerate().map(|(index, entry)| {
                let key = &entry.key;
                let fingerprint = shape_fingerprint(
                    &key.text,
                    key.size_bits,
                    key.face,
                    key.rich_spans
                        .iter()
                        .map(|span| (span.text.as_str(), span.color_bits, span.bold, span.italic)),
                );
                (fingerprint, index)
            }));
        self.lookup.sort_unstable();
        // Avoid retaining an index sized for an obsolete visible-text peak.
        // After retirement, metadata capacity is at most four times live rows;
        // a dialog's 128-row cap thus bounds this index to 512 tuple slots.
        if self.lookup.capacity() > self.lookup.len().saturating_mul(4) {
            self.lookup = std::mem::take(&mut self.lookup)
                .into_boxed_slice()
                .into_vec();
        }
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
        self.publish_usage();
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
        let fingerprint = run_fingerprint(run);
        let first = self.lookup.partition_point(|(hash, _)| *hash < fingerprint);
        let mut shaped = None;
        let mut reusable = None;
        for &(_, index) in self.lookup[first..]
            .iter()
            .take_while(|(hash, _)| *hash == fingerprint)
        {
            #[cfg(test)]
            {
                self.key_comparisons += 1;
            }
            let entry = &self.entries[index];
            let old_extent = (entry.key.width_px, entry.key.height_px);
            if matches_run(&entry.key, run, old_extent) {
                if old_extent == extent {
                    self.entries[index].last_used_frame = self.frame;
                    return (index, false);
                }
                shaped.get_or_insert(index);
                if entry.last_used_frame != self.frame {
                    reusable.get_or_insert(index);
                }
            }
        }
        if let Some(index) = reusable {
            // No current-frame text area references this entry. Relayout its
            // shared shaped paragraphs without cloning glyph payloads or retaining
            // an obsolete extent. The shaping fingerprint and index stay valid.
            let entry = &mut self.entries[index];
            entry
                .buffer
                .relayout(font_system, &mut self.layout_scratch, run, extent);
            entry.key.width_px = extent.0;
            entry.key.height_px = extent.1;
            entry.last_used_frame = self.frame;
            self.revision = self.revision.wrapping_add(1);
            #[cfg(test)]
            {
                self.shape_reuses += 1;
            }
            return (index, true);
        }
        let key = text_buffer_key(run, width, height);
        // Simultaneous extents share immutable shaping, never cloned glyph
        // vectors or copied paragraph strings. Each owns only its visible layout.
        let buffer = if let Some(index) = shaped {
            #[cfg(test)]
            {
                self.shape_reuses += 1;
            }
            let mut buffer = self.entries[index].buffer.fork_for_relayout();
            buffer.relayout(font_system, &mut self.layout_scratch, run, extent);
            buffer
        } else {
            TextLayout::new(font_system, &mut self.layout_scratch, run, extent)
        };
        self.revision = self.revision.wrapping_add(1);
        self.entries.push(CachedTextBuffer {
            key,
            buffer,
            last_used_frame: self.frame,
        });
        let index = self.entries.len() - 1;
        let insertion = self
            .lookup
            .partition_point(|item| *item < (fingerprint, index));
        self.lookup.insert(insertion, (fingerprint, index));
        (index, true)
    }
}

#[cfg(test)]
#[path = "text_buffer_cache_tests.rs"]
mod tests;
