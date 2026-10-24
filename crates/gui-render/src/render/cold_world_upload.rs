//! Retained-world upload continuation: one bounded submission per native turn.
use crate::{PreparedScene, Renderer, RetainedScene};

const CHUNK_BYTES: usize = 4 * 1024 * 1024;
const WORLD_STREAMS: usize = 8;

#[derive(Default)]
pub(crate) struct ColdWorldUploads {
    pub active: bool,
    chunks: u64,
    bytes: u64,
}

impl PreparedScene {
    pub(crate) fn requires_board_world(&self) -> bool {
        self.surface_passes()
            .iter()
            .any(|pass| pass.surface == crate::SceneSurface::Board)
    }
}

impl Renderer {
    pub(crate) fn pending_world_upload_bytes(&self) -> usize {
        self.world_glyph_points_gpu.pending_bytes()
            + self.world_glyph_materials_gpu.pending_bytes()
            + self.schematic_glyph_points_gpu.pending_bytes()
            + self.schematic_glyph_materials_gpu.pending_bytes()
            + self.world_vertices_gpu.pending_bytes()
            + self.world_strokes_gpu.pending_bytes()
            + self.schematic_world_vertices_gpu.pending_bytes()
            + self.schematic_world_strokes_gpu.pending_bytes()
            + self
                .qualified_schematic_gpu
                .entries
                .iter()
                .map(|e| e.pending_bytes())
                .sum::<usize>()
    }

    pub(crate) fn world_upload_sources_match(
        &self,
        prepared: &PreparedScene,
        retained: &RetainedScene,
        schematic: Option<&RetainedScene>,
    ) -> bool {
        let board_matches = if prepared.requires_board_world() {
            self.world_vertices_gpu
                .matches_source(&retained.world_vertices)
                && self
                    .world_glyph_points_gpu
                    .matches_source(&retained.glyph.points)
                && self
                    .world_glyph_materials_gpu
                    .matches_source(&retained.glyph.materials)
                && self
                    .world_strokes_gpu
                    .matches_source(retained.world_strokes())
        } else {
            self.world_vertices_gpu.buffer().is_none()
                && self.world_strokes_gpu.buffer().is_none()
                && self.world_glyph_points_gpu.buffer().is_none()
                && self.world_glyph_materials_gpu.buffer().is_none()
        };
        let qualified_matches = prepared
            .schematic_sources
            .iter()
            .filter(|(key, _, _)| key.is_some())
            .all(|(_, _, scene)| self.qualified_schematic_gpu.get(scene).is_some());
        let no_unused = self.qualified_schematic_gpu.entries.iter().all(|entry| {
            prepared
                .schematic_sources
                .iter()
                .any(|(key, _, scene)| key.is_some() && entry.matches(scene))
        });
        board_matches
            && qualified_matches
            && no_unused
            && match crate::gpu_surface_pass::prepare_schematic_pass(prepared, schematic) {
                Some(scene) => {
                    self.schematic_world_vertices_gpu
                        .matches_source(&scene.world_vertices)
                        && self
                            .schematic_glyph_points_gpu
                            .matches_source(&scene.glyph.points)
                        && self
                            .schematic_glyph_materials_gpu
                            .matches_source(&scene.glyph.materials)
                        && self
                            .schematic_world_strokes_gpu
                            .matches_source(scene.world_strokes())
                }
                None => {
                    self.schematic_world_vertices_gpu.buffer().is_none()
                        && self.schematic_world_strokes_gpu.buffer().is_none()
                        && self.schematic_glyph_points_gpu.buffer().is_none()
                        && self.schematic_glyph_materials_gpu.buffer().is_none()
                }
            }
    }

