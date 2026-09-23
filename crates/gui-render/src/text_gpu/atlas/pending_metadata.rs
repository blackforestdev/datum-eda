//! Atlas CPU record/page capacities share staging admission and fallible storage.
use super::*;

impl Atlas {
    pub(crate) fn lookup_metadata_bytes(&self) -> u64 {
        self.glyphs.allocated_bytes()
    }

    pub(crate) fn pending_metadata_bytes(&self) -> u64 {
        self.pending_uploads.allocated_bytes()
    }

    pub(crate) fn page_metadata_bytes(&self) -> u64 {
        self.pages.allocated_bytes()
    }

    pub(super) fn reserve_pending_metadata(&mut self) -> anyhow::Result<()> {
        if self.pending_uploads.len() < self.pending_uploads.capacity() {
            return Ok(());
        }
        let required = (self.pending_uploads.len() + 1).max(4);
        match self
            .pending_uploads
            .ensure_capacity(required, &self.staging_budget)
        {
            Err(_) if self.has_pending_uploads() => Err(UploadRequired.into()),
            result => result,
        }
    }

    pub(super) fn reserve_page_metadata(&mut self) -> anyhow::Result<()> {
        match self
            .pages
            .ensure_capacity(self.pages.len() + 1, &self.staging_budget)
        {
            Err(_) if self.has_pending_uploads() => Err(UploadRequired.into()),
            result => result,
        }
    }

    pub(super) fn release_empty_pending_metadata(&mut self) {
        if self.pending_uploads.is_empty() {
            self.pending_uploads = Default::default();
        }
    }
}
