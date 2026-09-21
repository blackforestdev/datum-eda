//! Prepare all consumers of the shared glyph atlas before encoding either one.
use super::*;

#[derive(Default)]
pub(crate) struct GlyphPreparation {
    prepared: Option<(u64, TextPrepareSignature)>,
    #[cfg(test)]
    pub(crate) forced_overlay_errors: usize,
    #[cfg(test)]
    pub(crate) workspace_prepares: usize,
    #[cfg(test)]
    pub(crate) atlas_retries: usize,
    #[cfg(test)]
    pub(crate) overlay_prepares: usize,
}

impl GlyphPreparation {
    #[cfg(all(test, feature = "visual"))]
    pub(crate) fn is_invalid(&self) -> bool {
        self.prepared.is_none()
    }
}

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

impl Renderer {
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
        self.text_buffers.begin_frame(if overlay_only {
            text_buffer_cache::Profile::Overlay
        } else {
            text_buffer_cache::Profile::Workspace
        });
        let has_workspace_text = !overlay_only && prepared.has_workspace_text();
        let has_overlay_text = prepared.has_overlay_text();
        if has_workspace_text || has_overlay_text {
            self.viewport.update(queue, Resolution { width, height });
        }
        let (workspace, stats) = if !has_workspace_text {
            (Vec::new(), TextBufferCacheStats::default())
        } else {
            self.text_buffers
                .indices(&mut self.font_system, &prepared.text_runs, width, height)
        };
        let overlay = if !has_overlay_text {
            Vec::new()
        } else {
            self.text_buffers
                .indices(
                    &mut self.font_system,
                    prepared.menu_overlay_text_runs(),
                    width,
                    height,
                )
                .0
        };
        let signature = has_workspace_text
            .then(|| text_prepare_signature(&workspace, &prepared.text_runs, width, height));
        let revision = self.text_buffers.revision();
        let reuse = signature.as_ref().is_some_and(|signature| {
            self.text_preparation
                .prepared
                .as_ref()
                .is_some_and(|(old_revision, old)| *old_revision == revision && old == signature)
        });
        // Prepare mutates glyph instances even on failure. No old signature may
        // survive a partial attempt, including one that fails its retry.
        self.text_preparation.prepared = None;
        let first = self.prepare_text_pair(
            device,
            queue,
            prepared,
            &workspace,
            &overlay,
            has_workspace_text && !reuse,
        );
        let retried = if let Err(initial) = first {
            // trim clears atlas residency protection. Re-prepare workspace FIRST
            // to protect its glyphs from overlay allocation, before either draw.
            #[cfg(test)]
            {
                self.text_preparation.atlas_retries += 1;
            }
            self.atlas.trim();
            self.prepare_text_pair(
                device,
                queue,
                prepared,
                &workspace,
                &overlay,
                has_workspace_text,
            )
            .map_err(|retry| {
                anyhow::anyhow!("prepare frame text after atlas trim: {retry}; initial: {initial}")
            })?;
            true
        } else {
            false
        };
        self.text_preparation.prepared = signature.map(|signature| (revision, signature));
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
                    &mut self.font_system,
                    &mut self.atlas,
                    &self.viewport,
                    build_text_areas(self.text_buffers.entries(), workspace, &prepared.text_runs),
                    &mut self.swash_cache,
                )
                .map_err(|error| anyhow::anyhow!("prepare workspace text: {error}"))?;
        }
        if !overlay.is_empty() {
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
                    &mut self.font_system,
                    &mut self.atlas,
                    &self.viewport,
                    build_text_areas(
                        self.text_buffers.entries(),
                        overlay,
                        prepared.menu_overlay_text_runs(),
                    ),
                    &mut self.swash_cache,
                )
                .map_err(|error| anyhow::anyhow!("prepare overlay text: {error}"))?;
        }
        Ok(())
    }
}
