//! Reusable glyph preparation identity and continuation lifetime.
use super::*;
use crate::text_buffer_cache::TextBufferCache;

#[derive(Default)]
pub(crate) struct GlyphPreparation {
    pub(super) upload_continuation: bool,
    pub(super) prepared: Option<(u64, u64, text_prepare_identity::AdmittedSignature)>,
    pub(super) overlay_prepared: Option<(u64, u64, text_prepare_identity::AdmittedSignature)>,
    #[cfg(test)]
    pub(crate) forced_overlay_errors: usize,
    #[cfg(test)]
    pub(crate) workspace_prepares: usize,
    #[cfg(test)]
    pub(crate) atlas_retries: usize,
    #[cfg(test)]
    pub(crate) overlay_prepares: usize,
}

impl GlyphPreparation {
    pub(crate) fn is_continuing(&self) -> bool {
        self.upload_continuation
    }

    pub(crate) fn cancel(&mut self) {
        self.upload_continuation = false;
        self.prepared = None;
        self.overlay_prepared = None;
    }

    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn force_overlay_errors(&mut self, count: usize) {
        self.overlay_prepared = None;
        self.forced_overlay_errors = count;
    }

    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn is_invalid(&self) -> bool {
        self.prepared.is_none()
    }
}

/// Shared pressure policy, using disjoint field borrows while upload plans borrow vertices.
pub(super) fn release_scratch(
    layouts: &mut TextBufferCache,
    raster: &mut crate::text_gpu::raster::Raster,
    fonts: &mut crate::text_layout::fonts::Fonts,
    host: &std::sync::Arc<crate::text_gpu::budget::Budget>,
    bytes: u64,
) {
    use crate::text_layout::fonts::Source;
    layouts.release_layout_scratch_for(bytes, host);
    raster.release_for(bytes, host);
    fonts.release_for(bytes);
}
