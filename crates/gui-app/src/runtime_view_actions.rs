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
    pub(super) fn activate_gui_local_menu_action(&mut self, action: &str) -> bool {
        match action {
            "view.fit" => {
                self.fit_camera();
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "view fit");
                true
            }
            "view.zoom_in" => {
                self.zoom_focused_view(1.2);
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "view zoom in");
                true
            }
            "view.zoom_out" => {
                self.zoom_focused_view(1.0 / 1.2);
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "view zoom out");
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
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "split pane vertical");
                true
            }
            "view.split_horizontal" => {
                self.pane_split_focused(datum_gui_protocol::SplitOrientation::Horizontal);
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "split pane horizontal");
                true
            }
            "view.close_pane" => {
                self.pane_close_focused();
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "close focused pane");
                true
            }
            "view.focus_next" => {
                self.pane_focus_next();
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "focus next pane");
                true
            }
            "view.focus_prev" => {
                self.pane_focus_prev();
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "focus previous pane");
                true
            }
            "view.maximize_pane" => {
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
            "view.preset_single" => {
                self.pane_apply_preset(datum_gui_protocol::WorkspacePreset::Single);
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "workspace preset single");
                true
            }
            "view.preset_board_schematic" => {
                self.pane_apply_preset(datum_gui_protocol::WorkspacePreset::BoardSchematic);
                self.log_console_echo(
                    ConsoleFeedbackSource::Viewport,
                    "workspace preset board and schematic",
                );
                true
            }
            "view.fill_board" => {
                self.pane_set_focused_content(datum_gui_protocol::PaneContent::Board);
                self.log_console_echo(ConsoleFeedbackSource::Viewport, "focused pane shows board");
                true
            }
            "view.fill_schematic" => {
                self.pane_set_focused_content(datum_gui_protocol::PaneContent::Schematic);
                self.log_console_echo(
                    ConsoleFeedbackSource::Viewport,
                    "focused pane shows schematic",
                );
                true
            }
            // Secondary view-local actions (crosshair radio group, decision 023)
            // and the unwired-narration fallback live in the runtime module to keep
            // this monolith under its source-health ceiling.
            other => self.activate_view_local_action(other),
        }
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
            "view.console_history" => self.toggle_console_history(),
            "view.console_duration.4s" => {
                self.set_console_duration(datum_gui_protocol::ConsoleFeedbackDuration::FourSeconds)
            }
            "view.console_duration.6s" => {
                self.set_console_duration(datum_gui_protocol::ConsoleFeedbackDuration::SixSeconds)
            }
            "view.console_duration.10s" => {
                self.set_console_duration(datum_gui_protocol::ConsoleFeedbackDuration::TenSeconds)
            }
            "view.console_duration.never" => {
                self.set_console_duration(datum_gui_protocol::ConsoleFeedbackDuration::Never)
            }
            other => {
                self.log_console_refusal_for_action(
                    ConsoleFeedbackSource::Viewport,
                    other,
                    "View action is unavailable",
                );
                self.invalidate_frame();
                true
            }
        }
    }

    pub(super) fn set_console_duration(
        &mut self,
        duration: datum_gui_protocol::ConsoleFeedbackDuration,
    ) -> bool {
        self.session
            .workspace_mut()
            .ui
            .console
            .set_duration_preference(duration);
        if console_preferences::persist_duration_preference(duration).is_err() {
            self.log_console_refusal(
                ConsoleFeedbackSource::Viewport,
                "Console duration applies for this session but could not be saved",
            );
            return true;
        }
        let label = match duration {
            datum_gui_protocol::ConsoleFeedbackDuration::FourSeconds => "4 seconds",
            datum_gui_protocol::ConsoleFeedbackDuration::SixSeconds => "6 seconds",
            datum_gui_protocol::ConsoleFeedbackDuration::TenSeconds => "10 seconds",
            datum_gui_protocol::ConsoleFeedbackDuration::Never => "never hide",
        };
        self.log_console_echo(
            ConsoleFeedbackSource::Viewport,
            format!("Console duration {label}"),
        );
        true
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
