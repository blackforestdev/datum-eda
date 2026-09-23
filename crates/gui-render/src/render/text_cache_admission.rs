//! Current layout admission; refusal releases derived storage, never source text.
use super::*;

pub(crate) trait FrameIndices {
    fn indices_mut(&mut self) -> &mut [usize];
    fn clear_indices(&mut self);
}
impl FrameIndices for crate::text_gpu::staging_vec::StagingVec<usize> {
    fn indices_mut(&mut self) -> &mut [usize] {
        self
    }
    fn clear_indices(&mut self) {
        self.clear();
    }
}
#[cfg(test)]
impl FrameIndices for Vec<usize> {
    fn indices_mut(&mut self) -> &mut [usize] {
        self
    }
    fn clear_indices(&mut self) {
        self.clear();
    }
}

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

    /// On refusal, cached layouts and every supplied index group are discarded.
    /// Retry must rebuild them from current TextRun input before requesting admission.
    pub(crate) fn admit_frame(
        &mut self,
        indices: &mut [&mut impl FrameIndices],
    ) -> anyhow::Result<()> {
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
                        for index in group.indices_mut().iter_mut() {
                            *index = remap[*index];
                            assert_ne!(*index, usize::MAX, "current text index was not pinned");
                        }
                    }
                    self.revision = self.revision.wrapping_add(1);
                    self.rebuild_lookup();
                }
            }
            if self.admission_payload_bytes() > limit {
                // Current layouts may fit once obsolete spare metadata slots
                // retire. Moving entries preserves their indices and shared
                // shape allocations; never reject solely for reusable slack.
                let changed = self.entries.capacity() > self.entries.len()
                    || self.lookup.capacity() > self.lookup.len();
                if changed {
                    self.entries = std::mem::take(&mut self.entries)
                        .into_boxed_slice()
                        .into_vec();
                    self.lookup = std::mem::take(&mut self.lookup)
                        .into_boxed_slice()
                        .into_vec();
                    self.revision = self.revision.wrapping_add(1);
                }
            }
            let required_bytes = self.admission_payload_bytes();
            if required_bytes > limit {
                // The frame will return an error before borrowing layout rows or
                // preparing glyphs. TextRun/model input remains authoritative for
                // retry; retaining rejected derived capacity cannot relieve pressure.
                self.entries = Vec::new();
                self.lookup = Vec::new();
                self.layout_scratch.clear();
                self.revision = self.revision.wrapping_add(1);
                self.layout_output_revision = self.revision;
                self.layout_output_bytes = 0;
                self.layout_output_tracking_bytes = 0;
                self.retained_revision = None;
                for group in indices.iter_mut() {
                    group.clear_indices();
                }
                self.published_bytes = self.retained_payload_bytes();
            } else {
                self.published_bytes = required_bytes;
            }
            budget::Admission {
                required_bytes,
                retained_bytes: self.published_bytes,
            }
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
