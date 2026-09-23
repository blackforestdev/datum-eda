//! Pending upload records share the same admission as their pixel payloads.
use super::*;

impl Atlas {
    pub(crate) fn pending_metadata_bytes(&self) -> u64 {
        crate::cpu_alloc::heap::capacity_bytes::<PendingUpload>(self.pending_uploads.capacity())
            as u64
    }

    pub(super) fn reserve_pending_metadata(&mut self) -> anyhow::Result<()> {
        if self.pending_uploads.len() < self.pending_uploads.capacity() {
            return Ok(());
        }
        let capacity = self.pending_uploads.capacity().saturating_mul(2).max(4);
        let bytes = crate::cpu_alloc::heap::capacity_bytes::<PendingUpload>(capacity) as u64;
        let reserve = || -> anyhow::Result<_> {
            Ok([
                self.staging_budget.reserve(bytes)?,
                super::super::budget::staging_process().reserve(bytes)?,
            ])
        };
        let permits = match reserve() {
            Err(_) if self.has_pending_uploads() => return Err(UploadRequired.into()),
            result => result?,
        };
        // Keep old capacity charged until it is freed; realloc-style admission
        // would miss the old/new storage overlap during growth.
        let mut replacement = Vec::with_capacity(capacity);
        replacement.append(&mut self.pending_uploads);
        self.pending_uploads = replacement;
        self.pending_metadata_permits = Some(permits);
        Ok(())
    }

    pub(super) fn release_empty_pending_metadata(&mut self) {
        if self.pending_uploads.is_empty() {
            self.pending_uploads = Vec::new();
            self.pending_metadata_permits = None;
        }
    }
}
