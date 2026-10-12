//! Pane-wide selected-emphasis planning; base paint and gesture feedback are separate.
use super::*;

// Retained plan capacities use the existing process admission owner. Growth
// reserves replacement overlap before allocation and releases old storage first.
struct Admission {
    _permit: Option<crate::text_gpu::budget::Permit>,
}
impl std::fmt::Debug for Admission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Admitted")
    }
}
impl PartialEq for Admission {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
#[derive(Debug)]
pub(crate) struct Metadata<T> {
    values: Vec<T>,
    admission: Admission,
}
impl<T> Default for Metadata<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            admission: Admission { _permit: None },
        }
    }
}
impl<T: PartialEq> PartialEq for Metadata<T> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl<T> std::ops::Deref for Metadata<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}
impl<T> std::ops::DerefMut for Metadata<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        &mut self.values
    }
}
impl<T> Metadata<T> {
    fn push(&mut self, value: T) -> anyhow::Result<()> {
        if self.values.len() == self.values.capacity() {
            let capacity = self
                .values
                .len()
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("cue metadata overflow"))?
                .max(self.values.capacity().saturating_mul(2));
            let bytes = crate::cpu_alloc::heap::capacity_bytes::<T>(capacity);
            let permit =
                crate::text_gpu::budget::staging_process().reserve(u64::try_from(bytes)?)?;
            let mut replacement = Vec::new();
            replacement.try_reserve_exact(capacity)?;
            anyhow::ensure!(
                replacement.capacity() == capacity,
                "cue metadata admission capacity"
            );
            replacement.append(&mut self.values);
            self.values = replacement;
            self.admission = Admission {
                _permit: Some(permit),
            };
        }
        self.values.push(value);
        Ok(())
    }
}
/// Counts emitted detail after culling/clipping, independently of minimum owners.
pub(crate) const DETAILED_PRIMITIVES: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScreenSpan {
    pub range: std::ops::Range<u32>,
    pub selected: bool,
    pub void: bool,
}

