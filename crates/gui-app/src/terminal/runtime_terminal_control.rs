//! Terminal control, session lifecycle and clipboard interactions.
use crate::*;

impl Runtime {
    // T0-C01 (DATUM_NATIVE_TERMINAL_SPEC.md) / decision 027 FT-001: there is
    // deliberately NO `push_terminal_line` here. Terminal cells are mutated
    // only by PTY bytes interpreted by the terminal core. Terminal lifecycle
    // and diagnostic messages stay in terminal chrome, never the Datum Console
    // or the terminal grid.

    pub(crate) fn handle_terminal_key_input(&mut self, event: &KeyEvent) -> bool {
        if self.handle_terminal_clipboard_confirmation_key(event) {
            return true;
        }
        if self.handle_terminal_link_confirmation_key(event) {
            return true;
        }
        if self.handle_terminal_search_key(event) {
            return true;
        }
        let application_cursor_keys = self.workspace().ui.terminal.application_cursor_keys;
        let application_keypad = self.workspace().ui.terminal.application_keypad;
        let action = terminal_key_action(
            event,
            self.modifiers,
            application_cursor_keys,
            application_keypad,
        );
        if let Some(handled) = self.handle_close_confirmation_action(&action) {
            return handled;
        }
        match action {
            TerminalKeyAction::CoreKey(input) => {
                match self.terminal_sessions.encode_active_key(&input) {
                    Ok(Some(bytes)) => self.write_foreign_shell_bytes(&bytes),
                    Ok(None) => true,
                    Err(err) => {
                        self.log_terminal_event(format!("terminal key encoding failed: {err}"));
                        true
                    }
                }
            }
            TerminalKeyAction::NewSession => self.spawn_terminal_session_tab(),
            TerminalKeyAction::SplitRight => {
                self.spawn_terminal_split(datum_gui_protocol::TerminalSplitDirection::SideBySide)
            }
            TerminalKeyAction::SplitDown => {
                self.spawn_terminal_split(datum_gui_protocol::TerminalSplitDirection::Stacked)
            }
            TerminalKeyAction::TerminateSession => {
                self.terminate_terminal_session();
                true
            }
            TerminalKeyAction::CloseSession => self.close_active_terminal_session(),
            TerminalKeyAction::RestartSession => {
                self.restart_terminal_session();
                true
            }
            TerminalKeyAction::ScrollbackPageUp => {
                self.scroll_terminal_scrollback(terminal_scrollback_page_step(self.workspace()));
                true
            }
            TerminalKeyAction::ScrollbackPageDown => {
                self.scroll_terminal_scrollback_down(terminal_scrollback_page_step(
                    self.workspace(),
                ));
                true
            }
            TerminalKeyAction::ScrollbackTop => {
                self.scroll_terminal_scrollback_to_top();
                true
            }
            TerminalKeyAction::ScrollbackBottom => {
                self.scroll_terminal_scrollback_to_bottom();
                true
            }
            TerminalKeyAction::CopyClipboard => self.copy_terminal_scrollback(),
            TerminalKeyAction::PasteClipboard => self.paste_terminal_input(),
            TerminalKeyAction::Search => {
                self.begin_terminal_search();
                true
            }
            TerminalKeyAction::FontZoomIn => self.adjust_terminal_font_zoom(1),
            TerminalKeyAction::FontZoomOut => self.adjust_terminal_font_zoom(-1),
            TerminalKeyAction::FontZoomReset => self.reset_terminal_font_zoom(),
            TerminalKeyAction::Ignore => false,
        }
    }

    pub(crate) fn scroll_terminal_scrollback(&mut self, delta: usize) {
        let max = self.terminal_sessions.active_render_row_count();
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.scroll_offset = (terminal.scroll_offset + delta).min(max);
        self.invalidate_frame();
    }

    pub(crate) fn scroll_terminal_scrollback_down(&mut self, delta: usize) {
        let terminal = &mut self.session.workspace_mut().ui.terminal;
        terminal.scroll_offset = terminal.scroll_offset.saturating_sub(delta);
        self.invalidate_frame();
    }

    pub(crate) fn scroll_terminal_scrollback_to_top(&mut self) {
        self.session.workspace_mut().ui.terminal.scroll_offset =
            self.terminal_sessions.active_render_row_count();
        self.invalidate_frame();
    }

    pub(crate) fn scroll_terminal_scrollback_to_bottom(&mut self) {
        self.session.workspace_mut().ui.terminal.scroll_offset = 0;
        self.invalidate_frame();
    }

    pub(crate) fn report_terminal_focus_event(&mut self, focused: bool) {
        if !self.terminal_sessions.active_attached() {
            return;
        }
        let input = if focused {
            datum_terminal_core::FocusInput::Gained
        } else {
            datum_terminal_core::FocusInput::Lost
        };
        match self.terminal_sessions.encode_active_focus(input) {
            Ok(Some(bytes)) => {
                if let Err(err) = self.terminal_sessions.active().write_bytes(&bytes) {
                    self.log_terminal_event(format!("terminal focus report failed: {err}"));
                }
            }
            Ok(None) => {}
            Err(err) => self.log_terminal_event(format!("terminal focus encoding failed: {err}")),
        }
    }

    pub(crate) fn terminate_terminal_session(&mut self) {
        match self
            .terminal_sessions
            .terminate_active(&mut self.session.workspace_mut().ui.terminal)
        {
            Ok(()) => {}
            Err(err) => self.log_terminal_event(format!("terminal terminate failed: {err}")),
        }
        self.sync_terminal_tabs();
        self.invalidate_frame();
    }

