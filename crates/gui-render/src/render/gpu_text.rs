//! Prepare all consumers of the shared glyph atlas before encoding either one.
use super::*;

#[path = "glyph_upload_continuation.rs"]
mod continuation;
const MAX_OVERLAY_SIGNATURE_RUNS: usize = 128;

#[path = "glyph_preparation.rs"]
mod preparation;
pub(crate) use preparation::GlyphPreparation;

fn has_text_payload(runs: &[TextRun]) -> bool {
    runs.iter().any(|run| {
        if run.rich_spans.is_empty() {
            !run.text.is_empty()
        } else {
            run.rich_spans.iter().any(|span| !span.text.is_empty())
        }
    })
}

impl PreparedScene {
    pub(crate) fn has_workspace_text(&self) -> bool {
        has_text_payload(&self.text_runs)
    }

    pub(crate) fn has_overlay_text(&self) -> bool {
        !self.menu_overlay_vertices().is_empty() && has_text_payload(self.menu_overlay_text_runs())
    }
}

macro_rules! visit_frame_uploads {
    ($this:ident, $buffers:ident) => {{
        gpu_vertex_upload::screen_streams!($this, stream, stream.append_uploads(&mut $buffers));
        $this.terminal_graphics.append_vertex_uploads(&mut $buffers);
        $this.uniform_buffer.append_uploads(&mut $buffers);
        for binding in &$this.surface_scene_uniforms {
            binding.buffer.append_uploads(&mut $buffers);
        }
        let text_bytes = $this.text_renderer.append_uploads(&mut $buffers);
        let overlay_bytes = $this
            .menu_overlay_text_renderer
            .append_uploads(&mut $buffers);
        (text_bytes, overlay_bytes)
    }};
}
pub(super) use visit_frame_uploads;

macro_rules! frame_buffers {
    ($this:ident) => {{
        let mut count = crate::text_gpu::upload::UploadCount::default();
        visit_frame_uploads!($this, count);
        let metadata = crate::text_gpu::staging_vec::StagingVec::<
            crate::text_gpu::upload::BufferUpload<'_>,
        >::capacity_bytes(count.entries)?;
        $this.release_text_scratch_for(metadata);
        let mut buffers = crate::text_gpu::staging_vec::StagingVec::new(
            count.entries,
            &$this.atlas.staging_budget,
        )?;
        let (text_bytes, overlay_bytes) = visit_frame_uploads!($this, buffers);
        (buffers, text_bytes, overlay_bytes)
    }};
}

