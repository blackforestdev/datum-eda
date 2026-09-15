// Glyph-buffer cache helpers for the `Renderer`, extracted from `gpu.rs` to keep
// it under its source-health ceiling (decision 022) as S4 threads the schematic
// interaction underlay through it. A real `#[path] mod` child of the crate root
// (declared in `gpu.rs`), so this inherent-impl block reaches the `Renderer`'s
// private fields and the crate-root text types/helpers via `use super::*` exactly
// as the inline methods did. Behaviour is unchanged — a verbatim move.

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

impl Renderer {
    /// Retain shaped buffers used by the immediately preceding frame only.
    /// Agent TUIs continuously rewrite status lines; retaining every historical
    /// whole-string buffer made lookup progressively slower and memory grow
    /// without bound. Current and previous-frame residency preserves stable
    /// frame reuse while bounding churn by visible scene complexity.
    pub(crate) fn begin_text_buffer_frame(&mut self) {
        self.text_buffer_frame = self.text_buffer_frame.wrapping_add(1).max(1);
        retain_recent_text_buffers(
            &mut self.text_buffer_cache,
            self.text_buffer_frame,
            |entry| entry.last_used_frame,
        );
    }

    pub(crate) fn cached_text_buffer_indices(
        &mut self,
        text_runs: &[TextRun],
        width: u32,
        height: u32,
    ) -> (Vec<usize>, TextBufferCacheStats) {
        let mut indices = Vec::with_capacity(text_runs.len());
        let mut stats = TextBufferCacheStats::default();
        for run in text_runs {
            let (index, missed) = self.ensure_text_buffer(run, width, height);
            if missed {
                stats.misses += 1;
            } else {
                stats.hits += 1;
            }
            indices.push(index);
        }
        (indices, stats)
    }

    fn ensure_text_buffer(&mut self, run: &TextRun, width: u32, height: u32) -> (usize, bool) {
        let key = text_buffer_key(run, width, height);
        if let Some(index) = self
            .text_buffer_cache
            .iter()
            .position(|entry| entry.key == key)
        {
            self.text_buffer_cache[index].last_used_frame = self.text_buffer_frame;
            return (index, false);
        }
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(run.size, run.size * 1.22),
        );
        let (buffer_width, buffer_height) = text_buffer_extent(run, width, height);
        buffer.set_size(
            &mut self.font_system,
            Some(buffer_width as f32),
            Some(buffer_height as f32),
        );
        let attrs = text_attrs(run.face);
        if run.rich_spans.is_empty() {
            buffer.set_text(
                &mut self.font_system,
                &run.text,
                &attrs,
                Shaping::Basic,
                None,
            );
        } else {
            buffer.set_rich_text(
                &mut self.font_system,
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
        buffer.shape_until_scroll(&mut self.font_system, false);
        self.text_buffer_cache.push(CachedTextBuffer {
            key,
            buffer,
            last_used_frame: self.text_buffer_frame,
        });
        (self.text_buffer_cache.len() - 1, true)
    }
}

#[cfg(test)]
mod tests {
    use super::retain_recent_text_buffers;

    #[derive(Debug)]
    struct SimulatedBuffer {
        key: String,
        last_used_frame: u64,
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