#[derive(Debug, PartialEq)]
pub(crate) struct PanePlan {
    pub viewport: RectPx,
    pub camera: CameraState,
    pub subject: Option<datum_gui_protocol::SelectionSubject>,
    pub basis: Vec<String>,
    pub screen: Metadata<ScreenSpan>,
    pub detail: Metadata<std::ops::Range<u32>>,
    pub detail_primitives: usize,
    pub palette: [f32; 3],
    _admission: Admission,
    failure: Option<String>,
}
impl PanePlan {
    pub(crate) fn new(
        state: &ReviewWorkspaceState,
        viewport: RectPx,
        camera: CameraState,
    ) -> anyhow::Result<Self> {
        let view = match &state.selection {
            SelectionTarget::Native(view) if !view.stale => Some(view),
            _ => None,
        };
        let string_bytes = |len: usize| crate::cpu_alloc::heap::capacity_bytes::<u8>(len);
        let mut bytes = std::mem::size_of::<Self>()
            + 2 * std::mem::size_of::<usize>()
            + crate::cpu_alloc::heap::capacity_bytes::<(
                datum_gui_protocol::PaneId,
                std::sync::Arc<Self>,
            )>(4);
        if let Some(view) = view {
            bytes += crate::cpu_alloc::heap::capacity_bytes::<String>(7)
                + string_bytes(36)
                + string_bytes(view.revision.0.len());
            for value in [
                state.review.producer_revision.as_ref(),
                state.review.source_fingerprint.as_ref(),
                state.review.artifact_path.as_ref(),
                state.checks.check_run_id.as_ref(),
                state.checks.model_revision.as_ref(),
            ] {
                bytes += string_bytes(value.map_or(1, |v| v.len() + 1));
            }
            if let datum_gui_protocol::SelectionSubject::Proposal(id)
            | datum_gui_protocol::SelectionSubject::Review(id)
            | datum_gui_protocol::SelectionSubject::Diagnostic(id) = &view.subject
            {
                bytes += string_bytes(id.len());
            }
        }
        let admission =
            crate::text_gpu::budget::staging_process().reserve(u64::try_from(bytes)?)?;
        let subject = view
            .filter(|view| datum_gui_protocol::selection_artifacts::is_artifact(&view.subject))
            .map(|view| view.subject.clone());
        let basis = if let Some(view) = view {
            let mut basis = Vec::new();
            basis.try_reserve_exact(7)?;
            anyhow::ensure!(basis.capacity() == 7, "cue basis admission capacity");
            basis.push(view.project.to_string());
            basis.push(view.revision.0.clone());
            for value in [
                state.review.producer_revision.as_ref(),
                state.review.source_fingerprint.as_ref(),
                state.review.artifact_path.as_ref(),
                state.checks.check_run_id.as_ref(),
                state.checks.model_revision.as_ref(),
            ] {
                // Preserve missing and present-empty producer fields distinctly.
                basis.push(value.map_or_else(
                    || "0".into(),
                    |value| {
                        let mut owned = String::with_capacity(value.len() + 1);
                        owned.push('1');
                        owned.push_str(value);
                        owned
                    },
                ));
            }
            basis
        } else {
            Vec::new()
        };
        let palette = match subject {
            Some(datum_gui_protocol::SelectionSubject::Proposal(_)) => PROPOSAL_OUTER,
            Some(datum_gui_protocol::SelectionSubject::Review(_)) => DIAGNOSTIC_FOCUS,
            _ => TEXT_ACCENT,
        };
        Ok(Self {
            viewport,
            camera,
            subject,
            basis,
            screen: Metadata::default(),
            detail: Metadata::default(),
            detail_primitives: 0,
            palette,
            _admission: Admission {
                _permit: Some(admission),
            },
            failure: None,
        })
    }
    pub(crate) fn freeze(self) -> anyhow::Result<Self> {
        anyhow::ensure!(
            self.failure.is_none(),
            "cue metadata admission failed: {:?}",
            self.failure
        );
        Ok(self)
    }
    pub(crate) fn overflow(&self, emitted_glyphs: usize) -> bool {
        self.detail_primitives.saturating_add(emitted_glyphs) > DETAILED_PRIMITIVES
    }
    pub(crate) fn base_output<'a>(
        &'a mut self,
        out: &'a mut Vec<Quad>,
        selected: bool,
    ) -> ScreenOutput<'a> {
        ScreenOutput {
            out,
            plan: self,
            selected,
            void: false,
            scratch: Vec::new(),
        }
    }
    pub(crate) fn finish_detail(&mut self, out: &mut Vec<Quad>, start: usize) {
        crate::hit_clipping::clip_content(
            out,
            &mut Vec::new(),
            &mut Vec::new(),
            start,
            0,
            0,
            self.viewport,
        );
        let count = out.len() - start;
        self.detail_primitives = self.detail_primitives.saturating_add(count);
        if count != 0 {
            let result = self.detail.push(
                u32::try_from(start.checked_mul(6).expect("vertex range")).expect("vertex index")
                    ..u32::try_from(out.len().checked_mul(6).expect("vertex range"))
                        .expect("vertex index"),
            );
            if let Err(error) = result {
                self.failure = Some(error.to_string());
            }
        }
    }
}