impl Renderer {
    pub(crate) fn flush_frame_uploads(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> anyhow::Result<Option<crate::text_gpu::upload::Batch>> {
        let (buffers, text_bytes, overlay_bytes) = frame_buffers!(self);
        // Admit and encode the complete mixed batch before consuming any producer.
        // Failure retains every pending update for cancellation/retry.
        // Reuse private layout scratch unless it competes with the real copy plan.
        let staging = self.atlas.required_staging_bytes(&buffers)?;
        preparation::release_scratch(
            &mut self.text_buffers,
            &mut self.swash_cache,
            &mut self.font_system,
            &self.atlas.staging_budget,
            staging,
        );

        let atlas_upload = self.atlas.flush_uploads(device, &buffers)?;
        drop(buffers);
        self.uniform_buffer.finish_uploads();
        for binding in &mut self.surface_scene_uniforms {
            binding.buffer.finish_uploads();
        }
        self.text_renderer.finish_uploads(text_bytes);
        self.menu_overlay_text_renderer
            .finish_uploads(overlay_bytes);
        self.finish_screen_uploads();
        Ok(atlas_upload)
    }

    pub(crate) fn hold_frame_submission(&mut self, queue: &wgpu::Queue) {
        let mut resources = self.vertex_submission_refs();
        resources.extend(self.surface_attachments.submission_ref());
        resources.extend(self.uniform_submission_refs());
        resources.extend(self.atlas.submission_refs());
        resources.extend(self.text_renderer.submission_ref());
        resources.extend(self.menu_overlay_text_renderer.submission_ref());
        text_gpu::hold_until_done(queue, resources);
        self.panel_gpu.retire_uncached_gpu();
        self.menu_overlay_gpu.retire_uncached_gpu();
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_frame_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
        overlay_only: bool,
    ) -> anyhow::Result<(TextBufferCacheStats, bool)> {
        let scope = self.text_cpu.clone();
        scope.with(|| {
            self.prepare_frame_text_inner(device, queue, prepared, width, height, overlay_only)
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_frame_text_inner(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedScene,
        width: u32,
        height: u32,
        overlay_only: bool,
    ) -> anyhow::Result<(TextBufferCacheStats, bool)> {
        // A previous frame may have failed after text preparation but before
        // submission. Only an explicit upload continuation preserves valid
        // preparation; its instance data remains pending for the final submission.
        let continued = std::mem::take(&mut self.text_preparation.upload_continuation);
        if !continued
            && (self.atlas.has_pending_uploads()
                || self.text_renderer.has_pending_uploads()
                || self.menu_overlay_text_renderer.has_pending_uploads())
        {
            self.atlas.repack();
            self.text_renderer.cancel_preparation();
            self.menu_overlay_text_renderer.cancel_preparation();
            self.text_preparation.prepared = None;
            self.text_preparation.overlay_prepared = None;
        }
        self.text_buffers.begin_frame(if overlay_only {
            text_buffer_cache::Profile::Overlay
        } else {
            text_buffer_cache::Profile::Workspace
        });
        let has_workspace_text = !overlay_only && prepared.has_workspace_text();
        let has_overlay_text = prepared.has_overlay_text();
        if has_workspace_text || has_overlay_text {
            self.text_resolution = [width, height];
        }
        let (mut workspace, stats) = if !has_workspace_text {
            (Default::default(), TextBufferCacheStats::default())
        } else {
            self.text_buffers.admitted_indices(
                &mut self.font_system,
                &prepared.text_runs,
                width,
                height,
                false,
                &self.atlas.staging_budget,
            )?
        };
        let (mut overlay, overlay_stats) = if !has_overlay_text {
            (Default::default(), TextBufferCacheStats::default())
        } else {
            self.text_buffers.admitted_indices(
                &mut self.font_system,
                prepared.menu_overlay_text_runs(),
                width,
                height,
                true,
                &self.atlas.staging_budget,
            )?
        };
        self.text_buffers
            .admit_layout_scratch(&self.atlas.staging_budget);
        if let Err(error) = self
            .text_buffers
            .admit_frame(&mut [&mut workspace, &mut overlay])
        {
            self.text_preparation.prepared = None;
            self.text_preparation.overlay_prepared = None;
            self.text_renderer.cancel_preparation();
            self.menu_overlay_text_renderer.cancel_preparation();
            return Err(error);
        }
        // Preserve workspace diagnostic semantics; dialog-only frames report
        // their actual text owner instead of an empty workspace statistic.
        let stats = if overlay_only { overlay_stats } else { stats };
        let signature = has_workspace_text
            .then(|| {
                text_prepare_identity::admitted_signature(
                    &workspace,
                    &prepared.text_runs,
                    width,
                    height,
                    &self.atlas.staging_budget,
                )
            })
            .transpose()?;
        let revision = self.text_buffers.revision();
        let reuse = signature.as_ref().is_some_and(|signature| {
            self.text_preparation.prepared.as_ref().is_some_and(
                |(old_revision, atlas_generation, old)| {
                    *old_revision == revision
                        && *atlas_generation == self.atlas.generation
                        && old.value == signature.value
                },
            )
        });
        let overlay_signature = (has_overlay_text && overlay.len() <= MAX_OVERLAY_SIGNATURE_RUNS)
            .then(|| {
                text_prepare_identity::admitted_signature(
                    &overlay,
                    prepared.menu_overlay_text_runs(),
                    width,
                    height,
                    &self.atlas.staging_budget,
                )
            })
            .transpose()?;
        let reuse_overlay = overlay_signature.as_ref().is_some_and(|signature| {
            self.text_preparation.overlay_prepared.as_ref().is_some_and(
                |(old_revision, atlas_generation, old)| {
                    *old_revision == revision
                        && *atlas_generation == self.atlas.generation
                        && old.value == signature.value
                },
            )
        });
        // Prepare mutates glyph instances even on failure. No old signature may
        // survive a partial attempt, including one that fails its retry.
        self.text_preparation.prepared = None;
        self.text_preparation.overlay_prepared = None;
        let first = self.prepare_text_pair(
            device,
            queue,
            prepared,
            &workspace,
            &overlay,
            has_workspace_text && !reuse,
            has_overlay_text && !reuse_overlay,
        );
        let retried = if let Err(initial) = first {
            if initial.is::<crate::text_gpu::UploadRequired>() {
                return Err(initial);
            }
            // trim clears atlas residency protection. Re-prepare workspace FIRST
            // to protect its glyphs from overlay allocation, before either draw.
            #[cfg(test)]
            {
                self.text_preparation.atlas_retries += 1;
            }
            self.atlas.repack();
            self.prepare_text_pair(
                device,
                queue,
                prepared,
                &workspace,
                &overlay,
                has_workspace_text,
                has_overlay_text,
            )
            .map_err(|retry| {
                retry.context(format!(
                    "prepare frame text after atlas trim; initial: {initial}"
                ))
            })?;
            true
        } else {
            false
        };
        self.text_preparation.prepared =
            signature.map(|signature| (revision, self.atlas.generation, signature));
        self.text_preparation.overlay_prepared =
            overlay_signature.map(|signature| (revision, self.atlas.generation, signature));
        Ok((stats, !has_workspace_text || (reuse && !retried)))
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_text_pair(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        prepared: &PreparedScene,
        workspace: &[usize],
        overlay: &[usize],
        prepare_workspace: bool,
        prepare_overlay: bool,
    ) -> anyhow::Result<()> {
        if prepare_workspace {
            #[cfg(test)]
            {
                self.text_preparation.workspace_prepares += 1;
            }
            self.text_renderer
                .prepare(
                    device,
                    queue,
                    &mut self.atlas,
                    &mut self.font_system,
                    &mut self.swash_cache,
                    self.text_resolution,
                    build_text_areas(self.text_buffers.entries(), workspace, &prepared.text_runs),
                )
                .map_err(|error| error.context("prepare workspace text"))?;
        }
        if prepare_overlay {
            #[cfg(test)]
            if self.text_preparation.forced_overlay_errors > 0 {
                self.text_preparation.forced_overlay_errors -= 1;
                anyhow::bail!("injected overlay allocation pressure");
            }
            #[cfg(test)]
            {
                self.text_preparation.overlay_prepares += 1;
            }
            self.menu_overlay_text_renderer
                .prepare(
                    device,
                    queue,
                    &mut self.atlas,
                    &mut self.font_system,
                    &mut self.swash_cache,
                    self.text_resolution,
                    build_text_areas(
                        self.text_buffers.entries(),
                        overlay,
                        prepared.menu_overlay_text_runs(),
                    ),
                )
                .map_err(|error| error.context("prepare overlay text"))?;
        }
        Ok(())
    }
}

impl Renderer {
    /// Retained workspace/overlay paint signatures; transient indices retire at
    /// the end of text preparation and use the same host/process staging budgets.
    pub fn text_preparation_storage_bytes(&self) -> u64 {
        self.text_preparation
            .prepared
            .as_ref()
            .map_or(0, |(_, _, s)| s.bytes)
            + self
                .text_preparation
                .overlay_prepared
                .as_ref()
                .map_or(0, |(_, _, s)| s.bytes)
    }
}

impl Renderer {
    /// Private retained raster scratch, excluding separately admitted pending pixels.
    pub fn raster_scratch_reserved_bytes(&self) -> u64 {
        self.swash_cache.reserved_bytes()
    }
}

impl Renderer {
    /// Reclaim cheap reusable layout/raster storage first, then font caches if
    /// required work still lacks admission. Font identity and returned shapes survive.
    pub(crate) fn release_text_scratch_for(&mut self, bytes: u64) {
        preparation::release_scratch(
            &mut self.text_buffers,
            &mut self.swash_cache,
            &mut self.font_system,
            &self.atlas.staging_budget,
            bytes,
        );
    }
    pub fn font_cache_reserved_bytes(&self) -> u64 {
        self.font_system.reserved_bytes()
    }
}
