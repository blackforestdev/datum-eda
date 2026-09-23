//! Retained raster pixels share host/process staging admission with GPU copies.
use super::*;

pub(super) const CHUNK_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct UploadRequired;
impl std::fmt::Display for UploadRequired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("upload pending glyph pixels before continuing raster preparation")
    }
}
impl std::error::Error for UploadRequired {}

impl Atlas {
    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn rasterization_count(&self) -> u64 {
        self.uploads.rasterizations
    }

    pub(crate) fn pending_cpu_bytes(&self) -> u64 {
        self.pending_uploads
            .iter()
            .map(|p| crate::cpu_alloc::heap::capacity_bytes::<u8>(p.pixels.capacity()) as u64)
            .sum()
    }

    pub(super) fn reserve_cpu_image(
        &self,
        bytes: u64,
        padded: u64,
    ) -> anyhow::Result<[super::super::budget::Permit; 2]> {
        let reserve = || -> anyhow::Result<_> {
            let process = super::super::budget::staging_process();
            let cpu = [self.staging_budget.reserve(bytes)?, process.reserve(bytes)?];
            // A retained image must leave room for its first bounded GPU copy.
            // These are preflight reservations; actual copies reserve atomically
            // again at submission and preserve pending pixels on refusal.
            let copy = (self.pending_staging_bytes() + padded).min(CHUNK_BYTES);
            let _host_copy = self.staging_budget.reserve(copy)?;
            let _process_copy = process.reserve(copy)?;
            Ok(cpu)
        };
        match reserve() {
            Err(_) if self.has_pending_uploads() => Err(UploadRequired.into()),
            result => result,
        }
    }
}

#[cfg(all(test, feature = "visual"))]
#[path = "cpu_images_tests.rs"]
mod tests;
