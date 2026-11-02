//! View-local menu-action dispatch and the cursor crosshair (decision 023
//! UVT-005). Split out of `main.rs`'s `impl Runtime` to give that monolith
//! source-health headroom (decision 022); behavior unchanged. Owns the
//! typed `gui_local` menu-action dispatch (camera/pane view ops) plus `view.cursor.*`
//! crosshair radio group and its keyboard cycle. All of this is consumer/session
//! UI state — camera, pane layout, crosshair style — and is NEVER journaled.
//!
//! Reaches the Runtime state and the crate's camera/pane helpers via `use
//! super::*`, exactly as the sibling runtime action modules do.

use super::*;

impl Runtime {
    pub(super) fn capture_native_authoring_view(
        &self,
        pane: Option<datum_gui_protocol::PaneId>,
    ) -> Result<datum_gui_protocol::AuthoringViewBasis, String> {
        datum_gui_protocol::AuthoringViewBasis::capture(
            self.workspace(),
            pane.ok_or("Authoring requires the actual source pane")?,
        )
    }

    pub(super) fn refuse_native_authoring_view(&mut self, reason: String) {
        // A queued immutable canonical request is not redirected or cancelled.
        if !self.native_edit.pending() {
            self.cancel_native_schematic();
            self.cancel_native_move();
            self.session.workspace_mut().authoring.gesture.view_basis = None;
        }
        self.log_console_refusal(ConsoleFeedbackSource::Tool, reason);
    }

    pub(super) fn guard_native_authoring_view(
        &mut self,
        pane: Option<datum_gui_protocol::PaneId>,
    ) -> bool {
        let valid = pane.is_some_and(|pane| {
            self.workspace()
                .authoring
                .gesture
                .view_basis
                .as_ref()
                .is_some_and(|basis| basis.is_current(self.workspace(), pane))
        });
        if !valid {
            self.refuse_native_authoring_view(
                "Authoring view changed; restart in the supported source pane".into(),
            );
        }
        valid
    }
}
#[path = "runtime_view_actions/action_evidence.rs"]
pub(crate) mod action_evidence;
use action_evidence::EntrySurface;

fn terminal_owns_maximize(focus: ApplicationFocus, active_dock: Option<DockTab>) -> bool {
    focus == ApplicationFocus::Terminal && active_dock == Some(DockTab::Terminal)
}

fn style_label(style: datum_gui_protocol::CrosshairStyle) -> &'static str {
    match style {
        datum_gui_protocol::CrosshairStyle::FullViewport => "full viewport",
        datum_gui_protocol::CrosshairStyle::Local => "local",
        datum_gui_protocol::CrosshairStyle::None => "off",
    }
}

