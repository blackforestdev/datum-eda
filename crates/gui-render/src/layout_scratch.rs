//! Track private layout scratch without inspecting dependency internals.
use super::allocation::{Admission, ProducedLine};
use glyphon::ShapeLine;

pub(crate) struct LayoutScratch {
    scope: crate::cpu_alloc::Scope,
    permits: Option<[crate::text_gpu::budget::Permit; 2]>,
    host: Option<std::sync::Arc<crate::text_gpu::budget::Budget>>,
    charged_bytes: u64,
}

impl Default for LayoutScratch {
    fn default() -> Self {
        Self {
            scope: crate::cpu_alloc::Scope::new("layout-scratch-and-output"),
            permits: None,
            host: None,
            charged_bytes: 0,
        }
    }
}

impl LayoutScratch {
    /// Stop at the first row rejected by the visible-extent consumer. Neither
    /// later row boundaries nor their output glyph storage are constructed.
    pub(super) fn for_each_row(
        &mut self,
        shape: &ShapeLine,
        size: f32,
        width: u32,
        admission: &Admission<'_>,
        mut consume: impl FnMut(ProducedLine) -> anyhow::Result<bool>,
    ) -> anyhow::Result<bool> {
        let mut rows = super::streaming_rows::Rows::new(shape, size, width, admission);
        while let Some(row) = self.scope.with(|| rows.next())? {
            let output = self.scope.with(|| rows.layout(&row))?;
            if !consume(output)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Call only after output containers have been consumed into cache rows.
    /// Every retained output glyph vector is public and counted by that cache;
    /// subtract its payload and tracking bytes to isolate transient Datum layout storage.
    pub(crate) fn private_bytes(
        &self,
        output_bytes: usize,
        output_tracking_bytes: usize,
    ) -> Option<u64> {
        let usage = self.scope.usage();
        usage.allocator_installed.then(|| {
            (usage.payload_bytes + usage.tracking_bytes)
                .checked_sub((output_bytes + output_tracking_bytes) as u64)
                .expect("layout outputs must belong to this scratch scope")
        })
    }

    pub(crate) fn admit(
        &mut self,
        output_bytes: usize,
        output_tracking_bytes: usize,
        host: &std::sync::Arc<crate::text_gpu::budget::Budget>,
    ) {
        let Some(bytes) = self.private_bytes(output_bytes, output_tracking_bytes) else {
            return;
        };
        if self.permits.is_some()
            && self.charged_bytes == bytes
            && self
                .host
                .as_ref()
                .is_some_and(|old| std::sync::Arc::ptr_eq(old, host))
        {
            return;
        }
        self.permits = None;
        let reserve = || -> anyhow::Result<[crate::text_gpu::budget::Permit; 2]> {
            Ok([
                host.reserve(bytes)?,
                crate::text_gpu::budget::staging_process().reserve(bytes)?,
            ])
        };
        match reserve() {
            Ok(permits) => {
                self.permits = Some(permits);
                self.host = Some(host.clone());
                self.charged_bytes = bytes;
            }
            // Only reusable scratch is discarded; returned glyph layouts survive.
            Err(_) => self.clear(),
        }
    }

    pub(crate) fn reserved_bytes(&self) -> u64 {
        self.charged_bytes
    }

    pub(crate) fn clear(&mut self) {
        self.permits = None;
        self.host = None;
        self.charged_bytes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipped_prefix_layout_peak_is_independent_of_hidden_paragraph_rows() {
        let mut fonts = crate::load_datum_fonts();
        let attrs = glyphon::AttrsList::new(&crate::text_attrs(crate::TextFace::Ui));
        let mut peaks = Vec::new();
        for repeats in [20, 2000] {
            let text = "a visible row followed by hidden content ".repeat(repeats);
            let shape =
                glyphon::ShapeLine::new(&mut fonts, &text, &attrs, glyphon::Shaping::Basic, 8);
            let mut scratch = LayoutScratch::default();
            let mut count = 0;
            let host = crate::text_gpu::budget::Budget::new(16 * 1024 * 1024);
            let admission = Admission {
                owner: None,
                host: &host,
            };
            assert!(
                !scratch
                    .for_each_row(&shape, 14.0, 120, &admission, |_| {
                        count += 1;
                        Ok(count < 3)
                    })
                    .unwrap()
            );
            assert_eq!(count, 3);
            let usage = scratch.scope.usage();
            assert!(usage.allocator_installed);
            assert_eq!(usage.payload_bytes + usage.tracking_bytes, 0);
            peaks.push(usage.peak_payload_bytes);
        }
        assert!(peaks[0] > 0);
        assert_eq!(
            peaks[0], peaks[1],
            "hidden rows must not grow layout output or scratch"
        );
    }

    #[test]
    fn private_layout_capacity_is_separate_from_outputs_and_evictable() {
        let mut fonts = crate::load_datum_fonts();
        let mut scratch = LayoutScratch::default();
        let run = crate::TextRun {
            text: "wrapped private scratch and surviving public glyphs ".repeat(40),
            rich_spans: Vec::new(),
            x: 0.0,
            y: 0.0,
            size: 14.0,
            color: [1.0; 3],
            face: crate::TextFace::Ui,
            clip_bounds: None,
            layout_size: None,
        };
        let mut layout =
            crate::text_layout::TextLayout::new(&mut fonts, &mut scratch, &run, (200, 300));
        let output = layout.layout_glyph_bytes();
        let output_tracking = layout.layout_glyph_tracking_bytes();
        assert!(output > 0);
        let private = scratch
            .private_bytes(output, output_tracking)
            .expect("test allocator installed");
        assert_eq!(private, 0, "streaming layout retains no private scratch");
        assert_eq!(
            scratch.scope.usage().payload_bytes + scratch.scope.usage().tracking_bytes,
            (output + output_tracking) as u64
        );
        let host = crate::text_gpu::budget::Budget::new(0);
        scratch.admit(output, output_tracking, &host);
        assert_eq!(host.used(), 0);
        let before: Vec<_> = layout
            .layout_runs()
            .map(|r| format!("{:?}", r.glyphs))
            .collect();
        // Refuse retention, not the already constructed required layout.
        scratch.admit(
            output,
            output_tracking,
            &crate::text_gpu::budget::Budget::new(0),
        );
        assert_eq!(host.used(), 0);
        assert_eq!(scratch.private_bytes(output, output_tracking), Some(0));
        assert_eq!(scratch.scope.usage().payload_bytes, output as u64);
        let after: Vec<_> = layout
            .layout_runs()
            .map(|r| format!("{:?}", r.glyphs))
            .collect();
        assert_eq!(before, after);
        layout.relayout(&mut fonts, &mut scratch, &run, (200, 300));
        let after: Vec<_> = layout
            .layout_runs()
            .map(|r| format!("{:?}", r.glyphs))
            .collect();
        assert_eq!(before, after);
        drop(layout);
        let private = scratch.private_bytes(0, 0).unwrap();
        assert_eq!(
            scratch.scope.usage().payload_bytes + scratch.scope.usage().tracking_bytes,
            private
        );
        scratch.clear();
        assert_eq!(scratch.scope.usage().payload_bytes, 0);
    }
}
