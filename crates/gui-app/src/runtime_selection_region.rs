//! Native gesture plumbing over the shared qualification and acquisition owners.
use super::*;
use datum_gui_protocol::selection_geometry_catalog::{
    QualificationGeometry as Geometry, qualification_catalog,
};
use datum_gui_viewport::{
    selection_acquisition::{AcquisitionEntry, AcquisitionMode},
    selection_gesture::SelectionGesture,
    selection_region::SelectionRegion,
};
impl Runtime {
    pub(super) fn begin_native_selection_press(&mut self) -> bool {
        if !self
            .workspace()
            .backing
            .as_ref()
            .is_some_and(|b| b.request.board_file.is_none())
        {
            return false;
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
            SceneSurface::Board => self.workspace().scene.scene_id.clone(),
            SceneSurface::Schematic => self
                .workspace()
                .schematic_scene
                .as_ref()
                .unwrap()
                .scene_id
                .clone(),
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
        self.selection_gesture = Some(gesture);
        true
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
        let changed = self.selection_gesture.take().is_some();
        self.session.workspace_mut().ui.selection_region = None;
        self.selection_tick = None;
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
        if scene.is_none_or(|s| s.scene_id != gesture.source_revision) {
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
        let region = match SelectionRegion::new(&gesture.boundary()) {
            Ok(r) => r,
            Err(e) => {
                self.selection_notice(format!("Region refused: {e:?}"));
                return Some(true);
            }
        };
        let state = self.workspace();
        let native = self.native_selection.as_ref().unwrap();
        let scene = if content == datum_gui_protocol::PaneContent::Schematic {
            state.schematic_scene.as_ref().unwrap()
        } else {
            &state.scene
        };
        let catalogue = qualification_catalog(&native.model, &native.authority, scene);
        let bindings = datum_gui_protocol::selection_scene::SceneSelectionBindings::capture(
            scene,
            &native.authority,
        );
        let eligible: std::collections::BTreeSet<_> = bindings
            .iter()
            .filter(|(_, binding)| {
                state.ui.filters.show_authored
                    && binding.visible(|layer| {
                        state
                            .ui
                            .filters
                            .layer_visibility
                            .get(layer)
                            .copied()
                            .unwrap_or(true)
                    })
            })
            .map(|(_, binding)| binding.acquired.clone())
            .collect();
        let mut candidates = std::collections::BTreeSet::new();
        for (id, geometry) in catalogue {
            if !eligible.contains(&id)
                || state
                    .ui
                    .filters
                    .excluded_selection_classes
                    .contains(&id.class)
            {
                continue;
            }
            let qualified = match geometry {
                Ok(Geometry::Anchors(anchors)) => region.anchors(&anchors),
                Ok(Geometry::Layout(layout)) => region.majority_layout(&layout),
                Ok(Geometry::Disk { center, radius }) => region.encloses_disk(center, radius),
                Ok(Geometry::Filled(polygons)) => {
                    let mut complete = !polygons.is_empty();
                    let mut error = None;
                    for polygon in polygons {
                        match region.encloses(&polygon) {
                            Ok(true) => {}
                            Ok(false) => complete = false,
                            Err(e) => {
                                error = Some(e);
                                break;
                            }
                        }
                    }
                    match error {
                        Some(e) => Err(e),
                        None => Ok(complete),
                    }
                }
                Err(reason) => {
                    self.selection_notice(reason);
                    return Some(true);
                }
            };
            match qualified {
                Ok(true) => {
                    candidates.insert(id);
                }
                Ok(false) => {}
                Err(e) => {
                    self.selection_notice(format!("Region refused atomically: {e:?}"));
                    return Some(true);
                }
            }
        }
        self.native_selection.as_mut().unwrap().reset_click_depth();
        Some(self.publish_native_acquisition(
            candidates,
            gesture.mode,
            AcquisitionEntry::Region,
            None,
        ))
    }
    pub(super) fn poll_native_selection_tick(&mut self) -> bool {
        if self
            .selection_tick
            .is_none_or(|due| due > std::time::Instant::now())
        {
            return false;
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