impl Runtime {
    pub(super) fn activate_gui_local_menu_action(
        &mut self,
        action: &str,
        surface: EntrySurface,
    ) -> bool {
        use datum_gui_protocol::gui_menu_model::action_registry::{
            self, ActionHandler, GuiLocalHandler,
        };
        let Some(handler) = action_registry::gui_local_handler(action) else {
            self.log_console_refusal_for_action(
                ConsoleFeedbackSource::Menu,
                action,
                action_registry::UNAVAILABLE_GUI_LOCAL_REASON,
            );
            self.invalidate_frame();
            return true;
        };
        match handler {
            GuiLocalHandler::Registered(_) => {
                match action_registry::admit(action, self.workspace()) {
                    Ok(ActionHandler::OpenLibrary) => {
                        self.open_library();
                        self.trace_action_attempt(action, surface, true);
                    }
                    Ok(ActionHandler::FocusNavigator) => {
                        self.focus_schematic_navigator();
                        self.trace_action_attempt(action, surface, true);
                    }
                    Ok(ActionHandler::FitCamera) => {
                        self.fit_camera();
                        self.trace_action_attempt(action, surface, true);
                        self.log_console_echo(ConsoleFeedbackSource::Viewport, "view fit");
                    }
                    Err(reason) => {
                        self.trace_action_attempt(action, surface, false);
                        self.log_console_refusal_for_action(
                            ConsoleFeedbackSource::Viewport,
                            action,
                            reason,
                        );
                        self.invalidate_frame();
                    }
                }
                true
            }
            GuiLocalHandler::SaveProject => self
                .queue_native_edit(datum_gui_protocol::native_edit_client::NativeEditAction::Save),
            GuiLocalHandler::NewProject => {
                self.close_project_open();
                self.open_new_project()
            }
            GuiLocalHandler::OpenProject => self.open_project(),
            GuiLocalHandler::PowerPort => {
                if let Some(reason) =
                    datum_gui_protocol::gui_menu_model::native_edit_unavailable_reason(
                        self.workspace(),
                        "datum.schematic.place_label",
                    )
                {
                    self.log_console_refusal_for_action(
                        ConsoleFeedbackSource::Menu,
                        action,
                        reason,
                    );
                    self.invalidate_frame();
                    true
                } else {
                    self.begin_native_label(eda_engine::schematic::LabelKind::Power)
                }
            }
            GuiLocalHandler::OpenRecentProject(index) => self.open_recent_project(index),
            GuiLocalHandler::GlobalPreferences => self.open_global_preferences(),
            GuiLocalHandler::ProjectPreferences => self.open_project_preferences(),
            GuiLocalHandler::ZoomIn => {
                self.zoom_focused_view(1.2);
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "view zoom in");
                true
            }
            GuiLocalHandler::ZoomOut => {
                self.zoom_focused_view(1.0 / 1.2);
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "view zoom out");
                true
            }
            GuiLocalHandler::ToggleTerminal => {
                if matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
                    self.close_active_dock()
                } else {
                    self.set_active_dock(DockTab::Terminal)
                }
            }
            // Workspace pane ops (decision 021). These reach the same warm pane-op
            // path the FEEL breakpoint proves is zero-re-resolve. Typed targets
            // keep their menu and keyboard entry surfaces on one dispatch path.
            GuiLocalHandler::SplitVertical => {
                self.pane_split_focused(datum_gui_protocol::SplitOrientation::Vertical);
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "split pane vertical");
                true
            }
            GuiLocalHandler::SplitHorizontal => {
                self.pane_split_focused(datum_gui_protocol::SplitOrientation::Horizontal);
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "split pane horizontal");
                true
            }
            GuiLocalHandler::ClosePane => {
                self.pane_close_focused();
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "close focused pane");
                true
            }
            GuiLocalHandler::FocusNext => {
                self.pane_focus_next();
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "focus next pane");
                true
            }
            GuiLocalHandler::FocusPrevious => {
                self.pane_focus_prev();
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "focus previous pane");
                true
            }
            GuiLocalHandler::MaximizePane => {
                if terminal_owns_maximize(
                    self.application_focus(),
                    self.workspace().ui.active_dock_tab,
                ) {
                    self.toggle_terminal_maximized();
                } else {
                    self.pane_toggle_zoom();
                }
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "toggle pane maximize");
                true
            }
            GuiLocalHandler::PresetSingle => {
                self.pane_apply_preset(datum_gui_protocol::WorkspacePreset::Single);
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "workspace preset single");
                true
            }
            GuiLocalHandler::PresetBoardSchematic => {
                self.pane_apply_preset(datum_gui_protocol::WorkspacePreset::BoardSchematic);
                self.log_console_echo(
                    ConsoleFeedbackSource::Viewport,
                    "workspace preset board and schematic",
                );
                true
            }
            GuiLocalHandler::FillBoard => {
                self.pane_set_focused_content(datum_gui_protocol::PaneContent::Board);
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "focused pane shows board");
                true
            }
            GuiLocalHandler::FillSchematic => {
                self.pane_set_focused_content(datum_gui_protocol::PaneContent::Schematic);
                self.log_console_echo(
                    ConsoleFeedbackSource::Viewport,
                    "focused pane shows schematic",
                );
                true
            }
            GuiLocalHandler::CursorFull => {
                self.set_crosshair_style(datum_gui_protocol::CrosshairStyle::FullViewport)
            }
            GuiLocalHandler::CursorSmall => {
                self.set_crosshair_style(datum_gui_protocol::CrosshairStyle::Local)
            }
            GuiLocalHandler::CursorNone => {
                self.set_crosshair_style(datum_gui_protocol::CrosshairStyle::None)
            }
            GuiLocalHandler::ConsoleHistory => self.toggle_console_history(),
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
            self.log_console_echo(
                ConsoleFeedbackSource::Viewport,
                format!("crosshair style {}", style_label(style)),
            );
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

#[cfg(test)]
mod terminal_maximize_tests {
    use super::*;

    #[test]
    fn maximize_action_follows_the_application_focus_authority() {
        assert!(terminal_owns_maximize(
            ApplicationFocus::Terminal,
            Some(DockTab::Terminal)
        ));
        assert!(!terminal_owns_maximize(
            ApplicationFocus::Editor(datum_gui_protocol::PaneId(1)),
            Some(DockTab::Terminal)
        ));
        assert!(!terminal_owns_maximize(ApplicationFocus::Terminal, None));
    }
}
