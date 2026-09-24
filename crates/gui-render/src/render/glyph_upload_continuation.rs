//! Yield oversized mixed text uploads before encoding a frame or allocating queries.
use super::*;
const CHUNK_BYTES: u64 = 4 * 1024 * 1024;

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_text_uploads(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
        overlay: bool,
        on_submitted: &mut dyn FnMut(wgpu::SubmissionIndex),
    ) -> anyhow::Result<Option<(TextBufferCacheStats, bool)>> {
        self.release_text_scratch_for(CHUNK_BYTES);

        match self.prepare_frame_text(device, queue, prepared, width, height, overlay) {
            Ok(result) => {
                if self.start_glyph_upload(device, queue, on_submitted)? {
                    Ok(None)
                } else {
                    Ok(Some(result))
                }
            }
            Err(error) if error.is::<crate::text_gpu::UploadRequired>() => {
                self.text_preparation.upload_continuation = true;
                anyhow::ensure!(
                    self.resume_glyph_upload(device, queue, on_submitted)?,
                    "glyph preparation yielded without pending uploads"
                );
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    pub(crate) fn start_glyph_upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        on_submitted: &mut dyn FnMut(wgpu::SubmissionIndex),
    ) -> anyhow::Result<bool> {
        let atlas_bytes = self.atlas.pending_staging_bytes();
        if atlas_bytes == 0 {
            return Ok(false);
        }
        let mut count = crate::text_gpu::upload::UploadCount::default();
        super::visit_frame_uploads!(self, count);
        if atlas_bytes + count.bytes <= CHUNK_BYTES {
            return Ok(false);
        }
        self.text_preparation.upload_continuation = true;
        self.resume_glyph_upload(device, queue, on_submitted)
    }

    pub(crate) fn resume_glyph_upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        on_submitted: &mut dyn FnMut(wgpu::SubmissionIndex),
    ) -> anyhow::Result<bool> {
        if !self.text_preparation.upload_continuation || !self.atlas.has_pending_uploads() {
            return Ok(false);
        }
        self.release_text_scratch_for(self.atlas.chunk_staging_bytes(CHUNK_BYTES)?);

        self.publish_resource_consumers();
        let submission = self.atlas.submit_chunk(device, queue, CHUNK_BYTES)?;
        on_submitted(submission);
        if let Some(measurements) = &mut self.measurements {
            self.cold_world.measurement_frame =
                Some(measurements.incomplete_upload_submission(self.cold_world.measurement_frame)?);
        }
        Ok(true)
    }
}

impl Renderer {
    /// Glyph lookup buckets, including cached misses and reusable empty slots.
    pub fn atlas_lookup_metadata_bytes(&self) -> u64 {
        self.atlas.lookup_metadata_bytes()
    }

    /// Atlas page-record capacity and headers, including reusable page ownership.
    pub fn atlas_page_metadata_bytes(&self) -> u64 {
        self.atlas.page_metadata_bytes()
    }

    /// Pending raster upload record capacity, charged to staging/scratch separately from pixels.
    pub fn pending_glyph_metadata_bytes(&self) -> u64 {
        self.atlas.pending_metadata_bytes()
    }

    /// Retained raster pixel capacities and Datum headers, also in staging/scratch usage.
    pub fn pending_glyph_pixel_bytes(&self) -> u64 {
        self.atlas.pending_cpu_bytes()
    }
}
