//! Cold text preparation with admitted transient rich input; exact hits stay borrowed.
use super::*;

impl TextBufferCache {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn fill_indices(
        &mut self,
        font_system: &mut impl crate::text_layout::fonts::Source,
        text_runs: &[TextRun],
        width: u32,
        height: u32,
        overlay: bool,
        indices: &mut impl Extend<usize>,
        host: &std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) -> anyhow::Result<TextBufferCacheStats> {
        let mut stats = TextBufferCacheStats::default();
        for run in text_runs {
            let (index, missed) =
                match self.ensure_text_buffer(font_system, run, width, height, host) {
                    Ok(result) => result,
                    Err(error) => {
                        // Earlier successful entries remain owned even if this batch refuses.
                        self.publish_usage();
                        return Err(error);
                    }
                };
            let entry = &mut self.entries[index];
            if overlay {
                entry.overlay_retained = true;
                entry.last_overlay_frame = self.frame;
            } else {
                entry.last_workspace_frame = self.frame;
            }
            if missed {
                stats.misses += 1;
            } else {
                stats.hits += 1;
            }
            indices.extend(std::iter::once(index));
        }
        self.publish_usage();
        Ok(stats)
    }

    fn ensure_text_buffer(
        &mut self,
        font_system: &mut impl crate::text_layout::fonts::Source,
        run: &TextRun,
        width: u32,
        height: u32,
        host: &std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) -> anyhow::Result<(usize, bool)> {
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
                    return Ok((index, false));
                }
                shaped.get_or_insert(index);
                if entry.last_used_frame != self.frame {
                    reusable.get_or_insert(index);
                }
            }
        }
        // The only concatenated input is temporary and admitted before allocation.
        // Exact hits above need no concatenation or staging permit.
        let needs_input = reusable
            .or(shaped)
            .is_none_or(|index| self.entries[index].buffer.needs_input());
        let spans = if needs_input {
            run.rich_spans.as_slice()
        } else {
            &[]
        };
        let len = spans.iter().try_fold(0usize, |len, span| {
            len.checked_add(span.text.len())
                .ok_or_else(|| anyhow::anyhow!("rich text input size overflow"))
        })?;
        let bytes = crate::text_gpu::staging_vec::StagingVec::<u8>::capacity_bytes(len)?;
        self.release_layout_scratch_for(bytes, host);
        font_system.release_for(bytes);
        let mut input = crate::text_gpu::staging_vec::StagingVec::new(len, host)?;
        for span in spans {
            input.extend_from_slice(span.text.as_bytes());
        }
        let rich_text = std::str::from_utf8(&input).expect("concatenated UTF-8 spans");
        if let Some(index) = reusable {
            // No current-frame text area references this entry. Relayout its
            // shared shaped paragraphs without cloning glyph payloads or retaining
            // an obsolete extent. The shaping fingerprint and index stay valid.
            let entry = &mut self.entries[index];
            entry.buffer.relayout_with_input(
                font_system,
                &mut self.layout_scratch,
                run,
                extent,
                rich_text,
            );
            entry.key.width_px = extent.0;
            entry.key.height_px = extent.1;
            entry.last_used_frame = self.frame;
            self.revision = self.revision.wrapping_add(1);
            #[cfg(test)]
            {
                self.shape_reuses += 1;
            }
            return Ok((index, true));
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
            buffer.relayout_with_input(
                font_system,
                &mut self.layout_scratch,
                run,
                extent,
                rich_text,
            );
            buffer
        } else {
            TextLayout::with_input(
                font_system,
                &mut self.layout_scratch,
                run,
                extent,
                rich_text,
            )
        };
        self.revision = self.revision.wrapping_add(1);
        self.entries.push(CachedTextBuffer {
            key,
            buffer,
            last_used_frame: self.frame,
            last_workspace_frame: 0,
            last_overlay_frame: 0,
            overlay_retained: false,
        });
        let index = self.entries.len() - 1;
        let insertion = self
            .lookup
            .partition_point(|item| *item < (fingerprint, index));
        self.lookup.insert(insertion, (fingerprint, index));
        Ok((index, true))
    }
}
