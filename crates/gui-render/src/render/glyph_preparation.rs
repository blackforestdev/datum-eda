//! Reusable glyph preparation identity and continuation lifetime.
use super::*;

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
