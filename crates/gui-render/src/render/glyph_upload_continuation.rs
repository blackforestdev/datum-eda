//! Yield oversized mixed text uploads before encoding a frame or allocating queries.
use super::*;
const CHUNK_BYTES: u64 = 4 * 1024 * 1024;

impl Renderer {
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
