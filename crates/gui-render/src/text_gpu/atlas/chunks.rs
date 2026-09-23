//! Partial glyph-page copies retain their prepared addresses until completion.
use super::*;

impl Atlas {
    pub(crate) fn pending_staging_bytes(&self) -> u64 {
        self.pending_copy_bytes
    }

    pub(crate) fn submit_chunk(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        limit: u64,
    ) -> anyhow::Result<wgpu::SubmissionIndex> {
        let mut remaining = limit;
        let mut uploads = Vec::new();
        let mut counts = Vec::new();
        for upload in &self.pending_uploads {
            let pitch = u64::from(
                upload
                    .stride
                    .next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
            );
            anyhow::ensure!(pitch <= limit, "glyph row exceeds staging chunk");
            let rows =
                (remaining / pitch).min(u64::from(upload.size[1] - upload.uploaded_rows)) as u32;
            if rows == 0 {
                break;
            }
            let first = (upload.uploaded_rows * upload.stride) as usize;
            let bytes = (rows * upload.stride) as usize;
            uploads.push(super::super::upload::TextureUpload {
                texture: &self.pages[upload.page].texture,
                origin: [upload.origin[0], upload.origin[1] + upload.uploaded_rows],
                size: [upload.size[0], rows],
                stride: upload.stride,
                pixels: &upload.pixels[first..first + bytes],
            });
            counts.push(rows);
            remaining -= pitch * u64::from(rows);
        }
        let mut batch = super::super::upload::batch(
            device,
            &self.owner,
            self.generation,
            &self.staging_budget,
            &uploads,
            &[],
        )?
        .ok_or_else(|| anyhow::anyhow!("glyph continuation has no pending upload"))?;
        let resources = self.submission_refs();
        let submission = queue.submit([batch.command()]);
        batch.hold(queue);
        super::super::hold_until_done(queue, resources);
        self.pending_copy_bytes -= limit - remaining;
        for (upload, rows) in self.pending_uploads.iter_mut().zip(counts) {
            upload.uploaded_rows += rows;
            self.uploads.bytes += u64::from(rows) * u64::from(upload.stride);
            self.uploads.writes += 1;
        }
        self.pending_uploads
            .retain(|upload| upload.uploaded_rows != upload.size[1]);
        Ok(submission)
    }
}
