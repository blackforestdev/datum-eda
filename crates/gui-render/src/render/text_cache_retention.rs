//! Workspace and label references share shaped storage with separate retention.
use super::*;

impl TextBufferCache {
    pub(crate) fn indices(
        &mut self,
        font_system: &mut FontSystem,
        text_runs: &[TextRun],
        width: u32,
        height: u32,
    ) -> (Vec<usize>, TextBufferCacheStats) {
        self.indices_for(font_system, text_runs, width, height, self.overlay_profile)
    }

    pub(crate) fn overlay_indices(
        &mut self,
        font_system: &mut FontSystem,
        text_runs: &[TextRun],
        width: u32,
        height: u32,
    ) -> (Vec<usize>, TextBufferCacheStats) {
        self.indices_for(font_system, text_runs, width, height, true)
    }

    /// Workspace/terminal keeps two generations; dialog history is trimmed
    /// after submission so every current-frame text-area borrow stays valid.
    pub(crate) fn begin_frame(&mut self, profile: Profile) {
        self.frame = self.frame.wrapping_add(1).max(1);
        self.overlay_profile = matches!(profile, Profile::Overlay);
        let old_len = self.entries.len();
        let frame = self.frame;
        self.entries.retain(|entry| {
            entry.overlay_retained
                || (entry.last_workspace_frame != 0
                    && text_buffer_frame_is_recent(entry.last_workspace_frame, frame))
        });
        if self.entries.len() != old_len {
            self.revision = self.revision.wrapping_add(1);
            self.rebuild_lookup();
        }
        self.publish_usage();
        budget::preparing(self.owner.id());
    }

    /// Called after glyph preparation/submission, when no text-area borrow is
    /// live. Bound retained buffers and key text; current-frame scratch can grow
    /// only with that frame's visible text. Glyph instances own their GPU data.
    pub(crate) fn trim_overlay(&mut self) {
        let labels = self.entries.iter().filter(|entry| entry.overlay_retained);
        let (count, bytes) = labels.fold((0, 0), |(count, bytes), entry| {
            (count + 1, bytes + key_text_bytes(&entry.key))
        });
        if count <= MAX_OVERLAY_BUFFERS && bytes <= MAX_OVERLAY_TEXT_BYTES {
            return;
        }
        let mut changed = self
            .entries
            .windows(2)
            .any(|pair| pair[0].last_overlay_frame < pair[1].last_overlay_frame);
        if changed {
            self.entries
                .sort_by_key(|entry| std::cmp::Reverse(entry.last_overlay_frame));
        }
        let (mut count, mut bytes) = (0, 0);
        let frame = self.frame;
        self.entries.retain_mut(|entry| {
            if entry.overlay_retained {
                let next = bytes + key_text_bytes(&entry.key);
                if count >= MAX_OVERLAY_BUFFERS || next > MAX_OVERLAY_TEXT_BYTES {
                    entry.overlay_retained = false;
                    changed = true;
                } else {
                    count += 1;
                    bytes = next;
                }
            }
            entry.overlay_retained
                || (entry.last_workspace_frame != 0
                    && text_buffer_frame_is_recent(entry.last_workspace_frame, frame))
        });
        if changed {
            // Only actual retirement/reordering invalidates index signatures.
            self.revision = self.revision.wrapping_add(1);
            self.rebuild_lookup();
        }
    }
}
