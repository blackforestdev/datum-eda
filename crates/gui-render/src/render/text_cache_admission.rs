//! Required current-frame layout admission without evicting visible text.
use super::*;

impl TextBufferCache {
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
