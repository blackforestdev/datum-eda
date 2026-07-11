// Glyph-buffer cache helpers for the `Renderer`, extracted from `gpu.rs` to keep
// it under its source-health ceiling (decision 022) as S4 threads the schematic
// interaction underlay through it. A real `#[path] mod` child of the crate root
// (declared in `gpu.rs`), so this inherent-impl block reaches the `Renderer`'s
// private fields and the crate-root text types/helpers via `use super::*` exactly
// as the inline methods did. Behaviour is unchanged — a verbatim move.

use super::*;

impl Renderer {
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
        buffer.set_text(
            &mut self.font_system,
            &run.text,
            &attrs,
            Shaping::Basic,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);
        self.text_buffer_cache
            .push(CachedTextBuffer { key, buffer });
        (self.text_buffer_cache.len() - 1, true)
    }
}
