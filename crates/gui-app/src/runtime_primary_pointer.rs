//! Primary-pointer dispatch ownership, extracted from the runtime monolith
//! under decision 022. This module resolves pane focus and routes the gesture;
//! it does not introduce a second mutation path.

use super::*;

impl Runtime {
    pub(super) fn trace_click(&self, message: String) {
        if std::env::var_os("DATUM_TRACE_CLICKS").is_some() {
            eprintln!("[datum-click] {message}");
        }
    }

    pub(super) fn handle_primary_click(&mut self) -> bool {
        if self.dismiss_marking_menu() {
            return true;
        }
        let Some((x, y)) = self.last_cursor_pos else {
            self.trace_click("primary click ignored: no cursor position".to_string());
            return false;
        };
        // Focus and dispatch are one gesture: after activating a different pane,
        // continue resolving this same click in that pane's camera/content.
        let mut focus_changed = false;
        if let Some(pane_id) = self.pane_at_screen(x, y) {
            // TF-01 deliberate exit: a canvas click is editor keyboard entry,
            // releasing any terminal/overlay key ownership before dispatch.
            self.set_application_focus(keyboard_focus::focus_after_canvas_click(pane_id));
            if pane_id != self.workspace().ui.layout.focused {
                self.swap_pane_focus(|layout| layout.focused = pane_id);
                self.log_console_echo_for_target(
                    ConsoleFeedbackSource::Viewport,
                    format!("pane:{}", pane_id.0),
                    "Pane focused",
                );
                self.trace_click(format!(
                    "primary click ({x:.1}, {y:.1}) focus-swapped to pane {}",
                    pane_id.0
                ));
                focus_changed = true;
            }
        }
        let prepared_started = std::time::Instant::now();
        let (prepared_target, world_point) = {
            let prepared = self.prepared_scene();
            (
                prepared.hit_test(x, y).cloned(),
                prepared.world_point_at_screen(x, y),
            )
        };
        let prepared_elapsed = prepared_started.elapsed();
        if self.terminal_clipboard_menu_active()
            && !matches!(
                prepared_target.as_ref(),
                Some(
                    HitTarget::TerminalClipboardCopy
                        | HitTarget::TerminalClipboardPaste
                        | HitTarget::TerminalProfileNext
                        | HitTarget::TerminalThemeNext
                        | HitTarget::TerminalLinkCopy
                        | HitTarget::TerminalLinkOpen
                )
            )
        {
            self.dismiss_terminal_clipboard_menu();
            return true;
        }
        if let Some(target) = prepared_target {
            self.trace_click(format!(
                "primary click ({x:.1}, {y:.1}) prepared target {target:?}; prepare {}ms; dock {:?}",
                prepared_elapsed.as_millis(),
                self.workspace().ui.active_dock_tab
            ));
            return self.select_hit_target(&target) || focus_changed;
        }
        if let Some((world_point, SceneSurface::Schematic)) = world_point {
            // UVT-004 resolves a schematic world point and symbol-region hit;
            // selection dispatch remains owned by the established session path.
            return self.resolve_schematic_primary_click((x, y), world_point) || focus_changed;
        }
        if let Some((world_point, SceneSurface::Board)) = world_point {
            let retained_started = std::time::Instant::now();
            let retained_target = {
                let retained = self.retained_scene.get_or_insert_with(|| {
                    RetainedScene::from_workspace_for_surface(
                        self.session.workspace(),
                        self.config.width,
                        self.config.height,
                        self.scale_factor,
                    )
                });
                retained
                    .hit_test_authored_world(world_point, self.session.workspace())
                    .cloned()
            };
            let retained_elapsed = retained_started.elapsed();
            let target_object_id = match &retained_target {
                Some(HitTarget::AuthoredObject(id)) | Some(HitTarget::ReviewAction(id)) => {
                    Some(id.clone())
                }
                _ => None,
            };
            if self.handle_authoring_canvas_click(world_point, target_object_id) {
                self.trace_click(format!(
                    "primary click ({x:.1}, {y:.1}) world ({}, {}) handled by authoring tool {}; prepare {}ms; retained {}ms",
                    world_point.x,
                    world_point.y,
                    self.workspace().tool.label(),
                    prepared_elapsed.as_millis(),
                    retained_elapsed.as_millis()
                ));
                return true;
            }
            if let Some(target) = retained_target {
                self.trace_click(format!(
                    "primary click ({x:.1}, {y:.1}) world ({}, {}) retained target {target:?}; prepare {}ms; retained {}ms; dock {:?}",
                    world_point.x,
                    world_point.y,
                    prepared_elapsed.as_millis(),
                    retained_elapsed.as_millis(),
                    self.workspace().ui.active_dock_tab
                ));
                return self.select_hit_target(&target) || focus_changed;
            }
            self.trace_click(format!(
                "primary click ({x:.1}, {y:.1}) world ({}, {}) no retained target; prepare {}ms; retained {}ms; dock {:?}",
                world_point.x,
                world_point.y,
                prepared_elapsed.as_millis(),
                retained_elapsed.as_millis(),
                self.workspace().ui.active_dock_tab
            ));
            return focus_changed;
        }
        self.trace_click(format!(
            "primary click ({x:.1}, {y:.1}) no prepared or viewport target; prepare {}ms; dock {:?}",
            prepared_elapsed.as_millis(),
            self.workspace().ui.active_dock_tab
        ));
        focus_changed
    }
}