    pub(crate) fn restart_terminal_session(&mut self) {
        match self.terminal_sessions.restart_active(
            &mut self.session.workspace_mut().ui.terminal,
            &self.terminal_launch_context,
        ) {
            Ok(()) => {
                self.log_terminal_event(
                    "terminal restart requested; waiting for verified teardown",
                );
                self.resize_terminal_to_dock();
            }
            Err(err) => self.log_terminal_event(format!("terminal restart failed: {err}")),
        }
        self.terminal_production_refresh_pending = false;
        self.terminal_workspace_refresh_pending = false;
        self.terminal_production_refresh_due = None;
        self.terminal_production_refresh_attempts = 0;
        self.sync_terminal_tabs();
        self.invalidate_frame();
    }

    pub(crate) fn activate_terminal_session(&mut self, session_id: &str) -> bool {
        if let Err(err) = self
            .terminal_sessions
            .activate_with_lane(session_id, &mut self.session.workspace_mut().ui.terminal)
        {
            self.log_terminal_event(format!("terminal session activate failed: {err}"));
            return true;
        }
        self.refresh_terminal_selection_context_if_changed();
        self.set_active_dock(DockTab::Terminal);
        self.refresh_terminal_activity_summary();
        self.sync_terminal_tabs();
        self.resize_terminal_to_dock();
        self.invalidate_frame();
        true
    }

    pub(crate) fn copy_terminal_scrollback(&mut self) -> bool {
        if !matches!(self.workspace().ui.active_dock_tab, Some(DockTab::Terminal)) {
            return false;
        }
        let text = match self.terminal_sessions.copy_active_selection() {
            Ok(text) => text,
            Err(_) => return false,
        };
        if self.write_clipboard_text(&text).is_err() {
            self.log_terminal_event("clipboard copy failed".to_string());
            return true;
        }
        self.log_terminal_event("terminal text copied".to_string());
        true
    }

    pub(crate) fn paste_terminal_input(&mut self) -> bool {
        let Ok(text) = self.read_clipboard_text() else {
            self.log_terminal_event("clipboard paste failed".to_string());
            return false;
        };
        if text.is_empty() {
            return false;
        }
        match self.terminal_input_owner() {
            keyboard_focus::TerminalInputOwner::AttachedPty => {
                match self.terminal_sessions.encode_active_paste(&text) {
                    Ok(Some(bytes)) => self.write_foreign_shell_bytes(&bytes),
                    Ok(None) => false,
                    Err(err) => {
                        self.log_terminal_event(format!("terminal paste encoding failed: {err}"));
                        true
                    }
                }
            }
            keyboard_focus::TerminalInputOwner::Unowned => false,
        }
    }

    pub(crate) fn read_clipboard_text(&mut self) -> Result<String> {
        if let Some(clipboard) = &mut self.clipboard
            && let Ok(text) = clipboard
                .get()
                .clipboard(LinuxClipboardKind::Clipboard)
                .text()
            && !text.is_empty()
        {
            return Ok(text);
        }
        self.read_clipboard_text_fallback()
    }

    pub(crate) fn write_clipboard_text(&mut self, text: &str) -> Result<()> {
        self.write_terminal_clipboard_text(
            datum_gui_protocol::TerminalClipboardSelection::Clipboard,
            text,
        )
    }

    pub(crate) fn write_terminal_clipboard_text(
        &mut self,
        selection: datum_gui_protocol::TerminalClipboardSelection,
        text: &str,
    ) -> Result<()> {
        let kind = match selection {
            datum_gui_protocol::TerminalClipboardSelection::Clipboard => {
                LinuxClipboardKind::Clipboard
            }
            datum_gui_protocol::TerminalClipboardSelection::Primary => LinuxClipboardKind::Primary,
        };
        if let Some(clipboard) = &mut self.clipboard
            && clipboard
                .set()
                .clipboard(kind)
                .text(text.to_string())
                .is_ok()
        {
            return Ok(());
        }
        self.write_clipboard_text_fallback(selection, text)
    }

    pub(crate) fn read_clipboard_text_fallback(&self) -> Result<String> {
        let output = Command::new("/usr/bin/xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
            .context("read clipboard with xclip")?;
        if !output.status.success() {
            anyhow::bail!("xclip clipboard read failed");
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub(crate) fn write_clipboard_text_fallback(
        &self,
        selection: datum_gui_protocol::TerminalClipboardSelection,
        text: &str,
    ) -> Result<()> {
        let selection = match selection {
            datum_gui_protocol::TerminalClipboardSelection::Clipboard => "clipboard",
            datum_gui_protocol::TerminalClipboardSelection::Primary => "primary",
        };
        let mut child = Command::new("/usr/bin/xclip")
            .args(["-selection", selection])
            .stdin(Stdio::piped())
            .spawn()
            .context("spawn xclip for clipboard write")?;
        let mut stdin = child.stdin.take().context("take xclip stdin")?;
        stdin
            .write_all(text.as_bytes())
            .context("write clipboard text to xclip")?;
        drop(stdin);
        let status = child.wait().context("wait for xclip clipboard write")?;
        if !status.success() {
            anyhow::bail!("xclip clipboard write failed");
        }
        Ok(())
    }

    pub(crate) fn log_terminal_event(&mut self, message: impl Into<String>) {
        self.session.workspace_mut().ui.terminal.status = message.into();
        self.invalidate_frame();
    }
}
