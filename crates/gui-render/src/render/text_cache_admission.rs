//! Required current-frame layout admission without evicting visible text.
use super::*;

impl TextBufferCache {
    pub(crate) fn admit_layout_scratch(
        &mut self,
        host: &std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) {
        if self.layout_output_revision != self.revision {
            self.layout_output_bytes = self
                .entries
                .iter()
                .map(|entry| entry.buffer.layout_glyph_bytes())
                .sum();
            self.layout_output_tracking_bytes = self
                .entries
                .iter()
                .map(|entry| entry.buffer.layout_glyph_tracking_bytes())
                .sum();
            self.layout_output_revision = self.revision;
        }
        self.layout_scratch.admit(
            self.layout_output_bytes,
            self.layout_output_tracking_bytes,
            host,
        );
    }

    pub(crate) fn release_layout_scratch_for(
        &mut self,
        bytes: u64,
        host: &std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) {
        if bytes > host.available()
            || bytes > crate::text_gpu::budget::staging_process().available()
        {
            self.layout_scratch.clear();
        }
    }

    fn admission_payload_bytes(&self) -> usize {
        if self.published_revision == self.revision {
            self.published_bytes
        } else {
            self.retained_payload_bytes()
        }
    }

    pub(crate) fn admit_frame(&mut self, indices: &mut [&mut Vec<usize>]) -> anyhow::Result<()> {
        let result = budget::admit(self.owner.id(), |limit| {
            if self.admission_payload_bytes() > limit {
                // All current-frame indices remain pinned. Unused history may
                // retire; remap indices atomically before any text-area borrow.
                let mut remap = vec![usize::MAX; self.entries.len()];
                let mut next = 0;
                let frame = self.frame;
                let mut old = 0;
                self.entries.retain(|entry| {
                    let keep = entry.last_used_frame == frame;
                    if keep {
                        remap[old] = next;
                        next += 1;
                    }
                    old += 1;
                    keep
                });
                if next != old {
                    for group in indices.iter_mut() {
                        for index in group.iter_mut() {
                            *index = remap[*index];
                            assert_ne!(*index, usize::MAX, "current text index was not pinned");
                        }
                    }
                    self.revision = self.revision.wrapping_add(1);
                    self.rebuild_lookup();
                }
            }
            self.published_bytes = self.admission_payload_bytes();
            self.published_bytes
        });
        self.published_revision = self.revision;
        result
    }
}

#[cfg(test)]
#[path = "text_cache_admission_tests.rs"]
mod tests;

impl crate::Renderer {
    /// Private retained layout scratch charged to the shared staging/scratch cap.
    /// Returned glyph vectors remain in public shaped-payload accounting.
    pub fn layout_scratch_reserved_bytes(&self) -> u64 {
        self.text_buffers.layout_scratch.reserved_bytes()
    }
}
