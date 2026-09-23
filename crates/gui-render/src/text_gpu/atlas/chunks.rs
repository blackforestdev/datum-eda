//! Partial glyph-page copies retain their prepared addresses until completion.
use super::*;

impl Atlas {
    pub(crate) fn pending_staging_bytes(&self) -> u64 {
        self.pending_copy_bytes
    }

    fn chunk_plan(&self, limit: u64) -> anyhow::Result<(usize, u64)> {
        let mut remaining = limit;
        let mut count = 0;
        for upload in self.pending_uploads.iter() {
            let pitch = u64::from(
                upload
                    .stride
                    .next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
            );
            anyhow::ensure!(pitch <= limit, "glyph row exceeds staging chunk");
            let rows = (remaining / pitch).min(u64::from(upload.size[1] - upload.uploaded_rows));
            if rows == 0 {
                break;
            }
            count += 1;
            remaining -= pitch * rows;
        }
        Ok((count, limit - remaining))
    }

    pub(crate) fn chunk_staging_bytes(&self, limit: u64) -> anyhow::Result<u64> {
        let (count, bytes) = self.chunk_plan(limit)?;
        Ok(bytes
            + StagingVec::<super::super::upload::TextureUpload<'_>>::capacity_bytes(count)?
            + StagingVec::<SubmissionRef>::capacity_bytes(self.pages.len())?
            + super::super::upload::retention_metadata_bytes(&[], false)?
            + super::super::upload::destination_metadata_bytes(count)?)
    }

    pub(crate) fn submit_chunk(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        limit: u64,
    ) -> anyhow::Result<wgpu::SubmissionIndex> {
        let (count, _) = self.chunk_plan(limit)?;
        let mut remaining = limit;
        let mut uploads = super::super::staging_vec::StagingVec::new(count, &self.staging_budget)?;
        for upload in self.pending_uploads.iter() {
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
                target: Some(self.pages[upload.page].texture.upload_target()),
                texture: &self.pages[upload.page].texture,
                origin: [upload.origin[0], upload.origin[1] + upload.uploaded_rows],
                size: [upload.size[0], rows],
                stride: upload.stride,
                pixels: &upload.pixels[first..first + bytes],
            });
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
        drop(uploads);
        let mut resources = StagingVec::new(self.pages.len(), &self.staging_budget)?;
        resources.extend(self.pages.iter().map(|page| page.texture.submission_ref()));
        let submission = queue.submit([batch.command()]);
        batch.hold(queue);
        queue.on_submitted_work_done(move || drop(resources));
        self.pending_copy_bytes -= limit - remaining;
        let mut copied = limit - remaining;
        for upload in self.pending_uploads.iter_mut() {
            let pitch = u64::from(
                upload
                    .stride
                    .next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT),
            );
            let rows =
                (copied / pitch).min(u64::from(upload.size[1] - upload.uploaded_rows)) as u32;
            if rows == 0 {
                break;
            }
            copied -= pitch * u64::from(rows);
            upload.uploaded_rows += rows;
            self.uploads.bytes += u64::from(rows) * u64::from(upload.stride);
            self.uploads.writes += 1;
        }
        self.pending_uploads
            .retain(|upload| upload.uploaded_rows != upload.size[1]);
        self.release_empty_pending_metadata();
        Ok(submission)
    }
}
