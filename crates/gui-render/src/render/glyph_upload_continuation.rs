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
        self.text_buffers
            .release_layout_scratch_for(CHUNK_BYTES, &self.atlas.staging_budget);
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
        let (buffers, _, _) = super::frame_buffers!(self);
        let bytes: u64 = buffers.iter().map(|upload| upload.bytes.len() as u64).sum();
        if atlas_bytes + bytes <= CHUNK_BYTES {
            return Ok(false);
        }
        drop(buffers);
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
        self.text_buffers
            .release_layout_scratch_for(CHUNK_BYTES, &self.atlas.staging_budget);
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
    /// Retained raster pixel capacities and Datum headers, also in staging/scratch usage.
    pub fn pending_glyph_pixel_bytes(&self) -> u64 {
        self.atlas.pending_cpu_bytes()
    }
}
