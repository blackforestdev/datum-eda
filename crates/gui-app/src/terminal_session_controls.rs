use crate::terminal_activity_snapshot::load_terminal_activity_summary_lines;

use super::{DockTab, Runtime};

impl Runtime {
    pub(super) fn refresh_terminal_activity_summary(&mut self) -> bool {
        let next = match load_terminal_activity_summary_lines(
            &self.terminal_sessions.active_event_log_path(),
            4,
        ) {
            Ok(lines) => lines,
            Err(err) => vec![format!("activity summary unavailable: {err}")],
        };
        let ui = &mut self.session.workspace_mut().ui;
        if ui.terminal.activity_summary == next {
            return false;
        }
        ui.terminal.activity_summary = next;
        self.sync_terminal_tabs();
        self.invalidate_frame();
        true
    }

    pub(super) fn sync_terminal_tabs(&mut self) {
        self.terminal_sessions
            .sync_lane_tabs(&mut self.session.workspace_mut().ui.terminal);
        self.session.workspace_mut().ui.terminal.rename_session_id =
            self.terminal_rename_session_id.clone();
    }

    pub(super) fn spawn_terminal_session_tab(&mut self) -> bool {
        match self
            .terminal_sessions
            .spawn_and_activate(&self.terminal_launch_context)
        {
            Ok(session_id) => {
                let session_id = session_id.to_string();
                self.push_terminal_line(format!("opened terminal session {session_id}"));
                self.set_active_dock(DockTab::Terminal);
                self.sync_terminal_tabs();
                self.resize_terminal_to_dock();
            }
            Err(err) => self.push_terminal_line(format!("terminal session open failed: {err}")),
        }
        true
    }

    pub(super) fn rename_active_terminal_session(&mut self) -> bool {
        let session_id = self.terminal_sessions.active().session_id().to_string();
        let label = self.terminal_sessions.active_label().to_string();
        self.terminal_rename_session_id = Some(session_id.clone());
        let ui = &mut self.session.workspace_mut().ui;
        ui.active_dock_tab = Some(DockTab::Terminal);
        ui.terminal.rename_session_id = Some(session_id);
        ui.terminal.input = label;
        ui.terminal.cursor = ui.terminal.input.chars().count();
        self.invalidate_frame();
        true
    }

    pub(super) fn submit_terminal_rename_input(&mut self) -> bool {
        let Some(session_id) = self.terminal_rename_session_id.clone() else {
            return false;
        };
        let label = self
            .session
            .workspace()
            .ui
            .terminal
            .input
            .trim()
            .to_string();
        if label.is_empty() {
            return self.cancel_terminal_rename();
        }
        match self.terminal_sessions.rename(&session_id, label.clone()) {
            Ok(()) => {
                self.push_terminal_line(format!("renamed active terminal session {label}"));
                self.clear_terminal_rename_editor();
            }
            Err(err) => self.push_terminal_line(format!("terminal session rename failed: {err}")),
        }
        true
    }

    pub(super) fn cancel_terminal_rename(&mut self) -> bool {
        if self.terminal_rename_session_id.is_none() {
            return false;
        }
        self.push_terminal_line("terminal session rename canceled".to_string());
        self.clear_terminal_rename_editor();
        true
    }

    fn clear_terminal_rename_editor(&mut self) {
        self.terminal_rename_session_id = None;
        let ui = &mut self.session.workspace_mut().ui;
        ui.terminal.rename_session_id = None;
        ui.terminal.input.clear();
        ui.terminal.cursor = 0;
        self.sync_terminal_tabs();
        self.invalidate_frame();
    }

    pub(super) fn close_active_terminal_session(&mut self) -> bool {
        self.clear_terminal_rename_editor();
        match self
            .terminal_sessions
            .close_active(&mut self.session.workspace_mut().ui.terminal)
        {
            Ok(()) => {
                self.refresh_terminal_context_snapshot();
                self.refresh_terminal_activity_summary();
                self.sync_terminal_tabs();
                self.resize_terminal_to_dock();
            }
            Err(err) => self.push_terminal_line(format!("terminal session close failed: {err}")),
        }
        true
    }

    pub(super) fn detach_active_terminal_session(&mut self) -> bool {
        self.clear_terminal_rename_editor();
        match self
            .terminal_sessions
            .detach_active(&mut self.session.workspace_mut().ui.terminal)
        {
            Ok(()) => {
                self.push_terminal_line("detached active terminal session".to_string());
                self.sync_terminal_tabs();
            }
            Err(err) => self.push_terminal_line(format!("terminal session detach failed: {err}")),
        }
        true
    }
}