/// Reuses the actual screen stream; mask intent stores only ordered span metadata.
/// The original emitter's void role survives clipping, preserving drill holes.
pub(crate) struct ScreenOutput<'a> {
    out: &'a mut Vec<Quad>,
    plan: &'a mut PanePlan,
    selected: bool,
    void: bool,
    scratch: Vec<Quad>,
}
impl std::ops::Deref for ScreenOutput<'_> {
    type Target = [Quad];
    fn deref(&self) -> &[Quad] {
        self.out
    }
}
impl std::ops::DerefMut for ScreenOutput<'_> {
    fn deref_mut(&mut self) -> &mut [Quad] {
        self.out
    }
}
impl Extend<Quad> for ScreenOutput<'_> {
    fn extend<T: IntoIterator<Item = Quad>>(&mut self, values: T) {
        for value in values {
            self.push(value);
        }
    }
}
impl Output<Quad> for ScreenOutput<'_> {
    fn push(&mut self, value: Quad) {
        self.scratch.clear();
        self.scratch.push(value);
        crate::hit_clipping::clip_content(
            &mut self.scratch,
            &mut Vec::new(),
            &mut Vec::new(),
            0,
            0,
            0,
            self.plan.viewport,
        );
        if self.scratch.is_empty() {
            return;
        }
        let first = u32::try_from(self.out.len().checked_mul(6).expect("vertex range"))
            .expect("vertex index");
        self.out.append(&mut self.scratch);
        let end = u32::try_from(self.out.len().checked_mul(6).expect("vertex range"))
            .expect("vertex index");
        if let Some(last) = self.plan.screen.last_mut()
            && last.range.end == first
            && last.selected == self.selected
            && last.void == self.void
        {
            last.range.end = end;
        } else {
            let result = self.plan.screen.push(ScreenSpan {
                range: first..end,
                selected: self.selected,
                void: self.void,
            });
            if let Err(error) = result {
                self.plan.failure = Some(error.to_string());
            }
        }
    }
    fn role(&mut self, role: crate::presentation_ranges::Role) {
        self.void = role == crate::presentation_ranges::Role::Void;
    }
    fn reserve(&mut self, additional: usize) {
        self.out.reserve(additional);
    }
}

