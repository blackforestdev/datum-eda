//! Retained-world upload continuation: one bounded submission per native turn.
use crate::{PreparedScene, Renderer, RetainedScene};

const CHUNK_BYTES: usize = 4 * 1024 * 1024;

#[derive(Default)]
pub(crate) struct ColdWorldUploads {
    pub active: bool,
    pub measurement_frame: Option<u64>,
    chunks: u64,
    bytes: u64,
}

impl Renderer {
    pub(crate) fn pending_world_upload_bytes(&self) -> usize {
        self.world_vertices_gpu.pending_bytes()
            + self.world_strokes_gpu.pending_bytes()
            + self.schematic_world_vertices_gpu.pending_bytes()
            + self.schematic_world_strokes_gpu.pending_bytes()
    }

    pub(crate) fn world_upload_sources_match(
        &self,
        prepared: &PreparedScene,
        retained: &RetainedScene,
        schematic: Option<&RetainedScene>,
    ) -> bool {
        self.world_vertices_gpu
            .matches_source(&retained.world_vertices)
            && self
                .world_strokes_gpu
                .matches_source(retained.world_strokes())
            && match crate::gpu_surface_pass::prepare_schematic_pass(prepared, schematic) {
                Some((_, _, _, scene)) => {
                    self.schematic_world_vertices_gpu
                        .matches_source(&scene.world_vertices)
                        && self
                            .schematic_world_strokes_gpu
                            .matches_source(scene.world_strokes())
                }
                None => {
                    self.schematic_world_vertices_gpu.pending_bytes() == 0
                        && self.schematic_world_strokes_gpu.pending_bytes() == 0
                }
            }
    }

    pub(crate) fn submit_world_upload_chunk(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        on_submitted: &mut dyn FnMut(wgpu::SubmissionIndex),
    ) -> anyhow::Result<()> {
        let mut remaining = CHUNK_BYTES;
        let mut uploads = Vec::new();
        let counts = [
            self.world_vertices_gpu
                .append_chunk(&mut remaining, &mut uploads),
            self.world_strokes_gpu
                .append_chunk(&mut remaining, &mut uploads),
            self.schematic_world_vertices_gpu
                .append_chunk(&mut remaining, &mut uploads),
            self.schematic_world_strokes_gpu
                .append_chunk(&mut remaining, &mut uploads),
        ];
        let mut batch = crate::text_gpu::upload::batch(
            device,
            &self.atlas.owner,
            self.atlas.generation,
            &self.atlas.staging_budget,
            &[],
            &uploads,
        )?
        .ok_or_else(|| anyhow::anyhow!("world continuation has no pending upload"))?;
        // Retain destinations independently of CPU owner replacement or host close.
        let resources = [
            self.world_vertices_gpu.submission_ref(),
            self.world_strokes_gpu.submission_ref(),
            self.schematic_world_vertices_gpu.submission_ref(),
            self.schematic_world_strokes_gpu.submission_ref(),
        ]
        .into_iter()
        .flatten()
        .collect();
        let submission = queue.submit([batch.command()]);
        batch.hold(queue);
        crate::text_gpu::hold_until_done(queue, resources);
        self.world_vertices_gpu.consume_chunk(counts[0]);
        self.world_strokes_gpu.consume_chunk(counts[1]);
        self.schematic_world_vertices_gpu.consume_chunk(counts[2]);
        self.schematic_world_strokes_gpu.consume_chunk(counts[3]);
        self.cold_world.active = self.pending_world_upload_bytes() != 0;
        self.cold_world.chunks += 1;
        self.cold_world.bytes += (CHUNK_BYTES - remaining) as u64;
        on_submitted(submission);
        // No partial multi-submission timing may masquerade as a complete sample.
        if let Some(measurements) = &mut self.measurements {
            self.cold_world.measurement_frame =
                Some(measurements.incomplete_upload_submission(self.cold_world.measurement_frame)?);
        }
        Ok(())
    }

    /// Actual retained-world copy submissions; warm camera/pointer frames add none.
    pub fn world_upload_chunk_count(&self) -> u64 {
        self.cold_world.chunks
    }
    pub fn world_upload_chunk_bytes(&self) -> u64 {
        self.cold_world.bytes
    }
}

#[cfg(all(test, feature = "visual"))]
#[path = "cold_world_upload_tests.rs"]
mod tests;
