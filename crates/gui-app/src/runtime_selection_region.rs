//! Native gesture plumbing over the shared qualification and acquisition owners.
use super::*;
use datum_gui_viewport::{
    selection_acquisition::{AcquisitionEntry, AcquisitionMode},
    selection_gesture::SelectionGesture,
};
impl Runtime {
    pub(super) fn begin_native_selection_press(&mut self) -> bool {
        if self.workspace().tool != WorkspaceTool::Select
            || self.workspace().schematic_authoring.active()
            || self.workspace().schematic_authoring.picker
        {
            return false;
        }
        if !self
            .workspace()
            .backing
            .as_ref()
            .is_some_and(|b| b.request.board_file.is_none())
        {
            return false;
        }
        if self.selection_evaluation.is_some() {
            self.cancel_native_selection_gesture();
            self.selection_release_suppressed = false;
        }
        let Some((x, y)) = self.last_cursor_pos else {
            return false;
        };
        if self.presented_hits.hit_test(x, y).is_some() {
            return false;
        }
        let Some(pane) = self.pane_at_screen(x, y) else {
            return false;
        };
        let Some((world, surface)) = self
            .prepared_scene()
            .and_then(|p| p.world_point_at_screen(x, y))
        else {
            return false;
        };
        let identity = match surface {
            SceneSurface::Board => format!(
                "{}@{}",
                self.workspace().scene.scene_id,
                self.workspace().scene.source_revision
            ),
            SceneSurface::Schematic => {
                let scene = self.workspace().schematic_scene.as_ref().unwrap();
                format!("{}@{}", scene.scene_id, scene.source_revision)
            }
        };
        let Some(gesture) = SelectionGesture::begin(
            pane,
            (x, y),
            world,
            self.modifiers.shift_key(),
            self.modifiers.control_key(),
            self.pan_gesture.space_is_held(),
            identity,
        ) else {
            return false;
        };
        self.set_application_focus(keyboard_focus::focus_after_canvas_click(pane));
        if pane != self.workspace().ui.layout.focused {
            self.swap_pane_focus(|l| l.focused = pane);
        }
        if gesture.mode != AcquisitionMode::Replace
            && let Err(reason) = self.prepare_native_region_basis(surface)
        {
            self.selection_notice(reason);
            self.selection_release_suppressed = true;
            return true;
        }
        self.selection_gesture = Some(gesture);
        true
    }
    fn prepare_native_region_basis(&mut self, surface: SceneSurface) -> Result<(), String> {
        match self.ensure_selection_authority() {
            Ok(true) => {}
            Ok(false) => return Err("Native region authority unavailable".into()),
            Err(error) => return Err(error.to_string()),
        }
        let native = self.native_selection.as_ref().unwrap();
        let scene = match surface {
            SceneSurface::Board => &self.workspace().scene,
            SceneSurface::Schematic => self
                .workspace()
                .schematic_scene
                .as_ref()
                .ok_or("Schematic region authority unavailable")?,
        };
        if scene.source_revision != native.model.model_revision.0
            || scene.project_uuid != native.model.project.project_id.to_string()
        {
            return Err("Region source revision does not match resolved authority".into());
        }
        let key = match surface {
            SceneSurface::Board => "board",
            SceneSurface::Schematic => "schematic",
        };
        if native
            .region_bases
            .get(key)
            .is_some_and(|basis| basis.matches(scene))
        {
            return Ok(());
        }
        let key = key.to_owned();
        let basis = std::sync::Arc::new(selection_region_evaluation::RegionBasis::capture(
            &native.model,
            &native.authority,
            scene,
        ));
        self.native_selection
            .as_mut()
            .unwrap()
            .region_bases
            .insert(key, basis);
        Ok(())
    }
    pub(super) fn advance_native_selection(&mut self, screen: (f32, f32)) -> bool {
        let Some(pane) = self.selection_gesture.as_ref().map(|g| g.pane) else {
            return false;
        };
        let Some(world) = self
            .prepared_scene()
            .and_then(|p| p.world_point_for_pane_unclipped(pane, screen.0, screen.1))
        else {
            self.cancel_native_selection_gesture();
            return true;
        };
        let gesture = self.selection_gesture.as_mut().unwrap();
        if !gesture.motion(screen, world) {
            return false;
        }
        if gesture.mode == AcquisitionMode::Replace {
            return true;
        }
        let feedback = datum_gui_protocol::SelectionRegionFeedback {
            pane,
            path: gesture.boundary(),
            subtractive: gesture.mode == AcquisitionMode::Remove,
            evaluating: None,
            phase: self
                .workspace()
                .ui
                .selection_region
                .as_ref()
                .map(|f| f.phase)
                .unwrap_or(0),
        };
        self.session.workspace_mut().ui.selection_region = Some(feedback);
        self.selection_tick =
            Some(std::time::Instant::now() + std::time::Duration::from_millis(33));
        self.invalidate_frame();
        true
    }
    pub(super) fn cancel_native_selection_gesture(&mut self) -> bool {
        let gesture_changed = self.selection_gesture.take().is_some();
        let evaluation_changed = self.selection_evaluation.take().is_some();
        let changed = gesture_changed || evaluation_changed;
        self.session.workspace_mut().ui.selection_region = None;
        self.selection_tick = None;
        self.refresh_selection_accessibility();
        if changed {
            self.selection_release_suppressed = true;
            self.invalidate_frame();
        }
        changed
    }
    pub(super) fn finish_native_selection_press(&mut self) -> Option<bool> {
        if std::mem::take(&mut self.selection_release_suppressed) {
            return Some(true);
        }
        let gesture = self.selection_gesture.take()?;
        self.session.workspace_mut().ui.selection_region = None;
        self.selection_tick = None;
        gesture.kind?;
        self.invalidate_frame();
        if gesture.mode == AcquisitionMode::Replace {
            return Some(true);
        } // Reserved plain drag is inert in S5A.
        let Some(content) = self.workspace().ui.layout.content_for(gesture.pane) else {
            return Some(true);
        };
        let scene = match content {
            datum_gui_protocol::PaneContent::Board => Some(&self.workspace().scene),
            datum_gui_protocol::PaneContent::Schematic => self.workspace().schematic_scene.as_ref(),
            _ => None,
        };
        if scene.is_none_or(|s| {
            format!("{}@{}", s.scene_id, s.source_revision) != gesture.source_revision
        }) {
            return Some(true);
        }
        match self.ensure_selection_authority() {
            Ok(true) => {}
            Ok(false) => return Some(true),
            Err(e) => {
                self.selection_notice(e.to_string());
                return Some(true);
            }
        }
        let state = self.workspace();
        let native = self.native_selection.as_ref().unwrap();
        let scene = if content == datum_gui_protocol::PaneContent::Schematic {
            state.schematic_scene.as_ref().unwrap()
        } else {
            &state.scene
        };
        let cache_key = if content == datum_gui_protocol::PaneContent::Schematic {
            "schematic"
        } else {
            "board"
        };
        let Some(basis) = native
            .region_bases
            .get(cache_key)
            .filter(|basis| basis.matches_source(scene))
            .cloned()
        else {
            self.selection_notice(
                "Region cancelled: source bounds unavailable; previous selection preserved".into(),
            );
            return Some(true);
        };
        let filters = state.ui.filters.clone();
        let revision = native.model.model_revision.0.clone();
        match selection_region_evaluation::RegionEvaluation::begin(
            gesture.pane,
            gesture.mode,
            revision,
            basis,
            filters,
            &gesture.boundary(),
        ) {
            Ok(evaluation) => {
                self.selection_evaluation = Some(evaluation);
                self.session.workspace_mut().ui.selection_region =
                    Some(datum_gui_protocol::SelectionRegionFeedback {
                        pane: gesture.pane,
                        path: gesture.boundary(),
                        subtractive: gesture.mode == AcquisitionMode::Remove,
                        phase: 0,
                        evaluating: Some(0),
                    });
                self.selection_tick = Some(std::time::Instant::now());
                self.advance_native_region_evaluation();
            }
            Err(reason) => self.selection_notice(reason),
        }
        Some(true)
    }
    fn advance_native_region_evaluation(&mut self) -> bool {
        let Some(mut evaluation) = self.selection_evaluation.take() else {
            return false;
        };
        let scene = self
            .workspace()
            .ui
            .layout
            .content_for(evaluation.pane)
            .and_then(|content| match content {
                datum_gui_protocol::PaneContent::Board => Some(&self.workspace().scene),
                datum_gui_protocol::PaneContent::Schematic => {
                    self.workspace().schematic_scene.as_ref()
                }
                _ => None,
            });
        if scene.is_none_or(|scene| !evaluation.basis.matches_source(scene))
            || &self.workspace().ui.filters != evaluation.filters()
        {
            self.session.workspace_mut().ui.selection_region = None;
            self.selection_tick = None;
            self.selection_notice(
                "Region cancelled: source or eligibility changed; previous selection preserved"
                    .into(),
            );
            return true;
        }
        // Resolve the current canonical basis before publication, never mix revisions.
        let outcome = evaluation.advance();
        match outcome {
            Ok(Some(candidates)) => {
                self.session.workspace_mut().ui.selection_region = None;
                self.selection_tick = None;
                match self.ensure_selection_authority() {
                    Ok(true) if self.native_selection.as_ref().unwrap().model.model_revision.0==evaluation.revision=> {
                        self.native_selection.as_mut().unwrap().reset_click_depth();
                        self.publish_native_acquisition(candidates,evaluation.mode,AcquisitionEntry::Region,None);
                    }
                    Ok(_)=>self.selection_notice("Region cancelled: resolved revision changed; no partial selection published".into()),
                    Err(error)=>self.selection_notice(error.to_string()),
                }
            }
            Ok(None) => {
                if let Some(feedback) = &mut self.session.workspace_mut().ui.selection_region {
                    feedback.evaluating = Some(evaluation.examined());
                }
                self.selection_evaluation = Some(evaluation);
                self.selection_tick =
                    Some(std::time::Instant::now() + std::time::Duration::from_millis(33));
            }
            Err(reason) => {
                self.session.workspace_mut().ui.selection_region = None;
                self.selection_tick = None;
                self.selection_notice(reason);
            }
        }
        self.refresh_selection_accessibility();
        self.invalidate_frame();
        true
    }
    pub(super) fn poll_native_selection_tick(&mut self) -> bool {
        if self
            .selection_tick
            .is_none_or(|due| due > std::time::Instant::now())
        {
            return false;
        }
        if self.selection_evaluation.is_some() {
            return self.advance_native_region_evaluation();
        }
        let Some(gesture) = &self.selection_gesture else {
            self.selection_tick = None;
            return false;
        };
        let Some(pane) = self
            .current_layout()
            .viewport_panes(&self.workspace().ui.layout)
            .panes
            .into_iter()
            .find(|p| p.id == gesture.pane)
        else {
            return self.cancel_native_selection_gesture();
        };
        let Some(screen) = self.last_cursor_pos else {
            return self.cancel_native_selection_gesture();
        };
        let delta = gesture.edge_pan(
            screen,
            [
                pane.rect.scene.x,
                pane.rect.scene.y,
                pane.rect.scene.width,
                pane.rect.scene.height,
            ],
        );
        if delta != (0.0, 0.0) {
            self.pan_selection_pane(gesture.pane, delta);
            self.advance_native_selection(screen);
        }
        let reduced = self.workspace().ui.global_preferences.reduced_motion;
        if let Some(feedback) = &mut self.session.workspace_mut().ui.selection_region
            && !reduced
        {
            feedback.phase = (feedback.phase + 1) % 12;
        }
        self.selection_tick =
            Some(std::time::Instant::now() + std::time::Duration::from_millis(33));
        self.invalidate_frame();
        true
    }
}
