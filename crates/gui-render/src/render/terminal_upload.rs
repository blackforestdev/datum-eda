//! Terminal pixel transfers share explicit staging and native queue receipts.
use super::*;

const CHUNK_BYTES: usize = 4 * 1024 * 1024;

impl TerminalGraphicsRenderer {
    fn upload_chunk(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &crate::text_gpu::Atlas,
    ) -> anyhow::Result<Option<wgpu::SubmissionIndex>> {
        if !self.textures.iter().any(|texture| {
            texture.pending
                && self
                    .draws
                    .iter()
                    .any(|draw| draw.texture_key == texture.key)
        }) {
            return Ok(None);
        }
        let mut remaining = CHUNK_BYTES;
        let mut uploads = Vec::new();
        let counts: Vec<_> = self
            .textures
            .iter()
            .map(|texture| {
                if self
                    .draws
                    .iter()
                    .any(|draw| draw.texture_key == texture.key)
                {
                    texture.append_chunk(&mut remaining, &mut uploads)
                } else {
                    0
                }
            })
            .collect();
        let Some(mut batch) = crate::text_gpu::upload::batch(
            device,
            &atlas.owner,
            atlas.generation,
            &atlas.staging_budget,
            &uploads,
            &[],
        )?
        else {
            return Ok(None);
        };
        let resources = self
            .textures
            .iter()
            .zip(&counts)
            .filter(|(_, count)| **count != 0)
            .map(|(texture, _)| texture.submission_ref())
            .collect();
        let submission = queue.submit([batch.command()]);
        batch.hold(queue);
        crate::text_gpu::hold_until_done(queue, resources);
        for (texture, count) in self.textures.iter_mut().zip(counts) {
            texture.consume_chunk(count);
        }
        self.upload_chunks += 1;
        self.upload_bytes += (CHUNK_BYTES - remaining) as u64;
        Ok(Some(submission))
    }
}

impl crate::Renderer {
    pub(crate) fn submit_terminal_upload_chunk(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        on_submitted: &mut dyn FnMut(wgpu::SubmissionIndex),
    ) -> anyhow::Result<bool> {
        let Some(submission) = self
            .terminal_graphics
            .upload_chunk(device, queue, &self.atlas)?
        else {
            return Ok(false);
        };
        on_submitted(submission);
        if let Some(measurements) = &mut self.measurements {
            self.cold_world.measurement_frame =
                Some(measurements.incomplete_upload_submission(self.cold_world.measurement_frame)?);
        }
        Ok(true)
    }

    pub fn terminal_upload_chunk_count(&self) -> u64 {
        self.terminal_graphics.upload_chunks
    }

    /// Actual padded staging bytes submitted for terminal pixels.
    pub fn terminal_upload_chunk_bytes(&self) -> u64 {
        self.terminal_graphics.upload_bytes
    }
}