    pub(crate) fn submit_world_upload_chunk(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        on_submitted: &mut dyn FnMut(wgpu::SubmissionIndex),
    ) -> anyhow::Result<()> {
        self.publish_resource_consumers();
        let world_streams = WORLD_STREAMS
            .checked_add(
                self.qualified_schematic_gpu
                    .entries
                    .len()
                    .checked_mul(4)
                    .ok_or_else(|| anyhow::anyhow!("document upload stream overflow"))?,
            )
            .ok_or_else(|| anyhow::anyhow!("world upload stream overflow"))?;
        let descriptor_bytes = crate::text_gpu::staging_vec::StagingVec::<
            crate::text_gpu::upload::BufferUpload<'_>,
        >::capacity_bytes(world_streams)?;
        self.release_text_scratch_for(
            CHUNK_BYTES as u64
                + descriptor_bytes
                + crate::text_gpu::upload::retention_metadata_bytes(&[], false)?
                + crate::text_gpu::upload::destination_metadata_bytes(world_streams)?,
        );
        let mut remaining = CHUNK_BYTES;
        let mut uploads = crate::text_gpu::staging_vec::StagingVec::new(
            world_streams,
            &self.atlas.staging_budget,
        )?;
        let counts = [
            self.world_vertices_gpu
                .append_chunk(&mut remaining, &mut uploads),
            self.world_strokes_gpu
                .append_chunk(&mut remaining, &mut uploads),
            self.schematic_world_vertices_gpu
                .append_chunk(&mut remaining, &mut uploads),
            self.schematic_world_strokes_gpu
                .append_chunk(&mut remaining, &mut uploads),
            self.world_glyph_points_gpu
                .append_chunk(&mut remaining, &mut uploads),
            self.world_glyph_materials_gpu
                .append_chunk(&mut remaining, &mut uploads),
            self.schematic_glyph_points_gpu
                .append_chunk(&mut remaining, &mut uploads),
            self.schematic_glyph_materials_gpu
                .append_chunk(&mut remaining, &mut uploads),
        ];
        let mut document_counts = crate::text_gpu::staging_vec::StagingVec::new(
            self.qualified_schematic_gpu.entries.len(),
            &self.atlas.staging_budget,
        )?;
        for entry in self.qualified_schematic_gpu.entries.iter() {
            document_counts.push(entry.append_chunk(&mut remaining, &mut uploads));
        }
        let mut batch = crate::text_gpu::upload::batch(
            device,
            &self.atlas.owner,
            self.atlas.generation,
            &self.atlas.staging_budget,
            &[],
            &uploads,
        )?
        .ok_or_else(|| anyhow::anyhow!("world continuation has no pending upload"))?;
        drop(uploads);
        // Retain destinations independently of CPU owner replacement or host close.
        let resources = [
            self.world_vertices_gpu.submission_ref(),
            self.world_strokes_gpu.submission_ref(),
            self.schematic_world_vertices_gpu.submission_ref(),
            self.schematic_world_strokes_gpu.submission_ref(),
            self.world_glyph_points_gpu.submission_ref(),
            self.world_glyph_materials_gpu.submission_ref(),
            self.schematic_glyph_points_gpu.submission_ref(),
            self.schematic_glyph_materials_gpu.submission_ref(),
        ];
        let mut document_resources = crate::text_gpu::staging_vec::StagingVec::new(
            self.qualified_schematic_gpu
                .entries
                .len()
                .checked_mul(4)
                .ok_or_else(|| anyhow::anyhow!("document upload hold overflow"))?,
            &self.atlas.staging_budget,
        )?;
        for entry in self.qualified_schematic_gpu.entries.iter() {
            for resource in entry.refs() {
                document_resources.push(resource);
            }
        }
        let (before, after) = self
            .begin_upload_measurement(device, "world")?
            .map_or((None, None), |(a, b)| (Some(a), Some(b)));
        let submission = queue.submit(before.into_iter().chain([batch.command()]).chain(after));
        batch.hold(queue);
        queue.on_submitted_work_done(move || drop((resources, document_resources)));
        self.world_vertices_gpu.consume_chunk(counts[0]);
        self.world_strokes_gpu.consume_chunk(counts[1]);
        self.schematic_world_vertices_gpu.consume_chunk(counts[2]);
        self.schematic_world_strokes_gpu.consume_chunk(counts[3]);
        self.world_glyph_points_gpu.consume_chunk(counts[4]);
        self.world_glyph_materials_gpu.consume_chunk(counts[5]);
        self.schematic_glyph_points_gpu.consume_chunk(counts[6]);
        self.schematic_glyph_materials_gpu.consume_chunk(counts[7]);
        for (entry, counts) in self
            .qualified_schematic_gpu
            .entries
            .iter_mut()
            .zip(document_counts.iter())
        {
            entry.consume_chunk(*counts);
        }
        self.cold_world.active = self.pending_world_upload_bytes() != 0;
        self.cold_world.chunks += 1;
        self.cold_world.bytes += (CHUNK_BYTES - remaining) as u64;
        on_submitted(submission);
        self.finish_upload_measurement(queue)?;
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
