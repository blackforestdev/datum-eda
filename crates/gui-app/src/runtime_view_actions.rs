//! View-local menu-action dispatch and the cursor crosshair (decision 023
//! UVT-005). Split out of `main.rs`'s `impl Runtime` to give that monolith
//! source-health headroom (decision 022); behavior unchanged. Owns the
//! `gui_local` menu-action match (camera/pane view ops) plus the `view.cursor.*`
//! crosshair radio group and its keyboard cycle. All of this is consumer/session
//! UI state — camera, pane layout, crosshair style — and is NEVER journaled.
//!
//! Reaches the Runtime state and the crate's camera/pane helpers via `use
//! super::*`, exactly as the sibling runtime action modules do.

use super::*;

impl Runtime {
    pub(super) fn activate_gui_local_menu_action(&mut self, action: &str) -> bool {
        match action {
            "view.fit" => {
                self.fit_camera();
                self.log_review_event("menu view.fit".to_string());
                true
            }
            "view.zoom_in" => {
                self.zoom_view_from_menu(1.2);
                self.log_review_event("menu view.zoom_in".to_string());
                true
            }
            "view.zoom_out" => {
                self.zoom_view_from_menu(1.0 / 1.2);
                self.log_review_event("menu view.zoom_out".to_string());
                true
            }
            "terminal.toggle" => {
                if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
                    self.close_active_dock()
                } else {
                    self.set_active_dock(DockTab::Terminal)
                }
            }
            // Workspace pane ops (decision 021). These reach the same warm pane-op
            // path the FEEL breakpoint proves is zero-re-resolve. The menu manifest
            // does not emit these ids yet (that is the later bindings pass); wiring
            // them here keeps the ops reachable through the one action dispatch.
            "view.split_vertical" => {
                self.pane_split_focused(datum_gui_protocol::SplitOrientation::Vertical);
                self.log_review_event("menu view.split_vertical".to_string());
                true
            }
            "view.split_horizontal" => {
                self.pane_split_focused(datum_gui_protocol::SplitOrientation::Horizontal);
                self.log_review_event("menu view.split_horizontal".to_string());
                true
            }
            "view.close_pane" => {
                self.pane_close_focused();
                self.log_review_event("menu view.close_pane".to_string());
                true
            }
            "view.focus_next" => {
                self.pane_focus_next();
                self.log_review_event("menu view.focus_next".to_string());
                true
            }
            "view.focus_prev" => {
                self.pane_focus_prev();
                self.log_review_event("menu view.focus_prev".to_string());
                true
            }
            "view.maximize_pane" => {
                self.pane_toggle_zoom();
                self.log_review_event("menu view.maximize_pane".to_string());
                true
            }
            "view.preset_single" => {
                self.pane_apply_preset(datum_gui_protocol::WorkspacePreset::Single);
                self.log_review_event("menu view.preset_single".to_string());
                true
            }
            "view.preset_board_schematic" => {
                self.pane_apply_preset(datum_gui_protocol::WorkspacePreset::BoardSchematic);
                self.log_review_event("menu view.preset_board_schematic".to_string());
                true
            }
            "view.fill_board" => {
                self.pane_set_focused_content(datum_gui_protocol::PaneContent::Board);
                self.log_review_event("menu view.fill_board".to_string());
                true
            }
            "view.fill_schematic" => {
                self.pane_set_focused_content(datum_gui_protocol::PaneContent::Schematic);
                self.log_review_event("menu view.fill_schematic".to_string());
                true
            }
            // Secondary view-local actions (crosshair radio group, decision 023)
            // and the unwired-narration fallback live in the runtime module to keep
            // this monolith under its source-health ceiling.
            other => self.activate_view_local_action(other),
        }
    }

    fn zoom_view_from_menu(&mut self, zoom_delta: f32) {
        let prepared = self.prepared_scene();
        let scene_viewport = prepared.scene_viewport;
        let bounds = self.workspace().scene.bounds.clone();
        self.camera.zoom_about_screen_point(
            scene_viewport,
            &bounds,
            scene_viewport.x + scene_viewport.width * 0.5,
            scene_viewport.y + scene_viewport.height * 0.5,
            zoom_delta,
        );
        self.invalidate_scene();
    }

    /// Secondary `gui_local` view actions dispatched from
    /// `activate_gui_local_menu_action`'s fallback: the `view.cursor.*` crosshair
    /// radio group (decision 023 UVT-005), else the "unwired" narration.
    pub(super) fn activate_view_local_action(&mut self, action: &str) -> bool {
        use datum_gui_protocol::CrosshairStyle;
        match action {
            "view.cursor.full" => self.set_crosshair_style(CrosshairStyle::FullViewport),
            "view.cursor.small" => self.set_crosshair_style(CrosshairStyle::Local),
            "view.cursor.none" => self.set_crosshair_style(CrosshairStyle::None),
            other => {
                self.push_terminal_line(format!("menu action {other} is view-local but unwired"));
                self.invalidate_frame();
                true
            }
        }
    }

    /// Set the cursor-crosshair style — a session UI preference, never journaled.
    /// Backs the `view.cursor.*` menu items and the crosshair-cycle keybinding.
    pub(super) fn set_crosshair_style(
        &mut self,
        style: datum_gui_protocol::CrosshairStyle,
    ) -> bool {
        if self.session.workspace().ui.crosshair_style != style {
            self.session.workspace_mut().ui.crosshair_style = style;
            self.log_review_event(format!("crosshair style {style:?}"));
            self.refresh_interaction_overlay();
        }
        true
    }

    /// Cycle FullViewport -> Local -> None -> FullViewport. Reachable now via the
    /// crosshair-cycle keybinding (`c`) so the preference is usable while the
    /// broader menu-action work matures.
    pub(super) fn cycle_crosshair_style(&mut self) -> bool {
        use datum_gui_protocol::CrosshairStyle;
        let next = match self.session.workspace().ui.crosshair_style {
            CrosshairStyle::FullViewport => CrosshairStyle::Local,
            CrosshairStyle::Local => CrosshairStyle::None,
            CrosshairStyle::None => CrosshairStyle::FullViewport,
        };
        self.set_crosshair_style(next)
    }
}
