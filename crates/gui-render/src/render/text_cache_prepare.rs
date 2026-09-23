//! Cold text preparation with admitted transient rich input; exact hits stay borrowed.
use super::*;

impl TextBufferCache {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn fill_indices<I: Extend<usize>>(
        &mut self,
        font_system: &mut impl crate::text_layout::fonts::Source,
        text_runs: &[TextRun],
        width: u32,
        height: u32,
        overlay: bool,
        indices: &mut I,
        host: &std::sync::Arc<crate::text_gpu::budget::Budget>,
        construction_admission: bool,
        mut admit: impl FnMut(&mut Self, &mut I, usize) -> anyhow::Result<()>,
    ) -> anyhow::Result<TextBufferCacheStats> {
        let mut stats = TextBufferCacheStats::default();
        for run in text_runs {
            let mut retired_history = false;
            let (index, missed) = loop {
                match self.ensure_text_buffer(
                    font_system,
                    run,
                    width,
                    height,
                    host,
                    construction_admission,
                ) {
                    Ok(result) => break result,
                    Err(error) => {
                        // A failed in-place relayout is never a cache hit. Publish
                        // its surviving storage before releasing construction leases.
                        self.revision = self.revision.wrapping_add(1);
                        self.publish_usage();
                        self.entry_construction = None;
                        self.lookup_construction = None;
                        for entry in &mut self.entries {
                            entry.buffer.published();
                        }
                        if !retired_history
                            && let Some(refusal) =
                                error.downcast_ref::<budget::ConstructionRefusal>()
                        {
                            admit(self, indices, refusal.bytes)?;
                            retired_history = true;
                            continue;
                        }
                        self.clear_derived_storage();
                        self.publish_usage();
                        return Err(error);
                    }
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
            if missed {
                admit(self, indices, 0)?;
            }
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
        construction_admission: bool,
    ) -> anyhow::Result<(usize, bool)> {
        self.publish_usage();
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
                if old_extent == extent && entry.buffer.is_valid() {
                    self.entries[index].last_used_frame = self.frame;
                    return Ok((index, false));
                }
                shaped.get_or_insert(index);
                if entry.last_used_frame != self.frame || !entry.buffer.is_valid() {
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
            let before = self.entries[index].buffer.layout_storage_bytes();
            self.entries[index].buffer.discard_rows();
            let old_layout = self.entries[index].buffer.layout_storage_bytes();
            self.publish_preparation_delta(before, old_layout);
            let entry = &mut self.entries[index];
            let old_shapes = entry.buffer.shape_allocations().count();
            entry.buffer.relayout_with_input(
                font_system,
                &mut self.layout_scratch,
                run,
                extent,
                rich_text,
                &crate::text_layout::Admission {
                    owner: construction_admission.then_some(&self.owner),
                    host,
                },
            )?;
            entry.key.width_px = extent.0;
            entry.key.height_px = extent.1;
            entry.last_used_frame = self.frame;
            let new_bytes = entry.buffer.layout_storage_bytes()
                + entry
                    .buffer
                    .shape_allocations()
                    .skip(old_shapes)
                    .map(|(_, bytes)| bytes)
                    .sum::<usize>();
            self.publish_preparation_delta(old_layout, new_bytes);
            self.entries[index].buffer.published();
            #[cfg(test)]
            {
                self.shape_reuses += 1;
            }
            return Ok((index, true));
        }
        let old_capacity = self.preparation_metadata_bytes();
        let shared_shapes = shaped.map_or(0, |index| {
            self.entries[index].buffer.shape_allocations().count()
        });
        let owner = construction_admission.then_some(&self.owner);
        construction::reserve_slot(&mut self.entries, &mut self.entry_construction, owner)?;
        construction::reserve_slot(&mut self.lookup, &mut self.lookup_construction, owner)?;
        let key = construction::key(run, width, height, owner)?;
        // Simultaneous extents share immutable shaping, never cloned glyph
        // vectors or copied paragraph strings. Each owns only its visible layout.
        let buffer = if let Some(index) = shaped {
            #[cfg(test)]
            {
                self.shape_reuses += 1;
            }
            let mut buffer = self.entries[index]
                .buffer
                .fork_for_relayout(&crate::text_layout::Admission { owner, host })?;
            buffer.relayout_with_input(
                font_system,
                &mut self.layout_scratch,
                run,
                extent,
                rich_text,
                &crate::text_layout::Admission {
                    owner: construction_admission.then_some(&self.owner),
                    host,
                },
            )?;
            buffer
        } else {
            TextLayout::with_input(
                font_system,
                &mut self.layout_scratch,
                run,
                extent,
                rich_text,
                &crate::text_layout::Admission {
                    owner: construction_admission.then_some(&self.owner),
                    host,
                },
            )?
        };
        let added = key_text_bytes(&key.key)
            + capacity_bytes::<TextBufferSpanKey>(key.key.rich_spans.capacity())
            + tracking_bytes::<u8>(key.key.text.capacity())
            + key
                .key
                .rich_spans
                .iter()
                .map(|span| tracking_bytes::<u8>(span.text.capacity()))
                .sum::<usize>()
            + buffer.layout_storage_bytes()
            + buffer
                .shape_allocations()
                .skip(shared_shapes)
                .map(|(_, bytes)| bytes)
                .sum::<usize>();
        self.entries.push(CachedTextBuffer {
            key: key.key,
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
        self.publish_preparation_delta(old_capacity, self.preparation_metadata_bytes() + added);
        self.entries[index].buffer.published();
        Ok((index, true))
    }
    fn preparation_metadata_bytes(&self) -> usize {
        capacity_bytes::<CachedTextBuffer>(self.entries.capacity())
            + capacity_bytes::<(u64, usize)>(self.lookup.capacity())
    }

    fn publish_preparation_delta(&mut self, removed: usize, added: usize) {
        self.published_bytes = self
            .published_bytes
            .checked_sub(removed)
            .and_then(|bytes| bytes.checked_add(added))
            .expect("text preparation accounting overflow");
        self.revision = self.revision.wrapping_add(1);
        self.published_revision = self.revision;
        self.owner.publish(self.published_bytes);
        self.entry_construction = None;
        self.lookup_construction = None;
    }
}

#[cfg(test)]
#[path = "text_batch_admission_tests.rs"]
mod tests;