#[derive(Debug)]
struct Cut {
    range: std::ops::Range<u32>,
    removed_before: u32,
}
impl PreparedScene {
    // Once quad producers already exceed the pane limit, their detailed paint
    // cannot be drawn. Remove it before GPU admission, retaining the exact base
    // silhouette stream, measured count and every other producer's draw order.
    fn compact_overflow_detail(&mut self) -> anyhow::Result<()> {
        let mut cuts = Metadata::<Cut>::default();
        for (pane, plan) in &self.pane_cues {
            anyhow::ensure!(
                std::sync::Arc::strong_count(plan) == 1,
                "shared cue plan before publication"
            );
            let count = self
                .pane_cues
                .iter()
                .filter(|(id, _)| id == pane)
                .try_fold(0usize, |total, (_, p)| {
                    total
                        .checked_add(p.detail_primitives)
                        .ok_or_else(|| anyhow::anyhow!("cue count overflow"))
                })?;
            if count > DETAILED_PRIMITIVES {
                for range in plan.detail.iter() {
                    cuts.push(Cut {
                        range: range.clone(),
                        removed_before: 0,
                    })?;
                }
            }
        }
        if cuts.is_empty() {
            return Ok(());
        }
        cuts.sort_by_key(|cut| cut.range.start);
        let mut removed = 0u32;
        let mut end = 0u32;
        for cut in cuts.iter_mut() {
            anyhow::ensure!(
                cut.range.start >= end
                    && cut.range.end >= cut.range.start
                    && cut.range.end as usize <= self.viewport_overlay_vertices.len(),
                "invalid cue detail range"
            );
            cut.removed_before = removed;
            removed = removed
                .checked_add(cut.range.end - cut.range.start)
                .ok_or_else(|| anyhow::anyhow!("cue range overflow"))?;
            end = cut.range.end;
        }
        let remap = |index: u32| {
            let prior = cuts.partition_point(|cut| cut.range.start < index);
            if prior == 0 {
                index
            } else {
                let cut = &cuts[prior - 1];
                index - cut.removed_before - (index.min(cut.range.end) - cut.range.start)
            }
        };
        self.selection_overlay_split = remap(u32::try_from(self.selection_overlay_split)?) as usize;
        let mut index = 0u32;
        let mut current = 0usize;
        self.viewport_overlay_vertices.retain(|_| {
            while current < cuts.len() && index >= cuts[current].range.end {
                current += 1;
            }
            let keep = current == cuts.len() || index < cuts[current].range.start;
            index += 1;
            keep
        });
        for (_, plan) in &mut self.pane_cues {
            let plan = std::sync::Arc::get_mut(plan).expect("unpublished plan checked above");
            for span in plan.screen.iter_mut() {
                span.range = remap(span.range.start)..remap(span.range.end);
            }
            for range in plan.detail.iter_mut() {
                *range = remap(range.start)..remap(range.end);
            }
            plan.detail.values.retain(|range| !range.is_empty());
        }
        Ok(())
    }
    /// Append once after the shared session applies every pane's warm camera.
    pub(crate) fn append_native_selection(
        &mut self,
        state: &ReviewWorkspaceState,
    ) -> anyhow::Result<()> {
        use crate::resource_consumers::{Consumer, Stream};
        self.selection_overlay_split = self.viewport_overlay_vertices.len();
        let mut minimum_count = 0usize;
        for pass in &self.surface_passes {
            let (scene, consumer) = match pass.surface {
                SceneSurface::Board => (&state.scene, Consumer::Board),
                SceneSurface::Schematic => {
                    let Some(scene) = state.schematic_scene.as_ref() else {
                        continue;
                    };
                    (scene, Consumer::Schematic)
                }
            };
            let surface = match pass.surface {
                SceneSurface::Board => datum_gui_protocol::PaneContent::Board,
                SceneSurface::Schematic => datum_gui_protocol::PaneContent::Schematic,
            };
            let mut quads = Vec::new();
            let mut plan = PanePlan::new(state, pass.scene_viewport, pass.camera)?;
            if authored_visible(state)
                && let SelectionTarget::Native(view) = &state.selection
                && let Some(mut request) = mask::Request::capture(view, scene, surface)?
            {
                let projection = Projection::new(
                    inset_rect(pass.scene_viewport, 10.0, 10.0, 10.0, 10.0),
                    &scene.bounds,
                    pass.camera,
                );
                let glyphs =
                    glyph_mesh_lookup::MeshIndex::new(&scene.glyph_mesh_assets, &mut quads);
                let lod = lod::Plan::capture(
                    view,
                    scene,
                    surface,
                    state,
                    &projection,
                    pass.scene_viewport,
                    glyphs.as_ref(),
                );
                minimum_count = minimum_count
                    .checked_add(lod.minimum_count())
                    .ok_or_else(|| anyhow::anyhow!("selection LOD count overflow"))?;
                request.configure(
                    &lod,
                    view,
                    scene,
                    surface,
                    state.ui.global_preferences.high_contrast_noncolor,
                    state.ui.filters.dim_unrelated,
                );
                self.selection_masks
                    .push((pass.pane_id, pass.surface, request));
            }
            let before_text = self.text_runs.len();
            let before_strokes = self.selection_strokes.len();
            push_native_impl(
                &mut self.selection_strokes,
                &mut quads,
                &mut self.text_runs,
                scene,
                state,
                pass.scene_viewport,
                pass.camera,
                surface,
                true,
            );
            if surface == datum_gui_protocol::PaneContent::Board
                && authored_visible(state)
                && let SelectionTarget::Native(view) = &state.selection
                && !view.stale
            {
                let projection = Projection::new(
                    inset_rect(pass.scene_viewport, 10.0, 10.0, 10.0, 10.0),
                    &scene.bounds,
                    pass.camera,
                );
                lock_cue::push_selected(&mut quads, view, &projection, pass.scene_viewport);
            }
            plan.finish_detail(&mut quads, 0);
            for run in &mut self.text_runs[before_text..] {
                run.layer = TextLayer::Selection(pass.pane_id);
            }
            if self
                .selection_masks
                .iter()
                .any(|(pane, _, _)| *pane == pass.pane_id)
            {
                self.consumers.include(Stream::Selection, consumer.into());
            }
            self.consumers.note(
                Stream::Selection,
                consumer,
                before_strokes,
                self.selection_strokes.len(),
            );
            push_region(
                &mut quads,
                &mut self.text_runs,
                state,
                pass.pane_id,
                scene,
                pass.scene_viewport,
                pass.camera,
            );
            diagnostics::push(
                &mut quads,
                &mut self.hit_regions,
                scene,
                state,
                pass.scene_viewport,
                pass.camera,
                surface,
                &mut plan,
            );
            let offset = u32::try_from(self.viewport_overlay_vertices.len())?;
            for span in plan.screen.iter_mut() {
                span.range.start = span
                    .range
                    .start
                    .checked_add(offset)
                    .ok_or_else(|| anyhow::anyhow!("cue range overflow"))?;
                span.range.end = span
                    .range
                    .end
                    .checked_add(offset)
                    .ok_or_else(|| anyhow::anyhow!("cue range overflow"))?;
            }
            for range in plan.detail.iter_mut() {
                range.start = range
                    .start
                    .checked_add(offset)
                    .ok_or_else(|| anyhow::anyhow!("cue range overflow"))?;
                range.end = range
                    .end
                    .checked_add(offset)
                    .ok_or_else(|| anyhow::anyhow!("cue range overflow"))?;
            }
            self.pane_cues
                .push((pass.pane_id, std::sync::Arc::new(plan.freeze()?)));
            TextRun::annotate(
                &mut self.text_runs[before_text..],
                TextOrigin::Viewport(pass.pane_id),
            );
            self.consumers
                .note(Stream::Text, consumer, before_text, self.text_runs.len());
            self.consumers
                .note(Stream::ViewportOverlay, consumer, 0, quads.len());
            self.viewport_overlay_vertices
                .extend(quads_to_vertices(&quads));
        }
        self.compact_overflow_detail()?;
        // Freeze one shared pane request after every selected-emphasis producer
        // has emitted its exact ranges. Text glyph counts join during preparation.
        for pass in &self.surface_passes {
            let plans = self
                .pane_cues
                .iter()
                .filter(|(pane, _)| *pane == pass.pane_id)
                .map(|(_, plan)| plan.clone());
            if let Some((_, _, request)) = self
                .selection_masks
                .iter_mut()
                .find(|(pane, _, _)| *pane == pass.pane_id)
            {
                request.set_screen(plans.clone())?;
            } else if plans
                .clone()
                .any(|plan| plan.screen.iter().any(|span| span.selected))
                && let SelectionTarget::Native(view) = &state.selection
            {
                let scene = match pass.surface {
                    SceneSurface::Board => &state.scene,
                    SceneSurface::Schematic => {
                        state.schematic_scene.as_ref().expect("schematic pass")
                    }
                };
                if let Some(mut request) = mask::Request::capture_screen(
                    view,
                    scene,
                    state.ui.global_preferences.high_contrast_noncolor,
                )? {
                    request.set_screen(plans.clone())?;
                    self.selection_masks
                        .push((pass.pane_id, pass.surface, request));
                }
            }
        }
        self.selection_visual_lod =
            matches!(state.selection, SelectionTarget::Native(_)).then(|| {
                let masks = self
                    .selection_masks
                    .iter()
                    .filter(|(_, _, request)| request.authored() || request.detailed_overflow(0))
                    .count();
                if masks != 0 {
                    SelectionVisualLod::ExactMask {
                        panes: masks,
                        minimum_cues: minimum_count,
                    }
                } else if !self.selection_masks.is_empty() {
                    SelectionVisualLod::Detailed {
                        panes: self.selection_masks.len(),
                        minimum_cues: minimum_count,
                    }
                } else {
                    SelectionVisualLod::Unavailable
                }
            });
        if let (Some((rect, scale)), Some(lod)) =
            (self.selection_lod_row, self.selection_visual_lod)
        {
            let summary = lod.to_string();
            let start = self.text_runs.len();
            draw_text(
                &summary,
                rect.x + 2.0 * scale,
                rect.y + 3.0 * scale,
                11.0 * scale,
                TEXT_PRIMARY,
                TextFace::Ui,
                &mut self.text_runs,
            );
            self.text_runs.last_mut().expect("LOD label").clip_bounds = Some(rect);
            self.consumers.note(
                Stream::Text,
                Consumer::Inspector,
                start,
                self.text_runs.len(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "cue_plan/tests.rs"]
mod tests;
