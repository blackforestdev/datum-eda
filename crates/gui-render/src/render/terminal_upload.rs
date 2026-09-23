//! Terminal pixel transfers share explicit staging and native queue receipts.
use super::*;
use crate::text_gpu::{
    lifetime::SubmissionRef,
    staging_vec::StagingVec,
    upload::{TextureUpload, UploadCount},
};

const CHUNK_BYTES: usize = 4 * 1024 * 1024;

impl TerminalGraphicsRenderer {
    fn upload_plan(&self) -> UploadCount {
        let mut count = UploadCount::default();
        let mut remaining = CHUNK_BYTES;
        for texture in &self.textures {
            if self
                .draws
                .iter()
                .any(|draw| draw.texture_key == texture.key)
            {
                texture.append_chunk(&mut remaining, &mut count);
            }
        }
        count
    }

    fn upload_metadata_bytes(count: usize) -> anyhow::Result<u64> {
        if count == 0 {
            return Ok(0);
        }
        Ok(StagingVec::<TextureUpload<'_>>::capacity_bytes(count)?
            + StagingVec::<(usize, usize)>::capacity_bytes(count)?
            + StagingVec::<SubmissionRef>::capacity_bytes(count)?
            + crate::text_gpu::upload::retention_metadata_bytes(&[], false)?)
    }

    fn upload_chunk(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &crate::text_gpu::Atlas,
        capacity: usize,
    ) -> anyhow::Result<Option<wgpu::SubmissionIndex>> {
        if capacity == 0 {
            return Ok(None);
        }
        let mut remaining = CHUNK_BYTES;
        let mut uploads = StagingVec::new(capacity, &atlas.staging_budget)?;
        let mut counts = StagingVec::new(capacity, &atlas.staging_budget)?;
        let mut resources = StagingVec::new(capacity, &atlas.staging_budget)?;
        for (index, texture) in self.textures.iter().enumerate() {
            if self
                .draws
                .iter()
                .any(|draw| draw.texture_key == texture.key)
            {
                let count = texture.append_chunk(&mut remaining, &mut uploads);
                if count != 0 {
                    counts.push((index, count));
                    resources.push(texture.submission_ref());
                }
            }
        }
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
        drop(uploads);
        let submission = queue.submit([batch.command()]);
        batch.hold(queue);
        queue.on_submitted_work_done(move || drop(resources));
        for &(index, count) in counts.iter() {
            self.textures[index].consume_chunk(count);
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
        let capacity = self.terminal_graphics.upload_plan().entries;
        self.release_text_scratch_for(
            CHUNK_BYTES as u64 + TerminalGraphicsRenderer::upload_metadata_bytes(capacity)?,
        );
        let Some(submission) =
            self.terminal_graphics
                .upload_chunk(device, queue, &self.atlas, capacity)?
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
